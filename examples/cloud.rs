use clap::Parser;
use reqwest::{
    header::{CONTENT_ENCODING, CONTENT_TYPE},
    Url,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

use scroll_proving_sdk::{
    config::{CloudProverConfig, Config},
    prover::{
        proving_service::{
            GetVkRequest, GetVkResponse, ProveRequest, ProveResponse, QueryTaskRequest,
            QueryTaskResponse, TaskStatus,
        },
        CircuitType, ProverBuilder, ProvingService,
    },
    utils::init_tracing,
};

#[derive(Parser, Debug)]
#[clap(disable_version_flag = true)]
struct Args {
    /// Path of config file
    #[arg(long = "config", default_value = "config.json")]
    config_file: String,
}

struct CloudProver {
    base_url: Url,
    api_key: String,
    client: ClientWithMiddleware,
    rt: tokio::runtime::Runtime,
}

#[derive(serde::Deserialize)]
struct VerificationKey {
    verification_key: String,
}

#[derive(serde::Deserialize)]
struct SindriTaskStatusResponse {
    pub proof_id: String,
    pub project_name: String,
    pub perfoman_verify: bool,
    pub status: SindriTaskStatus,
    pub compute_time_sec: Option<u64>,
    pub queue_time_sec: Option<u64>,
    pub verification_key: Option<VerificationKey>,
    pub proof: Option<String>,
    pub public: Option<String>,
    pub warnings: Option<Vec<String>>,
    pub error: Option<String>,
}

#[derive(serde::Deserialize)]
enum SindriTaskStatus {
    #[serde(rename = "Queued")]
    Queued,
    #[serde(rename = "In Progress")]
    Proving,
    #[serde(rename = "Ready")]
    Success,
    #[serde(rename = "Failed")]
    Failed,
}

impl From<SindriTaskStatus> for TaskStatus {
    fn from(status: SindriTaskStatus) -> Self {
        match status {
            SindriTaskStatus::Queued => TaskStatus::Queued,
            SindriTaskStatus::Proving => TaskStatus::Proving,
            SindriTaskStatus::Success => TaskStatus::Success,
            SindriTaskStatus::Failed => TaskStatus::Failed,
        }
    }
}

enum MethodClass {
    Circuit(CircuitType),
    Proof(String),
}

fn reformat_vk(vk_old: String) -> anyhow::Result<String> {
    log::debug!("vk_old: {:?}", vk_old);

    // decode base64 without padding
    let vk = base64::decode_config(vk_old, base64::URL_SAFE_NO_PAD)?;
    // encode with padding
    let vk_new = base64::encode_config(vk, base64::STANDARD);

    log::debug!("vk_new: {:?}", vk_new);

    Ok(vk_new)
}

impl ProvingService for CloudProver {
    fn is_local(&self) -> bool {
        false
    }

    fn get_vk(&self, req: GetVkRequest) -> GetVkResponse {
        if req.circuit_version != THIS_CIRCUIT_VERSION {
            return GetVkResponse {
                vk: String::new(),
                error: Some("circuit version mismatch".to_string()),
            };
        };

        #[derive(serde::Deserialize)]
        struct SindriGetDetailResponse {
            circuit_id: String,
            circuit_name: String,
            verification_key: VerificationKey,
        }

        match self
            .rt
            .block_on(self.get_with_token::<SindriGetDetailResponse>(
                MethodClass::Circuit(req.circuit_type),
                "detail",
                None,
            )) {
            Ok(resp) => match reformat_vk(resp.verification_key.verification_key) {
                Ok(vk) => GetVkResponse { vk, error: None },
                Err(e) => GetVkResponse {
                    vk: String::new(),
                    error: Some(e.to_string()),
                },
            },
            Err(e) => GetVkResponse {
                vk: String::new(),
                error: Some(e.to_string()),
            },
        }
    }

    fn prove(&self, req: ProveRequest) -> ProveResponse {
        if req.circuit_version != THIS_CIRCUIT_VERSION {
            return ProveResponse {
                task_id: String::new(),
                circuit_type: req.circuit_type,
                circuit_version: req.circuit_version,
                hard_fork_name: req.hard_fork_name,
                status: TaskStatus::Failed,
                created_at: 0,
                started_at: None,
                finished_at: None,
                compute_time_sec: None,
                input: Some(req.input.clone()),
                proof: None,
                vk: None,
                error: Some("circuit version mismatch".to_string()),
            };
        };

        #[derive(serde::Deserialize, serde::Serialize)]
        struct SindriProveRequest {
            proof_input: String,
            perform_verify: bool,
        }

        let sindri_req = SindriProveRequest {
            proof_input: req.input.clone(),
            perform_verify: true,
        };

        match self.rt.block_on(
            self.post_with_token::<SindriProveRequest, SindriTaskStatusResponse>(
                MethodClass::Circuit(req.circuit_type),
                "prove",
                &sindri_req,
            ),
        ) {
            Ok(resp) => ProveResponse {
                task_id: resp.proof_id,
                circuit_type: req.circuit_type,
                circuit_version: req.circuit_version,
                hard_fork_name: req.hard_fork_name,
                status: resp.status.into(),
                created_at: 0,     // TODO:
                started_at: None,  // TODO:
                finished_at: None, // TODO:
                compute_time_sec: resp.compute_time_sec,
                input: Some(req.input.clone()),
                proof: resp.proof,
                vk: resp.verification_key.map(|vk| vk.verification_key),
                error: resp.error,
            },
            Err(e) => ProveResponse {
                task_id: String::new(),
                circuit_type: req.circuit_type,
                circuit_version: req.circuit_version,
                hard_fork_name: req.hard_fork_name,
                status: TaskStatus::Failed,
                created_at: 0,
                started_at: None,
                finished_at: None,
                compute_time_sec: None,
                input: Some(req.input.clone()),
                proof: None,
                vk: None,
                error: Some(anyhow::anyhow!("failed to request proof: {e}").to_string()),
            },
        }
    }

    fn query_task(&self, req: QueryTaskRequest) -> QueryTaskResponse {
        let query_params: HashMap<String, String> = HashMap::from([
            ("include_proof".to_string(), "true".to_string()),
            ("include_public".to_string(), "true".to_string()),
            ("include_verification_key".to_string(), "true".to_string()),
        ]);

        match self
            .rt
            .block_on(self.get_with_token::<SindriTaskStatusResponse>(
                MethodClass::Proof(req.task_id.clone()),
                "detail",
                Some(query_params),
            )) {
            Ok(resp) => QueryTaskResponse {
                task_id: resp.proof_id,
                circuit_type: CircuitType::Undefined, // TODO:
                circuit_version: "".to_string(),
                hard_fork_name: "".to_string(),
                status: resp.status.into(),
                created_at: 0,     // TODO:
                started_at: None,  // TODO:
                finished_at: None, // TODO:
                compute_time_sec: resp.compute_time_sec,
                input: None,
                proof: resp.proof,
                vk: resp.verification_key.map(|vk| vk.verification_key),
                error: resp.error,
            },
            Err(e) => QueryTaskResponse {
                task_id: req.task_id,
                circuit_type: CircuitType::Undefined,
                circuit_version: "".to_string(),
                hard_fork_name: "".to_string(),
                status: TaskStatus::Queued,
                created_at: 0,
                started_at: None,
                finished_at: None,
                compute_time_sec: None,
                input: None,
                proof: None,
                vk: None,
                error: Some(anyhow::anyhow!("failed to query proof: {e}").to_string()),
            },
        }
    }
}

const THIS_CIRCUIT_VERSION: &str = "v0.13.1";

impl CloudProver {
    pub fn new(cfg: CloudProverConfig) -> Self {
        let client = ClientBuilder::new(reqwest::Client::new())
            // .with(RetryTransientMiddleware::new_with_policy(retry_policy)) // TODO: retry policy
            .build();

        let base_url = Url::parse(&cfg.base_url).expect("cannot parse cloud prover base_url");

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        Self {
            base_url,
            api_key: cfg.api_key,
            client,
            rt,
        }
    }

    fn build_url(
        &self,
        method_class: MethodClass,
        method: &str,
        query_params: Option<HashMap<String, String>>,
    ) -> anyhow::Result<Url> {
        let method_base = match method_class {
            MethodClass::Circuit(circuit_type) => {
                let circuit = match circuit_type {
                    CircuitType::Chunk => "chunk_prover",
                    CircuitType::Batch => "batch_prover",
                    CircuitType::Bundle => "bundle_prover",
                    CircuitType::Undefined => unreachable!("circuit type is undefined"),
                };
                format!("circuit/scroll-tech/{}:{}/", circuit, THIS_CIRCUIT_VERSION)
            }
            MethodClass::Proof(id) => format!("proof/{}/", id),
        };

        let mut url = self.base_url.join(&method_base)?.join(method)?;

        if let Some(params) = query_params {
            for (key, value) in params {
                url.query_pairs_mut().append_pair(&key, &value);
            }
        }

        Ok(url)
    }

    async fn post_with_token<Req, Resp>(
        &self,
        method_class: MethodClass,
        method: &str,
        req: &Req,
    ) -> anyhow::Result<Resp>
    where
        Req: ?Sized + Serialize,
        Resp: serde::de::DeserializeOwned,
    {
        let request_body = serde_json::to_string(req)?;

        self.request_with_token(method_class, method, None, Some(request_body))
            .await
    }

    async fn get_with_token<Resp>(
        &self,
        method_class: MethodClass,
        method: &str,
        query_params: Option<HashMap<String, String>>,
    ) -> anyhow::Result<Resp>
    where
        Resp: serde::de::DeserializeOwned,
    {
        self.request_with_token(method_class, method, query_params, None)
            .await
    }

    async fn request_with_token<Resp>(
        &self,
        method_class: MethodClass,
        method: &str,
        query_params: Option<HashMap<String, String>>,
        request_body: Option<String>,
    ) -> anyhow::Result<Resp>
    where
        Resp: serde::de::DeserializeOwned,
    {
        let url = self.build_url(method_class, method, query_params)?;

        log::info!("[sindri client]: {:?}", url.as_str());

        let resp_builder = match request_body {
            Some(request_body) => self.client.post(url).body(request_body),
            None => self.client.get(url),
        };

        let resp_builder = resp_builder
            // .timeout(self.send_timeout) // TODO: timeout
            .header(CONTENT_TYPE, "application/json")
            .header(CONTENT_ENCODING, "gzip")
            .bearer_auth(self.api_key.clone());

        let response = resp_builder.send().await?;

        let status = response.status();
        if !(status >= http::status::StatusCode::OK && status <= http::status::StatusCode::ACCEPTED){
            log::error!(
                "[sindir client], {method}, status not ok: {}",
                status
            );
            anyhow::bail!(
                "[sindir client], {method}, status not ok: {}",
                status
            )
        }

        let response_body = response.text().await?;

        log::info!("[sindir client], {method}, response: {response_body}");
        serde_json::from_str(&response_body).map_err(|e| anyhow::anyhow!(e))
    }
}

fn main() -> anyhow::Result<()> {
    init_tracing();

    let args = Args::parse();
    let cfg: Config = Config::from_file(args.config_file)?;
    let cloud_prover = CloudProver::new(cfg.prover.cloud.clone().unwrap());
    let prover = ProverBuilder::new(cfg)
        .with_proving_service(Box::new(cloud_prover))
        .build()?;

    Arc::new(prover).run()?;

    loop {}
}

use async_trait::async_trait;
use clap::Parser;
use core::time::Duration;
use reqwest::{
    header::{CONTENT_ENCODING, CONTENT_TYPE},
    Url,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    send_timeout: Duration,
    client: ClientWithMiddleware,
}

#[derive(Deserialize)]
struct VerificationKey {
    verification_key: String,
}

#[derive(Deserialize)]
struct SindriTaskStatusResponse {
    pub proof_id: String,
    pub project_name: String,
    pub perform_verify: bool,
    pub status: SindriTaskStatus,
    pub compute_time_sec: Option<f64>,
    pub queue_time_sec: Option<f64>,
    pub verification_key: Option<VerificationKey>,
    pub proof: Option<serde_json::Value>,
    pub public: Option<serde_json::Value>,
    pub warnings: Option<Vec<String>>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
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

// reencode the vk because the encoding scheme used in sindri is different from the one used in scroll internally
fn reformat_vk(vk_old: String) -> anyhow::Result<String> {
    log::debug!("vk_old: {:?}", vk_old);

    // decode base64 without padding
    let vk = base64::decode_config(vk_old, base64::URL_SAFE_NO_PAD)?;
    // encode with padding
    let vk_new = base64::encode_config(vk, base64::STANDARD);

    log::debug!("vk_new: {:?}", vk_new);

    Ok(vk_new)
}

#[async_trait]
impl ProvingService for CloudProver {
    fn is_local(&self) -> bool {
        false
    }

    async fn get_vk(&self, req: GetVkRequest) -> GetVkResponse {
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
            .get_with_token::<SindriGetDetailResponse>(
                MethodClass::Circuit(req.circuit_type),
                "detail",
                None,
            )
            .await
        {
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

    async fn prove(&self, req: ProveRequest) -> ProveResponse {
        if req.circuit_version != THIS_CIRCUIT_VERSION {
            return build_prove_error_response(&req, "circuit version mismatch");
        };

        let input = match reprocess_prove_input(&req) {
            Ok(input) => input,
            Err(e) => return build_prove_error_response(&req, &e.to_string()),
        };

        #[derive(serde::Deserialize, serde::Serialize)]
        struct SindriProveRequest {
            proof_input: String,
            perform_verify: bool,
        }

        let sindri_req = SindriProveRequest {
            proof_input: input,
            perform_verify: true,
        };

        match self
            .post_with_token::<SindriProveRequest, SindriTaskStatusResponse>(
                MethodClass::Circuit(req.circuit_type),
                "prove",
                &sindri_req,
            )
            .await
        {
            Ok(resp) => ProveResponse {
                task_id: resp.proof_id,
                circuit_type: req.circuit_type,
                circuit_version: req.circuit_version,
                hard_fork_name: req.hard_fork_name,
                status: resp.status.into(),
                created_at: 0.0,   // TODO:
                started_at: None,  // TODO:
                finished_at: None, // TODO:
                compute_time_sec: resp.compute_time_sec,
                input: Some(req.input.clone()),
                proof: serde_json::to_string(&resp.proof).ok(),
                vk: resp.verification_key.map(|vk| vk.verification_key),
                error: resp.error,
            },
            Err(e) => {
                return build_prove_error_response(&req, &format!("Failed to request proof: {}", e))
            }
        }
    }

    async fn query_task(&self, req: QueryTaskRequest) -> QueryTaskResponse {
        let query_params: HashMap<String, String> = [
            ("include_proof", "true"),
            ("include_public", "true"),
            ("include_verification_key", "true"),
        ]
        .iter()
        .map(|&(k, v)| (k.to_string(), v.to_string()))
        .collect();

        match self
            .get_with_token::<SindriTaskStatusResponse>(
                MethodClass::Proof(req.task_id.clone()),
                "detail",
                Some(query_params),
            )
            .await
        {
            Ok(resp) => QueryTaskResponse {
                task_id: resp.proof_id,
                circuit_type: CircuitType::Undefined, // TODO:
                circuit_version: "".to_string(),
                hard_fork_name: "".to_string(),
                status: resp.status.into(),
                created_at: 0.0,   // TODO:
                started_at: None,  // TODO:
                finished_at: None, // TODO:
                compute_time_sec: resp.compute_time_sec,
                input: None,
                proof: serde_json::to_string(&resp.proof).ok(),
                vk: resp.verification_key.map(|vk| vk.verification_key),
                error: resp.error,
            },
            Err(e) => {
                log::error!("Failed to query proof: {:?}", e);
                QueryTaskResponse {
                    task_id: req.task_id,
                    circuit_type: CircuitType::Undefined,
                    circuit_version: "".to_string(),
                    hard_fork_name: "".to_string(),
                    status: TaskStatus::Queued,
                    created_at: 0.0,
                    started_at: None,
                    finished_at: None,
                    compute_time_sec: None,
                    input: None,
                    proof: None,
                    vk: None,
                    error: Some(format!("Failed to query proof: {}", e)),
                }
            }
        }
    }
}

fn build_prove_error_response(req: &ProveRequest, error_msg: &str) -> ProveResponse {
    ProveResponse {
        task_id: String::new(),
        circuit_type: req.circuit_type,
        circuit_version: req.circuit_version.clone(),
        hard_fork_name: req.hard_fork_name.clone(),
        status: TaskStatus::Failed,
        created_at: 0.0,
        started_at: None,
        finished_at: None,
        compute_time_sec: None,
        input: Some(req.input.clone()),
        proof: None,
        vk: None,
        error: Some(error_msg.to_string()),
    }
}

// get rid of the "batch_proofs" layer because sindri expects the inner array as the input directly
fn reprocess_prove_input(req: &ProveRequest) -> anyhow::Result<String> {
    if req.circuit_type == CircuitType::Bundle {
        let bundle_task_detail: prover_darwin_v2::BundleProvingTask =
            serde_json::from_str(&req.input)?;
        Ok(serde_json::to_string(&bundle_task_detail.batch_proofs)?)
    } else {
        Ok(req.input.clone())
    }
}

// alternatively, we can just read it from the config
const THIS_CIRCUIT_VERSION: &str = "v0.13.1";

impl CloudProver {
    pub fn new(cfg: CloudProverConfig) -> Self {
        let retry_wait_duration = Duration::from_secs(cfg.retry_wait_time_sec);
        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(retry_wait_duration / 2, retry_wait_duration)
            .build_with_max_retries(cfg.retry_count);
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        let base_url = Url::parse(&cfg.base_url).expect("cannot parse cloud prover base_url");

        Self {
            base_url,
            api_key: cfg.api_key,
            send_timeout: Duration::from_secs(cfg.connection_timeout_sec),
            client,
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
            url.query_pairs_mut().extend_pairs(params);
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
            Some(body) => self.client.post(url).body(body),
            None => self.client.get(url),
        };

        let resp_builder = resp_builder
            .timeout(self.send_timeout)
            .header(CONTENT_TYPE, "application/json")
            .header(CONTENT_ENCODING, "gzip")
            .bearer_auth(&self.api_key);

        let response = resp_builder.send().await?;

        let status = response.status();
        if !(status >= http::status::StatusCode::OK && status <= http::status::StatusCode::ACCEPTED)
        {
            // log::error!("[sindir client], {method}, status not ok: {}", status);
            anyhow::bail!("[sindir client], {method}, status not ok: {}", status)
        }

        let response_body = response.text().await?;

        log::info!("[sindir client], {method}, received response");
        log::debug!("[sindir client], {method}, response: {response_body}");
        serde_json::from_str(&response_body).map_err(|e| anyhow::anyhow!(e))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let args = Args::parse();
    let cfg: Config = Config::from_file(args.config_file)?;
    let cloud_prover = CloudProver::new(
        cfg.prover
            .cloud
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Missing cloud prover configuration"))?,
    );
    let prover = ProverBuilder::new(cfg)
        .with_proving_service(Box::new(cloud_prover))
        .build()
        .await?;

    prover.run().await;

    Ok(())
}

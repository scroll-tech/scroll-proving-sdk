use async_trait::async_trait;
use clap::Parser;
use reqwest::Url;

use scroll_proving_sdk::{
    config::{CloudProverConfig, Config},
    prover::{
        proving_service::{
            GetVkRequest, GetVkResponse, ProveRequest, ProveResponse, QueryTaskRequest,
            QueryTaskResponse,
        },
        ProverBuilder, ProvingService,
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
}

#[async_trait]
impl ProvingService for CloudProver {
    fn is_local(&self) -> bool {
        false
    }
    async fn get_vk(&self, req: GetVkRequest) -> GetVkResponse {
        todo!()
    }
    async fn prove(&self, req: ProveRequest) -> ProveResponse {
        todo!()
    }
    async fn query_task(&self, req: QueryTaskRequest) -> QueryTaskResponse {
        todo!()
    }
}

impl CloudProver {
    pub fn new(cfg: CloudProverConfig) -> Self {
        let base_url = Url::parse(&cfg.base_url).expect("cannot parse cloud prover base_url");
        Self {
            base_url,
            api_key: cfg.api_key,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let args = Args::parse();
    let cfg: Config = Config::from_file_and_env(args.config_file)?;
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

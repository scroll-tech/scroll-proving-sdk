use async_trait::async_trait;
use clap::Parser;

use scroll_proving_sdk::{
    config::{Config, LocalProverConfig},
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

struct LocalProver {}

#[async_trait]
impl ProvingService for LocalProver {
    fn is_local(&self) -> bool {
        true
    }
    async fn get_vks(&self, req: GetVkRequest) -> GetVkResponse {
        todo!()
    }
    async fn prove(&self, req: ProveRequest) -> ProveResponse {
        todo!()
    }
    async fn query_task(&self, req: QueryTaskRequest) -> QueryTaskResponse {
        todo!()
    }
}

impl LocalProver {
    pub fn new(cfg: LocalProverConfig) -> Self {
        Self {}
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let args = Args::parse();
    let cfg: Config = Config::from_file_and_env(args.config_file)?;
    let local_prover = LocalProver::new(cfg.prover.local.clone().unwrap());
    let prover = ProverBuilder::new(cfg)
        .with_proving_service(Box::new(local_prover))
        .build()
        .await?;

    prover.run().await;

    Ok(())
}

use clap::Parser;
use std::sync::Arc;

use scroll_proving_sdk::{
    config::{CloudProverConfig, Config},
    prover::{
        proving_service::{ProveRequest, ProveResponse, QueryTaskRequest, QueryTaskResponse},
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
    endpoint: String,
    api_key: String,
}

impl ProvingService for CloudProver {
    fn is_local(&self) -> bool {
        false
    }
    fn get_vks(&self, req: GetVksRequest) -> Vec<String> {
        todo!()
    }
    fn prove(&self, req: ProveRequest) -> ProveResponse {
        todo!()
    }
    fn query_task(&self, req: QueryTaskRequest) -> QueryTaskResponse {
        todo!()
    }
}

impl CloudProver {
    pub fn new(cfg: CloudProverConfig) -> Self {
        Self {
            endpoint: cfg.endpoint,
            api_key: cfg.api_key,
        }
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

    Arc::new(prover).run()
}

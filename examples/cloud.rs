use std::sync::Arc;

use scroll_proving_sdk::{
    config::{CloudProverConfig, Config},
    prover::{
        proving_service::{GetTaskRequest, GetTaskResponse, ProveRequest, ProveResponse},
        ProverBuilder, ProvingService,
    },
};

struct CloudProver {
    endpoint: String,
    api_key: String,
}

impl ProvingService for CloudProver {
    fn prove(&self, req: ProveRequest) -> ProveResponse {
        todo!()
    }
    fn query_task(&self, req: GetTaskRequest) -> GetTaskResponse {
        todo!()
    }
    fn is_local(&self) -> bool {
        false
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
    // TODO: specify the path to the config file
    let cfg: Config = Config::from_file("config.json".to_owned())?;

    let cloud_prover = CloudProver::new(cfg.prover.cloud.clone().unwrap());

    let prover = ProverBuilder::new(cfg)
        .with_proving_service(Box::new(cloud_prover))
        .build()?;

    Arc::new(prover).run()
}

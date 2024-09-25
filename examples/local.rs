use std::sync::Arc;

use scroll_proving_sdk::{
    utils::init_tracing,
    config::{Config, LocalProverConfig},
    prover::{
        proving_service::{ProveRequest, ProveResponse, QueryTaskRequest, QueryTaskResponse},
        ProverBuilder, ProvingService,
    },
};

struct LocalProver {}

impl ProvingService for LocalProver {
    fn prove(&self, req: ProveRequest) -> ProveResponse {
        todo!()
    }
    fn query_task(&self, req: QueryTaskRequest) -> QueryTaskResponse {
        todo!()
    }
    fn is_local(&self) -> bool {
        true
    }
}

impl LocalProver {
    pub fn new(cfg: LocalProverConfig) -> Self {
        Self {}
    }
}

fn main() -> anyhow::Result<()> {
    init_tracing();

    // TODO: specify the path to the config file
    let cfg: Config = Config::from_file("config.json".to_owned())?;

    let local_prover = LocalProver::new(cfg.prover.local.clone().unwrap());

    let prover = ProverBuilder::new(cfg)
        .with_proving_service(Box::new(local_prover))
        .build()?;

    Arc::new(prover).run()
}

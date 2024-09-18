use scroll_proving_sdk::{
    config::{Config, LocalProverConfig},
    prover::{
        proving_service::{GetTaskRequest, GetTaskResponse, ProveRequest, ProveResponse},
        ProverBuilder, ProvingService,
    },
};

struct LocalProver {}

impl ProvingService for LocalProver {
    fn prove(&self, req: ProveRequest) -> ProveResponse {
        todo!()
    }
    fn get_task(&self, req: GetTaskRequest) -> GetTaskResponse {
        todo!()
    }
}

impl LocalProver {
    pub fn new(cfg: LocalProverConfig) -> Self {
        Self {}
    }
}

fn main() -> anyhow::Result<()> {
    // TODO: specify the path to the config file
    let cfg: Config = Config::from_file("config.json".to_owned())?;

    let local_prover = LocalProver::new(cfg.prover.local.clone().unwrap());

    let prover = ProverBuilder::new(cfg)
        .with_proving_service(Box::new(local_prover))
        .build()?;

    prover.run()
}

use crate::{config::Config, prover::Prover, prover::ProvingService};

struct ProverBuilder {
    cfg: Config,
    proving_service: Option<Box<dyn ProvingService>>,
}

impl ProverBuilder {
    pub fn new(cfg: Config) -> Self {
        ProverBuilder {
            cfg,
            proving_service: None,
        }
    }

    pub fn with_proving_service(mut self, proving_service: Box<dyn ProvingService>) -> Self {
        self.proving_service = Some(proving_service);
        self
    }

    pub fn build(&self) -> Prover {
        todo!()
    }
}

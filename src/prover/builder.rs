use crate::{
    config::Config,
    coordinator_handler::CoordinatorClient,
    prover::{Prover, ProvingService}, tracing_handler::L2gethClient,
};

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

    pub fn build(self) -> Prover {
        let coordinator_client = CoordinatorClient::new(self.cfg.coordinator.clone());
        let l2geth_client = match self.cfg.l2geth {
            Some(l2geth) => Some(L2gethClient::new(l2geth)),
            None => None,
        };
        let proving_service = self.proving_service.unwrap();

        Prover {
            coordinator_client: coordinator_client,
            l2geth_client: l2geth_client,
            proving_service: proving_service,
        }

        // todo!()
    }
}

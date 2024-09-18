use crate::{
    config::Config,
    coordinator_handler::CoordinatorClient,
    prover::{Prover, ProvingService},
    tracing_handler::L2gethClient,
};

use super::{CircuitType, Worker};

pub struct ProverBuilder {
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

    pub fn build(self) -> anyhow::Result<Prover> {
        if self.proving_service.is_none() {
            anyhow::bail!("proving_service is not provided");
        }
        if self.proving_service.as_ref().unwrap().is_local() && self.cfg.prover.n_workers > 1 {
            anyhow::bail!("cannot use multiple workers with local proving service");
        }

        if self.cfg.prover.circuit_type == CircuitType::Chunk && self.cfg.l2geth.is_none() {
            anyhow::bail!("circuit_type is chunk but l2geth config is not provided");
        }

        let coordinator_client = CoordinatorClient::new(self.cfg.coordinator.clone());
        let l2geth_client = match self.cfg.l2geth {
            Some(l2geth) => Some(L2gethClient::new(l2geth)),
            None => None,
        };
        let proving_service = self.proving_service.unwrap();

        let workers = (0..self.cfg.prover.n_workers)
            .map(|id| Worker::new(id))
            .collect::<Vec<_>>();

        Ok(Prover {
            coordinator_client,
            l2geth_client,
            proving_service,
            workers,
        })
    }
}

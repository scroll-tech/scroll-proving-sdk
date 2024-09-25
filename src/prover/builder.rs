use super::CircuitType;
use crate::{
    config::Config,
    coordinator_handler::{CoordinatorClient, KeySigner},
    prover::{Prover, ProvingService},
    tracing_handler::L2gethClient,
};
use std::path::PathBuf;

pub struct ProverBuilder {
    cfg: Config,
    proving_service: Option<Box<dyn ProvingService + Send + Sync>>,
}

impl ProverBuilder {
    pub fn new(cfg: Config) -> Self {
        ProverBuilder {
            cfg,
            proving_service: None,
        }
    }

    pub fn with_proving_service(
        mut self,
        proving_service: Box<dyn ProvingService + Send + Sync>,
    ) -> Self {
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

        let key_signers: Result<Vec<_>, _> = (0..self.cfg.prover.n_workers)
            .map(|i| {
                let key_path = PathBuf::from(&self.cfg.keys_dir).join(i.to_string());
                KeySigner::new(&key_path)
            })
            .collect();
        let key_signers = key_signers?;

        let coordinator_clients: Result<Vec<_>, _> = (0..self.cfg.prover.n_workers)
            .map(|i| {
                CoordinatorClient::new(
                    self.cfg.coordinator.clone(),
                    self.cfg.prover.circuit_type,
                    format!("{}{}", self.cfg.prover_name_prefix, i),
                    key_signers[i].clone(),
                )
            })
            .collect();
        let coordinator_clients = coordinator_clients?;

        let l2geth_client = match self.cfg.l2geth {
            Some(l2geth) => Some(L2gethClient::new(l2geth)?),
            None => None,
        };

        Ok(Prover {
            circuit_type: self.cfg.prover.circuit_type,
            coordinator_clients,
            l2geth_client,
            proving_service: self.proving_service.unwrap(),
            n_workers: self.cfg.prover.n_workers,
        })
    }
}

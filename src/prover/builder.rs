use super::CircuitType;
use crate::{
    config::Config,
    db::Db,
    coordinator_handler::{CoordinatorClient, KeySigner},
    prover::{
        proving_service::{GetVkRequest, ProvingService},
        Prover,
    },
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

    pub async fn build(self) -> anyhow::Result<Prover> {
        if self.proving_service.is_none() {
            anyhow::bail!("proving_service is not provided");
        }
        if self.proving_service.as_ref().unwrap().is_local() && self.cfg.prover.n_workers > 1 {
            anyhow::bail!("cannot use multiple workers with local proving service");
        }

        if self.cfg.prover.circuit_type == CircuitType::Chunk && self.cfg.l2geth.is_none() {
            anyhow::bail!("circuit_type is chunk but l2geth config is not provided");
        }

        let get_vk_request = GetVkRequest {
            circuit_type: self.cfg.prover.circuit_type,
            circuit_version: self.cfg.prover.circuit_version.clone(),
        };
        let get_vk_response = self
            .proving_service
            .as_ref()
            .unwrap()
            .get_vk(get_vk_request)
            .await;
        if let Some(error) = get_vk_response.error {
            anyhow::bail!("failed to get vk: {}", error);
        }

        let key_signers: Result<Vec<_>, _> = (0..self.cfg.prover.n_workers)
            .map(|i| {
                let key_path = PathBuf::from(&self.cfg.keys_dir).join(i.to_string());
                KeySigner::new(&key_path)
            })
            .collect();
        let key_signers =
            key_signers.map_err(|e| anyhow::anyhow!("cannot create key_signer, err: {e}"))?;

        let coordinator_clients: Result<Vec<_>, _> = (0..self.cfg.prover.n_workers)
            .map(|i| {
                CoordinatorClient::new(
                    self.cfg.coordinator.clone(),
                    self.cfg.prover.circuit_type,
                    vec![get_vk_response.vk.clone()],
                    self.cfg.prover.circuit_version.clone(),
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

        let db = Db::new(&self.cfg.db_path)?;

        Ok(Prover {
            circuit_type: self.cfg.prover.circuit_type,
            circuit_version: self.cfg.prover.circuit_version,
            coordinator_clients,
            l2geth_client,
            proving_service: self.proving_service.unwrap(),
            n_workers: self.cfg.prover.n_workers,
            health_listener_addr: self.cfg.health_listener_addr,
            db: db,
        })
    }
}

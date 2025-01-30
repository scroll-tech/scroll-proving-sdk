use tokio::sync::RwLock;

use super::{ProofType, ProverProviderType};
use crate::{
    config::Config,
    coordinator_handler::{CoordinatorClient, KeySigner},
    db::Db,
    prover::{
        proving_service::{GetVkRequest, ProvingService},
        Prover,
    },
    tracing_handler::L2gethClient,
    utils::format_cloud_prover_name,
};
use std::path::PathBuf;

pub struct ProverBuilder<Backend: ProvingService + Send + Sync + 'static> {
    cfg: Config,
    proving_service: Backend,
}

impl<Backend> ProverBuilder<Backend>
where
    Backend: ProvingService + Send + Sync + 'static,
{
    pub fn new(cfg: Config, service: Backend) -> Self {
        ProverBuilder {
            cfg,
            proving_service: service,
        }
    }

    pub async fn build(self) -> anyhow::Result<Prover<Backend>> {
        if self.proving_service.is_local() && self.cfg.prover.n_workers > 1 {
            anyhow::bail!("cannot use multiple workers with local proving service");
        }

        if self
            .cfg
            .prover
            .supported_proof_types
            .contains(&ProofType::Chunk)
            && self.cfg.l2geth.is_none()
        {
            anyhow::bail!("circuit_type is chunk but l2geth config is not provided");
        }

        let get_vk_request = GetVkRequest {
            proof_types: self.cfg.prover.supported_proof_types.clone(),
            circuit_version: self.cfg.prover.circuit_version.clone(),
        };
        let get_vk_response = self.proving_service.get_vks(get_vk_request).await;
        if let Some(error) = get_vk_response.error {
            anyhow::bail!("failed to get vk: {}", error);
        }

        let prover_provider_type = if self.proving_service.is_local() {
            ProverProviderType::Internal
        } else {
            ProverProviderType::External
        };

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
                let prover_name = if self.proving_service.is_local() {
                    self.cfg.prover_name_prefix.clone()
                } else {
                    format_cloud_prover_name(self.cfg.prover_name_prefix.clone(), i)
                };

                CoordinatorClient::new(
                    self.cfg.coordinator.clone(),
                    self.cfg.coordinator_prover_type(),
                    get_vk_response.vks.clone(),
                    prover_name,
                    prover_provider_type,
                    key_signers[i].clone(),
                )
            })
            .collect();
        let coordinator_clients = coordinator_clients?;

        let l2geth_client = match self.cfg.l2geth {
            Some(l2geth) => Some(L2gethClient::new(l2geth)?),
            None => None,
        };

        let db_path = self.cfg.db_path.unwrap_or_else(|| {
            panic!("Missing database path");
        });

        Ok(Prover {
            circuit_type: self.cfg.prover.circuit_type,
            proof_types: self.cfg.prover.supported_proof_types,
            circuit_version: self.cfg.prover.circuit_version,
            coordinator_clients,
            l2geth_client,
            proving_service: RwLock::new(self.proving_service),
            n_workers: self.cfg.prover.n_workers,
            health_listener_addr: self.cfg.health_listener_addr,
            db: Db::new(&db_path)?,
        })
    }
}

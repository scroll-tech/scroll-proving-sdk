use tokio::sync::RwLock;

use super::ProverProviderType;
use crate::{
    config::Config,
    coordinator_handler::{CoordinatorClient, KeySigner},
    db::Db,
    prover::{
        Prover,
        proving_service::{GetVkRequest, ProvingService},
    },
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

    pub async fn build(self) -> eyre::Result<Prover<Backend>> {
        if self.proving_service.is_local() && self.cfg.prover.n_workers > 1 {
            eyre::bail!("cannot use multiple workers with local proving service");
        }

        let get_vk_request = GetVkRequest {
            proof_types: self.cfg.prover.supported_proof_types.clone(),
            circuit_version: self.cfg.prover.circuit_version.clone(),
        };
        let get_vk_response = self.proving_service.get_vks(get_vk_request).await;
        if let Some(error) = get_vk_response.error {
            eyre::bail!("failed to get vk: {}", error);
        }

        // FIXME: should derive from `self.proving_service.is_local()`, but coordinator has a bug when handling external provers
        let prover_provider_type = ProverProviderType::Internal;

        let key_signers: Result<Vec<_>, _> = (0..self.cfg.prover.n_workers)
            .map(|i| {
                let keys_dir = PathBuf::from(&self.cfg.keys_dir);
                if !keys_dir.exists() {
                    std::fs::create_dir_all(&keys_dir).map_err(|e| {
                        eyre::eyre!(
                            "failed to create keys directory {}: {e}",
                            keys_dir.display()
                        )
                    })?;
                }
                let key_path = keys_dir.join(i.to_string());
                KeySigner::new(&key_path)
            })
            .collect();
        let key_signers =
            key_signers.map_err(|e| eyre::eyre!("cannot create key_signer, err: {e}"))?;

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
                    self.cfg.coordinator.suppress_empty_task_error,
                    get_vk_response.vks.clone(),
                    prover_name,
                    prover_provider_type,
                    key_signers[i].clone(),
                )
            })
            .collect();
        let coordinator_clients = coordinator_clients?;

        Ok(Prover {
            proof_types: self.cfg.prover.supported_proof_types,
            circuit_version: self.cfg.prover.circuit_version,
            coordinator_clients,
            proving_service: RwLock::new(self.proving_service),
            n_workers: self.cfg.prover.n_workers,
            health_listener_addr: self.cfg.health_listener_addr,
            db: self
                .cfg
                .db_path
                .as_ref()
                .map(|path| Db::new(path.as_str()))
                .transpose()?,
            poll_interval_sec: self.cfg.prover.poll_interval_sec,
            randomized_delay_sec: self.cfg.prover.randomized_delay_sec,
            suppress_empty_task_error: self.cfg.prover.suppress_empty_task_error,
        })
    }
}

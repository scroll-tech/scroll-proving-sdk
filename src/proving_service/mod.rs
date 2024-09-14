mod local;
mod sindri;
mod snarkify;
mod types;

use std::collections::HashSet;

use crate::config::{DbConfig, L2GethConfig, ProvingServiceConfig};
pub use types::*;

pub trait ProvingServiceExtension {
    fn prove(&self, req: ProveRequest) -> ProveResponse;
    fn get_task(&self, req: GetTaskRequest) -> GetTaskResponse;
    fn total_workers(&self) -> usize;
    fn idle_workers(&self) -> usize;
}

pub struct ProvingService {
    extensions: Vec<Box<dyn ProvingServiceExtension>>,
    total_workers: usize, // sum of total workers of all extensions
    supported_circuit_types: HashSet<CircuitType>,
}

impl ProvingService {
    pub fn new(
        cfg: ProvingServiceConfig,
        db_cfg: DbConfig,
        l2geth_cfg: Option<L2GethConfig>,
    ) -> Self {
        // config sanity check
        if cfg.local.is_none() && cfg.sindri.is_none() && cfg.snarkify.is_none() {
            panic!("No proving service is configured");
        }
        if cfg.local.is_some() {
            if cfg.local.clone().unwrap().circuit_type == CircuitType::Chunk && l2geth_cfg.is_none()
            {
                panic!("Local chunk proving service is specified but no l2geth is provided");
            }
        }

        let mut proving_service = ProvingService {
            extensions: Vec::new(),
            total_workers: 0,
            supported_circuit_types: HashSet::new(),
        };

        if cfg.local.is_some() {
            let local_cfg = cfg.local.unwrap();
            let local_proving_service = local::LocalProvingService::new(local_cfg.clone(), db_cfg);
            proving_service
                .extensions
                .push(Box::new(local_proving_service));
            proving_service.total_workers += 1;
            proving_service
                .supported_circuit_types
                .insert(local_cfg.circuit_type);
        }

        if cfg.sindri.is_some() {
            let sindri_cfg = cfg.sindri.unwrap();
            let sindri_proving_service = sindri::SindriProvingService::new(sindri_cfg.clone());
            proving_service
                .extensions
                .push(Box::new(sindri_proving_service));
            proving_service.total_workers += sindri_cfg.n_workers;
            proving_service
                .supported_circuit_types
                .insert(CircuitType::Batch);
            proving_service
                .supported_circuit_types
                .insert(CircuitType::Bundle);
            if l2geth_cfg.is_some() {
                proving_service
                    .supported_circuit_types
                    .insert(CircuitType::Chunk);
            }
        }

        if cfg.snarkify.is_some() {
            let snarkify_cfg = cfg.snarkify.unwrap();
            let snarkify_proving_service =
                snarkify::SnarkifyProvingService::new(snarkify_cfg.clone());
            proving_service
                .extensions
                .push(Box::new(snarkify_proving_service));
            proving_service.total_workers += snarkify_cfg.n_workers;
            proving_service
                .supported_circuit_types
                .insert(CircuitType::Batch);
            proving_service
                .supported_circuit_types
                .insert(CircuitType::Bundle);
            if l2geth_cfg.is_some() {
                proving_service
                    .supported_circuit_types
                    .insert(CircuitType::Chunk);
            }
        }

        proving_service
    }

    fn start_prove(&mut self) -> anyhow::Result<()> {
        // sort extensions by idle workers
        let mut extensions: Vec<&mut Box<dyn ProvingServiceExtension>> =
            self.extensions.iter_mut().collect();
        extensions.sort_by_key(|e| e.idle_workers());
        extensions.reverse();
        if extensions.first().unwrap().idle_workers() <= 0 {
            anyhow::bail!("No idle workers");
        }

        // TODO:
        // e.prove(ProveRequest {});
        Ok(())
    }
}

mod local;
mod sindri;
mod snarkify;
mod types;

use crate::config::ProvingServiceConfig;
pub use types::*;

pub trait ProvingServiceExtension {
    fn prove(&self, req: ProveRequest) -> ProveResponse;
    fn get_task(&self, req: GetTaskRequest) -> GetTaskResponse;
    fn idle_workers(&self) -> usize;
}

struct ProvingService {
    extensions: Vec<Box<dyn ProvingServiceExtension>>,
}

impl ProvingService {
    fn new(cfg: ProvingServiceConfig) -> Self {
        if cfg.local.is_none() && cfg.sindri.is_none() && cfg.snarkify.is_none() {
            panic!("No proving service is configured");
        }

        let mut proving_service = ProvingService {
            extensions: Vec::new(),
        };

        if cfg.local.is_some() {
            let local_proving_service = local::LocalProvingService::new(cfg.local.unwrap());
            proving_service
                .extensions
                .push(Box::new(local_proving_service));
        }

        if cfg.sindri.is_some() {
            let sindri_proving_service = sindri::SindriProvingService::new(cfg.sindri.unwrap());
            proving_service
                .extensions
                .push(Box::new(sindri_proving_service));
        }

        if cfg.snarkify.is_some() {
            let snarkify_proving_service =
                snarkify::SnarkifyProvingService::new(cfg.snarkify.unwrap());
            proving_service
                .extensions
                .push(Box::new(snarkify_proving_service));
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
            return Err(anyhow::Error::msg("No idle workers"));
        }

        // e.prove(ProveRequest {});
        Ok(())
    }
}

use super::types::*;
use super::ProvingServiceExtension;
use crate::config::SnarkifyConfig;

pub struct SnarkifyProvingService {
    endpoint: String,
    api_key: String,
    n_workers: usize,
}

impl SnarkifyProvingService {
    pub fn new(cfg: SnarkifyConfig) -> Self {
        Self {
            endpoint: cfg.endpoint,
            api_key: cfg.api_key,
            n_workers: cfg.n_workers,
        }
    }
}

impl ProvingServiceExtension for SnarkifyProvingService {
    fn prove(&self, req: ProveRequest) -> ProveResponse {
        todo!()
    }

    fn get_task(&self, req: GetTaskRequest) -> GetTaskResponse {
        todo!()
    }

    fn total_workers(&self) -> usize {
        self.n_workers
    }

    fn idle_workers(&self) -> usize {
        todo!()
    }
}

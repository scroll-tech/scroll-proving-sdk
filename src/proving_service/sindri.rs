use super::types::*;
use super::ProvingServiceExtension;
use crate::config::SindriConfig;

pub struct SindriProvingService {
    endpoint: String,
    api_key: String,
}

impl SindriProvingService {
    pub fn new(cfg: SindriConfig) -> Self {
        Self {
            endpoint: cfg.endpoint,
            api_key: cfg.api_key,
        }
    }
}

impl ProvingServiceExtension for SindriProvingService {
    fn prove(&self, req: ProveRequest) -> ProveResponse {
        todo!()
    }

    fn get_task(&self, req: GetTaskRequest) -> GetTaskResponse {
        todo!()
    }

    fn idle_workers(&self) -> usize {
        todo!()
    }
}

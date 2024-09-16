use super::types::*;
use super::ProvingServiceExtension;
use crate::config::SnarkifyConfig;

pub struct SnarkifyProvingService {
    endpoint: String,
    api_key: String,
}

impl SnarkifyProvingService {
    pub fn new(cfg: SnarkifyConfig) -> Self {
        Self {
            endpoint: cfg.endpoint,
            api_key: cfg.api_key,
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
}

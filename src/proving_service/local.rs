use super::types::*;
use super::ProvingServiceExtension;
use crate::config::DbConfig;
use crate::config::LocalPoverConfig;

pub struct LocalProvingService {}

impl LocalProvingService {
    pub fn new(cfg: LocalPoverConfig, db_cfg: DbConfig) -> Self {
        Self {}
    }
}

impl ProvingServiceExtension for LocalProvingService {
    fn prove(&self, req: ProveRequest) -> ProveResponse {
        todo!()
    }

    fn get_task(&self, req: GetTaskRequest) -> GetTaskResponse {
        todo!()
    }

    fn total_workers(&self) -> usize {
        1
    }

    fn idle_workers(&self) -> usize {
        todo!()
    }
}

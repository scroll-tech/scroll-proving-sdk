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
        // TODO:
        // save the related info to db
        // spin a new thread to do the proving; in the thread, when the proving is done, update the related into db

        todo!()
    }

    fn get_task(&self, req: GetTaskRequest) -> GetTaskResponse {
        // read the related info from db
        todo!()
    }
}

use super::CoordinatorTask;
use crate::config::CoordinatorConfig;

pub struct CoordinatorClient {
    pub base_url: String,
}

impl CoordinatorClient {
    pub fn new(cfg: CoordinatorConfig) -> Self {
        Self {
            base_url: cfg.base_url,
        }
    }

    pub fn get_task(&self) -> Option<CoordinatorTask> {
        todo!()
    }
}

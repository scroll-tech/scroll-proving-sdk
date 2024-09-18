use std::result;

use crate::{
    config::Config, coordinator_handler::CoordinatorClient,
    tracing_handler::L2gethClient,
};

struct Manager {
    coordinator_client: CoordinatorClient,
    l2geth_client: Option<L2gethClient>,
    // TODO:
    // keys: HashMap<string, Key>,
    // db: Db,
}

impl Manager {
    pub fn new(cfg: Config) -> Manager {
        todo!()
    }

    pub fn run(&self) {
        todo!()
    }
}

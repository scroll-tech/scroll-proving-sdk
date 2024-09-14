use std::result;

use crate::{
    config::Config, coordinator_handler::CoordinatorClient, proving_service::ProvingService,
    tracing_handler::L2gethClient,
};

struct Manager {
    proving_service: ProvingService,
    coordinator_client: CoordinatorClient,
    l2geth_client: Option<L2gethClient>,
    // TODO:
    // keys: HashMap<string, Key>,
    // db: Db,
}

impl Manager {
    pub fn new(cfg: Config) -> Manager {
        let ps = ProvingService::new(cfg.proving_service, cfg.db, cfg.l2geth.clone());
        let cc = CoordinatorClient::new(cfg.coordinator);
        let l2geth = if cfg.l2geth.is_some() {
            Some(L2gethClient::new(cfg.l2geth.unwrap()))
        } else {
            None
        };

        // TODO:
        // generate keys for proving_service.total_workers
        // create db connection

        Manager {
            proving_service: ps,
            coordinator_client: cc,
            l2geth_client: l2geth,
        }
    }

    pub fn run(&self) {
        // TODO:

        // 1. spin a thread to deal with [chunk] task, if supported
        // if there's any idle workers, sign the request, get [chunk] task from coordinator
        // [get block_trace from l2geth]
        // send [chunk] task to proving_service
        // periodically check task result, retry if needed
        // sign the result, send task result back to coordinator

        // 2. spin a thread to deal with [batch] task, if supported
        // if there's any idle workers, sign the request, get [batch] task from coordinator
        // send [batch] task to proving_service
        // periodically check task result, retry if needed
        // sign the result, send task result back to coordinator

        // 3. spin a thread to deal with [bundle] task, if supported
        // if there's any idle workers, sign the request, get [bundle] task from coordinator
        // send [bundle] task to proving_service
        // periodically check task result, retry if needed
        // sign the result, send task result back to coordinator

        todo!()
    }
}

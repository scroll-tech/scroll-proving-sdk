use crate::{
    config::Config,
    coordinator_handler::CoordinatorClient,
    proving_service::{self, ProvingService},
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
}

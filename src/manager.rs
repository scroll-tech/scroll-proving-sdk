use crate::{
    config::Config, coordinator_handler::CoordinatorClient, proving_service::ProvingService,
};

struct Manager {
    proving_service: ProvingService,
    coordinator_client: CoordinatorClient,
    // TODO:
    // l2gethClient: Option<L2GethClient>,
    // keys: HashMap<string, Key>,
}

impl Manager {
    pub fn new(cfg: Config) -> Manager {
        let ps = ProvingService::new(cfg.proving_service, cfg.db);
        let cc = CoordinatorClient::new(cfg.coordinator);
        Manager {
            proving_service: ps,
            coordinator_client: cc,
        }
    }
}

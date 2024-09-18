pub mod builder;
pub mod proving_service;

pub use proving_service::ProvingService;

use crate::{coordinator_handler::CoordinatorClient, tracing_handler::L2gethClient};

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum CircuitType {
    Chunk,
    Batch,
    Bundle,
}

struct Prover {
    coordinator_client: CoordinatorClient,
    l2geth_client: Option<L2gethClient>,
    proving_service: Box<dyn ProvingService>,
    // TODO:
    // keys: HashMap<string, Key>,
    // db: Db,
}

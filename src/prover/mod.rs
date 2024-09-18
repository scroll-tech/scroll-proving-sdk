pub mod builder;
pub mod proving_service;

pub use {builder::ProverBuilder, proving_service::ProvingService};

use crate::{coordinator_handler::CoordinatorClient, tracing_handler::L2gethClient};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum CircuitType {
    Chunk,
    Batch,
    Bundle,
}

pub struct Prover {
    coordinator_client: CoordinatorClient,
    l2geth_client: Option<L2gethClient>,
    proving_service: Box<dyn ProvingService>,
    // TODO:
    // keys: HashMap<string, Key>,
    // db: Db,
}

impl Prover {
    pub fn run(&self) -> anyhow::Result<()> {
        todo!()
    }
}

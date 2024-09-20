pub mod builder;
pub mod proving_service;
pub use {builder::ProverBuilder, proving_service::ProvingService};

use crate::{
    coordinator_handler::{CoordinatorClient, CoordinatorTask, KeySigner},
    tracing_handler::L2gethClient,
};
use anyhow::Ok;
use proving_service::{GetTaskRequest, ProveRequest, TaskStatus};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::thread;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum CircuitType {
    Chunk,
    Batch,
    Bundle,
}

const WORKER_SLEEP_SEC: u64 = 20;

pub struct Prover {
    prover_name_prefix: String,
    circuit_type: CircuitType,
    coordinator_client: CoordinatorClient,
    l2geth_client: Option<L2gethClient>,
    proving_service: Box<dyn ProvingService + Send + Sync>,
    n_workers: usize,
    key_signers: Vec<KeySigner>,
    db: Db,
}

impl Prover {
    pub fn run(self: std::sync::Arc<Self>) -> anyhow::Result<()> {
        for i in 0..self.n_workers {
            let self_clone = std::sync::Arc::clone(&self);
            // spin up a thread
            thread::spawn(move || {
                self_clone.working_loop(i);
            });
        }

        Ok(())
    }

    fn working_loop(&self, i: usize) {
        let worker_name = format!("{}{}", self.prover_name_prefix, i);
        let key_signer = self.key_signers[i].clone();

        loop {
            let coordinator_task = self.coordinator_client.get_task();

            if coordinator_task.is_none() {
                thread::sleep(std::time::Duration::from_secs(WORKER_SLEEP_SEC));
                continue;
            }

            let proving_input = self.build_proving_input(coordinator_task.unwrap());
            let proving_task = self.proving_service.prove(proving_input);
            if proving_task.error.is_some() {
                // TODO: log error
                continue; // retry
            } else {
                loop {
                    let task = self.proving_service.query_task(GetTaskRequest {
                        task_id: proving_task.task_id.clone(),
                    });
                    match task.status {
                        TaskStatus::Success => {
                            // TODO: send back proof
                        }
                        TaskStatus::Failed => {
                            // TODO: log error
                            // TODO: send back error
                        }
                        _ => {
                            thread::sleep(std::time::Duration::from_secs(WORKER_SLEEP_SEC));
                        }
                    }
                }
            }
        }
    }

    fn build_proving_input(&self, task: CoordinatorTask) -> ProveRequest {
        match self.circuit_type {
            CircuitType::Chunk => {}
            CircuitType::Batch => {}
            CircuitType::Bundle => {}
        }

        todo!()
    }
}

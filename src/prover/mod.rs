pub mod builder;
pub mod proving_service;
pub mod types;
pub use {builder::ProverBuilder, proving_service::ProvingService, types::*};

use crate::{
    coordinator_handler::{CoordinatorClient, ErrorCode, GetTaskRequest, GetTaskResponseData},
    tracing_handler::L2gethClient,
};
use proving_service::{ProveRequest, QueryTaskRequest, TaskStatus};
use std::thread;

const WORKER_SLEEP_SEC: u64 = 20;

pub struct Prover {
    circuit_type: CircuitType,
    coordinator_clients: Vec<CoordinatorClient>,
    l2geth_client: Option<L2gethClient>,
    proving_service: Box<dyn ProvingService + Send + Sync>,
    n_workers: usize,
    // TODO:
    // db: Db,
}

impl Prover {
    pub fn run(self: std::sync::Arc<Self>) -> anyhow::Result<()> {
        assert!(self.n_workers == self.coordinator_clients.len());
        if self.circuit_type == CircuitType::Chunk {
            assert!(self.l2geth_client.is_some());
        }

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
        loop {
            let get_task_request = self.build_get_task_request();
            let coordinator_task = self.coordinator_clients[i].get_task(&get_task_request);

            if let Err(e) = coordinator_task {
                log::error!("failed to get task: {:?}", e);
                thread::sleep(std::time::Duration::from_secs(WORKER_SLEEP_SEC));
                continue;
            } else if coordinator_task.as_ref().unwrap().errcode != ErrorCode::Success {
                log::error!(
                    "failed to get task, errcode: {:?}, errmsg: {:?}",
                    coordinator_task.as_ref().unwrap().errcode,
                    coordinator_task.as_ref().unwrap().errmsg
                );
                thread::sleep(std::time::Duration::from_secs(WORKER_SLEEP_SEC));
                continue;
            } else if coordinator_task.as_ref().unwrap().data.is_none() {
                log::error!("no task is available");
                thread::sleep(std::time::Duration::from_secs(WORKER_SLEEP_SEC));
                continue;
            }

            let coordinator_task = coordinator_task.unwrap().data.unwrap();

            let proving_input = self.build_proving_input(coordinator_task);
            let proving_task = self.proving_service.prove(proving_input);
            if proving_task.error.is_some() {
                log::error!("failed to prove: {:?}", proving_task.error);
                continue;
            } else {
                loop {
                    let task = self.proving_service.query_task(QueryTaskRequest {
                        task_id: proving_task.task_id.clone(),
                    });
                    match task.status {
                        TaskStatus::Success => {
                            // TODO: send back proof
                        }
                        TaskStatus::Failed => {
                            log::error!("task failed: {:?}", task.error.unwrap());
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

    fn build_get_task_request(&self) -> GetTaskRequest {
        let prover_height = self.l2geth_client.as_ref().and_then(|l2geth_client| {
            l2geth_client
                .block_number_sync()
                .ok()
                .and_then(|block_number| block_number.as_number())
        });

        GetTaskRequest {
            task_types: vec![self.circuit_type],
            prover_height,
        }
    }

    fn build_proving_input(&self, task: GetTaskResponseData) -> ProveRequest {
        match self.circuit_type {
            CircuitType::Undefined => {
                unreachable!();
            }
            CircuitType::Chunk => {}
            CircuitType::Batch => {}
            CircuitType::Bundle => {}
        }

        todo!()
    }
}

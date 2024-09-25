pub mod builder;
pub mod proving_service;
pub mod types;
pub use {builder::ProverBuilder, proving_service::ProvingService, types::*};

use crate::{
    coordinator_handler::{
        CoordinatorClient, ErrorCode, GetTaskRequest, GetTaskResponseData, ProofFailureType,
        ProofStatus, SubmitProofRequest,
    },
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

    // TODO: print more prover_name
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
            let coordinator_task_uuid = coordinator_task.uuid.clone();
            let coordinator_task_id = coordinator_task.task_id.clone();

            let proving_input = match self.build_proving_input(&coordinator_task) {
                Ok(input) => input,
                Err(e) => {
                    log::error!("failed to build proving input: {:?}", e);
                    continue;
                }
            };

            let proving_task = self.proving_service.prove(proving_input);
            if proving_task.error.is_some() {
                log::error!("failed to prove: {:?}", proving_task.error);
                continue;
            } else {
                let proving_service_task_id = proving_task.task_id;
                loop {
                    let task = self.proving_service.query_task(QueryTaskRequest {
                        task_id: proving_service_task_id.clone(),
                    });
                    match task.status {
                        TaskStatus::Queued => {
                            log::info!("task queued");
                            thread::sleep(std::time::Duration::from_secs(WORKER_SLEEP_SEC));
                        }
                        TaskStatus::Proving => {
                            log::info!("task proving");
                            thread::sleep(std::time::Duration::from_secs(WORKER_SLEEP_SEC));
                        }
                        TaskStatus::Success => {
                            let submit_proof_req = SubmitProofRequest {
                                uuid: coordinator_task_uuid,
                                task_id: coordinator_task_id,
                                task_type: task.circuit_type,
                                status: ProofStatus::Ok,
                                proof: task.proof.unwrap(),
                                failure_type: None,
                                failure_msg: None,
                            };
                            if let Ok(_) =
                                self.coordinator_clients[i].submit_proof(&submit_proof_req)
                            {
                                log::info!("proof submitted");
                            } else {
                                log::error!("failed to submit proof");
                            }
                            break;
                        }
                        TaskStatus::Failed => {
                            let task_err = task.error.unwrap();
                            log::error!("task failed: {:?}", task_err);
                            let submit_proof_req = SubmitProofRequest {
                                uuid: coordinator_task_uuid,
                                task_id: coordinator_task_id,
                                task_type: task.circuit_type,
                                status: ProofStatus::Error,
                                proof: "".to_string(),
                                failure_type: Some(ProofFailureType::Panic), // TODO: handle ProofFailureType::NoPanic
                                failure_msg: Some(task_err),
                            };
                            if let Ok(_) =
                                self.coordinator_clients[i].submit_proof(&submit_proof_req)
                            {
                                log::info!("proof submitted");
                            } else {
                                log::error!("failed to submit proof");
                            }
                            break;
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

    fn build_proving_input(&self, task: &GetTaskResponseData) -> anyhow::Result<ProveRequest> {
        anyhow::ensure!(
            task.task_type == self.circuit_type,
            "task type mismatch. self: {:?}, task: {:?}, uuid: {:?}, coordinator_task_id: {:?}",
            self.circuit_type,
            task.task_type,
            task.uuid,
            task.task_id
        );

        match task.task_type {
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

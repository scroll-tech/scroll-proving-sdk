pub mod builder;
pub mod proving_service;
pub mod types;
use tokio::task::JoinSet;
pub use {builder::ProverBuilder, proving_service::ProvingService, types::*};

use crate::{
    coordinator_handler::{
        ChunkTaskDetail, CoordinatorClient, ErrorCode, GetTaskRequest, GetTaskResponseData,
        ProofFailureType, ProofStatus, SubmitProofRequest,
    },
    tracing_handler::L2gethClient,
};
use proving_service::{ProveRequest, QueryTaskRequest, TaskStatus};
use std::thread;

const WORKER_SLEEP_SEC: u64 = 20;

pub struct Prover {
    circuit_type: CircuitType,
    circuit_version: String,
    coordinator_clients: Vec<CoordinatorClient>,
    l2geth_client: Option<L2gethClient>,
    proving_service: Box<dyn ProvingService + Send + Sync>,
    n_workers: usize,
}

impl Prover {
    pub async fn run(self) {
        assert!(self.n_workers == self.coordinator_clients.len());
        if self.circuit_type == CircuitType::Chunk {
            assert!(self.l2geth_client.is_some());
        }

        let mut provers = JoinSet::new();
        let self_arc = std::sync::Arc::new(self);
        for i in 0..self_arc.n_workers {
            let self_clone = std::sync::Arc::clone(&self_arc);
            provers.spawn(async move {
                self_clone.working_loop(i).await;
            });
        }

        while provers.join_next().await.is_some() {}
    }

    async fn working_loop(&self, i: usize) {
        loop {
            let coordinator_client = &self.coordinator_clients[i];
            let prover_name = coordinator_client.prover_name.clone();

            log::info!("{:?}: getting task from coordinator", prover_name);

            let get_task_request = self.build_get_task_request().await;
            let coordinator_task = coordinator_client.get_task(&get_task_request).await;

            if let Err(e) = coordinator_task {
                log::error!("{:?}: failed to get task: {:?}", prover_name, e);
                thread::sleep(std::time::Duration::from_secs(WORKER_SLEEP_SEC));
                continue;
            } else if coordinator_task.as_ref().unwrap().errcode != ErrorCode::Success {
                log::error!(
                    "{:?}: failed to get task, errcode: {:?}, errmsg: {:?}",
                    prover_name,
                    coordinator_task.as_ref().unwrap().errcode,
                    coordinator_task.as_ref().unwrap().errmsg
                );
                thread::sleep(std::time::Duration::from_secs(WORKER_SLEEP_SEC));
                continue;
            } else if coordinator_task.as_ref().unwrap().data.is_none() {
                log::error!("{:?}: no task is available", prover_name);
                thread::sleep(std::time::Duration::from_secs(WORKER_SLEEP_SEC));
                continue;
            }

            let coordinator_task = coordinator_task.unwrap().data.unwrap();
            let coordinator_task_uuid = coordinator_task.uuid.clone();
            let coordinator_task_id = coordinator_task.task_id.clone();
            let task_type = coordinator_task.task_type;

            let proving_input = match self.build_proving_input(&coordinator_task).await {
                Ok(input) => input,
                Err(e) => {
                    log::error!(
                        "{:?}: failed to build proving input. task_type: {:?}, coordinator_task_uuid: {:?}, coordinator_task_id: {:?}, err: {:?}",
                        prover_name,
                        task_type,
                        coordinator_task_uuid,
                        coordinator_task_id,
                        e,
                    );
                    continue;
                }
            };

            let proving_task = self.proving_service.prove(proving_input);
            if proving_task.error.is_some() {
                log::error!(
                    "{:?}: failed to request proving_service to prove. task_type: {:?}, coordinator_task_uuid: {:?}, coordinator_task_id: {:?}, err: {:?}",
                    prover_name,
                    task_type,
                    coordinator_task_uuid,
                    coordinator_task_id,
                    proving_task.error,
                );
                continue;
            } else {
                let proving_service_task_id = proving_task.task_id;
                loop {
                    let task = self.proving_service.query_task(QueryTaskRequest {
                        task_id: proving_service_task_id.clone(),
                    });
                    match task.status {
                        TaskStatus::Queued => {
                            log::info!(
                                "{:?}: task queued. task_type: {:?}, coordinator_task_uuid: {:?}, coordinator_task_id: {:?}, proving_service_task_id: {:?}", 
                                prover_name,
                                task_type,
                                coordinator_task_uuid,
                                coordinator_task_id,
                                proving_service_task_id.clone(),
                            );
                            thread::sleep(std::time::Duration::from_secs(WORKER_SLEEP_SEC));
                        }
                        TaskStatus::Proving => {
                            log::info!(
                                "{:?}: task proving. task_type: {:?}, coordinator_task_uuid: {:?}, coordinator_task_id: {:?}, proving_service_task_id: {:?}",
                                prover_name,
                                task_type,
                                coordinator_task_uuid,
                                coordinator_task_id,
                                proving_service_task_id.clone(),
                            );
                            thread::sleep(std::time::Duration::from_secs(WORKER_SLEEP_SEC));
                        }
                        TaskStatus::Success => {
                            log::info!(
                                "{:?}: task proved successfully. task_type: {:?}, coordinator_task_uuid: {:?}, coordinator_task_id: {:?}, proving_service_task_id: {:?}",
                                prover_name,
                                task_type,
                                coordinator_task_uuid,
                                coordinator_task_id,
                                proving_service_task_id.clone(),
                            );
                            let submit_proof_req = SubmitProofRequest {
                                uuid: coordinator_task_uuid.clone(),
                                task_id: coordinator_task_id.clone(),
                                task_type: task.circuit_type,
                                status: ProofStatus::Ok,
                                proof: task.proof.unwrap(),
                                failure_type: None,
                                failure_msg: None,
                            };
                            match coordinator_client.submit_proof(&submit_proof_req).await {
                                Ok(_) => {
                                    log::info!(
                                        "{:?}: proof submitted. task_type: {:?}, coordinator_task_uuid: {:?}, coordinator_task_id: {:?}, proving_service_task_id: {:?}",
                                        prover_name,
                                        task_type,
                                        coordinator_task_uuid,
                                        coordinator_task_id,
                                        proving_service_task_id.clone(),
                                    );
                                }
                                Err(e) => {
                                    log::error!(
                                        "{:?}: failed to submit proof. task_type: {:?}, coordinator_task_uuid: {:?}, coordinator_task_id: {:?}, proving_service_task_id: {:?}, submission err: {:?}",
                                        prover_name,
                                        task_type,
                                        coordinator_task_uuid,
                                        coordinator_task_id,
                                        proving_service_task_id.clone(),
                                        e,
                                    );
                                }
                            };
                            break;
                        }
                        TaskStatus::Failed => {
                            let task_err = task.error.unwrap();
                            log::error!(
                                "{:?}: task failed. task_type: {:?}, coordinator_task_uuid: {:?}, coordinator_task_id: {:?}, proving_service_task_id: {:?}, err: {:?}",
                                prover_name,
                                task_type,
                                coordinator_task_uuid,
                                coordinator_task_id,
                                proving_service_task_id.clone(),
                                task_err,
                            );
                            let submit_proof_req = SubmitProofRequest {
                                uuid: coordinator_task_uuid.clone(),
                                task_id: coordinator_task_id.clone(),
                                task_type: task.circuit_type,
                                status: ProofStatus::Error,
                                proof: "".to_string(),
                                failure_type: Some(ProofFailureType::Panic), // TODO: handle ProofFailureType::NoPanic
                                failure_msg: Some(task_err),
                            };
                            match coordinator_client.submit_proof(&submit_proof_req).await {
                                Ok(_) => {
                                    log::info!(
                                        "{:?}: proof_err submitted. task_type: {:?}, coordinator_task_uuid: {:?}, coordinator_task_id: {:?}, proving_service_task_id: {:?}",
                                        prover_name,
                                        task_type,
                                        coordinator_task_uuid,
                                        coordinator_task_id,
                                        proving_service_task_id.clone(),
                                    );
                                }
                                Err(e) => {
                                    log::error!(
                                        "{:?}: failed to submit proof_err. task_type: {:?}, coordinator_task_uuid: {:?}, coordinator_task_id: {:?}, proving_service_task_id: {:?}, submission err: {:?}",
                                        prover_name,
                                        task_type,
                                        coordinator_task_uuid,
                                        coordinator_task_id,
                                        proving_service_task_id.clone(),
                                        e,
                                    );
                                }
                            };
                            break;
                        }
                    }
                }
            }
        }
    }

    async fn build_get_task_request(&self) -> GetTaskRequest {
        let prover_height = match &self.l2geth_client {
            None => None,
            Some(l2geth_client) => match l2geth_client.block_number().await {
                Ok(block_number) => block_number.as_number(),
                Err(_) => None,
            },
        };

        GetTaskRequest {
            task_types: vec![self.circuit_type],
            prover_height,
        }
    }

    async fn build_proving_input(
        &self,
        task: &GetTaskResponseData,
    ) -> anyhow::Result<ProveRequest> {
        anyhow::ensure!(
            task.task_type == self.circuit_type,
            "task type mismatch. self: {:?}, task: {:?}, coordinator_task_uuid: {:?}, coordinator_task_id: {:?}",
            self.circuit_type,
            task.task_type,
            task.uuid,
            task.task_id
        );

        match task.task_type {
            CircuitType::Undefined => {
                unreachable!();
            }
            CircuitType::Chunk => {
                let chunk_task_detail: ChunkTaskDetail = serde_json::from_str(&task.task_data)?;
                let traces = self
                    .l2geth_client
                    .as_ref()
                    .unwrap()
                    .get_sorted_traces_by_hashes(&chunk_task_detail.block_hashes)
                    .await?;
                let input = serde_json::to_string(&traces)?;

                Ok(ProveRequest {
                    circuit_type: task.task_type,
                    circuit_version: self.circuit_version.clone(),
                    hard_fork_name: task.hard_fork_name.clone(),
                    input,
                })
            }
            CircuitType::Batch => Ok(ProveRequest {
                circuit_type: task.task_type,
                circuit_version: self.circuit_version.clone(),
                hard_fork_name: task.hard_fork_name.clone(),
                input: task.task_data.clone(),
            }),
            CircuitType::Bundle => Ok(ProveRequest {
                circuit_type: task.task_type,
                circuit_version: self.circuit_version.clone(),
                hard_fork_name: task.hard_fork_name.clone(),
                input: task.task_data.clone(),
            }),
        }
    }
}

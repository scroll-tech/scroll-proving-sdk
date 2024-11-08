pub mod builder;
pub mod proving_service;
pub mod types;
use crate::{
    coordinator_handler::{
        ChunkTaskDetail, CoordinatorClient, ErrorCode, GetTaskRequest, GetTaskResponseData,
        ProofFailureType, ProofStatus, SubmitProofRequest,
    },
    db::Db,
    tracing_handler::L2gethClient,
};
use axum::{routing::get, Router};
use proving_service::{ProveRequest, QueryTaskRequest, TaskStatus};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};
use tracing::{error, info, instrument};

pub use {builder::ProverBuilder, proving_service::ProvingService, types::*};

const WORKER_SLEEP_SEC: u64 = 20;

pub struct Prover {
    circuit_type: CircuitType,
    circuit_version: String,
    coordinator_clients: Vec<CoordinatorClient>,
    l2geth_client: Option<L2gethClient>,
    proving_service: Box<dyn ProvingService + Send + Sync>,
    n_workers: usize,
    health_listener_addr: String,
    db: Db,
}

impl Prover {
    pub async fn run(self) {
        assert!(self.n_workers == self.coordinator_clients.len());
        if self.circuit_type == CircuitType::Chunk {
            assert!(self.l2geth_client.is_some());
        }

        self.test_coordinator_connection().await;

        let app = Router::new().route("/", get(|| async { "OK" }));
        let addr = SocketAddr::from_str(&self.health_listener_addr)
            .expect("Failed to parse socket address");
        let server = axum::Server::bind(&addr).serve(app.into_make_service());
        let health_check_server_task = tokio::spawn(server);

        let mut provers = JoinSet::new();
        let self_arc = std::sync::Arc::new(self);
        for i in 0..self_arc.n_workers {
            let self_clone = std::sync::Arc::clone(&self_arc);
            provers.spawn(async move {
                self_clone.working_loop(i).await;
            });
        }

        tokio::select! {
            _ = health_check_server_task => {},
            _ = async { while provers.join_next().await.is_some() {} } => {},
        }
    }

    async fn test_coordinator_connection(&self) {
        self.coordinator_clients[0]
            .get_token(true)
            .await
            .expect("Failed to login to coordinator");
    }

    #[instrument(skip(self))]
    async fn working_loop(&self, i: usize) {
        loop {
            let coordinator_client = &self.coordinator_clients[i];
            let prover_name = &coordinator_client.prover_name;

            info!(?prover_name, "Getting task from coordinator");

            if let Err(e) = self.handle_task(coordinator_client).await {
                error!(?prover_name, ?e, "Error handling task");
            }

            sleep(Duration::from_secs(WORKER_SLEEP_SEC)).await;
        }
    }

    async fn handle_task(&self, coordinator_client: &CoordinatorClient) -> anyhow::Result<()> {
        let public_key = coordinator_client.key_signer.get_public_key();
        if let (Some(coordinator_task), Some(proving_task_id)) = (
            self.db.get_coordinator_task_by_public_key(public_key.clone()),
            self.db.get_proving_task_id_by_public_key(public_key),
        ) {
            return self
                .handle_proving_progress(coordinator_client, &coordinator_task, proving_task_id)
                .await;
        }
        
        let coordinator_task = self.get_coordinator_task(coordinator_client).await?;
        let proving_task = self.request_proving(&coordinator_task).await?;
        self.handle_proving_progress(coordinator_client, &coordinator_task, proving_task.task_id)
            .await
    }

    async fn get_coordinator_task(
        &self,
        coordinator_client: &CoordinatorClient,
    ) -> anyhow::Result<GetTaskResponseData> {
        let get_task_request = self.build_get_task_request().await;
        let coordinator_task = coordinator_client.get_task(&get_task_request).await?;

        if coordinator_task.errcode != ErrorCode::Success {
            anyhow::bail!(
                "Failed to get task, errcode: {:?}, errmsg: {:?}",
                coordinator_task.errcode,
                coordinator_task.errmsg
            );
        }

        coordinator_task
            .data
            .ok_or_else(|| anyhow::anyhow!("No task available"))
    }

    async fn request_proving(
        &self,
        coordinator_task: &GetTaskResponseData,
    ) -> anyhow::Result<proving_service::ProveResponse> {
        let proving_input = self.build_proving_input(coordinator_task).await?;
        let proving_task = self.proving_service.prove(proving_input).await;

        if let Some(error) = proving_task.error {
            anyhow::bail!(
                "Failed to request proving_service to prove. task_type: {:?}, coordinator_task_uuid: {:?}, coordinator_task_id: {:?}, err: {:?}",
                coordinator_task.task_type,
                coordinator_task.uuid,
                coordinator_task.task_id,
                error,
            );
        }

        Ok(proving_task)
    }

    async fn handle_proving_progress(
        &self,
        coordinator_client: &CoordinatorClient,
        coordinator_task: &GetTaskResponseData,
        proving_service_task_id: String,
    ) -> anyhow::Result<()> {
        let prover_name = &coordinator_client.prover_name;
        let public_key = &coordinator_client.key_signer.get_public_key();
        let task_type = coordinator_task.task_type;
        let coordinator_task_uuid = &coordinator_task.uuid;
        let coordinator_task_id = &coordinator_task.task_id;

        loop {
            let task = self
                .proving_service
                .query_task(QueryTaskRequest {
                    task_id: proving_service_task_id.clone(),
                })
                .await;

            match task.status {
                TaskStatus::Queued | TaskStatus::Proving => {
                    info!(
                        ?prover_name,
                        ?task_type,
                        ?coordinator_task_uuid,
                        ?coordinator_task_id,
                        ?proving_service_task_id,
                        status = ?task.status,
                        "Task status update"
                    );
                    self.db.set_coordinator_task_by_public_key(public_key.clone(), coordinator_task);
                    self.db.set_proving_task_id_by_public_key(public_key.clone(), proving_service_task_id.clone());
                    sleep(Duration::from_secs(WORKER_SLEEP_SEC)).await;
                }
                TaskStatus::Success => {
                    info!(
                        ?prover_name,
                        ?task_type,
                        ?coordinator_task_uuid,
                        ?coordinator_task_id,
                        ?proving_service_task_id,
                        "Task proved successfully"
                    );
                    self.submit_proof(
                        coordinator_client,
                        coordinator_task,
                        task,
                        ProofStatus::Ok,
                        None,
                    )
                    .await?;
                    self.db.delete_coordinator_task_by_public_key(public_key.clone());
                    self.db.delete_proving_task_id_by_public_key(public_key.clone());
                    break;
                }
                TaskStatus::Failed => {
                    let task_err = task.error.clone().unwrap();
                    error!(
                        ?prover_name,
                        ?task_type,
                        ?coordinator_task_uuid,
                        ?coordinator_task_id,
                        ?proving_service_task_id,
                        ?task_err,
                        "Task failed"
                    );
                    self.submit_proof(
                        coordinator_client,
                        coordinator_task,
                        task,
                        ProofStatus::Error,
                        Some(task_err),
                    )
                    .await?;
                    self.db.delete_coordinator_task_by_public_key(public_key.clone());
                    self.db.delete_proving_task_id_by_public_key(public_key.clone());
                    break;
                }
            }
        }

        Ok(())
    }

    async fn submit_proof(
        &self,
        coordinator_client: &CoordinatorClient,
        coordinator_task: &GetTaskResponseData,
        task: proving_service::QueryTaskResponse,
        status: ProofStatus,
        failure_msg: Option<String>,
    ) -> anyhow::Result<()> {
        let submit_proof_req = SubmitProofRequest {
            uuid: coordinator_task.uuid.clone(),
            task_id: coordinator_task.task_id.clone(),
            task_type: coordinator_task.task_type,
            status,
            proof: task.proof.unwrap_or_default(),
            failure_type: failure_msg.as_ref().map(|_| ProofFailureType::Panic), // TODO: handle ProofFailureType::NoPanic
            failure_msg,
        };

        coordinator_client.submit_proof(&submit_proof_req).await?;
        info!(
            prover_name = ?coordinator_client.prover_name,
            ?coordinator_task.task_type,
            ?coordinator_task.uuid,
            ?coordinator_task.task_id,
            ?task.task_id,
            "Proof submitted"
        );
        Ok(())
    }

    async fn build_get_task_request(&self) -> GetTaskRequest {
        let prover_height = match &self.l2geth_client {
            None => None,
            Some(l2geth_client) => match l2geth_client.block_number().await {
                Ok(block_number) => block_number.as_number().map(|num| num.as_u64()),
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
            CircuitType::Batch | CircuitType::Bundle => Ok(ProveRequest {
                circuit_type: task.task_type,
                circuit_version: self.circuit_version.clone(),
                hard_fork_name: task.hard_fork_name.clone(),
                input: task.task_data.clone(),
            }),
        }
    }
}

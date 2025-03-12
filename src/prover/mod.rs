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
use anyhow::bail;
use axum::{routing::get, Router};
use ethers_core::types::H256;
use ethers_providers::Middleware;
use proving_service::{ProveRequest, QueryTaskRequest, TaskStatus};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::time::{sleep, Duration};
use tokio::{sync::RwLock, task::JoinSet};
use tracing::{error, info, instrument};

pub use {builder::ProverBuilder, proving_service::ProvingService, types::*};

const WORKER_SLEEP_SEC: u64 = 20;

pub struct Prover<Backend: ProvingService + Send + Sync + 'static> {
    circuit_type: CircuitType,
    proof_types: Vec<ProofType>,
    circuit_version: String,
    coordinator_clients: Vec<CoordinatorClient>,
    l2geth_client: Option<L2gethClient>,
    proving_service: RwLock<Backend>,
    n_workers: usize,
    health_listener_addr: String,
    db: Db,
}

impl<Backend> Prover<Backend>
where
    Backend: ProvingService + Send + Sync + 'static,
{
    pub async fn run(self) {
        assert!(self.n_workers == self.coordinator_clients.len());
        if self.proof_types.contains(&ProofType::Chunk) {
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
        if let (Some(coordinator_task), Some(mut proving_task_id)) = self
            .db
            .get_task(coordinator_client.key_signer.get_public_key())
        {
            if self.proving_service.read().await.is_local() {
                let proving_task = self.request_proving(&coordinator_task).await?;
                proving_task_id = proving_task.task_id
            }
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
        let proving_task = self
            .proving_service
            .write()
            .await
            .prove(proving_input)
            .await;

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
                .write()
                .await
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
                    self.db.set_task(
                        public_key.clone(),
                        coordinator_task,
                        proving_service_task_id.clone(),
                    );
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
                    self.db.delete_task(public_key.clone());
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
                    self.db.delete_task(public_key.clone());
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
            task_types: self.proof_types.clone(),
            prover_height,
        }
    }

    async fn build_proving_input(
        &self,
        task: &GetTaskResponseData,
    ) -> anyhow::Result<ProveRequest> {
        anyhow::ensure!(
            self.proof_types.contains(&task.task_type),
            "unsupported task type. self: {:?}, task: {:?}, coordinator_task_uuid: {:?}, coordinator_task_id: {:?}",
            self.proof_types,
            task.task_type,
            task.uuid,
            task.task_id
        );

        match self.circuit_type {
            CircuitType::Halo2 => self.build_halo2_input(task).await,
            CircuitType::OpenVM => {
                #[cfg(not(feature = "openvm"))]
                anyhow::bail!("OpenVM is not enabled in this build");
                #[cfg(feature = "openvm")]
                self.build_openvm_input(task).await
            }
            _ => anyhow::bail!("unsupported circuit type: {:?}", self.circuit_type),
        }
    }

    async fn build_halo2_input(&self, task: &GetTaskResponseData) -> anyhow::Result<ProveRequest> {
        match task.task_type {
            ProofType::Undefined => {
                unreachable!();
            }
            ProofType::Chunk => {
                let chunk_task_detail: ChunkTaskDetail = serde_json::from_str(&task.task_data)?;
                let serialized_traces = self
                    .l2geth_client
                    .as_ref()
                    .unwrap()
                    .get_traces_by_hashes(&chunk_task_detail.block_hashes)
                    .await?;
                // Note: Manually join pre-serialized traces since they are already in JSON format.
                // Using serde_json::to_string would escape the JSON strings, creating invalid nested JSON.
                let input = format!("[{}]", serialized_traces.join(","));

                Ok(ProveRequest {
                    proof_type: task.task_type,
                    circuit_version: self.circuit_version.clone(),
                    hard_fork_name: task.hard_fork_name.clone(),
                    input,
                })
            }
            ProofType::Batch | ProofType::Bundle => Ok(ProveRequest {
                proof_type: task.task_type,
                circuit_version: self.circuit_version.clone(),
                hard_fork_name: task.hard_fork_name.clone(),
                input: task.task_data.clone(),
            }),
        }
    }

    #[cfg(feature = "openvm")]
    async fn build_openvm_input(&self, task: &GetTaskResponseData) -> anyhow::Result<ProveRequest> {
        match task.task_type {
            ProofType::Chunk => {
                let chunk_task_detail: ChunkTaskDetail = serde_json::from_str(&task.task_data)?;
                let mut witnesses = vec![];
                for block_hash in chunk_task_detail.block_hashes {
                    witnesses.push(self.build_block_witness(block_hash).await?);
                }
                witnesses.sort_by(|a, b| a.header.number.cmp(&b.header.number));

                let input_map = serde_json::json!({
                    "witnesses": witnesses,
                    "prev_msg_queue_hash": chunk_task_detail.prev_msg_queue_hash
                });

                Ok(ProveRequest {
                    proof_type: task.task_type,
                    circuit_version: self.circuit_version.clone(),
                    hard_fork_name: task.hard_fork_name.clone(),
                    input: serde_json::to_string(&input_map)?,
                })
            }
            ProofType::Batch | ProofType::Bundle => Ok(ProveRequest {
                proof_type: task.task_type,
                circuit_version: self.circuit_version.clone(),
                hard_fork_name: task.hard_fork_name.clone(),
                input: task.task_data.clone(),
            }),
            _ => bail!("unsupported task type: {:?}", task.task_type),
        }
    }

    #[cfg(feature = "openvm")]
    async fn build_block_witness(
        &self,
        hash: H256,
    ) -> anyhow::Result<sbv_primitives::types::BlockWitness> {
        use sbv_utils::rpc::ProviderExt;

        let client = self.l2geth_client.as_ref().unwrap();
        let block = client
            .provider
            .get_block(hash)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Block not found"))?;
        let block_num = block.number.expect("block hash without number").as_u64();

        let provider =
            alloy::providers::ProviderBuilder::<_, _, sbv_primitives::Network>::default()
                .on_http(client.provider.provider().url().clone());

        let witness = provider.dump_block_witness(block_num.into()).await?;
        witness.ok_or_else(|| anyhow::anyhow!("Failed to dump block witness"))
    }
}

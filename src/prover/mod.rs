pub mod builder;
pub mod proving_service;
pub mod types;

use crate::{
    coordinator_handler::{
        CoordinatorClient, ErrorCode, GetTaskRequest, GetTaskResponseData, ProofFailureType,
        ProofStatus, SubmitProofRequest,
    },
    db::Db,
};
use axum::{routing::get, Router};
use proving_service::{ProveRequest, QueryTaskRequest, TaskStatus};
use std::future::IntoFuture;
use std::net::SocketAddr;
use std::str::FromStr;
use rand::Rng;
use tokio::net::TcpListener;
use tokio::time::{sleep, Duration};
use tokio::{sync::RwLock, task::JoinSet};
use tracing::Level;
use tracing::{error, info, instrument};

pub use {builder::ProverBuilder, proving_service::ProvingService, types::*};

pub struct Prover<Backend: ProvingService + Send + Sync + 'static> {
    proof_types: Vec<ProofType>,
    circuit_version: String,
    coordinator_clients: Vec<CoordinatorClient>,
    proving_service: RwLock<Backend>,
    n_workers: usize,
    health_listener_addr: String,
    db: Option<Db>,
    poll_interval_sec: u64,
    randomized_delay_sec: u64,
    suppress_empty_task_error: bool,
}

impl<Backend> Prover<Backend>
where
    Backend: ProvingService + Send + Sync + 'static,
{
    pub async fn run(self) {
        assert_eq!(self.n_workers, self.coordinator_clients.len());

        self.test_coordinator_connection().await;

        let app = Router::new().route("/", get(|| async { "OK" }));
        let addr = SocketAddr::from_str(&self.health_listener_addr)
            .expect("Failed to parse socket address");
        let listener = TcpListener::bind(addr)
            .await
            .expect("Failed to bind health check listener");
        let server = axum::serve(listener, app).into_future();
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

    pub async fn one_shot(
        self: std::sync::Arc<Self>,
        tasks: &[String],
        task_type: ProofType,
    ) -> bool {
        assert_eq!(self.n_workers, self.coordinator_clients.len());

        self.test_coordinator_connection().await;

        let mut provers = JoinSet::new();
        let mut work_set = Vec::from_iter(0..self.n_workers);
        for task in tasks {
            while work_set.is_empty() {
                let ret = provers.join_next().await.expect("joinset must not empty");
                if let Ok(i) = ret {
                    work_set.push(i)
                } else {
                    // quit since one task has failed
                    return false;
                }
            }

            let self_clone = self.clone();
            let task_str = task.to_string();
            let i = work_set.pop().expect("can not be empty");
            provers.spawn(async move {
                // Soft start delay to stagger the provers
                sleep(self_clone.poll_delay()).await;

                let coordinator_client = &self_clone.coordinator_clients[i];
                let prover_name = &coordinator_client.prover_name;

                info!(?prover_name, "Getting task from coordinator");

                if let Err(e) = self_clone
                    .handle_task(coordinator_client, Some((task_type, task_str.as_str())))
                    .await
                {
                    error!(prover_name, error = e.to_string(), "Error handling task");
                    panic!("task fail");
                }
                i
            });
        }

        // wait until all tasks has been done
        while let Some(r) = provers.join_next().await {
            let Ok(r) = r else {
                // quit since one task has failed
                return false;
            };
            info!("worker {r} has completed");
        }
        true
    }

    async fn test_coordinator_connection(&self) {
        self.coordinator_clients[0]
            .get_token(true)
            .await
            .expect("Failed to login to coordinator");
    }

    #[instrument(skip(self), level = Level::DEBUG)]
    async fn working_loop(&self, i: usize) {
        // Soft start delay to stagger the provers
        sleep(self.poll_delay()).await;
        loop {
            let coordinator_client = &self.coordinator_clients[i];
            if let Err(e) = self.handle_task(coordinator_client, None).await {
                error!(prover_name = %coordinator_client.prover_name, error = e.to_string(), "Error handling task");
            }
            sleep(self.poll_delay()).await;
        }
    }

    #[instrument(skip(self, coordinator_client), level = Level::DEBUG)]
    async fn handle_task(
        &self,
        coordinator_client: &CoordinatorClient,
        task_spec: Option<(ProofType, &str)>,
    ) -> eyre::Result<()> {
        if let (Some(coordinator_task), Some(mut proving_task_id)) = self
            .db
            .as_ref()
            .map(|db| db.get_task(coordinator_client.key_signer.get_public_key()))
            .unwrap_or_default()
        {
            let task_id = coordinator_task.clone().task_id;
            debug!(task_id = %task_id, "got previous task from db");
            if self.proving_service.read().await.is_local() {
                let proving_task = self
                    .request_proving(coordinator_client, &coordinator_task)
                    .await?;
                proving_task_id = proving_task.task_id
            }
            return self
                .handle_proving_progress(coordinator_client, &coordinator_task, proving_task_id)
                .await;
        }

        let mut get_task_request = self.build_get_task_request(None)?;
        if let Some((t, s)) = task_spec {
            get_task_request.task_types = vec![t];
            get_task_request.task_id.replace(s.to_string());
        }
        let Some(coordinator_task) = self
            .get_coordinator_task(coordinator_client, &get_task_request)
            .await?
        else {
            return Ok(());
        };
        info!(prover_name = %coordinator_client.prover_name, "Got task from coordinator");
        let proving_task = self
            .request_proving(coordinator_client, &coordinator_task)
            .await?;
        self.handle_proving_progress(coordinator_client, &coordinator_task, proving_task.task_id)
            .await
    }

    async fn get_coordinator_task(
        &self,
        coordinator_client: &CoordinatorClient,
        request: &GetTaskRequest,
    ) -> eyre::Result<Option<GetTaskResponseData>> {
        let coordinator_task = coordinator_client.get_task(request).await?;

        if coordinator_task.errcode == ErrorCode::ErrCoordinatorEmptyProofData
            && self.suppress_empty_task_error
        {
            return Ok(None);
        }

        if coordinator_task.errcode != ErrorCode::Success {
            eyre::bail!(
                "Failed to get task, errcode: {:?}, errmsg: {:?}",
                coordinator_task.errcode,
                coordinator_task.errmsg
            );
        }

        Ok(coordinator_task.data)
    }

    async fn request_proving(
        &self,
        coordinator_client: &CoordinatorClient,
        coordinator_task: &GetTaskResponseData,
    ) -> eyre::Result<proving_service::ProveResponse> {
        let proving_input = match self.get_proving_input(coordinator_task) {
            Ok(result) => result,
            Err(error) => {
                self.submit_proof(
                    coordinator_client,
                    coordinator_task,
                    proving_service::QueryTaskResponse::default(),
                    ProofStatus::Error,
                    Some(format!("failed to build proving input: error {:?}", error)),
                )
                .await?;
                eyre::bail!(
                    "Failed to build proving input. task_type: {:?}, coordinator_task_uuid: {:?}, coordinator_task_id: {:?}, err: {:?}",
                    coordinator_task.task_type,
                    coordinator_task.uuid,
                    coordinator_task.task_id,
                    error,
                );
            }
        };

        let proving_task = self
            .proving_service
            .write()
            .await
            .prove(proving_input)
            .await;
        if let Some(error) = proving_task.error {
            self.submit_proof(
                coordinator_client,
                coordinator_task,
                proving_service::QueryTaskResponse::default(),
                ProofStatus::Error,
                Some(format!("failed to request proving: error {:?}", error)),
            )
            .await?;
            eyre::bail!(
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
    ) -> eyre::Result<()> {
        let prover_name = &coordinator_client.prover_name;
        let public_key = &coordinator_client.key_signer.get_public_key();
        let task_type = coordinator_task.task_type;
        let coordinator_task_uuid = &coordinator_task.uuid;
        let coordinator_task_id = &coordinator_task.task_id;

        // Track last observed status to avoid spamming logs when status hasn't changed.
        let mut last_status: Option<TaskStatus> = None;

        loop {
            let task = self
                .proving_service
                .write()
                .await
                .query_task(QueryTaskRequest {
                    task_id: proving_service_task_id.clone(),
                })
                .await;

            let current_status = task.status; // capture for comparison

            match current_status {
                TaskStatus::Queued | TaskStatus::Proving => {
                    if last_status != Some(current_status) {
                        info!(
                            ?prover_name,
                            ?task_type,
                            ?coordinator_task_uuid,
                            ?coordinator_task_id,
                            ?proving_service_task_id,
                            status = ?current_status,
                            "Task status update"
                        );
                    }
                    last_status.replace(current_status);
                    if let Some(db) = &self.db {
                        db.set_task(
                            public_key.clone(),
                            coordinator_task,
                            proving_service_task_id.clone(),
                        );
                    }
                    sleep(self.poll_delay()).await;
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
                    if let Some(db) = &self.db {
                        db.delete_task(public_key.clone());
                    }
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
                    if let Some(db) = &self.db {
                        db.delete_task(public_key.clone());
                    }
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
    ) -> eyre::Result<()> {
        let submit_proof_req = SubmitProofRequest {
            universal: true,
            uuid: coordinator_task.uuid.clone(),
            task_id: coordinator_task.task_id.clone(),
            task_type: coordinator_task.task_type,
            status,
            proof: task.proof.unwrap_or_default(),
            failure_type: failure_msg.as_ref().map(|_| ProofFailureType::Panic), // TODO: handle ProofFailureType::NoPanic
            failure_msg,
        };

        let submit_proof_result = match coordinator_client.submit_proof(&submit_proof_req).await {
            Ok(result) => result,
            Err(e) => {
                info!(
                    prover_name = ?coordinator_client.prover_name,
                    ?coordinator_task.task_type,
                    ?coordinator_task.uuid,
                    ?coordinator_task.task_id,
                    ?task.task_id,
                    error = ?e,
                    "Failed to submit proof due to a http error"
                );
                return Ok(());
            }
        };

        if submit_proof_result.errcode != ErrorCode::Success {
            info!(
                prover_name = ?coordinator_client.prover_name,
                ?coordinator_task.task_type,
                ?coordinator_task.uuid,
                ?coordinator_task.task_id,
                ?task.task_id,
                errcode = ?submit_proof_result.errcode,
                errmsg = ?submit_proof_result.errmsg,
                "Failed to submit proof due to coordinator error"
            );
        } else {
            info!(
                prover_name = ?coordinator_client.prover_name,
                ?coordinator_task.task_type,
                ?coordinator_task.uuid,
                ?coordinator_task.task_id,
                ?task.task_id,
                "Proof submitted successfully"
            );
        }
        Ok(())
    }

    fn build_get_task_request(&self, prover_height: Option<u64>) -> eyre::Result<GetTaskRequest> {
        Ok(GetTaskRequest {
            task_types: self.proof_types.clone(),
            prover_height,
            universal: true,
            task_id: None,
        })
    }

    fn get_proving_input(&self, task: &GetTaskResponseData) -> eyre::Result<ProveRequest> {
        eyre::ensure!(
            self.proof_types.contains(&task.task_type),
            "unsupported task type. self: {:?}, task: {:?}, coordinator_task_uuid: {:?}, coordinator_task_id: {:?}",
            self.proof_types,
            task.task_type,
            task.uuid,
            task.task_id
        );

        Ok(ProveRequest {
            proof_type: task.task_type,
            circuit_version: self.circuit_version.clone(),
            hard_fork_name: task.hard_fork_name.clone(),
            input: task.task_data.clone(),
        })
    }

    fn poll_delay(&self) -> Duration {
        let base_delay = Duration::from_secs(self.poll_interval_sec);
        if self.randomized_delay_sec == 0 {
            return base_delay;
        }
        let mut rng = rand::rng();
        let random_delay = rng.random_range(0..self.randomized_delay_sec * 1000);
        base_delay + Duration::from_millis(random_delay)
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::prover::{
        proving_service::{
            GetVkRequest, GetVkResponse, ProveRequest, ProveResponse, QueryTaskRequest,
            QueryTaskResponse,
        },
        ProverBuilder, ProvingService,
    };
    use async_trait::async_trait;
    use tokio;

    struct MockProver {}

    #[async_trait]
    impl ProvingService for MockProver {
        fn is_local(&self) -> bool {
            true
        }
        async fn get_vks(&self, _: GetVkRequest) -> GetVkResponse {
            GetVkResponse {
                ..Default::default()
            }
        }
        async fn prove(&mut self, _: ProveRequest) -> ProveResponse {
            ProveResponse {
                ..Default::default()
            }
        }
        async fn query_task(&mut self, _: QueryTaskRequest) -> QueryTaskResponse {
            QueryTaskResponse {
                ..Default::default()
            }
        }
    }

    #[tokio::test]
    async fn test_build_get_task_request() {
        let cfg = Config::from_file("conf/config.json".to_string()).unwrap();
        let prover_service = MockProver {};
        let prover = ProverBuilder::new(cfg, prover_service)
            .build()
            .await
            .unwrap();

        let get_task_request = prover.build_get_task_request(None);
        assert!(get_task_request.is_ok())
    }
}

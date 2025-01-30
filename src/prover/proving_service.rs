use super::ProofType;
use async_trait::async_trait;

#[async_trait]
pub trait ProvingService {
    fn is_local(&self) -> bool;
    async fn get_vks(&self, req: GetVkRequest) -> GetVkResponse;
    async fn prove(&mut self, req: ProveRequest) -> ProveResponse;
    async fn query_task(&mut self, req: QueryTaskRequest) -> QueryTaskResponse;
}

pub struct GetVkRequest {
    pub proof_types: Vec<ProofType>,
    pub circuit_version: String,
}

pub struct GetVkResponse {
    pub vks: Vec<String>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct ProveRequest {
    pub proof_type: ProofType,
    pub circuit_version: String,
    pub hard_fork_name: String,
    pub input: String,
}

#[derive(Default)]
pub struct ProveResponse {
    pub task_id: String,
    pub proof_type: ProofType,
    pub circuit_version: String,
    pub hard_fork_name: String,
    pub status: TaskStatus,
    pub created_at: f64,
    pub started_at: Option<f64>,
    pub finished_at: Option<f64>,
    pub compute_time_sec: Option<f64>,
    pub input: Option<String>,
    pub proof: Option<String>,
    pub vk: Option<String>,
    pub error: Option<String>,
}

pub struct QueryTaskRequest {
    pub task_id: String,
}

#[derive(Default)]
pub struct QueryTaskResponse {
    pub task_id: String,
    pub proof_type: ProofType,
    pub circuit_version: String,
    pub hard_fork_name: String,
    pub status: TaskStatus,
    pub created_at: f64,
    pub started_at: Option<f64>,
    pub finished_at: Option<f64>,
    pub compute_time_sec: Option<f64>,
    pub input: Option<String>,
    pub proof: Option<String>,
    pub vk: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, PartialEq, Default)]
pub enum TaskStatus {
    #[default]
    Queued,
    Proving,
    Success,
    Failed,
}

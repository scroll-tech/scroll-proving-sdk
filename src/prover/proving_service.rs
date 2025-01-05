use super::CircuitType;
use async_trait::async_trait;

#[async_trait]
pub trait ProvingService {
    fn is_local(&self) -> bool;
    async fn get_vks(&self, req: GetVkRequest) -> GetVkResponse;
    async fn prove(&self, req: ProveRequest) -> ProveResponse;
    async fn query_task(&self, req: QueryTaskRequest) -> QueryTaskResponse;
}

pub struct GetVkRequest {
    pub circuit_types: Vec<CircuitType>,
    pub circuit_version: String,
}

pub struct GetVkResponse {
    pub vks: Vec<String>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct ProveRequest {
    pub circuit_type: CircuitType,
    pub circuit_version: String,
    pub hard_fork_name: String,
    pub input: String,
}

pub struct ProveResponse {
    pub task_id: String,
    pub circuit_type: CircuitType,
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

pub struct QueryTaskResponse {
    pub task_id: String,
    pub circuit_type: CircuitType,
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

#[derive(Debug, PartialEq)]
pub enum TaskStatus {
    Queued,
    Proving,
    Success,
    Failed,
}

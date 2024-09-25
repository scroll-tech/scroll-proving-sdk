use super::CircuitType;

pub trait ProvingService {
    fn prove(&self, req: ProveRequest) -> ProveResponse; // TODO: Result<ProveResponse, Error>
    fn query_task(&self, req: QueryTaskRequest) -> QueryTaskResponse; // TODO: Result<QueryTaskResponse, Error>
    fn is_local(&self) -> bool;
}

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
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    pub compute_time_sec: Option<u64>,
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
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub finished_at: Option<u64>,
    pub compute_time_sec: Option<u64>,
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

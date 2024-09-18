trait ProvingService {
    fn prove(&self, req: ProveRequest) -> ProveResponse;
    fn get_task(&self, req: GetTaskRequest) -> GetTaskResponse;
}

pub struct ProveRequest {
    pub circuit_type: CircuitType,
    pub circuit_version: String,
    pub input: String,
}

pub struct ProveResponse {
    pub task_id: String,
    pub circuit_type: CircuitType,
    pub circuit_version: String,
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

pub struct GetTaskRequest {
    pub task_id: String,
}

pub struct GetTaskResponse {
    pub task_id: String,
    pub circuit_version: String,
    pub circuit_type: CircuitType,
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

pub enum TaskStatus {
    Queued,
    Proving,
    Success,
    Failed,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum CircuitType {
    Chunk,
    Batch,
    Bundle,
}
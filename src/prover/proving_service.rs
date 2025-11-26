use super::ProofType;
use async_trait::async_trait;
use std::fmt;

#[async_trait]
pub trait ProvingService {
    fn is_local(&self) -> bool;
    async fn get_vks(&self, req: GetVkRequest) -> GetVkResponse;
    async fn prove(&mut self, req: ProveRequest) -> ProveResponse;
    async fn query_task(&mut self, req: QueryTaskRequest) -> QueryTaskResponse;
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GetVkRequest {
    pub proof_types: Vec<ProofType>,
    pub circuit_version: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct GetVkResponse {
    pub vks: Vec<String>,
    pub error: Option<String>,
}

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct ProveRequest {
    pub proof_type: ProofType,
    pub circuit_version: String,
    pub hard_fork_name: String,
    pub input: String,
}

impl fmt::Debug for ProveRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProveRequest")
            .field("proof_type", &self.proof_type)
            .field("circuit_version", &self.circuit_version)
            .field("hard_fork_name", &self.hard_fork_name)
            .field("input", &"...")
            .finish()
    }
}

#[derive(Default, Clone, PartialEq)]
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

impl fmt::Debug for ProveResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt = f.debug_struct("ProveResponse");
        fmt.field("task_id", &self.task_id)
            .field("proof_type", &self.proof_type)
            .field("circuit_version", &self.circuit_version)
            .field("hard_fork_name", &self.hard_fork_name)
            .field("status", &self.status)
            .field("created_at", &self.created_at);
        if let Some(started_at) = &self.started_at {
            fmt.field("started_at", started_at);
        }
        if let Some(finished_at) = &self.finished_at {
            fmt.field("finished_at", finished_at);
        }
        if let Some(compute_time_sec) = &self.compute_time_sec {
            fmt.field("compute_time_sec", compute_time_sec);
        }
        if self.input.is_some() {
            fmt.field("input", &"..."); // Hide actual input for brevity
        }
        if self.proof.is_some() {
            fmt.field("proof", &"..."); // Hide actual proof for brevity
        }
        if let Some(vk) = &self.vk {
            fmt.field("vk", vk);
        }
        if let Some(error) = &self.error {
            fmt.field("error", error);
        }
        fmt.finish()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QueryTaskRequest {
    pub task_id: String,
}

#[derive(Default, Clone, PartialEq)]
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

impl fmt::Debug for QueryTaskResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmt = f.debug_struct("QueryTaskResponse");
        fmt.field("task_id", &self.task_id)
            .field("proof_type", &self.proof_type)
            .field("circuit_version", &self.circuit_version)
            .field("hard_fork_name", &self.hard_fork_name)
            .field("status", &self.status)
            .field("created_at", &self.created_at);
        if let Some(started_at) = &self.started_at {
            fmt.field("started_at", started_at);
        }
        if let Some(finished_at) = &self.finished_at {
            fmt.field("finished_at", finished_at);
        }
        if let Some(compute_time_sec) = &self.compute_time_sec {
            fmt.field("compute_time_sec", compute_time_sec);
        }
        if self.input.is_some() {
            fmt.field("input", &"..."); // Hide actual input for brevity
        }
        if self.proof.is_some() {
            fmt.field("proof", &"..."); // Hide actual proof for brevity
        }
        if let Some(vk) = &self.vk {
            fmt.field("vk", vk);
        }
        if let Some(error) = &self.error {
            fmt.field("error", error);
        }
        fmt.finish()
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    #[default]
    Queued,
    Proving,
    Success,
    Failed,
}

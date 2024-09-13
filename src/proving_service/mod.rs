trait ProvingServiceExtension {
    fn prove(&self, req: ProveRequest) -> ProveResponse;
    fn get_task(&self, req: GetTaskRequest) -> GetTaskResponse;
}

struct ProveRequest {
    circuit_version: String,
    input: String,
}

struct ProveResponse {
    task_id: String,
    circuit_version: String,
    status: TaskStatus,
    created_at: u64,
    started_at: Option<u64>,
    finished_at: Option<u64>,
    compute_time_sec: Option<u64>,
    input: Option<String>,
    proof: Option<String>,
    vk: Option<String>,
    error: Option<String>,
}

struct GetTaskRequest {
    task_id: String,
}

struct GetTaskResponse {
    task_id: String,
    circuit_version: String,
    status: TaskStatus,
    created_at: u64,
    started_at: Option<u64>,
    finished_at: Option<u64>,
    compute_time_sec: Option<u64>,
    input: Option<String>,
    proof: Option<String>,
    vk: Option<String>,
    error: Option<String>,
}

enum TaskStatus {
    Queued,
    Proving,
    Success,
    Failed,
}
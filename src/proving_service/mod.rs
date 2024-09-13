mod types;
pub use types::*;

trait ProvingServiceExtension {
    fn prove(&self, req: ProveRequest) -> ProveResponse;
    fn get_task(&self, req: GetTaskRequest) -> GetTaskResponse;
}

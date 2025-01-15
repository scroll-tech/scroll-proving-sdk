use crate::coordinator_handler::GetTaskResponseData;
use rocksdb::DB;

pub struct Db {
    db: DB,
}

impl Db {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let db = DB::open_default(path)?;
        Ok(Self { db })
    }

    pub fn get_task(&self, public_key: String) -> (Option<GetTaskResponseData>, Option<String>) {
        (
            self.get_coordinator_task_by_public_key(public_key.clone()),
            self.get_proving_task_id_by_public_key(public_key),
        )
    }

    pub fn set_task(
        &self,
        public_key: String,
        coordinator_task: &GetTaskResponseData,
        proving_task_id: String,
    ) {
        self.set_coordinator_task_by_public_key(public_key.clone(), coordinator_task);
        self.set_proving_task_id_by_public_key(public_key, proving_task_id);
    }

    pub fn delete_task(&self, public_key: String) {
        self.delete_coordinator_task_by_public_key(public_key.clone());
        self.delete_proving_task_id_by_public_key(public_key);
    }

    fn get_coordinator_task_by_public_key(
        &self,
        public_key: String,
    ) -> Option<GetTaskResponseData> {
        self.db
            .get(fmt_coordinator_task_key(public_key))
            .ok()?
            .as_ref()
            .and_then(|v| serde_json::from_slice(v).ok())
    }

    fn get_proving_task_id_by_public_key(&self, public_key: String) -> Option<String> {
        self.db
            .get(fmt_proving_task_id_key(public_key))
            .ok()?
            .and_then(|v| String::from_utf8(v).ok())
    }

    fn set_coordinator_task_by_public_key(
        &self,
        public_key: String,
        coordinator_task: &GetTaskResponseData,
    ) {
        let _ = serde_json::to_vec(coordinator_task)
            .map(|bytes| self.db.put(fmt_coordinator_task_key(public_key), bytes));
    }

    fn set_proving_task_id_by_public_key(&self, public_key: String, proving_task_id: String) {
        let _ = self.db.put(
            fmt_proving_task_id_key(public_key),
            proving_task_id.as_bytes(),
        );
    }

    fn delete_coordinator_task_by_public_key(&self, public_key: String) {
        let _ = self.db.delete(fmt_coordinator_task_key(public_key));
    }

    fn delete_proving_task_id_by_public_key(&self, public_key: String) {
        let _ = self.db.delete(fmt_proving_task_id_key(public_key));
    }
}

fn fmt_coordinator_task_key(public_key: String) -> String {
    format!("last_coordinator_task_{}", public_key)
}

fn fmt_proving_task_id_key(public_key: String) -> String {
    format!("last_proving_task_id_{}", public_key)
}

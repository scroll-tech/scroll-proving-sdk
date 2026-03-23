use crate::coordinator_handler::GetTaskResponse;
use rocksdb::{DB, WriteBatch};
use std::path::Path;
use tracing::Level;

pub struct Db {
    db: DB,
}

impl Db {
    pub fn new(path: impl AsRef<Path>) -> eyre::Result<Self> {
        tracing::info!(
            "Apply locol storage at {}",
            path.as_ref().to_str().unwrap_or("WRONG PATH")
        );
        let db = DB::open_default(path)?;
        Ok(Self { db })
    }

    #[instrument(skip(self), level = Level::DEBUG)]
    pub fn get_task(&self, public_key: &str) -> Option<(GetTaskResponse, String)> {
        self.get_task_inner(public_key)
            .inspect_err(|e| {
                tracing::error!("Failed to get task for public_key {public_key}: {e:?}");
            })
            .unwrap_or_default()
    }

    fn get_task_inner(&self, public_key: &str) -> eyre::Result<Option<(GetTaskResponse, String)>> {
        let coordinator_task_key = fmt_coordinator_task_key(public_key);
        let proving_task_id_key = fmt_proving_task_id_key(public_key);

        let mut results = self
            .db
            .multi_get([coordinator_task_key, proving_task_id_key])
            .into_iter();

        let Some(coordinator_task_bytes) = results.next().unwrap()? else {
            return Ok(None);
        };
        let Some(proving_task_id_bytes) = results.next().unwrap()? else {
            return Ok(None);
        };

        let coordinator_task: GetTaskResponse = serde_json::from_slice(&coordinator_task_bytes)?;
        let proving_task_id = String::from_utf8(proving_task_id_bytes)?;

        Ok(Some((coordinator_task, proving_task_id)))
    }

    #[instrument(skip_all, fields(public_key = %public_key), level = Level::DEBUG)]
    pub fn set_task(
        &self,
        public_key: &str,
        coordinator_task: &GetTaskResponse,
        proving_task_id: &str,
    ) {
        if let Err(e) = self.set_task_inner(public_key, coordinator_task, proving_task_id) {
            tracing::error!("Failed to set task for public_key {public_key}: {e:?}");
        }
    }

    fn set_task_inner(
        &self,
        public_key: &str,
        coordinator_task: &GetTaskResponse,
        proving_task_id: &str,
    ) -> eyre::Result<()> {
        let task = serde_json::to_vec(coordinator_task)?;

        let mut write_batch = WriteBatch::default();
        // set_coordinator_task_by_public_key
        write_batch.put(fmt_coordinator_task_key(public_key), task);
        // set_proving_task_id_by_public_key
        write_batch.put(
            fmt_proving_task_id_key(public_key),
            proving_task_id.as_bytes(),
        );
        self.db.write(write_batch)?;

        Ok(())
    }

    pub fn delete_task(&self, public_key: &str) {
        if let Err(e) = self.delete_task_inner(public_key) {
            tracing::error!("Failed to delete task for public_key {public_key}: {e:?}");
        }
    }

    fn delete_task_inner(&self, public_key: &str) -> eyre::Result<()> {
        let mut write_batch = WriteBatch::default();

        // delete_coordinator_task_by_public_key
        write_batch.delete(fmt_coordinator_task_key(public_key));
        // delete_proving_task_id_by_public_key
        write_batch.delete(fmt_proving_task_id_key(public_key));

        self.db.write(write_batch)?;
        Ok(())
    }

    /// Expose the inner RocksDB instance for advanced operations.
    pub fn inner(&self) -> &DB {
        &self.db
    }
}

/// Format keys for storing and retrieving tasks in the database.
pub static COORDINATOR_TASK_KEY_PREFIX: &str = "last_coordinator_task_";

/// Format keys for storing and retrieving proving task IDs in the database.
pub static PROVING_TASK_ID_KEY_PREFIX: &str = "last_proving_task_id_";

#[inline]
pub fn fmt_coordinator_task_key(public_key: &str) -> String {
    format!("{COORDINATOR_TASK_KEY_PREFIX}{public_key}")
}

#[inline]
pub fn fmt_proving_task_id_key(public_key: &str) -> String {
    format!("{PROVING_TASK_ID_KEY_PREFIX}{public_key}")
}

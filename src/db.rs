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

    pub fn get_coordinator_task_by_public_key(
        &self,
        public_key: String,
    ) -> Option<GetTaskResponseData> {
        todo!()
        // self.db.get(fmt_coordinator_task_key(public_key))
    }

    pub fn get_proving_task_id_by_public_key(&self, public_key: String) -> Option<String> {
        match self.db.get(fmt_proving_task_id_key(public_key)) {
            Ok(value) => match value {
                Some(v) => match String::from_utf8(v) {
                    Ok(task_id) => Some(task_id),
                    Err(e) => None,
                },
                None => None,
            },
            Err(e) => None,
        }
    }
}

fn fmt_coordinator_task_key(public_key: String) -> String {
    todo!()
}

fn fmt_proving_task_id_key(public_key: String) -> String {
    todo!()
}

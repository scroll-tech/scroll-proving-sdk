use rocksdb::DB;

pub struct Db {
    db: DB,
}


impl Db {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let db = DB::open_default(path)?;
        Ok(Self { db })
    }

    pub fn get(&self, key: &[u8]) -> Option<String> {
        self.db.get(key).unwrap()
    }
}

fn fmt_coordinator_task_key() -> String {
}

fn fmt_proving_task_id_key() -> String {
}
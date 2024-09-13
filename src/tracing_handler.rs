use crate::config::L2GethConfig;

pub struct L2gethClient {
    pub endpoint: String,
}

impl L2gethClient {
    pub fn new(cfg: L2GethConfig) -> Self {
        Self {
            endpoint: cfg.endpoint,
        }
    }
}

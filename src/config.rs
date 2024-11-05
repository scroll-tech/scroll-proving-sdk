use crate::prover::CircuitType;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub prover_name_prefix: String,
    pub keys_dir: String,
    pub coordinator: CoordinatorConfig,
    pub l2geth: Option<L2GethConfig>,
    pub prover: ProverConfig,
    #[serde(default = "default_health_listener_addr")]
    pub health_listener_addr: String,
}

fn default_health_listener_addr() -> String {
    "0.0.0.0:80".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoordinatorConfig {
    pub base_url: String,
    pub retry_count: u32,
    pub retry_wait_time_sec: u64,
    pub connection_timeout_sec: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct L2GethConfig {
    pub endpoint: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProverConfig {
    pub circuit_type: CircuitType,
    pub circuit_version: String,
    pub n_workers: usize,
    pub cloud: Option<CloudProverConfig>,
    pub local: Option<LocalProverConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudProverConfig {
    pub base_url: String,
    pub api_key: String,
    pub retry_count: u32,
    pub retry_wait_time_sec: u64,
    pub connection_timeout_sec: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalProverConfig {
    // TODO:
    // params path
    // assets path
    // DB config
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbConfig {}

impl Config {
    pub fn from_reader<R>(reader: R) -> anyhow::Result<Self>
    where
        R: std::io::Read,
    {
        serde_json::from_reader(reader).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn from_file(file_name: String) -> anyhow::Result<Self> {
        let file = File::open(file_name)?;
        Config::from_reader(&file)
    }

    pub fn from_file_and_env(file_name: String) -> anyhow::Result<Self> {
        let mut cfg = Config::from_file(file_name)?;
        cfg.override_with_env()?;
        Ok(cfg)
    }

    fn override_with_env(&mut self) -> anyhow::Result<()> {
        dotenv().ok();

        // read circuit_type from env if set
        if let Some(circuit_type) = std::env::var_os("CIRCUIT_TYPE") {
            let circuit_type = circuit_type
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("CIRCUIT_TYPE env var is not valid UTF-8"))?
                .parse::<u8>()?;
            self.prover.circuit_type = CircuitType::from_u8(circuit_type);
        }

        // TODO: PROVER_NAME_PREFIX, KEYS_DIR, COORDINATOR_BASE_URL, L2GETH_ENDPOINT, PROVING_SERVICE_BASE_URL, PROVING_SERVICE_API_KEY

        Ok(())
    }
}

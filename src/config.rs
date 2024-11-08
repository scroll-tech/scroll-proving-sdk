use crate::prover::CircuitType;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub prover_name_prefix: String,
    pub keys_dir: String,
    pub db_path: String,
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

    fn get_env_var(key: &str) -> anyhow::Result<Option<String>> {
        Ok(std::env::var_os(key).map(|val| {
            val.to_str()
                .ok_or_else(|| anyhow::anyhow!("{key} env var is not valid UTF-8"))
                .map(String::from)
        }).transpose()?)
    }

    fn override_with_env(&mut self) -> anyhow::Result<()> {
        dotenv().ok();

        if let Some(val) = Self::get_env_var("PROVER_NAME_PREFIX")? {
            self.prover_name_prefix = val;
        }
        if let Some(val) = Self::get_env_var("KEYS_DIR")? {
            self.keys_dir = val;
        }
        if let Some(val) = Self::get_env_var("COORDINATOR_BASE_URL")? {
            self.coordinator.base_url = val;
        }
        if let Some(val) = Self::get_env_var("L2GETH_ENDPOINT")? {
            if let Some(l2geth) = &mut self.l2geth {
                l2geth.endpoint = val;
            }
        }
        if let Some(val) = Self::get_env_var("CIRCUIT_TYPE")? {
            self.prover.circuit_type = CircuitType::from_u8(val.parse()?);
        }
        if let Some(val) = Self::get_env_var("PROVING_SERVICE_BASE_URL")? {
            if let Some(cloud) = &mut self.prover.cloud {
                cloud.base_url = val;
            }
        }
        if let Some(val) = Self::get_env_var("PROVING_SERVICE_API_KEY")? {
            if let Some(cloud) = &mut self.prover.cloud {
                cloud.api_key = val;
            }
        }
        if let Some(val) = Self::get_env_var("DB_PATH")? {
            self.db_path = val;
        }

        Ok(())
    }
}

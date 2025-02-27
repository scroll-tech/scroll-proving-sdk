use crate::prover::CircuitType;
use anyhow::{anyhow, Result};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub prover_name_prefix: String,
    pub keys_dir: String,
    pub db_path: Option<String>,
    pub coordinator: CoordinatorConfig,
    pub l2geth: Option<L2GethConfig>,
    pub prover: ProverConfig,
    #[serde(default = "default_health_listener_addr")]
    pub health_listener_addr: String,
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
    pub circuit_types: Vec<CircuitType>,
    pub circuit_version: String,
    #[serde(default = "default_n_workers")]
    pub n_workers: usize,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbConfig {}

fn default_health_listener_addr() -> String {
    "0.0.0.0:80".to_string()
}

fn default_n_workers() -> usize {
    1
}

impl Config {
    pub fn from_reader<R>(reader: R) -> Result<Self>
    where
        R: std::io::Read,
    {
        serde_json::from_reader(reader).map_err(|e| anyhow!(e))
    }

    pub fn from_file(file_name: String) -> Result<Self> {
        let file = File::open(file_name)?;
        Config::from_reader(&file)
    }

    pub fn from_file_and_env(file_name: String) -> Result<Self> {
        let mut cfg = Config::from_file(file_name)?;
        cfg.override_with_env()?;
        Ok(cfg)
    }

    fn get_env_var(key: &str) -> Result<Option<String>> {
        std::env::var_os(key)
            .map(|val| {
                val.to_str()
                    .ok_or_else(|| anyhow!("{key} env var is not valid UTF-8"))
                    .map(String::from)
            })
            .transpose()
    }

    pub fn override_with_env(&mut self) -> Result<()> {
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
        if let Some(val) = Self::get_env_var("CIRCUIT_TYPES")? {
            let values_vec: Vec<&str> = val
                .trim_matches(|c| c == '[' || c == ']')
                .split(',')
                .map(|s| s.trim())
                .collect();

            self.prover.circuit_types = values_vec
                .iter()
                .map(|value| match value.parse::<u8>() {
                    Ok(num) => CircuitType::from_u8(num),
                    Err(e) => {
                        panic!("Failed to parse circuit type: {}", e);
                    }
                })
                .collect::<Vec<CircuitType>>();
        }

        if let Some(val) = Self::get_env_var("N_WORKERS")? {
            self.prover.n_workers = val.parse()?;
        }

        if let Some(val) = Self::get_env_var("DB_PATH")? {
            self.db_path = Option::from(val);
        }

        Ok(())
    }
}

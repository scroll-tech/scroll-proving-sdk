use crate::{coordinator_handler::ProverType, prover::ProofType};
use dotenvy::dotenv;
use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub prover_name_prefix: String,
    pub keys_dir: String,
    pub db_path: Option<String>,
    pub coordinator: CoordinatorConfig,
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
pub struct ProverConfig {
    pub supported_proof_types: Vec<ProofType>,
    pub circuit_version: String,
    #[serde(default = "default_n_workers")]
    pub n_workers: usize,
    /// Interval between polling the coordinator for new tasks.
    #[serde(default = "default_poll_interval_sec")]
    pub poll_interval_sec: u64,
    /// Delay the timer by a randomly selected, evenly distributed amount of time between 0 and the
    /// specified time value. Defaults to 0, indicating that no randomized delay shall be applied.
    #[serde(default)]
    pub randomized_delay_sec: u64,
    #[serde(default)]
    pub suppress_empty_task_error: bool,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbConfig {}

fn default_health_listener_addr() -> String {
    "0.0.0.0:80".to_string()
}

const fn default_n_workers() -> usize {
    1
}

const fn default_poll_interval_sec() -> u64 {
    20
}

impl Config {
    pub fn from_reader<R>(reader: R) -> Result<Self>
    where
        R: std::io::Read,
    {
        serde_json::from_reader(reader).map_err(|e| eyre!(e))
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
                    .ok_or_else(|| eyre!("{key} env var is not valid UTF-8"))
                    .map(String::from)
            })
            .transpose()
    }

    fn override_with_env(&mut self) -> Result<()> {
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

        if let Some(val) = Self::get_env_var("PROOF_TYPES")? {
            let values_vec: Vec<&str> = val
                .trim_matches(|c| c == '[' || c == ']')
                .split(',')
                .map(|s| s.trim())
                .collect();

            self.prover.supported_proof_types = values_vec
                .iter()
                .map(|value| match value.parse::<u8>() {
                    Ok(num) => ProofType::from_u8(num),
                    Err(e) => {
                        panic!("Failed to parse circuit type: {}", e);
                    }
                })
                .collect::<Vec<ProofType>>();
        }

        if let Some(val) = Self::get_env_var("N_WORKERS")? {
            self.prover.n_workers = val.parse()?;
        }

        if let Some(val) = Self::get_env_var("DB_PATH")? {
            self.db_path = Option::from(val);
        }

        if let Some(val) = Self::get_env_var("POLL_INTERVAL_SEC")? {
            self.prover.poll_interval_sec = val.parse()?;
        }
        if let Some(val) = Self::get_env_var("RANDOMIZED_DELAY_SEC")? {
            self.prover.randomized_delay_sec = val.parse()?;
        }

        if Self::get_env_var("SUPPRESS_EMPTY_TASK_ERR")?.is_some() {
            self.prover.suppress_empty_task_error = true;
        }

        Ok(())
    }

    pub fn coordinator_prover_type(&self) -> Vec<ProverType> {
        vec![ProverType::OpenVM]
    }
}

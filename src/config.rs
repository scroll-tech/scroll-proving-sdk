use crate::prover::CircuitType;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub db: DbConfig,
    pub coordinator: CoordinatorConfig,
    pub l2geth: Option<L2GethConfig>,
    pub prover: ProverConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbConfig {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoordinatorConfig {
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct L2GethConfig {
    pub endpoint: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProverConfig {
    pub circuit_type: CircuitType,
    pub n_workers: usize,
    pub cloud: Option<CloudProverConfig>,
    pub local: Option<LocalProverConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudProverConfig {
    pub endpoint: String,
    pub api_key: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalProverConfig {
    // TODO:
    // params path
    // assets path
}

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
}

use std::path;

use crate::proving_service::CircuitType;

#[derive(Clone)]
pub struct Config {
    pub db: DbConfig,
    pub coordinator: CoordinatorConfig,
    pub l2geth: Option<L2GethConfig>,
    pub proving_service: ProvingServiceConfig,
}

#[derive(Clone)]
pub struct DbConfig {}

#[derive(Clone)]
pub struct CoordinatorConfig {
    pub base_url: String,
}

#[derive(Clone)]
pub struct L2GethConfig {
    pub endpoint: String,
}

#[derive(Clone)]
pub struct ProvingServiceConfig {
    pub circuit_type: CircuitType,
    pub n_workers: usize,
    pub snarkify: Option<SnarkifyConfig>,
    pub sindri: Option<SindriConfig>,
    pub local: Option<LocalPoverConfig>,
}

#[derive(Clone)]
pub struct SnarkifyConfig {
    pub endpoint: String,
    pub api_key: String,
}

#[derive(Clone)]
pub struct SindriConfig {
    pub endpoint: String,
    pub api_key: String,
}

#[derive(Clone)]
pub struct LocalPoverConfig {
    // TODO:
    // params path
    // assets path
}

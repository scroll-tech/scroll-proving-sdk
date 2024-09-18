use crate::prover::CircuitType;

#[derive(Clone)]
pub struct Config {
    pub db: DbConfig,
    pub coordinator: CoordinatorConfig,
    pub l2geth: Option<L2GethConfig>,
    pub prover: ProverConfig,
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
pub struct ProverConfig {
    pub circuit_type: CircuitType,
    pub n_workers: usize,
    pub cloud: Option<CloudPoverConfig>,
    pub local: Option<LocalPoverConfig>,
}

#[derive(Clone)]
pub struct CloudPoverConfig {
    pub endpoint: String,
    pub api_key: String,
}

#[derive(Clone)]
pub struct LocalPoverConfig {
    // TODO:
    // params path
    // assets path
}

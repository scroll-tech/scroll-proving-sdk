pub struct Config {
    pub db: DbConfig,
    pub coordinator: CoordinatorConfig,
    pub l2geth: Option<L2GethConfig>,
    pub proving_service: ProvingServiceConfig,
}

pub struct DbConfig {}

pub struct CoordinatorConfig {
    pub base_url: String,
}

pub struct L2GethConfig {
    pub endpoint: String,
}

pub struct ProvingServiceConfig {
    pub snarkify: Option<SnarkifyConfig>,
    pub sindri: Option<SindriConfig>,
    pub local: Option<LocalPoverConfig>,
}

pub struct SnarkifyConfig {
    pub endpoint: String,
    pub api_key: String,
}

pub struct SindriConfig {
    pub endpoint: String,
    pub api_key: String,
}

pub struct LocalPoverConfig {}

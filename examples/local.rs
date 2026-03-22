use async_trait::async_trait;
use clap::Parser;
use eyre::{Result, eyre};
use scroll_proving_sdk::{
    config::Config as SdkConfig,
    prover::{
        ProverBuilder, ProvingService,
        proving_service::{
            GetVkRequest, GetVkResponse, ProveRequest, ProveResponse, QueryTaskRequest,
            QueryTaskResponse,
        },
    },
    utils::init_tracing,
};
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Parser, Debug)]
#[clap(disable_version_flag = true)]
struct Args {
    /// Path of config file
    #[arg(long = "config", default_value = "config.json")]
    config_file: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocalProverConfig {
    pub sdk_config: SdkConfig,
    pub conf1: String,
    pub conf2: String,
}

impl LocalProverConfig {
    pub fn from_reader<R>(reader: R) -> Result<Self>
    where
        R: std::io::Read,
    {
        serde_json::from_reader(reader).map_err(|e| eyre!(e))
    }

    pub fn from_file(file_name: String) -> Result<Self> {
        let file = File::open(file_name)?;
        Self::from_reader(&file)
    }

    pub fn from_file_and_env(file_name: String) -> Result<Self> {
        let cfg = Self::from_file(file_name)?;
        Ok(cfg)
    }
}

struct LocalProver {}

#[async_trait]
impl ProvingService for LocalProver {
    fn is_local(&self) -> bool {
        true
    }
    async fn get_vks(&self, _req: GetVkRequest) -> GetVkResponse {
        todo!()
    }
    async fn prove(&mut self, _req: ProveRequest) -> ProveResponse {
        todo!()
    }
    async fn query_task(&mut self, _req: QueryTaskRequest) -> QueryTaskResponse {
        todo!()
    }
}

impl LocalProver {
    pub fn new(_cfg: LocalProverConfig) -> Self {
        Self {}
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    init_tracing();

    let args = Args::parse();
    let cfg = LocalProverConfig::from_file_and_env(args.config_file)?;
    let sdk_config = cfg.sdk_config.clone();
    let local_prover = LocalProver::new(cfg);
    let prover = ProverBuilder::new(sdk_config, local_prover).build().await?;

    prover.run().await;

    Ok(())
}

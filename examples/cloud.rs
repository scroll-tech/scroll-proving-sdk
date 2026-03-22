#![allow(dead_code)]
use async_trait::async_trait;
use clap::Parser;
use eyre::{Result, eyre};
use reqwest::Url;
use std::fs::File;

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

#[derive(Parser, Debug)]
#[clap(disable_version_flag = true)]
struct Args {
    /// Path of config file
    #[arg(long = "config", default_value = "config.json")]
    config_file: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudProverConfig {
    pub sdk_config: SdkConfig,
    pub base_url: String,
    pub api_key: String,
}

impl CloudProverConfig {
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

    fn get_env_var(key: &str) -> Result<Option<String>> {
        std::env::var_os(key)
            .map(|val| {
                val.to_str()
                    .ok_or_else(|| eyre!("{key} env var is not valid UTF-8"))
                    .map(String::from)
            })
            .transpose()
    }

    pub fn from_file_and_env(file_name: String) -> Result<Self> {
        let mut cfg = Self::from_file(file_name)?;

        if let Some(val) = Self::get_env_var("PROVING_SERVICE_BASE_URL")? {
            cfg.base_url = val;
        }

        if let Some(val) = Self::get_env_var("PROVING_SERVICE_API_KEY")? {
            cfg.api_key = val;
        }

        Ok(cfg)
    }
}

struct CloudProver {
    base_url: Url,
    api_key: String,
}

#[async_trait]
impl ProvingService for CloudProver {
    fn is_local(&self) -> bool {
        false
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

impl CloudProver {
    pub fn new(cfg: CloudProverConfig) -> Self {
        let base_url = Url::parse(&cfg.base_url).expect("cannot parse cloud prover base_url");
        Self {
            base_url,
            api_key: cfg.api_key,
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    init_tracing();

    let args = Args::parse();
    let cfg = CloudProverConfig::from_file_and_env(args.config_file)?;
    let sdk_config = cfg.sdk_config.clone();
    let cloud_prover = CloudProver::new(cfg);
    let prover = ProverBuilder::new(sdk_config, cloud_prover).build().await?;

    prover.run().await;

    Ok(())
}

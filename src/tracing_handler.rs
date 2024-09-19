use crate::config::L2GethConfig;
use anyhow::Result;
use ethers_core::types::BlockNumber;
use ethers_core::types::H256;
use ethers_providers::{Http, Provider};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

pub type CommonHash = H256;

pub struct L2gethClient {
    // id: String, // TODO: prover_name
    provider: Provider<Http>,
}

impl L2gethClient {
    pub fn new(cfg: L2GethConfig) -> anyhow::Result<Self> {
        let provider = Provider::<Http>::try_from(cfg.endpoint)?;

        Ok(Self { provider })
    }

    pub async fn get_block_trace_by_hash<T>(&self, hash: &CommonHash) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Debug + Send,
    {
        log::info!(
            "l2geth_client calling get_block_trace_by_hash, hash: {:#?}",
            hash
        );

        let trace = self
            .provider
            .request("scroll_getBlockTraceByNumberOrHash", [format!("{hash:#x}")])
            .await?;
        Ok(trace)
    }

    pub async fn block_number(&self) -> Result<BlockNumber> {
        log::info!("l2geth_client calling block_number");

        let trace = self.provider.request("eth_blockNumber", ()).await?;
        Ok(trace)
    }
}

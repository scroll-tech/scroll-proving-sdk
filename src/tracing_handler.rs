use crate::config::L2GethConfig;
use ethers_core::types::BlockNumber;
use ethers_core::types::H256;
use ethers_providers::{Http, Provider};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use tokio::runtime::Runtime;

pub type CommonHash = H256;

pub struct L2gethClient {
    provider: Provider<Http>,
    rt: Runtime,
}

impl L2gethClient {
    pub fn new(cfg: L2GethConfig) -> anyhow::Result<Self> {
        let provider = Provider::<Http>::try_from(cfg.endpoint)?;
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        Ok(Self { provider, rt })
    }

    async fn get_block_trace_by_hash_async<T>(&self, hash: &CommonHash) -> anyhow::Result<T>
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

    pub fn get_block_trace_by_hash_sync<T>(&self, hash: &CommonHash) -> anyhow::Result<T>
    where
        T: Serialize + DeserializeOwned + Debug + Send,
    {
        self.rt.block_on(self.get_block_trace_by_hash_async(hash))
    }

    async fn block_number_async(&self) -> anyhow::Result<BlockNumber> {
        log::info!("l2geth_client calling block_number");

        let trace = self.provider.request("eth_blockNumber", ()).await?;
        Ok(trace)
    }

    pub fn block_number_sync(&self) -> anyhow::Result<BlockNumber> {
        self.rt.block_on(self.block_number_async())
    }
}

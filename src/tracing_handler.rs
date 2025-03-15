use crate::config::L2GethConfig;
use ethers_core::types::BlockNumber;
use ethers_core::types::H256;
use ethers_providers::{Http, Provider};
pub type CommonHash = H256;

pub struct L2gethClient {
    pub provider: Provider<Http>,
}

impl L2gethClient {
    pub fn new(cfg: L2GethConfig) -> anyhow::Result<Self> {
        let provider = Provider::<Http>::try_from(cfg.endpoint)?;
        Ok(Self { provider })
    }

    pub async fn get_block_trace_by_hash(&self, hash: &CommonHash) -> anyhow::Result<String> {
        log::info!(
            "l2geth_client calling get_block_trace_by_hash, hash: {:#?}",
            hash
        );

        let trace: serde_json::Value = self
            .provider
            .request("scroll_getBlockTraceByNumberOrHash", [format!("{hash:#x}")])
            .await?;

        let trace = serde_json::to_string(&trace)?;
        Ok(trace)
    }

    pub async fn block_number(&self) -> anyhow::Result<BlockNumber> {
        log::info!("l2geth_client calling block_number");

        let block_number = self.provider.request("eth_blockNumber", ()).await?;
        log::info!("l2geth_client received block_number: {:#?}", block_number);

        Ok(block_number)
    }

    pub async fn get_traces_by_hashes(
        &self,
        block_hashes: &[CommonHash],
    ) -> anyhow::Result<Vec<String>> {
        if block_hashes.is_empty() {
            log::error!("failed to get sorted traces: block_hashes are empty");
            anyhow::bail!("block_hashes are empty")
        }

        let mut block_traces = Vec::new();
        for hash in block_hashes.iter() {
            let trace = self.get_block_trace_by_hash(hash).await?;
            block_traces.push(trace);
        }

        Ok(block_traces)
    }
}

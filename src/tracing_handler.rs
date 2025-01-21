use crate::config::L2GethConfig;
use ethers_core::types::BlockNumber;
use ethers_core::types::H256;
use ethers_providers::{Http, Provider};
use prover_darwin_v2::BlockTrace;
use std::cmp::Ordering;

pub type CommonHash = H256;

pub struct L2gethClient {
    provider: Provider<Http>,
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

        let trace = self
            .provider
            .request("scroll_getBlockTraceByNumberOrHash", [format!("{hash:#x}")])
            .await?;
        Ok(trace)
    }

    pub async fn block_number(&self) -> anyhow::Result<BlockNumber> {
        log::info!("l2geth_client calling block_number");

        let trace = self.provider.request("eth_blockNumber", ()).await?;
        Ok(trace)
    }

    pub async fn get_sorted_traces_by_hashes(
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

        block_traces.sort_by(|a, b| {
            if get_block_number_from_trace(a).is_none() {
                Ordering::Less
            } else if get_block_number_from_trace(b).is_none() {
                Ordering::Greater
            } else {
                get_block_number_from_trace(a)
                    .unwrap()
                    .cmp(&get_block_number_from_trace(b).unwrap())
            }
        });

        let block_numbers: Vec<u64> = block_traces
            .iter()
            .map(|trace| get_block_number_from_trace(trace).unwrap_or(0))
            .collect();
        let mut i = 0;
        while i < block_numbers.len() - 1 {
            if block_numbers[i] + 1 != block_numbers[i + 1] {
                log::error!(
                    "block numbers are not continuous, got {} and {}",
                    block_numbers[i],
                    block_numbers[i + 1]
                );
                anyhow::bail!(
                    "block numbers are not continuous, got {} and {}",
                    block_numbers[i],
                    block_numbers[i + 1]
                )
            }
            i += 1;
        }

        Ok(block_traces)
    }
}

fn get_block_number_from_trace(block_trace: &String) -> Option<u64> {
    let block_trace: BlockTrace = serde_json::from_str(block_trace).unwrap();
    block_trace.header.number.map(|n| n.as_u64())
}

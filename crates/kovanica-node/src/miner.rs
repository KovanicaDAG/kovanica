//! Node mining worker and difficulty calculation for Chain 5 (Hybrid PoW testnet).

use kovanica_consensus::daa::{calculate_next_target_bits, DaaConfig};
use kovanica_consensus::mining::{mine_header_async, MiningWorker};
use kovanica_types::block::BlockHeader;

/// Node mining service wrapper.
#[allow(dead_code)]
pub struct NodeMiner {
    worker: MiningWorker,
    /// DAA configuration for this miner.
    pub daa_config: DaaConfig,
}

#[allow(dead_code)]
impl Default for NodeMiner {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl NodeMiner {
    /// Create a new node miner with default Chain 5 parameters.
    #[must_use]
    pub fn new() -> Self {
        Self {
            worker: MiningWorker::new(),
            daa_config: DaaConfig::chain5_default(),
        }
    }

    /// Calculate next target bits for a candidate block from ancestor timestamps.
    #[must_use]
    pub fn next_target_bits(&self, current_target_bits: u32, timestamps: &[u64]) -> u32 {
        calculate_next_target_bits(current_target_bits, timestamps, &self.daa_config)
    }

    /// Synchronously mine `header` up to `max_iterations`.
    pub fn mine(&mut self, header: BlockHeader, max_iterations: u64) -> Option<BlockHeader> {
        self.worker.mine_header(header, max_iterations)
    }

    /// Asynchronously mine `header` up to `max_iterations`.
    pub async fn mine_async(
        &self,
        header: BlockHeader,
        max_iterations: u64,
    ) -> Option<BlockHeader> {
        mine_header_async(header, max_iterations).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_types::hash::Hash;

    #[tokio::test]
    async fn node_miner_lifecycle() {
        let mut miner = NodeMiner::new();
        // Allow high target in test environment
        miner.daa_config.max_target_bits = 0x207fffff;
        miner.daa_config.target_block_time_ms = 4000;

        let target_bits = miner.next_target_bits(0x207fffff, &[1000, 5000, 9000]);
        assert_eq!(target_bits, 0x207fffff);

        let header = BlockHeader::new(5, 1, 0, vec![], Hash::zero(), 13000, target_bits, 0);
        let mined = miner
            .mine(header.clone(), 1_000)
            .expect("should mine block");
        assert!(mined.verify_pow().is_ok());

        let mined_async = miner
            .mine_async(header, 1_000)
            .await
            .expect("should mine block async");
        assert!(mined_async.verify_pow().is_ok());
    }
}

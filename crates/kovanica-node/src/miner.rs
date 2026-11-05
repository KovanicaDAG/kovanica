//! Node mining worker and difficulty calculation for Chain 5 (Hybrid PoW testnet).

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use kovanica_consensus::daa::{calculate_next_target_bits, DaaConfig};
use kovanica_consensus::ghostdag::GhostdagCommitter;
use kovanica_consensus::mining::{mine_header_async, MiningWorker};
use kovanica_dag::DagStore;
use kovanica_mempool::Mempool;
use kovanica_types::block::{BlockHeader, BlockReference, StatementBlock};
use kovanica_types::hash::Hash;
use kovanica_types::CommittedSubDag;
use tokio::sync::{mpsc, watch};
use tracing::{error, info, warn};

use crate::config::MiningConfig;
use crate::NetworkCommand;

/// Node mining service wrapper.
#[allow(dead_code)]
pub struct NodeMiner {
    worker: MiningWorker,
    /// DAA configuration for this miner.
    pub daa_config: DaaConfig,
}

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

/// Select parent hashes from recent DAG tips for block authoring on Chain 5.
#[must_use]
pub fn select_parents_from_tips(dag_store: &DagStore, max_parents: usize) -> Vec<Hash> {
    let max_round = match dag_store.max_dag_round() {
        Ok(Some(r)) => r,
        _ => return Vec::new(),
    };

    let mut parents = Vec::new();
    let mut round = max_round;
    loop {
        if let Ok(blocks) = dag_store.get_blocks_by_round(round) {
            for block in blocks {
                if !parents.contains(&block.digest) {
                    parents.push(block.digest);
                    if parents.len() >= max_parents {
                        return parents;
                    }
                }
            }
        }
        if round == 0 || parents.len() >= max_parents {
            break;
        }
        round = round.saturating_sub(1);
        if max_round.saturating_sub(round) > 2 {
            break;
        }
    }
    parents
}

/// Extract timestamps in ascending order from ancestor blocks for DAA.
#[must_use]
pub fn extract_ancestor_timestamps(
    dag_store: &DagStore,
    parent_hashes: &[Hash],
    chain_id: u64,
    limit: usize,
) -> Vec<u64> {
    let mut timestamps = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    for h in parent_hashes {
        if visited.insert(*h) {
            queue.push_back(*h);
        }
    }

    while let Some(h) = queue.pop_front() {
        if let Ok(block) = dag_store.get_block(&h) {
            if let Ok(header) = block.extract_pow_header(chain_id) {
                timestamps.push(header.timestamp);
            }
            if timestamps.len() >= limit {
                break;
            }
            for p in &block.parents {
                if visited.insert(p.digest) {
                    queue.push_back(p.digest);
                }
            }
        }
    }

    timestamps.sort_unstable();
    timestamps
}

/// Run the background PoW mining loop for Chain 5.
#[allow(clippy::too_many_arguments)]
pub async fn run_mining_loop(
    mining_config: MiningConfig,
    chain_id: u64,
    dag_store: Arc<DagStore>,
    mempool: Arc<Mempool>,
    network_cmd_tx: mpsc::UnboundedSender<NetworkCommand>,
    exec_tx: mpsc::UnboundedSender<CommittedSubDag>,
    ghostdag: Arc<parking_lot::Mutex<GhostdagCommitter>>,
    shutdown_rx: watch::Receiver<bool>,
) {
    if !mining_config.enabled {
        return;
    }

    info!(
        chain_id,
        threads = mining_config.threads,
        "Chain 5 PoW mining worker loop started"
    );

    let miner = NodeMiner::new();

    loop {
        if *shutdown_rx.borrow() {
            info!("PoW miner loop received shutdown signal; stopping");
            break;
        }

        let parents = select_parents_from_tips(&dag_store, kovanica_params::GHOSTDAG_K as usize);
        let round = if parents.is_empty() {
            0
        } else {
            let mut max_r = 0u64;
            for p in &parents {
                if let Ok(b) = dag_store.get_block(p) {
                    max_r = max_r.max(b.round);
                }
            }
            max_r.saturating_add(1)
        };

        let parent_refs: Vec<BlockReference> = parents
            .iter()
            .map(|h| {
                if let Ok(b) = dag_store.get_block(h) {
                    b.reference()
                } else {
                    BlockReference {
                        digest: *h,
                        round: round.saturating_sub(1),
                        author: 0,
                    }
                }
            })
            .collect();

        let target_bits = if parents.is_empty() {
            kovanica_params::MAX_TARGET_BITS
        } else {
            let timestamps = extract_ancestor_timestamps(
                &dag_store,
                &parents,
                chain_id,
                kovanica_params::DAA_WINDOW_BLOCKS as usize,
            );
            miner.next_target_bits(kovanica_params::MAX_TARGET_BITS, &timestamps)
        };

        let txs = mempool.get_next_transactions(kovanica_types::MAX_TXS_PER_BLOCK);
        let merkle_root = StatementBlock::compute_merkle_root(&txs);
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let header = BlockHeader::new(
            chain_id,
            round,
            0, // author
            parent_refs,
            merkle_root,
            now_ms,
            target_bits,
            0,
        );

        // Mine with bounded iterations so loop cooperates with tokio and shutdown
        const CHUNK_ITERATIONS: u64 = 50_000;
        if let Some(mined_header) = miner.mine_async(header, CHUNK_ITERATIONS).await {
            info!(
                round = mined_header.round,
                pow_hash = ?mined_header.pow_hash,
                nonce = mined_header.nonce,
                "successfully mined new PoW block on Chain 5"
            );

            let statement_block = mined_header.to_statement_block(txs);
            if let Err(e) = dag_store.put_block(&statement_block) {
                error!(error = %e, "failed to persist mined block in dag_store");
                continue;
            }

            let subdags = {
                let mut gd = ghostdag.lock();
                gd.add_block(statement_block.clone());
                gd.commit_uncommitted_selected_chain()
            };

            for subdag in subdags {
                let _ = exec_tx.send(subdag);
            }

            if let Err(e) = network_cmd_tx.send(NetworkCommand::BroadcastBlock(statement_block)) {
                warn!(error = %e, "failed to broadcast mined block to P2P network");
            }
        } else {
            // Yield briefly to let other tasks run
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_storage::Storage;
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

    #[test]
    fn test_select_parents_and_extract_timestamps() {
        let dir = tempfile::tempdir().unwrap();
        let storage = Storage::new(dir.path().join("dag.redb")).unwrap();
        let dag = DagStore::new(storage).unwrap();

        // Empty dag returns no parents
        let tips = select_parents_from_tips(&dag, 5);
        assert!(tips.is_empty());

        let mut header = BlockHeader::new(5, 1, 0, vec![], Hash::zero(), 10_000, 0x207fffff, 0);
        header.pow_hash = Some(header.compute_pow_hash());
        let block = header.to_statement_block(vec![]);
        let digest = block.digest;
        dag.put_block(&block).unwrap();

        let tips = select_parents_from_tips(&dag, 5);
        assert_eq!(tips, vec![digest]);

        let timestamps = extract_ancestor_timestamps(&dag, &tips, 5, 10);
        assert_eq!(timestamps, vec![10_000]);
    }
}

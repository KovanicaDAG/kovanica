//! GHOSTDAG committer with k=5 for Chain 5 (Hybrid PoW + GHOSTDAG).
//!
//! Implements GHOSTDAG block sequencing without stake weighting,
//! committee votes, or certificates (ADR 0003).

use crate::ghostdag_scoped::colour_mergeset;
use crate::non_blue::non_blue_refs;
use kovanica_types::block::{compact_to_target, BlockReference, StatementBlock};
use kovanica_types::hash::Hash;
use kovanica_types::CommittedSubDag;
use std::collections::{HashMap, HashSet, VecDeque};

/// Calculate scalar work estimate from compact target bits.
#[must_use]
pub fn calculate_work_from_target(target_bits: u32) -> u128 {
    let target = compact_to_target(target_bits);
    if target == [0u8; 32] {
        return 0;
    }
    let high = u128::from_be_bytes(target[0..16].try_into().unwrap());
    if high == 0 {
        u128::MAX
    } else {
        (u128::MAX / high).max(1)
    }
}

/// Whether `chain_id` is a hybrid PoW + GHOSTDAG network.
pub fn is_hybrid_chain(chain_id: u64) -> bool {
    kovanica_types::signing::chain_id::is_hybrid(chain_id)
}

/// GHOSTDAG committer for sequencing PoW blocks on hybrid chains.
#[derive(Debug, Clone)]
pub struct GhostdagCommitter {
    /// GHOSTDAG cluster parameter k (default 5 for Chain 5).
    pub k: usize,
    /// In-memory cache of blocks indexed by digest.
    pub blocks: HashMap<Hash, StatementBlock>,
    /// Selected parent for each block in the DAG.
    pub selected_parent: HashMap<Hash, Option<Hash>>,
    /// Blue score of each block.
    pub blue_score: HashMap<Hash, u64>,
    /// Cumulative PoW difficulty work of each block.
    pub cumulative_work: HashMap<Hash, u128>,
    /// Blue blocks in each block's mergeset, ordered topologically.
    pub blue_sets: HashMap<Hash, Vec<Hash>>,
    /// Red (non-blue) block references in each block's mergeset.
    pub red_sets: HashMap<Hash, Vec<BlockReference>>,
    /// All block digests that have already been committed into a [`CommittedSubDag`].
    pub committed_blocks: HashSet<Hash>,
    /// Selected tip of the DAG with highest cumulative work / blue score.
    pub selected_tip: Option<Hash>,
    /// Last committed selected parent on the main chain.
    pub last_committed_selected_parent: Option<Hash>,
}

impl Default for GhostdagCommitter {
    fn default() -> Self {
        Self::new(kovanica_params::GHOSTDAG_K as usize)
    }
}

impl GhostdagCommitter {
    /// Create a new GHOSTDAG committer with the specified k parameter (typically 5).
    #[must_use]
    pub fn new(k: usize) -> Self {
        Self {
            k,
            blocks: HashMap::new(),
            selected_parent: HashMap::new(),
            blue_score: HashMap::new(),
            cumulative_work: HashMap::new(),
            blue_sets: HashMap::new(),
            red_sets: HashMap::new(),
            committed_blocks: HashSet::new(),
            selected_tip: None,
            last_committed_selected_parent: None,
        }
    }

    /// Add a validated PoW block to the GHOSTDAG committer.
    ///
    /// Computes selected parent, mergeset, scoped GHOSTDAG colouring, blue score,
    /// and updates the selected tip if this block establishes a new heaviest path.
    pub fn add_block(&mut self, block: StatementBlock) {
        let digest = block.digest;
        if self.blocks.contains_key(&digest) {
            return;
        }

        let work = if let Some((_, target_bits, _, _)) =
            kovanica_types::decode_pow_statement(&block.statements)
        {
            calculate_work_from_target(target_bits)
        } else if let Ok(header) =
            block.extract_pow_header(kovanica_params::CHAIN_ID_HYBRID_TESTNET)
        {
            calculate_work_from_target(header.target_bits)
        } else {
            1u128
        };

        // Determine selected parent among known parents
        let valid_parents: Vec<Hash> = block
            .parents
            .iter()
            .map(|p| p.digest)
            .filter(|h| self.blocks.contains_key(h))
            .collect();

        let sel_parent = valid_parents
            .iter()
            .max_by(|a, b| {
                let work_a = self.cumulative_work.get(*a).copied().unwrap_or(0);
                let work_b = self.cumulative_work.get(*b).copied().unwrap_or(0);
                let score_a = self.blue_score.get(*a).copied().unwrap_or(0);
                let score_b = self.blue_score.get(*b).copied().unwrap_or(0);
                (work_a, score_a, a.0).cmp(&(work_b, score_b, b.0))
            })
            .copied();

        let parent_work = sel_parent
            .and_then(|p| self.cumulative_work.get(&p).copied())
            .unwrap_or(0);
        let parent_score = sel_parent
            .and_then(|p| self.blue_score.get(&p).copied())
            .unwrap_or(0);

        let cum_work = parent_work.saturating_add(work);

        // Compute mergeset: past cone of block minus past cone of selected parent
        let mergeset_hashes = self.compute_mergeset_hashes(&valid_parents, sel_parent);
        let mergeset_blocks: Vec<StatementBlock> = mergeset_hashes
            .iter()
            .filter_map(|h| self.blocks.get(h).cloned())
            .collect();

        // Run GHOSTDAG colouring on mergeset with k=5
        let previous_tips: Vec<Hash> = sel_parent.into_iter().collect();
        let colouring = colour_mergeset(&mergeset_blocks, self.k, &previous_tips);

        let blue_ordered = colouring.blue_ordered();
        let red_refs = non_blue_refs(&mergeset_blocks, &colouring.blue, &digest);

        let score = parent_score
            .saturating_add(colouring.blue.len() as u64)
            .saturating_add(1);

        self.blocks.insert(digest, block);
        self.selected_parent.insert(digest, sel_parent);
        self.blue_score.insert(digest, score);
        self.cumulative_work.insert(digest, cum_work);
        self.blue_sets.insert(digest, blue_ordered);
        self.red_sets.insert(digest, red_refs);

        // Update selected tip if this block has higher cumulative work / blue score
        let is_new_tip = match self.selected_tip {
            None => true,
            Some(current_tip) => {
                let cur_work = self.cumulative_work.get(&current_tip).copied().unwrap_or(0);
                let cur_score = self.blue_score.get(&current_tip).copied().unwrap_or(0);
                (cum_work, score, digest.0) > (cur_work, cur_score, current_tip.0)
            }
        };

        if is_new_tip {
            self.selected_tip = Some(digest);
        }

        crate::metrics::set_ghostdag_tips(self.tips().len());
    }

    /// Compute past cone ancestor hashes of `block_parents` excluding past cone of `sel_parent`.
    fn compute_mergeset_hashes(
        &self,
        block_parents: &[Hash],
        sel_parent: Option<Hash>,
    ) -> Vec<Hash> {
        let mut past_of_selected: HashSet<Hash> = HashSet::new();
        if let Some(sp) = sel_parent {
            past_of_selected.insert(sp);
            let mut queue = VecDeque::new();
            queue.push_back(sp);
            while let Some(curr) = queue.pop_front() {
                if let Some(blk) = self.blocks.get(&curr) {
                    for p in &blk.parents {
                        if past_of_selected.insert(p.digest) {
                            queue.push_back(p.digest);
                        }
                    }
                }
            }
        }

        let mut mergeset = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        for p in block_parents {
            if Some(*p) != sel_parent && !past_of_selected.contains(p) && visited.insert(*p) {
                queue.push_back(*p);
            }
        }

        while let Some(curr) = queue.pop_front() {
            if past_of_selected.contains(&curr) {
                continue;
            }
            mergeset.push(curr);
            if let Some(blk) = self.blocks.get(&curr) {
                for p in &blk.parents {
                    if !past_of_selected.contains(&p.digest) && visited.insert(p.digest) {
                        queue.push_back(p.digest);
                    }
                }
            }
        }

        mergeset
    }

    /// Commit uncommitted blocks along the selected parent chain up to the selected tip.
    ///
    /// Returns ordered [`CommittedSubDag`]s ready for state execution and reward minting.
    pub fn commit_uncommitted_selected_chain(&mut self) -> Vec<CommittedSubDag> {
        let Some(tip) = self.selected_tip else {
            return Vec::new();
        };

        // Trace selected parent chain backwards from tip to last_committed
        let mut chain_rev = Vec::new();
        let mut curr = Some(tip);

        while let Some(h) = curr {
            if Some(h) == self.last_committed_selected_parent {
                break;
            }
            chain_rev.push(h);
            curr = self.selected_parent.get(&h).copied().flatten();
        }

        chain_rev.reverse();

        let mut subdags = Vec::new();

        for selected_hash in chain_rev {
            let Some(leader_block) = self.blocks.get(&selected_hash).cloned() else {
                continue;
            };

            let blue_order = self
                .blue_sets
                .get(&selected_hash)
                .cloned()
                .unwrap_or_default();
            let red_refs = self
                .red_sets
                .get(&selected_hash)
                .cloned()
                .unwrap_or_default();

            // Collect uncommitted blue blocks in topological order
            let mut ordered_blocks = Vec::new();
            for blue_hash in blue_order {
                if !self.committed_blocks.contains(&blue_hash) {
                    if let Some(blk) = self.blocks.get(&blue_hash) {
                        ordered_blocks.push(blk.clone());
                        self.committed_blocks.insert(blue_hash);
                    }
                }
            }

            // Include leader block if not already committed
            if !self.committed_blocks.contains(&selected_hash) {
                ordered_blocks.push(leader_block.clone());
                self.committed_blocks.insert(selected_hash);
            }

            // Collect uncommitted red blocks
            let mut uncommitted_red = Vec::new();
            for r in red_refs {
                if !self.committed_blocks.contains(&r.digest) {
                    self.committed_blocks.insert(r.digest);
                    uncommitted_red.push(r);
                }
            }

            crate::metrics::record_ghostdag_blue_blocks(ordered_blocks.len() as u64);
            crate::metrics::record_ghostdag_red_blocks(uncommitted_red.len() as u64);

            subdags.push(CommittedSubDag {
                leader: leader_block.clone(),
                blocks: ordered_blocks,
                leader_round: leader_block.round,
                leader_author: leader_block.author,
                non_blue: uncommitted_red,
            });

            self.last_committed_selected_parent = Some(selected_hash);
        }

        subdags
    }

    /// Return the active DAG tips (blocks with no children in memory).
    #[must_use]
    pub fn tips(&self) -> Vec<Hash> {
        let mut referenced_parents = HashSet::new();
        for b in self.blocks.values() {
            for p in &b.parents {
                referenced_parents.insert(p.digest);
            }
        }
        self.blocks
            .keys()
            .filter(|h| !referenced_parents.contains(h))
            .copied()
            .collect()
    }

    /// Return the current selected tip.
    #[must_use]
    pub fn selected_tip(&self) -> Option<Hash> {
        self.selected_tip
    }

    /// Return the blue score of a block if tracked.
    #[must_use]
    pub fn blue_score(&self, hash: &Hash) -> Option<u64> {
        self.blue_score.get(hash).copied()
    }

    /// Return the cumulative PoW work of a block if tracked.
    #[must_use]
    pub fn cumulative_work(&self, hash: &Hash) -> Option<u128> {
        self.cumulative_work.get(hash).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_types::block::BlockHeader;

    fn make_pow_block(
        round: u64,
        author: u16,
        parents: Vec<BlockReference>,
        nonce: u64,
    ) -> StatementBlock {
        let mut header = BlockHeader::new(
            kovanica_params::CHAIN_ID_HYBRID_TESTNET,
            round,
            author,
            parents,
            Hash::zero(),
            1_700_000_000_000 + round * 4000,
            0x207fffff,
            nonce,
        );
        header.pow_hash = Some(header.compute_pow_hash());
        header.to_statement_block(Vec::new())
    }

    #[test]
    fn ghostdag_committer_single_chain_linearization() {
        let mut committer = GhostdagCommitter::new(5);

        // Genesis
        let b0 = make_pow_block(0, 0, vec![], 0);
        committer.add_block(b0.clone());

        // B1 -> B0
        let b1 = make_pow_block(1, 1, vec![b0.reference()], 1);
        committer.add_block(b1.clone());

        // B2 -> B1
        let b2 = make_pow_block(2, 2, vec![b1.reference()], 2);
        committer.add_block(b2.clone());

        assert_eq!(committer.selected_tip, Some(b2.digest));

        let subdags = committer.commit_uncommitted_selected_chain();
        assert_eq!(subdags.len(), 3);
        assert_eq!(subdags[0].leader.digest, b0.digest);
        assert_eq!(subdags[1].leader.digest, b1.digest);
        assert_eq!(subdags[2].leader.digest, b2.digest);

        // Second commit without new blocks yields empty
        let empty = committer.commit_uncommitted_selected_chain();
        assert!(empty.is_empty());
    }

    #[test]
    fn ghostdag_committer_mergeset_and_tip_selection() {
        let mut committer = GhostdagCommitter::new(5);

        // B0
        let b0 = make_pow_block(0, 0, vec![], 0);
        committer.add_block(b0.clone());

        // Two concurrent blocks pointing to B0: B1a and B1b
        let b1a = make_pow_block(1, 1, vec![b0.reference()], 10);
        let b1b = make_pow_block(1, 2, vec![b0.reference()], 20);
        committer.add_block(b1a.clone());
        committer.add_block(b1b.clone());

        // B2 points to both B1a and B1b
        let b2 = make_pow_block(2, 3, vec![b1a.reference(), b1b.reference()], 30);
        committer.add_block(b2.clone());

        assert_eq!(committer.selected_tip, Some(b2.digest));
        let sel_p = committer.selected_parent.get(&b2.digest).copied().flatten();
        assert!(sel_p == Some(b1a.digest) || sel_p == Some(b1b.digest));

        let subdags = committer.commit_uncommitted_selected_chain();
        assert!(!subdags.is_empty());

        // All 4 blocks must be committed
        let all_committed_hashes: HashSet<Hash> = subdags
            .iter()
            .flat_map(|s| s.blocks.iter().map(|b| b.digest))
            .collect();
        assert!(all_committed_hashes.contains(&b0.digest));
        assert!(all_committed_hashes.contains(&b1a.digest));
        assert!(all_committed_hashes.contains(&b1b.digest));
        assert!(all_committed_hashes.contains(&b2.digest));
    }
}

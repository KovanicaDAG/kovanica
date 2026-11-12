//! Consensus for Kovanica.
//!
//! Chains 1 and 2 order PoW blocks with GHOSTDAG (`ghostdag`, `daa`, `mining`)
//! and commit per block. The Mysticeti-style wave/leader engine (`engine`,
//! `committer`, `mysticghost`, leader selection) is the legacy path: kept in
//! tree, but not used for chains 1 and 2.

#![deny(unsafe_code)]
#![allow(missing_docs)]
#![allow(clippy::result_large_err)]
#![allow(clippy::large_enum_variant)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod committer;
pub mod committer_mysticghost;
pub mod daa;
pub mod engine;
pub mod genesis;
pub mod ghostdag;
pub mod ghostdag_scoped;
pub mod linearizer;
pub mod metrics;
pub mod mining;
pub mod mysticghost;
pub mod non_blue;
pub mod skip;
pub mod types;

pub use committer::{uncommitted_history, BaseCommitter, UniversalCommitter};
pub use daa::{
    calculate_next_target_bits, extract_ancestor_timestamps, mul_div_u256, required_target_bits,
    select_parents_from_tips, try_required_target_bits, DaaConfig, DaaError,
};
pub use engine::{
    BlockBroadcaster, ConsensusConfig, ConsensusEngine, ConsensusError, TimeoutBroadcaster,
    ValidatorState, VoteBroadcaster, VoteRejection,
};
pub use genesis::is_genesis_subdag;
pub use ghostdag::{calculate_work_from_target, GhostdagCommitter};
pub use kovanica_types::Vote;
pub use linearizer::Linearizer;
pub use metrics::{
    metrics_text, observe_finality_latency, record_active_validators, record_block_height,
    record_colouring_duration_ms, record_commit_latency, record_committed_leader,
    record_connected_peers, record_current_round, record_mempool_size, record_mergeset_size,
    record_mergeset_size_metric, record_peer_count, record_pruned_blocks, record_pruned_waves,
    record_rss_bytes, record_rss_proxy, record_skipped_leader, record_sync_error,
    record_timeout_signed, record_vote_timeout_equivocation, registry, set_pow_target_bits,
    SkipReason, SyncErrorKind,
};
pub use mining::{mine_header, mine_header_async, MiningWorker};
pub use non_blue::{non_blue_refs, non_blue_transactions};
pub use types::{
    AuthorityInfo, CommitResult, CommittedSubDag, CommitteeInfo, CommitteeInfoError, LeaderInfo,
    LeaderStatus,
};

use kovanica_types::{Round, WAVE_LENGTH};

/// Returns the wave number for a given round.
#[inline]
pub fn wave_of(round: Round) -> u64 {
    round / WAVE_LENGTH
}

/// Returns the offset inside the wave (0 = leader, 1 = vote, 2 = decide).
#[inline]
pub fn offset_in_wave(round: Round) -> u64 {
    round % WAVE_LENGTH
}

/// True if this round is a leader round.
#[inline]
pub fn is_leader_round(round: Round) -> bool {
    offset_in_wave(round) == 0
}

/// True if this round is a vote round.
#[inline]
pub fn is_vote_round(round: Round) -> bool {
    offset_in_wave(round) == 1
}

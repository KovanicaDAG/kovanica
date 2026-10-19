//! Core types for the Kovanica (KUNA) blockchain.
//!
//! This crate contains all fundamental data structures used across the system:
//! blocks, transactions, addresses, rounds, committee, etc.

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod account_tree;
pub mod address;
pub mod block;
pub mod colouring;
pub mod commit;
pub mod committee;
pub mod crypto;
pub mod error;
pub mod hash;
pub mod light_client;
pub mod merkle;
pub mod signing;
pub mod transaction;
pub mod vote;

pub use account_tree::{
    account_leaf_hash, account_root, verify_account_proof, AccountProof, AccountSmt,
};
pub use address::Address;
pub use block::{Block, BlockReference, StatementBlock};
pub use colouring::{
    colour_mergeset, mysticghost_commitment, ColourBlock, ColourError, ColouringResult,
    GhostCommitment,
};
pub use commit::CommittedSubDag;
pub use committee::{Authority, Committee, Stake};
pub use crypto::{PublicKey, Signature, SigningKey};
pub use error::TypesError;
pub use hash::Hash;
pub use light_client::{
    committee_seal_hash, executed_tx_hashes, leader_header, light_vrf_input, light_vrf_priority,
    verify_colouring, vrf_hash_prefix, vrf_priority_cmp, ColouringCertificate, CommitSeal,
    LeaderHeader, LightClientCheckpoint, SealAnnouncement, SealError, StateProof,
    WaveCommitCertificate, LIGHT_VRF_DOMAIN,
};
pub use merkle::{MerkleProof, MerkleStep};
pub use signing::SigningContext;
pub use transaction::{Transaction, TransactionKind};
pub use vote::Vote;

/// Logical round number in the DAG.
pub type Round = u64;

/// Unique authority index within the current committee.
pub type AuthorityIndex = u16;

/// Wave length used by the consensus (Mysticeti-style = 3).
pub const WAVE_LENGTH: u64 = 3;

/// Maximum allowed gap, in rounds, between a block and any of its parents.
///
/// Parent selection walks backwards from `round - 1` until it finds the most
/// recent populated round; validation rejects any parent further back than
/// this, keeping the DAG causally connected without unbounded edges.
pub const MAX_PARENT_ROUND_GAP: u64 = 300;

/// Maximum number of transactions per block (tunable).
pub const MAX_TXS_PER_BLOCK: usize = 10_000;

/// Canonical per-transaction gas cap for `Call` transactions.
///
/// `Call.gas_limit` is charged as interpreter (wasmi) fuel, so an uncapped
/// value is unbounded per-transaction work. This is the **single source of
/// truth**: `kovanica-runtime` re-exports it, `kovanica-dag`'s block manager
/// enforces it on the consensus path, and the mempool's default
/// `MempoolConfig::max_gas_limit` is derived from it. Change it here only.
pub const MAX_TX_GAS_LIMIT: u64 = 10_000_000;

/// Round from which the gas-hardening consensus rules take effect.
///
/// Three new rules are gated on `block.round >= GAS_HARDENING_ACTIVE_FROM_ROUND`
/// (below it every block validates and executes exactly as before):
///
/// 1. per-transaction `Call` gas cap ([`MAX_TX_GAS_LIMIT`]) in block-manager
///    `validate_transaction`;
/// 2. per-block gas budget in block-manager `validate_block_content`;
/// 3. per-call module-size gas floor in the execution layer (`Call` gas must be
///    at least `bytecode_len * kovanica_runtime::GAS_PER_MODULE_BYTE`).
///
/// Defaults to `u64::MAX` so the rules are **inert** on the currently running
/// testnet: activating them on a live chain without a coordinated upgrade would
/// fork it, because a node that has not yet been updated would still accept an
/// over-cap block that an updated node rejects. Set to `0` at the next genesis /
/// testnet reset, after every validator runs a build containing the hardening.
pub const GAS_HARDENING_ACTIVE_FROM_ROUND: u64 = u64::MAX;

/// Whether the gas-hardening consensus rules apply to a block/leader at
/// `round`.
///
/// Equivalent to `round >= GAS_HARDENING_ACTIVE_FROM_ROUND`. This is the single
/// predicate every gate uses (block-manager proposal truncation and block
/// validation, execution-time module-size floor), so the activation rule has
/// exactly one definition.
///
/// The lint is allowed because the default activation round is `u64::MAX` (so
/// the rules are inert on the running testnet); the comparison is deliberate,
/// not an accident, and would move here should the constant ever be redefined.
#[allow(clippy::absurd_extreme_comparisons)]
pub fn gas_hardening_active_from(round: Round) -> bool {
    round >= GAS_HARDENING_ACTIVE_FROM_ROUND
}

/// Genesis round.
pub const GENESIS_ROUND: Round = 0;

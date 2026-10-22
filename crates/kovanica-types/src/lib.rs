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
    committee_seal_hash, decode_with_version, encode_with_version, executed_tx_hashes,
    leader_header, light_vrf_input, light_vrf_priority, snapshot_seal_members, verify_colouring,
    vrf_hash_prefix, vrf_priority_cmp, ColouringCertificate, CommitSeal, LeaderHeader,
    LightClientCheckpoint, SealAnnouncement, SealCommitteeMember, SealCommitteeSnapshotMember,
    SealError, StateProof, WaveCommitCertificate, EPOCH_COMMITTEE_VERSION, LIGHT_VRF_DOMAIN,
    SNAPSHOT_MAGIC, STAKING_STATE_VERSION,
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

/// Epoch rotation length in committed leader rounds (≈ 1 day at 12 s/leader:
/// `86_400 / 12 = 7_200`).
///
/// This is the single canonical definition of the epoch unit. Every component
/// that derives an epoch — consensus vote/seal signing
/// (`leader_round / EPOCH_ROUNDS`), committee construction, RPC
/// `getCommittee`/`signingContext`, and the staking state — must use
/// [`epoch_for_round`] so all nodes compute the SAME epoch for the same round.
/// `kovanica_staking` re-exports this constant for backward-compatible paths.
pub const EPOCH_ROUNDS: u64 = 7_200;

/// Epoch that a committed leader round belongs to.
///
/// The epoch is a pure function of the round: `round / EPOCH_ROUNDS`. It is
/// deliberately decoupled from wall-clock time, committed height, and boot
/// time, so it is derived identically everywhere and cannot drift across a
/// restart. This is the one function the consensus signing context, the
/// network vote/seal edge, the committee builder, and the RPC light-client
/// paths must share.
#[must_use]
pub fn epoch_for_round(round: Round) -> u64 {
    round / EPOCH_ROUNDS
}

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

/// Round from which the contract-hardening consensus rules take effect.
///
/// Three new rules are gated on `block.round >= CONTRACT_HARDENING_ACTIVE_FROM_ROUND`
/// (below it every `Deploy`/`Call` validates and executes exactly as before,
/// so an un-updated node never diverges):
///
/// 1. `Deploy` bytecode validation — a deploy whose `code` is not a valid wasm
///    module (compiled with the same wasmi engine the interpreter uses,
///    hosted in `kovanica-execution`) is rejected **before** any account/code
///    row is written; fee and nonce are still charged, matching the existing
///    failure accounting;
/// 2. `Deploy` overwrite guard — a deploy that would replace the code of an
///    account that already holds non-empty code is rejected;
/// 3. wasm entry-point gating — a `Call` whose `method` is not one of the 16
///    canonical `kovanica_execution::ENTRY_POINTS` is rejected before the
///    interpreter runs (the native `dispatch_entry` path already gated these).
///
/// Defaults to `u64::MAX` so the rules are **inert** on the currently running
/// testnet: activating them on a live chain without a coordinated upgrade would
/// fork it, because a node that has not yet been updated would still accept a
/// deploy/call that an updated node rejects. Set to `0` at the next genesis /
/// testnet reset, after every validator runs a build containing the hardening.
pub const CONTRACT_HARDENING_ACTIVE_FROM_ROUND: u64 = u64::MAX;

/// Whether the contract-hardening consensus rules apply to a block/leader at
/// `round`.
///
/// Equivalent to `round >= CONTRACT_HARDENING_ACTIVE_FROM_ROUND`. This is the
/// single predicate the execution layer uses for deploy bytecode validation,
/// the deploy overwrite guard, and wasm entry-point gating, so the activation
/// rule has exactly one definition.
///
/// The lint is allowed because the default activation round is `u64::MAX` (so
/// the rules are inert on the running testnet); the comparison is deliberate,
/// not an accident, and would move here should the constant ever be redefined.
#[allow(clippy::absurd_extreme_comparisons)]
pub fn contract_hardening_active_from(round: Round) -> bool {
    round >= CONTRACT_HARDENING_ACTIVE_FROM_ROUND
}

/// Maximum serialized size of a single transaction, in bytes (512 KiB).
///
/// Measured on the bincode encoding, which is what storage and the network
/// use, so every node computes the same size for the same transaction. The
/// cap keeps a hostile or buggy sender from forcing peers to buffer, relay
/// and store arbitrarily large payloads; a wasm `Deploy` is the only kind
/// that can realistically approach it.
pub const MAX_TX_BYTES: usize = 512 * 1024;

/// Maximum serialized size of a block, in bytes (960 KiB).
///
/// Must stay comfortably below the network frame cap (`MAX_TRANSMIT_SIZE`,
/// 1 MiB) so a block that passes consensus validation always fits in one
/// message; `kovanica-network` asserts that relationship in a test.
pub const MAX_BLOCK_BYTES: usize = 960 * 1024;

/// Round from which the consensus size limits take effect.
///
/// Two block-validity rejections are gated on
/// `block.round >= SIZE_LIMITS_ACTIVE_FROM_ROUND` (below it every block
/// validates exactly as before):
///
/// 1. a transaction whose bincode encoding exceeds [`MAX_TX_BYTES`];
/// 2. a block whose bincode encoding exceeds [`MAX_BLOCK_BYTES`].
///
/// Defaults to `u64::MAX` so the rules are **inert** on the currently running
/// testnet: activating them on a live chain without a coordinated upgrade
/// would fork it, because a node that has not yet been updated would still
/// accept an oversized block that an updated node rejects. Set to `0` at the
/// next genesis / testnet reset, after every validator runs a build containing
/// the limits.
pub const SIZE_LIMITS_ACTIVE_FROM_ROUND: u64 = u64::MAX;

/// Whether the consensus size limits apply to a block at `round`.
///
/// Equivalent to `round >= SIZE_LIMITS_ACTIVE_FROM_ROUND`. This is the single
/// predicate the block manager uses, so the activation rule has exactly one
/// definition.
///
/// The lint is allowed because the default activation round is `u64::MAX` (so
/// the rules are inert on the running testnet); the comparison is deliberate,
/// not an accident, and would move here should the constant ever be redefined.
#[allow(clippy::absurd_extreme_comparisons)]
pub fn size_limits_active_from(round: Round) -> bool {
    round >= SIZE_LIMITS_ACTIVE_FROM_ROUND
}

/// Genesis round.
pub const GENESIS_ROUND: Round = 0;

#[cfg(test)]
mod epoch_tests {
    use super::{epoch_for_round, EPOCH_ROUNDS};

    /// The epoch unit is frozen at 7_200 rounds. Every crate that derives an
    /// epoch from a round must agree on this value; changing it is a consensus
    /// change (it would fork the signing context at every boundary).
    #[test]
    fn epoch_rounds_is_the_frozen_unit() {
        assert_eq!(EPOCH_ROUNDS, 7_200);
    }

    /// `epoch_for_round` is the one shared epoch function. It must be a pure
    /// floor-division of the round so consensus, the network edge, the
    /// committee builder, and the RPC paths all compute the same epoch.
    #[test]
    fn epoch_for_round_is_floor_division_of_the_round() {
        assert_eq!(epoch_for_round(0), 0);
        assert_eq!(epoch_for_round(EPOCH_ROUNDS - 1), 0);
        assert_eq!(epoch_for_round(EPOCH_ROUNDS), 1);
        assert_eq!(epoch_for_round(EPOCH_ROUNDS + 1), 1);
        assert_eq!(epoch_for_round(5 * EPOCH_ROUNDS), 5);
    }
}

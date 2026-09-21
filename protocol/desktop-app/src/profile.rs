//! Network profiles for the desktop app.
//!
//! Mirrors `NetworkProfile` in `crates/kovanica-node/src/explorer.rs` (private
//! to that crate). These are **consensus genesis parameters**: changing them
//! changes the genesis block id and must never diverge from the network's real
//! parameters without tripping the genesis-parity gate
//! ([`crate::service::NodeService::verify_genesis_parity`]).
//!
//! ## Which parameters are live?
//!
//! **RFC-006 is LIVE on testnet** (activated 2026-09-20 as a consensus fork
//! that wiped all pre-RFC-006 balances). The live testnet now runs the
//! RFC-006-era chain: `k:3`, subsidy `10 * ATOM` (10 KVNC/block), premine
//! `200_000 * ATOM` (200,000 KVNC = 0.2M KVNC), founder seed 1, treasury
//! `10 × 1M KVNC` vaults, `finality_depth 100`, `payload_pruning_depth 1000`.
//! This reproduces the live genesis `9565fc20…` byte-for-byte (verified by
//! `examples/probe_genesis.rs` and `tests/genesis_parity.rs`).
//!
//! The old pre-RFC-006-era chain (genesis `3beecbeb…`, subsidy 200 KVNC,
//! premine 200 KVNC, no treasury) is **obsolete** — the activation fork reset
//! the chain. These old values are kept only for historical reference.

use kovanica_state::{RFC006_GENESIS_SUBSIDY, RFC006_PREMINE};

/// Live testnet network id (`/api/bootstrap → network`).
pub const NETWORK_TESTNET: &str = "kovanica-testnet";
/// Dormant mainnet network id (never booted implicitly).
pub const NETWORK_MAINNET: &str = "kovanica-mainnet";
/// Founder actor seed used by the testnet genesis (deterministic keys).
pub const FOUNDER_SEED: u64 = 1;
/// Finality depth of the live testnet (blocks below this blue score become final).
pub const TESTNET_FINALITY_DEPTH: u64 = 100;
/// Payload pruning depth of the live testnet (below this score payloads are evicted).
pub const TESTNET_PAYLOAD_PRUNING_DEPTH: u64 = 1000;
/// 1 KVNC = 10^8 atoms (same value as `explorer.rs::ATOM`).
pub const ATOM: u64 = 100_000_000;

/// RFC-006 live testnet issuance: 10 KVNC/block (atoms).
///
/// The genesis-parity gate verified this is the *only* value that reproduces
/// the live genesis `9565fc20…` (see the module docs and
/// `examples/probe_genesis.rs`).
pub const TESTNET_LIVE_SUBSIDY: u64 = 10 * ATOM;
/// RFC-006 live founder premine: 200,000 KVNC (atoms).
///
/// Matches `kovanica-node`'s explorer `GENESIS_PREMINE` (RFC-006 era).
pub const TESTNET_LIVE_PREMINE: u64 = 200_000 * ATOM;

/// A network identity: id, genesis parameters, and data-dir isolation (slice B
/// wires the data dir / `network` marker file enforcement).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetworkProfile {
    /// Network id — reported by `/api/bootstrap`, `/api/head` and the snapshot.
    pub id: &'static str,
    /// GHOSTDAG `k` parameter for this network's genesis.
    pub genesis_k: u16,
    /// Per-block subsidy cap at genesis (atoms).
    pub genesis_subsidy: u64,
    /// Founder premine minted by the genesis coinbase (atoms).
    pub genesis_premine: u64,
    /// Founder actor seed (deterministic keys).
    pub founder_seed: u64,
    /// Finality depth: blocks more than this many blue score below the tip
    /// become final. `u64::MAX` disables finality pruning.
    pub finality_depth: u64,
    /// Payload pruning depth: blocks more than this many blue score below the
    /// tip have their payloads evicted. `u64::MAX` disables payload pruning.
    pub payload_pruning_depth: u64,
    /// Dormant placeholder (mainnet): genesis parameters are TBD and the
    /// profile refuses to boot.
    pub dormant: bool,
}

impl NetworkProfile {
    /// The live testnet — the default profile.
    ///
    /// Verified construction (2026-09-21, post-RFC-006 activation): the live
    /// network runs the **RFC-006-era** chain — `k:3`, subsidy `10 * ATOM`,
    /// premine `200_000 * ATOM`, founder seed 1, treasury `10 × 1M KVNC`
    /// placeholder vaults, `finality_depth 100`, `payload_pruning_depth 1000`.
    /// Only these values reproduce the live genesis `9565fc20…` (see
    /// `examples/probe_genesis.rs` and `tests/genesis_parity.rs`).
    pub fn testnet() -> Self {
        Self {
            id: NETWORK_TESTNET,
            genesis_k: 3,
            genesis_subsidy: TESTNET_LIVE_SUBSIDY,
            genesis_premine: TESTNET_LIVE_PREMINE,
            founder_seed: FOUNDER_SEED,
            finality_depth: TESTNET_FINALITY_DEPTH,
            payload_pruning_depth: TESTNET_PAYLOAD_PRUNING_DEPTH,
            dormant: false,
        }
    }

    /// Mainnet profile with RFC-006 parameters but still **DORMANT** (mirrors
    /// the explorer's fail-fast guard: mainnet is never activated implicitly).
    ///
    /// Note: the live mainnet treasury keys come from a key ceremony
    /// (`KOVANICA_TREASURY_SEED` in the explorer). Slice B surfaces the
    /// override as an explicit, warned user action — never a default.
    pub fn mainnet() -> Self {
        Self {
            id: NETWORK_MAINNET,
            genesis_k: 3,
            genesis_subsidy: RFC006_GENESIS_SUBSIDY,
            genesis_premine: RFC006_PREMINE,
            founder_seed: FOUNDER_SEED,
            finality_depth: 1000,
            payload_pruning_depth: 10_000,
            dormant: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn testnet_and_mainnet_ids_are_distinct() {
        assert_ne!(NETWORK_TESTNET, NETWORK_MAINNET);
    }

    #[test]
    fn mainnet_is_dormant_by_default() {
        assert!(NetworkProfile::mainnet().dormant);
        assert!(!NetworkProfile::testnet().dormant);
    }
}

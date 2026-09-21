//! Offline genesis-parity regression suite (Slice A gate).
//!
//! Fixtures captured from the **live network on 2026-09-21** (post-RFC-006 activation):
//! - `GET https://explorer.kovanica.online/api/bootstrap` → genesis, network
//! - `GET https://explorer.kovanica.online/api/head` → genesis, blocks
//!
//! **RFC-006 is LIVE on testnet** (activated 2026-09-20 as a consensus fork
//! that wiped all pre-RFC-006 balances). The live chain now runs the
//! RFC-006-era parameters: `k:3`, subsidy `10 * ATOM` (10 KVNC/block), premine
//! `200_000 * ATOM` (200,000 KVNC = 0.2M KVNC), founder seed 1, treasury
//! `10 × 1M KVNC` vaults, `finality_depth 100`, `payload_pruning_depth 1000`.
//! This reproduces the live genesis `9565fc20…` byte-for-byte (verified by
//! `examples/probe_genesis.rs`).
//!
//! The old pre-RFC-006-era chain (genesis `3beecbeb…`, subsidy 200 KVNC,
//! premine 200 KVNC, no treasury) is **obsolete** — the activation fork reset
//! the chain.
//!
//! Refresh this fixture only on a deliberate network reset, together with
//! `NetworkProfile::testnet()` and `examples/probe_genesis.rs`. See
//! `examples/genesis_parity_live.rs` for the live, network-dependent check.

use kovanica_desktop::profile::{
    NetworkProfile, ATOM, NETWORK_TESTNET, TESTNET_FINALITY_DEPTH, TESTNET_LIVE_PREMINE,
    TESTNET_LIVE_SUBSIDY, TESTNET_PAYLOAD_PRUNING_DEPTH,
};
use kovanica_desktop::service::NodeService;

/// Live testnet genesis id, reported by `/api/bootstrap` + `/api/head`
/// (captured 2026-09-21, post-RFC-006 activation). The embedded node must reproduce this exactly.
const LIVE_GENESIS: &str = "9565fc20cb465eec0198a65c07da6b825e4211c4060d581a2c7dac6c96bafc97";

/// Live chain facts used by the gate (captured 2026-09-21, post-RFC-006 activation).
const LIVE_NETWORK: &str = NETWORK_TESTNET;
const LIVE_K: u16 = 3;
const LIVE_SUBSIDY_ATOMS: u64 = 10 * ATOM; // 10 KVNC/block — live /api/head (RFC-006)
const LIVE_PREMINE_ATOMS: u64 = 200_000 * ATOM; // 200,000 KVNC — RFC-006 genesis coinbase
const LIVE_FOUNDER_SEED: u64 = 1;
const LIVE_FINALITY_DEPTH: u64 = TESTNET_FINALITY_DEPTH;
const LIVE_PAYLOAD_PRUNING_DEPTH: u64 = TESTNET_PAYLOAD_PRUNING_DEPTH;

#[test]
fn local_genesis_matches_live_network() {
    let mut service = NodeService::new(NetworkProfile::testnet());
    service
        .verify_genesis_parity(LIVE_GENESIS)
        .expect("embedded node must reproduce the live testnet genesis id");
    assert_eq!(service.profile().id, LIVE_NETWORK);
}

#[test]
fn testnet_profile_matches_the_verified_live_chain() {
    // The construction `examples/probe_genesis.rs` proved reproduces the live
    // genesis byte-for-byte. Keep it pinned; the old pre-RFC-006-era schedule is
    // obsolete — the activation fork reset the chain.
    let live = NetworkProfile {
        id: LIVE_NETWORK,
        genesis_k: LIVE_K,
        genesis_subsidy: LIVE_SUBSIDY_ATOMS,
        genesis_premine: LIVE_PREMINE_ATOMS,
        founder_seed: LIVE_FOUNDER_SEED,
        finality_depth: LIVE_FINALITY_DEPTH,
        payload_pruning_depth: LIVE_PAYLOAD_PRUNING_DEPTH,
        dormant: false,
    };
    assert_eq!(
        NetworkProfile::testnet(),
        live,
        "testnet profile drifted from the verified live chain"
    );
}

#[test]
fn live_subsidy_and_premine_are_the_rfc006_values() {
    // Guards against a well-meaning "downgrade" to the old pre-RFC-006-era constants
    // (200 KVNC / 200 KVNC), which the probe shows does NOT reproduce the live
    // genesis. If this fails, the network was reset to old-era parameters:
    // update `NetworkProfile::testnet()`, this fixture and
    // `examples/probe_genesis.rs` together — never one alone.
    assert_eq!(TESTNET_LIVE_SUBSIDY, 10 * ATOM);
    assert_eq!(TESTNET_LIVE_PREMINE, 200_000 * ATOM);
    assert_ne!(NetworkProfile::testnet().genesis_subsidy, 200 * ATOM);
    assert_ne!(NetworkProfile::testnet().genesis_premine, 200 * ATOM);
}

#[test]
fn atom_and_kvnc_units_match_live_reports() {
    // /api/bootstrap and /api/head both report atom = 100_000_000.
    assert_eq!(ATOM, 100_000_000);
}

#[test]
fn boot_reports_genesis_and_founder() {
    let mut service = NodeService::new(NetworkProfile::testnet());
    let (genesis, founder) = service.boot().expect("testnet must boot");
    assert_eq!(genesis, LIVE_GENESIS);
    // Founder is actor seed 1 (deterministic keys).
    let expected_founder = kovanica_node::Node::address(LIVE_FOUNDER_SEED).to_string();
    assert_eq!(founder, expected_founder);
    assert_eq!(service.block_count(), 1); // genesis only
    assert!(service.selected_tip().is_some());
}

#[test]
fn dormant_mainnet_refuses_to_boot() {
    let mut service = NodeService::new(NetworkProfile::mainnet());
    let err = service
        .boot()
        .expect_err("dormant profiles must refuse to boot");
    assert!(
        matches!(err, kovanica_desktop::BootError::DormantNetwork(_)),
        "expected DormantNetwork, got {err}"
    );
}

#[test]
fn parity_rejects_a_wrong_genesis() {
    let mut service = NodeService::new(NetworkProfile::testnet());
    let err = service
        .verify_genesis_parity("0000000000000000000000000000000000000000000000000000000000000000")
        .expect_err("a mismatched live genesis must fail the gate");
    assert!(
        matches!(err, kovanica_desktop::ParityError::Mismatch { .. }),
        "expected Mismatch, got {err}"
    );
}

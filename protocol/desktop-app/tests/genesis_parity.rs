//! Offline genesis-parity regression suite (Slice A gate).
//!
//! Fixtures captured from the **live network on 2026-09-17**:
//! - `GET https://explorer.kovanica.online/api/bootstrap` → genesis, network
//! - `GET https://explorer.kovanica.online/api/head` → genesis, blocks
//!
//! The live chain is a **pre-RFC-006-era** chain: `k:3`, `200 * ATOM` subsidy
//! and premine, founder seed 1, no treasury. These are the only values that
//! reproduce the live genesis `3beecbeb…b74056e` (verified by
//! `examples/probe_genesis.rs`). The deployed binary's
//! `/api/bootstrap → light_config` reports RFC-006-era numbers (`subsidy:
//! 10 KVNC`, `premine: 200_000 KVNC`) that do **not** reproduce the live
//! genesis — it is RFC-006-era code serving a pre-reset chain. Do not use
//! those fields as the parity fixture.
//!
//! Refresh this fixture only on a deliberate network reset (e.g. the RFC-006
//! activation the protocol repo describes), together with
//! `NetworkProfile::testnet()` and `examples/probe_genesis.rs`. See
//! `examples/genesis_parity_live.rs` for the live, network-dependent check.

use kovanica_desktop::profile::{
    NetworkProfile, ATOM, NETWORK_TESTNET, TESTNET_FINALITY_DEPTH, TESTNET_LIVE_PREMINE,
    TESTNET_LIVE_SUBSIDY, TESTNET_PAYLOAD_PRUNING_DEPTH,
};
use kovanica_desktop::service::NodeService;

/// Live testnet genesis id, reported by `/api/bootstrap` + `/api/head`
/// (captured 2026-09-17). The embedded node must reproduce this exactly.
const LIVE_GENESIS: &str = "3beecbebb6103ee24d1617fd87e920c949d613febbbcf6ca1453f3a4bf74056e";

/// Live chain facts used by the gate (captured 2026-09-17).
const LIVE_NETWORK: &str = NETWORK_TESTNET;
const LIVE_K: u16 = 3;
const LIVE_SUBSIDY_ATOMS: u64 = 200 * ATOM; // 200 KVNC/block — live /api/head
const LIVE_PREMINE_ATOMS: u64 = 200 * ATOM; // 200 KVNC — old-era genesis coinbase
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
    // genesis byte-for-byte. Keep it pinned; the RFC-006-era schedule is a
    // future reset, not the live network.
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
fn live_subsidy_and_premine_are_the_old_era_values() {
    // Guards against a well-meaning "upgrade" to the RFC-006-era constants
    // (10 KVNC / 200,000 KVNC), which the probe shows does NOT reproduce the
    // live genesis. If this fails, the network was reset to RFC-006-era
    // parameters: update `NetworkProfile::testnet()`, this fixture and
    // `examples/probe_genesis.rs` together — never one alone.
    assert_eq!(TESTNET_LIVE_SUBSIDY, 200 * ATOM);
    assert_eq!(TESTNET_LIVE_PREMINE, 200 * ATOM);
    assert_ne!(NetworkProfile::testnet().genesis_subsidy, 10 * ATOM);
    assert_ne!(NetworkProfile::testnet().genesis_premine, 200_000 * ATOM);
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

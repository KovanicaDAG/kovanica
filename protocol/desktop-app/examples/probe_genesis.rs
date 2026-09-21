//! Throwaway probe: reproduce the live testnet genesis id from candidate
//! construction paths. Network-independent.
//!
//! **RFC-006 is LIVE on testnet** (activated 2026-09-20). The live chain
//! runs the RFC-006-era parameters with treasury vaults:
//! `k:3`, subsidy `10*ATOM`, premine `200_000*ATOM`, founder_seed=1,
//! treasury = 10 × 1M KVNC placeholder vaults.
//!
//! The old pre-RFC-006-era chain (genesis `3beecbeb…`, subsidy 200 KVNC,
//! premine 200 KVNC, no treasury) is **obsolete** — the activation fork
//! reset the chain.

const LIVE_GENESIS: &str = "9565fc20cb465eec0198a65c07da6b825e4211c4060d581a2c7dac6c96bafc97";
const ATOM: u64 = 100_000_000;

fn boot(
    k: u16,
    subsidy: u64,
    premine: u64,
    founder_seed: u64,
    treasury: Option<kovanica_node::TreasuryGenesis>,
) -> String {
    let mut node = kovanica_node::Node::new();
    let (genesis, founder) = node
        .genesis_with_finality(k, subsidy, premine, founder_seed, treasury, 100, 1000)
        .expect("genesis boots");
    eprintln!("  founder: {founder}");
    genesis.to_string()
}

fn main() {
    println!("target  {}", LIVE_GENESIS);
    println!("A) RFC-006-era with placeholder treasury (LIVE)");
    let a = boot(
        3,
        10 * ATOM,
        200_000 * ATOM,
        1,
        Some(kovanica_node::TreasuryGenesis::placeholder()),
    );
    println!(
        "   => {a}  {}",
        if a == LIVE_GENESIS { "MATCH" } else { "no" }
    );
    println!("B) RFC-006-era, no treasury (differs from live)");
    let b = boot(3, 10 * ATOM, 200_000 * ATOM, 1, None);
    println!(
        "   => {b}  {}",
        if b == LIVE_GENESIS { "MATCH" } else { "no" }
    );
    println!("C) Old pre-RFC-006-era (OBSOLETE — does not match live)");
    let c = boot(3, 200 * ATOM, 200 * ATOM, 1, None);
    println!(
        "   => {c}  {}",
        if c == LIVE_GENESIS { "MATCH" } else { "no" }
    );
}

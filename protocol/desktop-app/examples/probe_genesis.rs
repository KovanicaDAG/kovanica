//! Throwaway probe: reproduce the live testnet genesis id from candidate
//! construction paths. Network-independent.
//!
//! Hypotheses being tested (AGENTS.md Slice 9a documented the old-era
//! reproducer `LightConfig { k:3, subsidy:200*ATOM, founder_amount:200*ATOM,
//! founder_seed:1 }`):
//!   A) old-era, treasury=None, subsidy=200*ATOM, premine=200*ATOM
//!   B) RFC-006-era, treasury=None,  subsidy=10*ATOM, premine=200_000*ATOM
//!   C) RFC-006-era, treasury placeholder, 10*ATOM / 200_000*ATOM

const LIVE_GENESIS: &str = "3beecbebb6103ee24d1617fd87e920c949d613febbbcf6ca1453f3a4bf74056e";
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
    println!("A) old-era, no treasury, 200 KVNC sub + 200 KVNC premine");
    let a = boot(3, 200 * ATOM, 200 * ATOM, 1, None);
    println!(
        "   => {a}  {}",
        if a == LIVE_GENESIS { "MATCH" } else { "no" }
    );
    println!("B) RFC006-era, no treasury, 10 KVNC sub + 200,000 KVNC premine");
    let b = boot(3, 10 * ATOM, 200_000 * ATOM, 1, None);
    println!(
        "   => {b}  {}",
        if b == LIVE_GENESIS { "MATCH" } else { "no" }
    );
    println!("C) RFC006-era with placeholder treasury");
    let c = boot(
        3,
        10 * ATOM,
        200_000 * ATOM,
        1,
        Some(kovanica_node::TreasuryGenesis::placeholder()),
    );
    println!(
        "   => {c}  {}",
        if c == LIVE_GENESIS { "MATCH" } else { "no" }
    );
}

//! End-to-end checks for `ops/genesis/ceremony.sh`.
//!
//! The ceremony is the only supported way to produce a chain-2 (canonical
//! hybrid testnet) genesis file set. These tests run the real script and the
//! real binary: they check that a `no-stake-82m` chain-2 genesis is produced
//! with an explicit policy, no stake field, a recorded treasury and a founder
//! premine file, and that every input a hybrid node would refuse fails closed
//! instead of being silently rewritten.
//!
//! Addresses are throwaway values derived from a fixed byte pattern. No test
//! prints or writes secret material; `--generate-keys` is not exercised here.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const NODE_BIN: &str = env!("CARGO_BIN_EXE_kovanica-node");

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/kovanica-node sits under the workspace root")
        .to_path_buf()
}

/// A canonical PRO-21 address. `seed` selects a distinct throwaway key.
fn pro21(seed: u8) -> String {
    kovanica_types::Address([seed; 32]).encode()
}

fn write_validators(dir: &Path, addresses: &[String]) -> PathBuf {
    let path = dir.join("validators.json");
    let entries: Vec<String> = addresses
        .iter()
        .map(|address| {
            format!(
                r#"{{"address": "{address}", "public_key": "{}"}}"#,
                "11".repeat(32)
            )
        })
        .collect();
    std::fs::write(&path, format!("[{}]\n", entries.join(","))).expect("write validators");
    path
}

fn ceremony(current_dir: &Path, args: &[&str]) -> Output {
    Command::new("bash")
        .arg(repo_root().join("ops/genesis/ceremony.sh"))
        .args(args)
        .env("KOVANICA_NODE_BIN", NODE_BIN)
        .current_dir(current_dir)
        .output()
        .expect("running ops/genesis/ceremony.sh")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

#[test]
fn chain2_ceremony_produces_a_no_stake_82m_genesis() {
    let root = tempfile::tempdir().expect("tempdir");
    let validators = write_validators(root.path(), &[pro21(0x21), pro21(0x22)]);
    let treasury = pro21(0x31);
    let founder = pro21(0x32);
    let out = root.path().join("genesis-out");

    let output = ceremony(
        root.path(),
        &[
            "--network",
            "testnet",
            "--validators",
            validators.to_str().unwrap(),
            "--treasury-address",
            &treasury,
            "--founder-address",
            &founder,
            "--output",
            out.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "ceremony failed:\nstdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );

    // genesis_validators.toml: chain 2, explicit no-stake-82m policy, the real
    // treasury recorded, and no stake field at all.
    let toml_text = read(&out.join("genesis_validators.toml"));
    let parsed: toml::Value = toml::from_str(&toml_text).expect("genesis_validators.toml parses");
    assert_eq!(parsed["chain_id"].as_integer(), Some(2));
    assert_eq!(
        parsed["emission_policy"].as_str(),
        Some("no-stake-82m"),
        "the ceremony must pass and record the hybrid policy:\n{toml_text}"
    );
    assert_eq!(parsed["treasury_address"].as_str(), Some(treasury.as_str()));
    assert!(
        !toml_text.contains("stake =") && !toml_text.contains("stake="),
        "a no-stake genesis carries no stake field:\n{toml_text}"
    );
    let declared = parsed["validator"]
        .as_array()
        .expect("[[validator]] entries");
    assert_eq!(declared.len(), 2);
    for entry in declared {
        assert!(entry.get("stake").is_none(), "no stake field: {entry:?}");
    }

    // founder_premine.hex is what init_genesis reads to mint the 200k premine.
    let premine = read(&out.join("founder_premine.hex"));
    assert_eq!(premine.trim(), founder);

    // genesis.json preview: no genesis validator allocation and no locked
    // stake. The founder and treasury allocations are not balances in this
    // file — the node derives them from founder_premine.hex and
    // KOVANICA_TREASURY_ADDRESS.
    let genesis: serde_json::Value =
        serde_json::from_str(&read(&out.join("genesis.json"))).expect("genesis.json parses");
    let allocations = &genesis["allocations"];
    assert_eq!(
        allocations["emission_policy"].as_str(),
        Some("no-stake-82m")
    );
    assert_eq!(
        allocations["genesis_validator_allocation_atoms"].as_u64(),
        Some(0),
        "no-stake-82m mints no genesis validator allocation"
    );
    assert_eq!(
        allocations["mining_subsidy_budget_kovanica"].as_u64(),
        Some(82_000_000)
    );
    for validator in allocations["validators"].as_array().expect("validators") {
        assert_eq!(validator["stake"].as_u64(), Some(0));
        assert_eq!(validator["genesis_share_atoms"].as_u64(), Some(0));
    }
    assert!(
        genesis["staking_state"]["validators"]
            .as_array()
            .is_some_and(|validators| validators.is_empty()),
        "a no-stake genesis must not join any validator:\n{genesis}"
    );
    assert_eq!(genesis["staking_state"]["total_staked"].as_u64(), Some(0));

    // The removed placeholder addresses must never appear in any output.
    for name in [
        "genesis.json",
        "genesis_validators.toml",
        "founder_premine.hex",
        "bootnodes.toml",
    ] {
        assert!(
            !read(&out.join(name)).contains("kvncqqqqqqqqqqq"),
            "{name} contains a placeholder address"
        );
    }
}

#[test]
fn legacy_hex_validators_are_refused_on_chain2() {
    let root = tempfile::tempdir().expect("tempdir");
    let legacy = repo_root().join("testnet/validators/validators.json");
    assert!(
        legacy.exists(),
        "the committed legacy validator set is the input this test refuses"
    );
    let out = root.path().join("genesis-out");

    let output = ceremony(
        root.path(),
        &[
            "--network",
            "testnet",
            "--validators",
            legacy.to_str().unwrap(),
            "--treasury-address",
            &pro21(0x31),
            "--founder-address",
            &pro21(0x32),
            "--output",
            out.to_str().unwrap(),
        ],
    );
    assert!(
        !output.status.success(),
        "legacy hex validators were accepted on chain 2:\n{}",
        stdout(&output)
    );
    let message = stderr(&output);
    assert!(
        message.contains("PRO-21"),
        "the error must say why and must not offer a conversion: {message}"
    );
    assert!(
        !out.join("genesis_validators.toml").exists(),
        "no file set may be written when an input is refused"
    );
}

#[test]
fn legacy_hex_founder_or_treasury_is_refused_on_chain2() {
    let root = tempfile::tempdir().expect("tempdir");
    let validators = write_validators(root.path(), &[pro21(0x21)]);
    let hex = "aa".repeat(32);

    for (flag, other, address) in [
        ("--treasury-address", "--founder-address", hex.as_str()),
        ("--founder-address", "--treasury-address", hex.as_str()),
    ] {
        let out = root
            .path()
            .join(format!("out{}", flag.trim_start_matches('-')));
        let placeholder = pro21(0x40);
        let mut args = vec![
            "--network",
            "testnet",
            "--validators",
            validators.to_str().unwrap(),
            "--output",
            out.to_str().unwrap(),
        ];
        if flag == "--treasury-address" {
            args.extend([flag, address, other, placeholder.as_str()]);
        } else {
            args.extend([other, placeholder.as_str(), flag, address]);
        }
        let output = ceremony(root.path(), &args);
        assert!(!output.status.success(), "{flag} hex was accepted");
        assert!(
            stderr(&output).contains("PRO-21"),
            "got:\n{}",
            stderr(&output)
        );
    }
}

#[test]
fn founder_and_treasury_flags_are_mandatory_and_write_nothing() {
    let root = tempfile::tempdir().expect("tempdir");
    let validators = write_validators(root.path(), &[pro21(0x21)]);

    let missing_founder = root.path().join("missing-founder");
    let output = ceremony(
        root.path(),
        &[
            "--network",
            "testnet",
            "--validators",
            validators.to_str().unwrap(),
            "--treasury-address",
            &pro21(0x31),
            "--output",
            missing_founder.to_str().unwrap(),
        ],
    );
    assert!(!output.status.success(), "a missing founder was accepted");
    assert!(
        stderr(&output).contains("--founder-address"),
        "got:\n{}",
        stderr(&output)
    );
    assert!(
        !missing_founder.exists(),
        "nothing may be created when a required address is missing"
    );

    let missing_treasury = root.path().join("missing-treasury");
    let output = ceremony(
        root.path(),
        &[
            "--network",
            "testnet",
            "--validators",
            validators.to_str().unwrap(),
            "--founder-address",
            &pro21(0x32),
            "--output",
            missing_treasury.to_str().unwrap(),
        ],
    );
    assert!(!output.status.success(), "a missing treasury was accepted");
    assert!(
        stderr(&output).contains("--treasury-address"),
        "got:\n{}",
        stderr(&output)
    );
    assert!(
        !missing_treasury.exists(),
        "nothing may be created when a required address is missing"
    );
}

#[test]
fn committed_genesis_artefacts_are_never_the_output() {
    let _root = tempfile::tempdir().expect("tempdir");
    let validators = write_validators(_root.path(), &[pro21(0x21)]);

    for reserved in [
        "testnet",
        "testnet/node-data",
        "mainnet",
        "ops/docker/validators",
        "ops/docker/chain2",
    ] {
        let output_dir = repo_root().join(reserved).display().to_string();
        let output = ceremony(
            &repo_root(),
            &[
                "--network",
                "testnet",
                "--validators",
                validators.to_str().unwrap(),
                "--treasury-address",
                &pro21(0x31),
                "--founder-address",
                &pro21(0x32),
                "--output",
                &output_dir,
                "--force",
            ],
        );
        assert!(
            !output.status.success(),
            "--output {reserved} (a committed artefact directory) was accepted"
        );
        assert!(
            stderr(&output).contains("committed"),
            "got:\n{}",
            stderr(&output)
        );
    }
}

/// The node's own refusal is what makes the ceremony's flag non-optional, so it
/// is asserted directly: `--chain-id 2` with the legacy policy fails closed.
#[test]
fn chain2_genesis_refuses_the_legacy_emission_policy() {
    let root = tempfile::tempdir().expect("tempdir");
    let validators = write_validators(root.path(), &[pro21(0x21)]);

    let output = Command::new(NODE_BIN)
        .args([
            "genesis",
            "--chain-id",
            "2",
            "--emission-policy",
            "legacy-81m",
            "--validators",
            validators.to_str().unwrap(),
            "--treasury-address",
            &pro21(0x31),
            "--founder-address",
            &pro21(0x32),
            "--output",
            root.path().join("genesis.json").to_str().unwrap(),
        ])
        .output()
        .expect("running kovanica-node genesis");

    assert!(
        !output.status.success(),
        "chain 2 accepted legacy-81m:\n{}",
        stdout(&output)
    );
    assert!(
        stderr(&output).contains("no-stake-82m"),
        "the error must name the required policy:\n{}",
        stderr(&output)
    );
}

/// Tokenomics arithmetic only — not an on-chain balance. The 90.2M supply is
/// founder 200k + treasury 8M + the 82M mining budget; no-stake-82m mints no
/// genesis validator allocation. The faucet pre-funding in
/// `funded_accounts.toml` is extra and deliberately not part of this sum.
#[test]
fn no_stake_82m_policy_closes_the_supply() {
    use kovanica_types::emission::EmissionPolicy;

    let policy = EmissionPolicy::NoStake82M;
    assert_eq!(
        policy.mining_budget(),
        82_000_000 * kovanica_params::ONE_KVNC
    );
    assert_eq!(policy.genesis_validator_allocation(), 0);
    assert_eq!(
        kovanica_params::FOUNDER_PREMINE + kovanica_params::TREASURY_TOTAL + policy.mining_budget(),
        kovanica_params::TOTAL_SUPPLY
    );
    assert_eq!(
        kovanica_params::TOTAL_SUPPLY,
        90_200_000 * kovanica_params::ONE_KVNC
    );
    assert_eq!(
        kovanica_params::FOUNDER_PREMINE,
        200_000 * kovanica_params::ONE_KVNC
    );
    assert_eq!(
        kovanica_params::TREASURY_TOTAL,
        8_000_000 * kovanica_params::ONE_KVNC
    );
}

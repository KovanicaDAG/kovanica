//! End-to-end checks for `kovanica-node genesis-bundle`.
//!
//! The unit tests in `src/genesis_bundle.rs` cover the archive format. These
//! tests run the real binary: they check that the tag it prints is the tag a
//! node computes for the same file set, that a rebuild is byte-identical, and
//! that the command refuses the mistakes that would publish a wrong bundle.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const NODE_BIN: &str = env!("CARGO_BIN_EXE_kovanica-node");

/// Canonical chain-2 address and key of the chain-2 faucet account. Reused
/// verbatim so the fixture carries no key material of its own.
const FAUCET_ADDRESS: &str = "kvnck2559l6vnpcchmtkuf2eslmdtxc6wtfm9nf9zqqructs4336nla58y78mdag";
const FAUCET_PUBLIC_KEY: &str = "b2a942ff4c98718bed76e255987f6d59b1a72d3b2cd2510003e6170ac63a9ffb";

fn run(args: &[&str]) -> Output {
    Command::new(NODE_BIN)
        .args(args)
        .output()
        .expect("running kovanica-node")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Write a minimal chain-2 genesis file set and return its directory.
fn chain2_fixture(root: &Path) -> PathBuf {
    let source = root.join("genesis");
    std::fs::create_dir_all(&source).expect("create fixture dir");
    std::fs::write(
        source.join("genesis_validators.toml"),
        format!(
            "# chain 2 fixture\n\
             chain_id = 2\n\
             emission_policy = \"no-stake-82m\"\n\
             \n\
             [[validator]]\n\
             address = \"{FAUCET_ADDRESS}\"\n\
             public_key = \"{FAUCET_PUBLIC_KEY}\"\n"
        ),
    )
    .expect("write genesis validators");
    std::fs::write(
        source.join("founder_premine.hex"),
        format!("{FAUCET_ADDRESS}\n"),
    )
    .expect("write founder premine");
    std::fs::write(
        source.join("funded_accounts.toml"),
        format!("[[account]]\naddress = \"{FAUCET_ADDRESS}\"\nbalance = 100000000000000\n"),
    )
    .expect("write funded accounts");
    source
}

/// The tag printed by `genesis-bundle`, which is the last line's value.
fn printed_tag(output: &Output) -> String {
    let text = stdout(output);
    let line = text
        .lines()
        .find(|line| line.trim_start().starts_with("network tag"))
        .unwrap_or_else(|| panic!("no network tag line in:\n{text}"));
    line.split_whitespace()
        .nth(2)
        .expect("tag after `network tag`")
        .to_string()
}

#[test]
fn bundle_tag_is_the_tag_a_node_computes_for_the_same_files() {
    let root = tempfile::tempdir().expect("tempdir");
    let source = chain2_fixture(root.path());
    let out = root.path().join("kovanica-chain2-genesis.tar.gz");

    let bundle = run(&[
        "genesis-bundle",
        "--chain-id",
        "2",
        "--source",
        source.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert!(bundle.status.success(), "failed:\n{}", stderr(&bundle));
    let tag = printed_tag(&bundle);
    assert!(
        tag.starts_with("2-") && tag.len() == "2-".len() + 16,
        "tag {tag} is not <chain-id>-<16 hex>"
    );

    // The tag a node computes from the installed file set must be the one the
    // bundle advertises, otherwise every node that installs it is isolated.
    let data = root.path().join("data");
    std::fs::create_dir_all(&data).expect("create data dir");
    let config = root.path().join("kovanica.toml");
    std::fs::write(
        &config,
        format!("chain_id = 2\ndata_dir = \"{}\"\n", data.display()),
    )
    .expect("write config");
    for name in [
        "genesis_validators.toml",
        "founder_premine.hex",
        "funded_accounts.toml",
    ] {
        std::fs::copy(source.join(name), data.join(name)).expect("copy genesis file");
    }
    let node_tag = run(&["network-tag", "--config", config.to_str().unwrap()]);
    assert!(node_tag.status.success(), "failed:\n{}", stderr(&node_tag));
    assert_eq!(stdout(&node_tag).trim(), tag, "bundle tag vs node tag");

    // `expected-tag` inside the archive is what install.sh compares against.
    let extracted = root.path().join("extracted");
    std::fs::create_dir_all(&extracted).expect("create extracted dir");
    let status = Command::new("tar")
        .args(["-xzf", out.to_str().unwrap(), "-C"])
        .arg(&extracted)
        .status();
    match status {
        Ok(status) if status.success() => {
            let expected = std::fs::read_to_string(extracted.join("expected-tag"))
                .expect("expected-tag in the bundle");
            assert_eq!(expected.trim(), tag);

            let sums = std::fs::read_to_string(extracted.join("SHA256SUMS"))
                .expect("SHA256SUMS in the bundle");
            // `sha256sum -c` accepts exactly this format from the data dir.
            for line in sums.lines() {
                let (_, name) = line
                    .split_once("  ")
                    .unwrap_or_else(|| panic!("bad SHA256SUMS line: {line}"));
                assert_eq!(
                    std::fs::read(extracted.join(name)).expect("listed file exists"),
                    std::fs::read(source.join(name)).expect("source file"),
                    "{name} differs from the source"
                );
            }
            let check = Command::new("sha256sum")
                .arg("-c")
                .arg("SHA256SUMS")
                .current_dir(&extracted)
                .output();
            if let Ok(check) = check {
                assert!(
                    check.status.success(),
                    "sha256sum -c failed:\n{}",
                    stderr(&check)
                );
            }
        }
        _ => eprintln!("skipping archive extraction check: tar is not available"),
    }
}

#[test]
fn a_rebuild_is_byte_identical() {
    let root = tempfile::tempdir().expect("tempdir");
    let source = chain2_fixture(root.path());
    let first = root.path().join("first.tar.gz");
    let second = root.path().join("second.tar.gz");

    for out in [&first, &second] {
        let output = run(&[
            "genesis-bundle",
            "--chain-id",
            "2",
            "--source",
            source.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ]);
        assert!(output.status.success(), "failed:\n{}", stderr(&output));
    }

    assert_eq!(
        std::fs::read(&first).expect("first bundle"),
        std::fs::read(&second).expect("second bundle"),
        "the same files must produce the same bytes"
    );
}

#[test]
fn files_that_are_not_genesis_files_are_refused() {
    let root = tempfile::tempdir().expect("tempdir");
    let source = chain2_fixture(root.path());
    std::fs::write(source.join("node1_signing.key"), b"secret\n").expect("write key");

    let output = run(&[
        "genesis-bundle",
        "--chain-id",
        "2",
        "--source",
        source.to_str().unwrap(),
        "--out",
        root.path().join("bundle.tar.gz").to_str().unwrap(),
    ]);
    assert!(!output.status.success(), "a key in the source was accepted");
    assert!(
        stderr(&output).contains("node1_signing.key"),
        "error should name the offending file:\n{}",
        stderr(&output)
    );
    assert!(
        !root.path().join("bundle.tar.gz").exists(),
        "no archive may be written when the source is refused"
    );
}

#[test]
fn an_existing_archive_needs_force() {
    let root = tempfile::tempdir().expect("tempdir");
    let source = chain2_fixture(root.path());
    let out = root.path().join("bundle.tar.gz");
    std::fs::write(&out, b"old").expect("write placeholder");

    let refused = run(&[
        "genesis-bundle",
        "--chain-id",
        "2",
        "--source",
        source.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert!(
        !refused.status.success(),
        "an existing archive was overwritten"
    );
    assert!(stderr(&refused).contains("--force"), "{}", stderr(&refused));

    let forced = run(&[
        "genesis-bundle",
        "--chain-id",
        "2",
        "--source",
        source.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
        "--force",
    ]);
    assert!(forced.status.success(), "failed:\n{}", stderr(&forced));
    assert_ne!(std::fs::read(&out).expect("archive"), b"old");
}

#[test]
fn an_unregistered_chain_id_is_refused() {
    let root = tempfile::tempdir().expect("tempdir");
    let source = chain2_fixture(root.path());

    let output = run(&[
        "genesis-bundle",
        "--chain-id",
        "4",
        "--source",
        source.to_str().unwrap(),
        "--out",
        root.path().join("bundle.tar.gz").to_str().unwrap(),
    ]);
    assert!(!output.status.success(), "chain 4 was accepted");
    assert!(
        stderr(&output).contains("not a registered chain"),
        "{}",
        stderr(&output)
    );
}

/// The workflow publishes exactly one archive name, and the default `--out`
/// must produce it: the release job never passes `--out`.
#[test]
fn the_default_archive_name_is_the_published_chain2_asset() {
    let root = tempfile::tempdir().expect("tempdir");
    let source = chain2_fixture(root.path());
    let workdir = root.path().join("work");
    std::fs::create_dir_all(&workdir).expect("create workdir");

    let bundle = Command::new(NODE_BIN)
        .args([
            "genesis-bundle",
            "--chain-id",
            "2",
            "--source",
            source.to_str().unwrap(),
        ])
        .current_dir(&workdir)
        .output()
        .expect("running kovanica-node");
    assert!(bundle.status.success(), "failed:\n{}", stderr(&bundle));

    let archive = workdir.join("kovanica-chain2-genesis.tar.gz");
    assert!(
        archive.exists(),
        "default output must be kovanica-chain2-genesis.tar.gz, got: {:?}",
        std::fs::read_dir(&workdir)
            .expect("read workdir")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name())
            .collect::<Vec<_>>()
    );

    let extracted = root.path().join("extracted");
    std::fs::create_dir_all(&extracted).expect("create extracted dir");
    match Command::new("tar")
        .args(["-xzf", archive.to_str().unwrap(), "-C"])
        .arg(&extracted)
        .status()
    {
        Ok(status) if status.success() => {
            let expected = std::fs::read_to_string(extracted.join("expected-tag"))
                .expect("expected-tag in the bundle");
            assert_eq!(expected.trim(), printed_tag(&bundle));

            let sums = std::fs::read_to_string(extracted.join("SHA256SUMS"))
                .expect("SHA256SUMS in the bundle");
            let listed: Vec<&str> = sums
                .lines()
                .map(|line| {
                    line.split_once("  ")
                        .unwrap_or_else(|| panic!("bad SHA256SUMS line: {line}"))
                        .1
                })
                .collect();
            assert_eq!(
                listed,
                [
                    "founder_premine.hex",
                    "funded_accounts.toml",
                    "genesis_validators.toml"
                ],
                "SHA256SUMS must cover the genesis files only"
            );
        }
        _ => eprintln!("skipping archive extraction check: tar is not available"),
    }
}

/// The release workflow must package the chain-2 bundle the ceremony produces,
/// not the retired chain-4 files. This is a text-level guard: it fails a PR that
/// re-points the workflow back without touching this test.
#[test]
fn the_release_workflow_packages_the_chain2_bundle() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/kovanica-node sits under the workspace root");
    let workflow = std::fs::read_to_string(repo.join(".github/workflows/release-node.yml"))
        .expect("read release-node.yml");

    for needle in [
        "genesis-bundle",
        "--chain-id \"$GENESIS_CHAIN_ID\"",
        "kovanica-chain2-genesis.tar.gz",
        "testnet/chain2",
        "expected-tag",
        "SHA256SUMS",
    ] {
        assert!(
            workflow.contains(needle),
            "release-node.yml must contain {needle:?}"
        );
    }
    for forbidden in [
        "testnet/chain4",
        "kovanica-chain4-genesis.tar.gz",
        "4-0eacd5699210b145",
    ] {
        assert!(
            !workflow.contains(forbidden),
            "release-node.yml still references the retired chain-4 bundle: {forbidden:?}"
        );
    }
}

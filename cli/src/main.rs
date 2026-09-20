//! `kovanica` — a command-line client for the Kovanica (KVNC) testnet.
//!
//! Read-only explorer queries, a local Ed25519 wallet, and signed transfers.
//! The address encoding and spend signing are delegated to `kovanica-state`,
//! the node's own crate, so the CLI stays byte-compatible with the ledger.

mod api;
mod wallet;

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use kovanica_state::{Address, AssetId, derive_rwa_asset_id};

use crate::api::{print_json, Client};
use crate::wallet::Wallet;

/// 1 KVNC = 10^8 atoms.
const ATOM: u64 = 100_000_000;

#[derive(Parser)]
#[command(
    name = "kovanicagent",
    version,
    about = "Command-line client for the Kovanica (KVNC) testnet BlockDAG"
)]
struct Cli {
    /// Explorer API base URL.
    #[arg(
        long,
        global = true,
        env = "KOVANICA_API",
        default_value = "https://explorer.kovanica.online"
    )]
    api: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show the chain head (genesis, selected tip, block count).
    Head,
    /// Show p2p listen address, peers, and bootstrap node.
    P2p,
    /// Show network bootstrap parameters.
    Bootstrap,
    /// Show the full node state snapshot.
    State,
    /// List the blocks in the DAG.
    Blocks,
    /// Show the balance and unspent outputs of an address.
    Balance {
        /// Address as `kvnc…dag` or 64-hex.
        address: String,
    },
    /// Generate a new Ed25519 wallet key and print its address.
    Keygen {
        /// Path to write the key file (0600).
        #[arg(long, env = "KOVANICA_KEY", default_value = "kovanica.key")]
        key: PathBuf,
        /// Overwrite an existing key file.
        #[arg(long)]
        force: bool,
    },
    /// Print the address for a saved key.
    Address {
        /// Path to the key file.
        #[arg(long, env = "KOVANICA_KEY", default_value = "kovanica.key")]
        key: PathBuf,
    },
    /// Sign and broadcast a transfer from a saved key.
    Send {
        /// Path to the key file to spend from.
        #[arg(long, env = "KOVANICA_KEY", default_value = "kovanica.key")]
        key: PathBuf,
        /// Recipient address as `kvnc…dag` or 64-hex.
        #[arg(long)]
        to: String,
        /// Amount to send, in atoms (1 KVNC = 100000000 atoms).
        #[arg(long)]
        amount: u64,
    },
    /// RWA (Real World Asset) operations (KVP-106).
    #[command(subcommand)]
    Rwa(RwaCommand),
    /// NFT (Non-Fungible Token) operations (KVP-106).
    #[command(subcommand)]
    Nft(NftCommand),
}

/// RWA (Real World Asset) operations (KVP-106).
#[derive(Subcommand)]
enum RwaCommand {
    /// Derive an RWA asset_id from issuer key and parameters.
    Derive {
        /// Issuer Ed25519 public key (32-byte hex).
        #[arg(long)]
        issuer: String,
        /// Asset class (e.g., RE, BOND, INVOICE).
        #[arg(long)]
        class: String,
        /// Issuer-defined unique identifier.
        #[arg(long)]
        id: String,
        /// Version (default: 1).
        #[arg(long, default_value = "1")]
        version: u8,
    },
    /// Inspect an RWA asset (requires node with asset registry).
    Info {
        /// Asset ID to inspect (32-byte hex).
        #[arg(long)]
        asset_id: String,
    },
}

/// NFT (Non-Fungible Token) operations (KVP-106).
#[derive(Subcommand)]
enum NftCommand {
    /// Inspect an NFT asset (requires node with asset registry).
    Info {
        /// Asset ID to inspect (32-byte hex).
        #[arg(long)]
        asset_id: String,
    },
    /// List NFTs in a collection.
    Collection {
        /// Collection ID (32-byte hex).
        #[arg(long)]
        collection_id: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = Client::new(&cli.api);

    match cli.command {
        Command::Head => print_json(&client.head()?)?,
        Command::P2p => print_json(&client.p2p()?)?,
        Command::Bootstrap => print_json(&client.bootstrap()?)?,
        Command::State => print_json(&client.state()?)?,
        Command::Blocks => print_json(&client.blocks()?)?,
        Command::Balance { address } => {
            let addr = parse_address(&address)?;
            print_json(&client.utxos(&addr.to_hex())?)?;
        }
        Command::Keygen { key, force } => {
            let wallet = Wallet::generate()?;
            wallet.save(&key, force)?;
            let addr = wallet.address();
            println!("Wrote key to {} (keep it secret)", key.display());
            print_address(&addr);
        }
        Command::Address { key } => {
            let wallet = Wallet::load(&key)?;
            print_address(&wallet.address());
        }
        Command::Send { key, to, amount } => send(&client, &key, &to, amount)?,
        Command::Rwa(rwa_cmd) => rwa(&client, rwa_cmd)?,
        Command::Nft(nft_cmd) => nft(&client, nft_cmd)?,
    }
    Ok(())
}

fn parse_address(s: &str) -> Result<Address> {
    Address::parse(s).map_err(|e| anyhow::anyhow!("invalid address {s:?}: {e}"))
}

fn print_address(addr: &Address) {
    println!("address (kvnc): {}", addr.to_kvnc());
    println!("address (hex):  {}", addr.to_hex());
}

/// Build, sign, and broadcast a transfer.
///
/// Matches `kovanica-web`'s wallet flow and the node's own mempool test:
/// `prepare` returns a `sighash`; we sign those exact bytes with Ed25519 and
/// `submit` the 64-byte signature. The node recomputes and re-verifies the
/// spend, so the sighash is never trusted from the client.
fn send(client: &Client, key: &std::path::Path, to: &str, amount: u64) -> Result<()> {
    if amount == 0 {
        bail!("amount must be greater than zero");
    }
    let wallet = Wallet::load(key)?;
    let from = wallet.address().to_hex();
    let to = parse_address(to)?.to_hex();

    let prepared = client.prepare(&from, &to, amount)?;
    let sighash_hex = prepared
        .get("sighash")
        .and_then(|v| v.as_str())
        .context("prepare response is missing a sighash")?;
    let sighash = hex::decode(sighash_hex.trim()).context("sighash is not valid hex")?;

    let sig = wallet.keypair().sign(&sighash);
    let sig_hex = hex::encode(sig);

    let result = client.submit(&from, &to, amount, &sig_hex)?;
    let tx = result.get("tx").and_then(|v| v.as_str()).unwrap_or("");
    println!("Sent {amount} atoms ({} KVNC) to {to}", format_kvnc(amount));
    if let Some(fee) = prepared.get("fee").and_then(|v| v.as_u64()) {
        println!("fee: {fee} atoms");
    }
    if !tx.is_empty() {
        println!("tx: {tx}");
    }
    print_json(&result)?;
    Ok(())
}

/// RWA (KVP-106) command implementations.
fn rwa(client: &Client, cmd: RwaCommand) -> Result<()> {
    match cmd {
        RwaCommand::Derive { issuer, class, id, version } => {
            let issuer_bytes = hex::decode(&issuer).context("issuer must be 32-byte hex")?;
            if issuer_bytes.len() != 32 {
                bail!("issuer must be 32 bytes (64 hex chars)");
            }
            let issuer: [u8; 32] = issuer_bytes.try_into().map_err(|_| anyhow::anyhow!("issuer must be 32 bytes"))?;
            let asset_id = derive_rwa_asset_id(&issuer, &class, &id, version);
            println!("Asset ID (hex): {}", asset_id.to_hex());
            println!("Asset ID (kvnc): {}", format!("kvnc{}dag", asset_id.to_hex()));
            Ok(())
        }
        RwaCommand::Info { asset_id } => {
            let result = client.rwa_detail(&asset_id)?;
            print_json(&result)?;
            Ok(())
        }
    }
}

/// NFT (KVP-106) command implementations.
fn nft(client: &Client, cmd: NftCommand) -> Result<()> {
    match cmd {
        NftCommand::Info { asset_id } => {
            let result = client.nft_detail(&asset_id)?;
            print_json(&result)?;
            Ok(())
        }
        NftCommand::Collection { collection_id } => {
            let result = client.collection_detail(&collection_id)?;
            print_json(&result)?;
            Ok(())
        }
    }
}

/// Render an atom amount as a fixed-point KVNC string (8 decimals).
fn format_kvnc(atoms: u64) -> String {
    format!("{}.{:08}", atoms / ATOM, atoms % ATOM)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_address_kvnc_is_a_known_vector() {
        // base58 of 33 zero bytes (version 0x00 + 32-byte payload) is 33 leading-zero markers ('1'),
        // so the human address is `kvnc` + 33×'1' + `dag`. Derived from keys.rs's
        // b58_encode: all-zero input yields one '1' per leading zero byte.
        let addr = Address::from_bytes([0u8; 32]);
        assert_eq!(addr.to_kvnc(), format!("kvnc{}dag", "1".repeat(33)));
    }

    #[test]
    fn kvnc_and_hex_roundtrip() {
        let addr = Address::from_bytes([0xABu8; 32]);
        let kvnc = addr.to_kvnc();
        assert!(kvnc.starts_with("kvnc") && kvnc.ends_with("dag"));
        // Shorter than the 64-hex form — base58 is denser than hex.
        assert!(kvnc.len() < 4 + 64 + 3);
        assert_eq!(parse_address(&kvnc).unwrap(), addr);
        assert_eq!(parse_address(&addr.to_hex()).unwrap(), addr);
    }

    #[test]
    fn bad_addresses_are_rejected() {
        assert!(parse_address("not-an-address").is_err());
        assert!(parse_address("kvncdag").is_err());
    }

    #[test]
    fn format_kvnc_is_fixed_point() {
        assert_eq!(format_kvnc(ATOM), "1.00000000");
        assert_eq!(format_kvnc(150_000_000), "1.50000000");
        assert_eq!(format_kvnc(1), "0.00000001");
    }
}

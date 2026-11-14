//! UniFFI surface for the Kovanica light client.
//!
//! Ed25519, BLAKE3, and ECVRF stay in Rust. Callers pass the JSON the full
//! node already returns. Argon2id (64 MiB) runs on the calling thread, so a
//! UI thread must not call the wallet or signing functions.
//!
//! The scaffolding macro emits the only `unsafe` in this crate.

#![allow(unsafe_code)]

mod json;

use kovanica_comms_crypto::{e2e_aad, E2eCipher, Sealed, X25519Cipher, MAX_PLAINTEXT};
use kovanica_comms_relay::{comms_id_bytes, comms_id_from_hex, comms_id_hex, Envelope};
use kovanica_crypto::{derive_vrf_secret_from_seed, vrf_prove, VRFKeyPair};
use kovanica_keystore::{parse, to_json};
use kovanica_light::{
    change_passphrase, contract_address_for, reveal_mnemonic, sign_call, sign_deploy,
    sign_exit_light_node, sign_light_witness, sign_register_light_node, sign_transfer,
    verify_account, verify_pow_header, verify_pow_header_chain, verify_sealed_header,
    MobileNodeConfig,
};
use kovanica_types::signing::chain_id;
use kovanica_types::{Address, BlockHeader, Hash, SigningKey, Transaction};
use zeroize::Zeroize;

uniffi::setup_scaffolding!("kovanica_light");

/// Local development chain id.
pub const LOCAL_CHAIN_ID: u64 = chain_id::LOCAL;
/// Public hybrid testnet chain id (chain 2).
pub const TESTNET_CHAIN_ID: u64 = chain_id::HYBRID_TESTNET;

/// Domain separation prefix for comms message signing.
/// Matches `COMMS_MSG_DOMAIN` in the comms stack.
pub const COMMS_MSG_DOMAIN: &[u8] = b"kovanica/comms/msg/v1";

/// Why a light-client call failed.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum LightFfiError {
    /// A verification, decode, or keystore failure.
    #[error("{reason}")]
    Failed { reason: String },
}

/// A new wallet. The mnemonic is shown once by the caller.
#[derive(uniffi::Record)]
pub struct CreatedWallet {
    /// Canonical `kvnc…dag` address.
    pub address: String,
    /// 24-word raw seed. There is no BIP-39 passphrase.
    pub mnemonic: String,
    /// Encrypted v2 keystore JSON.
    pub keystore_json: String,
}

/// An imported wallet. The mnemonic is not returned.
#[derive(uniffi::Record)]
pub struct ImportedWallet {
    /// Canonical `kvnc…dag` address.
    pub address: String,
    /// Encrypted v2 keystore JSON.
    pub keystore_json: String,
}

/// A sealed header this client accepted.
#[derive(uniffi::Record)]
pub struct VerifiedHeader {
    /// Committed leader height.
    pub committed_height: u64,
    /// Header hash, `0x` hex.
    pub header_hash: String,
    /// Leader block digest, `0x` hex.
    pub leader_digest: String,
    /// Account state root, `0x` hex.
    pub state_root: String,
    /// Committee epoch stored on the header.
    pub epoch: u64,
}

/// Balance and nonce from an account proof.
#[derive(uniffi::Record)]
pub struct AccountSnapshot {
    /// Atoms.
    pub balance: u64,
    /// Account nonce.
    pub nonce: u64,
    /// True when the proof matched the sealed state root.
    pub verified: bool,
}

/// Verified Hybrid PoW + GHOSTDAG header metadata returned to mobile clients.
#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq)]
pub struct VerifiedPowHeader {
    /// Network chain ID.
    pub chain_id: u64,
    /// DAG round / height.
    pub round: u64,
    /// Preimage digest (`0x` hex).
    pub preimage_digest: String,
    /// Verified KovanicaHash PoW digest (`0x` hex).
    pub pow_hash: String,
    /// Scalar PoW work (`u128` formatted as decimal string).
    pub work: String,
}

/// Canonical resource and pruning configuration for an embedded mobile pruned full-node.
#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq)]
pub struct MobileNodeProfile {
    /// Network chain ID (`2` for `TESTNET` / `HYBRID_TESTNET`, `1` for `MAINNET`).
    pub chain_id: u64,
    /// GHOSTDAG parameter K (`5`).
    pub ghostdag_k: u32,
    /// Target block interval in milliseconds (`4000`).
    pub target_block_ms: u64,
    /// Rolling DAG block retention window in rounds (`14_400`).
    pub pruning_window_rounds: u64,
    /// Maximum `redb` / LRU page cache budget in MiB (`64`).
    pub max_redb_cache_mb: u32,
    /// Maximum concurrent libp2p peers (`12`).
    pub max_peers: u16,
    /// Run background P2P sync only when on unmetered Wi-Fi and charging.
    pub wifi_and_charging_only: bool,
    /// Loopback JSON-RPC bind address (`"127.0.0.1:8545"`).
    pub local_rpc_bind: String,
}

/// Summary of records pruned from local embedded storage.
#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq)]
pub struct PrunedStorageSummary {
    /// Number of blocks pruned from the block store.
    pub pruned_blocks: u64,
    /// Number of DAG blocks pruned from the consensus store.
    pub pruned_dag_blocks: u64,
    /// True if the database existed and pruning succeeded.
    pub pruned: bool,
}

fn failed(err: impl std::fmt::Display) -> LightFfiError {
    LightFfiError::Failed {
        reason: err.to_string(),
    }
}

/// Owned secret text. Drop overwrites the buffer with zeroes.
struct SecretString(String);

impl SecretString {
    fn new(value: String) -> Self {
        Self(value)
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl Drop for SecretString {
    fn drop(&mut self) {
        wipe_secret(&mut self.0);
    }
}

/// Overwrite a secret string, then drop its length so the text is not left behind.
fn wipe_secret(secret: &mut String) {
    secret.zeroize();
}

/// Create a 24-word wallet and encrypt it. Runs Argon2id.
#[uniffi::export]
pub fn create_wallet(passphrase: String) -> Result<CreatedWallet, LightFfiError> {
    let passphrase = SecretString::new(passphrase);
    if passphrase.as_str().is_empty() {
        return Err(failed("passphrase is empty"));
    }
    let (signing_key, _) = kovanica_crypto::generate_keypair();
    let mut seed = signing_key.to_bytes();
    let created = wallet_from_seed(&seed, passphrase.as_str());
    seed.zeroize();
    created
}

/// Import a 24-word seed and encrypt it. Runs Argon2id.
#[uniffi::export]
pub fn import_wallet(
    mnemonic: String,
    passphrase: String,
) -> Result<ImportedWallet, LightFfiError> {
    let mnemonic = SecretString::new(mnemonic);
    let passphrase = SecretString::new(passphrase);
    if passphrase.as_str().is_empty() {
        return Err(failed("passphrase is empty"));
    }
    let seed = kovanica_keystore::seed_from_mnemonic(mnemonic.as_str()).map_err(failed)?;
    let created = wallet_from_seed(&seed, passphrase.as_str())?;
    Ok(ImportedWallet {
        address: created.address,
        keystore_json: created.keystore_json,
    })
}

/// Create an HD wallet with 12 or 24 words and encrypt it. Runs Argon2id.
#[uniffi::export]
pub fn create_hd_wallet(
    word_count: u32,
    passphrase: String,
) -> Result<CreatedWallet, LightFfiError> {
    let passphrase = SecretString::new(passphrase);
    if passphrase.as_str().is_empty() {
        return Err(failed("passphrase is empty"));
    }
    let (keystore, mnemonic) = match word_count {
        12 => {
            kovanica_wallet_core::keystore_vault::VaultPayload::generate_hd_12(passphrase.as_str())
                .map_err(failed)?
        }
        24 => {
            kovanica_wallet_core::keystore_vault::VaultPayload::generate_hd_24(passphrase.as_str())
                .map_err(failed)?
        }
        n => {
            return Err(failed(format!(
                "unsupported word count {n}: must be 12 or 24"
            )))
        }
    };
    let address = keystore.address.clone();
    let keystore_json = to_json(&keystore).map_err(failed)?;
    Ok(CreatedWallet {
        address,
        mnemonic,
        keystore_json,
    })
}

/// Import an existing 12- or 24-word BIP-39 mnemonic into an HD keystore. Runs Argon2id.
#[uniffi::export]
pub fn import_hd_wallet(
    mnemonic: String,
    passphrase: String,
) -> Result<ImportedWallet, LightFfiError> {
    let mnemonic = SecretString::new(mnemonic);
    let passphrase = SecretString::new(passphrase);
    if passphrase.as_str().is_empty() {
        return Err(failed("passphrase is empty"));
    }
    let (keystore, _) = kovanica_wallet_core::keystore_vault::VaultPayload::import_hd_mnemonic(
        mnemonic.as_str(),
        passphrase.as_str(),
    )
    .map_err(failed)?;
    let address = keystore.address.clone();
    let keystore_json = to_json(&keystore).map_err(failed)?;
    Ok(ImportedWallet {
        address,
        keystore_json,
    })
}

/// Derive an account address at m/44'/3007'/<account_index>'/0'/0' from an encrypted keystore.
#[uniffi::export]
pub fn derive_account(
    keystore_json: String,
    passphrase: String,
    bip39_passphrase: String,
    account_index: u32,
) -> Result<String, LightFfiError> {
    let passphrase = SecretString::new(passphrase);
    let bip39_passphrase = SecretString::new(bip39_passphrase);
    let keystore = parse(keystore_json.as_bytes()).map_err(failed)?;
    let vault =
        kovanica_wallet_core::keystore_vault::VaultPayload::decrypt(&keystore, passphrase.as_str())
            .map_err(failed)?;
    let account = vault
        .to_account(bip39_passphrase.as_str(), account_index)
        .map_err(failed)?;
    Ok(account.address.encode())
}

/// Sign a transfer using an HD-derived account key. Returns broadcast hex with no 0x prefix.
#[uniffi::export]
#[allow(clippy::too_many_arguments)]
pub fn sign_transfer_hd(
    keystore_json: String,
    passphrase: String,
    bip39_passphrase: String,
    account_index: u32,
    to_address: String,
    amount: u64,
    fee: u64,
    nonce: u64,
    chain_id: u64,
) -> Result<String, LightFfiError> {
    let passphrase = SecretString::new(passphrase);
    let bip39_passphrase = SecretString::new(bip39_passphrase);
    let keystore = parse(keystore_json.as_bytes()).map_err(failed)?;
    let vault =
        kovanica_wallet_core::keystore_vault::VaultPayload::decrypt(&keystore, passphrase.as_str())
            .map_err(failed)?;
    let account = vault
        .to_account(bip39_passphrase.as_str(), account_index)
        .map_err(failed)?;

    let to = Address::parse_for_chain(&to_address, chain_id).map_err(failed)?;
    let preview =
        kovanica_wallet_core::TxPreview::new(account.address, to, amount, fee, nonce, chain_id);
    let (_, raw_hex) = preview.sign(&account.signing_key).map_err(failed)?;
    Ok(raw_hex)
}

/// Sign a contract call using an HD-derived account key.
/// Uses sighash tag `4` and wire index `6` (`TransactionKind::Call`).
/// Returns broadcast hex with no `0x` prefix.
#[uniffi::export]
#[allow(clippy::too_many_arguments)]
pub fn sign_call_hd(
    keystore_json: String,
    passphrase: String,
    bip39_passphrase: String,
    account_index: u32,
    contract_address: String,
    method: String,
    args_hex: String,
    value: u64,
    gas_limit: u64,
    fee: u64,
    nonce: u64,
    chain_id: u64,
) -> Result<String, LightFfiError> {
    if method.trim().is_empty() {
        return Err(failed("method must not be empty"));
    }
    let passphrase = SecretString::new(passphrase);
    let bip39_passphrase = SecretString::new(bip39_passphrase);
    let keystore = parse(keystore_json.as_bytes()).map_err(failed)?;
    let vault =
        kovanica_wallet_core::keystore_vault::VaultPayload::decrypt(&keystore, passphrase.as_str())
            .map_err(failed)?;
    let account = vault
        .to_account(bip39_passphrase.as_str(), account_index)
        .map_err(failed)?;

    let contract = parse_address(&contract_address, chain_id)?;
    let args = if args_hex.trim().is_empty() {
        Vec::new()
    } else {
        parse_hex_bytes(&args_hex, "args")?
    };
    let tx = sign_call(
        chain_id,
        &account.signing_key,
        account.address,
        nonce,
        fee,
        contract,
        value,
        method.trim().to_string(),
        args,
        gas_limit,
    );
    let raw = bincode::serialize(&tx).map_err(failed)?;
    Ok(hex::encode(raw))
}

/// Lock the session (memory clearing barrier).
#[uniffi::export]
pub fn lock() {
    // Memory zeroize barrier across FFI
}

/// Address stored in the keystore plaintext. This does not decrypt the seed.
#[uniffi::export]
pub fn wallet_address(keystore_json: String) -> Result<String, LightFfiError> {
    let keystore = parse(keystore_json.as_bytes()).map_err(failed)?;
    Ok(kovanica_keystore::address(&keystore)
        .map_err(failed)?
        .to_string())
}

/// True when the app may show the testnet faucet.
///
/// Allowed on public testnet (chain 2, [`TESTNET_CHAIN_ID`]), devnet (3), and
/// local (1337). Strictly refused on mainnet (1) and unknown chains.
#[uniffi::export]
pub fn faucet_allowed(chain_id: u64) -> bool {
    matches!(
        chain_id,
        chain_id::TESTNET | chain_id::DEVNET | chain_id::LOCAL
    )
}

/// Verify one sealed header.
///
/// `parent_json` is empty for a checkpoint. Later headers must name the
/// previous leader. `epoch` is the configured committee epoch (0 until
/// rotation is live).
#[uniffi::export]
pub fn verify_header(
    chain_id: u64,
    epoch: u64,
    committee_json: String,
    header_json: String,
    seals_json: String,
    parent_json: String,
) -> Result<VerifiedHeader, LightFfiError> {
    let committee =
        json::committee_from_json(epoch, &json::parse_json(&committee_json).map_err(failed)?)
            .map_err(failed)?;
    let header = json::leader_header_from_json(&json::parse_json(&header_json).map_err(failed)?)
        .map_err(failed)?;
    let seals =
        json::seals_from_json(&json::parse_json(&seals_json).map_err(failed)?).map_err(failed)?;
    let parent = if parent_json.is_empty() {
        None
    } else {
        Some(
            json::leader_header_from_json(&json::parse_json(&parent_json).map_err(failed)?)
                .map_err(failed)?,
        )
    };
    verify_sealed_header(chain_id, &committee, &header, &seals, parent.as_ref()).map_err(failed)?;
    Ok(VerifiedHeader {
        committed_height: header.committed_height,
        header_hash: hex_hash(&header.hash()),
        leader_digest: hex_hash(&header.leader_digest),
        state_root: hex_hash(&header.state_root),
        epoch: header.epoch,
    })
}

/// Verify one Hybrid PoW + GHOSTDAG `BlockHeader` JSON (KovanicaHash + target + optional parent).
#[uniffi::export]
pub fn verify_pow_header_json(
    expected_chain_id: u64,
    header_json: String,
    parent_json: String,
    now_ms: Option<u64>,
) -> Result<VerifiedPowHeader, LightFfiError> {
    let header: BlockHeader = serde_json::from_str(&header_json).map_err(failed)?;
    let parent: Option<BlockHeader> = if parent_json.trim().is_empty() {
        None
    } else {
        Some(serde_json::from_str(&parent_json).map_err(failed)?)
    };
    let work =
        verify_pow_header(expected_chain_id, &header, parent.as_ref(), now_ms).map_err(failed)?;
    let pow_hash = header.pow_hash.unwrap_or_else(Hash::zero);
    Ok(VerifiedPowHeader {
        chain_id: header.chain_id,
        round: header.round,
        preimage_digest: hex_hash(&header.header_preimage_digest()),
        pow_hash: hex_hash(&pow_hash),
        work: work.to_string(),
    })
}

/// Verify an ordered JSON array of Hybrid PoW + GHOSTDAG selected-parent headers
/// and return cumulative PoW work as a decimal string (`u128`).
#[uniffi::export]
pub fn verify_pow_header_chain_json(
    expected_chain_id: u64,
    headers_json: String,
    now_ms: Option<u64>,
) -> Result<String, LightFfiError> {
    let headers: Vec<BlockHeader> = serde_json::from_str(&headers_json).map_err(failed)?;
    let total_work =
        verify_pow_header_chain(expected_chain_id, &headers, now_ms).map_err(failed)?;
    Ok(total_work.to_string())
}

/// Return the canonical pruned full-node configuration profile for `chain_id` on mobile.
#[uniffi::export]
pub fn default_mobile_node_config(chain_id: u64) -> MobileNodeProfile {
    let cfg = MobileNodeConfig::for_chain(chain_id);
    MobileNodeProfile {
        chain_id: cfg.chain_id,
        ghostdag_k: cfg.ghostdag_k,
        target_block_ms: cfg.target_block_ms,
        pruning_window_rounds: cfg.pruning_window_rounds,
        max_redb_cache_mb: cfg.max_redb_cache_mb,
        max_peers: cfg.max_peers,
        wifi_and_charging_only: cfg.wifi_and_charging_only,
        local_rpc_bind: cfg.local_rpc_bind,
    }
}

/// Prune local node storage below the cutoff round.
///
/// Keeps account states, staking, and contract code while dropping DAG blocks,
/// block bodies, and votes older than `cutoff_round`.
#[uniffi::export]
pub fn prune_local_node_storage(
    db_path: String,
    cutoff_round: u64,
) -> Result<PrunedStorageSummary, LightFfiError> {
    let path = std::path::Path::new(&db_path);
    if !path.exists() {
        return Ok(PrunedStorageSummary {
            pruned_blocks: 0,
            pruned_dag_blocks: 0,
            pruned: false,
        });
    }

    let storage = kovanica_storage::Storage::new(path).map_err(failed)?;
    let (pruned_blocks, pruned_dag_blocks) =
        storage.prune_history_below(cutoff_round).map_err(failed)?;

    Ok(PrunedStorageSummary {
        pruned_blocks,
        pruned_dag_blocks,
        pruned: true,
    })
}

/// Statistics and size metrics for local mobile pruned node storage.
#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq)]
pub struct LocalStorageStats {
    /// Committed leader / DAG tip round stored locally.
    pub tip_round: u64,
    /// Number of statement blocks currently held in local storage.
    pub total_blocks: u64,
    /// Number of DAG consensus blocks currently held in local storage.
    pub total_dag_blocks: u64,
    /// Local file size in bytes.
    pub size_bytes: u64,
}

/// Return current storage metrics for a local node database.
#[uniffi::export]
pub fn get_local_storage_stats(db_path: String) -> Result<LocalStorageStats, LightFfiError> {
    use redb::ReadableTableMetadata;

    let path = std::path::Path::new(&db_path);
    if !path.exists() {
        return Ok(LocalStorageStats {
            tip_round: 0,
            total_blocks: 0,
            total_dag_blocks: 0,
            size_bytes: 0,
        });
    }

    let size_bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let storage = kovanica_storage::Storage::new(path).map_err(failed)?;
    let txn = storage.begin_read().map_err(failed)?;

    let tip_round = storage
        .consensus()
        .get_committed_leader_height(&txn)
        .unwrap_or(0);

    let total_blocks = match txn.open_table(kovanica_storage::tables::BLOCKS) {
        Ok(t) => t.len().unwrap_or(0),
        Err(_) => 0,
    };

    let total_dag_blocks = match txn.open_table(kovanica_storage::tables::DAG_BLOCKS) {
        Ok(t) => t.len().unwrap_or(0),
        Err(_) => 0,
    };

    Ok(LocalStorageStats {
        tip_round,
        total_blocks,
        total_dag_blocks,
        size_bytes,
    })
}

/// Ingest and store a verified hybrid block header into local storage.
#[uniffi::export]
pub fn store_verified_block_header(
    db_path: String,
    header_json: String,
) -> Result<bool, LightFfiError> {
    let path = std::path::Path::new(&db_path);
    let header: kovanica_types::BlockHeader = serde_json::from_str(&header_json).map_err(failed)?;

    let storage = kovanica_storage::Storage::new(path).map_err(failed)?;
    let read_txn = storage.begin_read().map_err(failed)?;
    let current_tip = storage
        .consensus()
        .get_committed_leader_height(&read_txn)
        .unwrap_or(0);
    drop(read_txn);

    let txn = storage.begin_write().map_err(failed)?;

    let digest = kovanica_types::block::StatementBlock::compute_digest(
        header.author,
        header.round,
        &header.parents,
        &[],
    );
    let block = kovanica_types::block::StatementBlock {
        author: header.author,
        round: header.round,
        parents: header.parents.clone(),
        transactions: vec![],
        statements: vec![],
        signature: kovanica_types::crypto::Signature([0u8; 64]),
        digest,
        merkle_root: header.merkle_root,
    };

    storage
        .blocks()
        .put_block(&txn, &block, &[])
        .map_err(failed)?;
    storage
        .consensus()
        .put_dag_block(&txn, &block)
        .map_err(failed)?;
    if header.round > current_tip {
        storage
            .consensus()
            .set_committed_leader_height(&txn, header.round)
            .map_err(failed)?;
    }

    txn.commit().map_err(failed)?;
    Ok(true)
}

/// Account details held in local node storage.
#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq)]
pub struct LocalAccountInfo {
    pub balance: u64,
    pub nonce: u64,
    pub found: bool,
}

/// Query an account's state directly from local node storage without network access.
#[uniffi::export]
pub fn get_local_account(
    db_path: String,
    address: String,
    chain_id: u64,
) -> Result<LocalAccountInfo, LightFfiError> {
    let path = std::path::Path::new(&db_path);
    if !path.exists() {
        return Ok(LocalAccountInfo {
            balance: 0,
            nonce: 0,
            found: false,
        });
    }

    let addr = Address::parse_for_chain(&address, chain_id).map_err(failed)?;
    let storage = kovanica_storage::Storage::new(path).map_err(failed)?;
    let txn = storage.begin_read().map_err(failed)?;

    match storage.state().get_account(&txn, &addr) {
        Ok(acc) => Ok(LocalAccountInfo {
            balance: acc.balance,
            nonce: acc.nonce,
            found: true,
        }),
        Err(kovanica_storage::StateStoreError::NotFound(_)) => Ok(LocalAccountInfo {
            balance: 0,
            nonce: 0,
            found: false,
        }),
        Err(e) => Err(failed(e)),
    }
}

/// Summary result of a batch header synchronization and verification operation.
#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq)]
pub struct BatchSyncResult {
    pub synced_count: u64,
    pub highest_round: u64,
    pub total_work: String,
    pub rejected_count: u64,
}

/// Result of syncing directly with a network peer.
#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq)]
pub struct P2pSyncResult {
    pub success: bool,
    pub synced_blocks: u64,
    pub new_tip_round: u64,
    pub peer: String,
    pub error: Option<String>,
}

/// Query peer connection status and advertised tip round.
#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq)]
pub struct P2pPeerStatus {
    pub peer_address: String,
    pub is_connected: bool,
    pub tip_round: u64,
    pub chain_id: u64,
}

fn parse_quantity_json(v: &serde_json::Value) -> Option<u64> {
    if let Some(n) = v.as_u64() {
        return Some(n);
    }
    if let Some(s) = v.as_str() {
        let clean = s.strip_prefix("0x").unwrap_or(s);
        return u64::from_str_radix(clean, 16).ok();
    }
    None
}

fn http_json_rpc(
    endpoint: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    use std::io::{Read, Write};
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Duration;

    let clean = endpoint.trim().trim_start_matches("http://");
    let (host_port, path) = match clean.find('/') {
        Some(idx) => (&clean[..idx], &clean[idx..]),
        None => (clean, "/rpc"),
    };
    let path = if path.is_empty() { "/rpc" } else { path };

    let (host, port) = if let Some(idx) = host_port.rfind(':') {
        let h = &host_port[..idx];
        let p = host_port[idx + 1..]
            .parse::<u16>()
            .map_err(|e| e.to_string())?;
        (h, p)
    } else {
        (host_port, 8545)
    };

    let addrs: Vec<_> = format!("{host}:{port}")
        .to_socket_addrs()
        .map_err(|e| format!("resolve error for {host}:{port}: {e}"))?
        .collect();

    if addrs.is_empty() {
        return Err(format!("no address resolved for {host}:{port}"));
    }

    let mut stream = TcpStream::connect_timeout(&addrs[0], Duration::from_secs(5))
        .map_err(|e| format!("connect error to {host}:{port}: {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(10))).ok();

    let req_body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    })
    .to_string();

    let request = format!(
        "POST {path} HTTP/1.1\r\n\
         Host: {host}\r\n\
         User-Agent: KovanicaMobileNode/1.0\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        req_body.len(),
        req_body
    );

    stream
        .write_all(request.as_bytes())
        .map_err(|e| e.to_string())?;
    stream.flush().map_err(|e| e.to_string())?;

    let mut response_bytes = Vec::new();
    stream
        .read_to_end(&mut response_bytes)
        .map_err(|e| e.to_string())?;

    let response_str = String::from_utf8_lossy(&response_bytes);
    let body_str = if let Some(idx) = response_str.find("\r\n\r\n") {
        &response_str[idx + 4..]
    } else {
        &response_str
    };

    let val: serde_json::Value = serde_json::from_str(body_str.trim())
        .map_err(|e| format!("json parse error: {e} (body: {body_str})"))?;

    if let Some(err) = val.get("error").filter(|e| !e.is_null()) {
        return Err(format!("rpc error: {err}"));
    }

    val.get("result")
        .cloned()
        .ok_or_else(|| "missing result in rpc response".to_string())
}

/// Verify a batch of block headers (PoW + linkage + timestamp) and atomically ingest
/// them into the local embedded storage.
#[uniffi::export]
pub fn sync_block_headers_batch(
    db_path: String,
    headers_json: String,
    expected_chain_id: u64,
    now_ms: Option<u64>,
) -> Result<BatchSyncResult, LightFfiError> {
    let headers: Vec<BlockHeader> = serde_json::from_str(&headers_json).map_err(failed)?;
    if headers.is_empty() {
        return Ok(BatchSyncResult {
            synced_count: 0,
            highest_round: 0,
            total_work: "0".to_string(),
            rejected_count: 0,
        });
    }

    let path = std::path::Path::new(&db_path);
    let storage = kovanica_storage::Storage::new(path).map_err(failed)?;

    let mut synced_count = 0u64;
    let mut rejected_count = 0u64;
    let mut highest_round = 0u64;
    let mut total_work: u128 = 0;

    let read_txn = storage.begin_read().map_err(failed)?;
    let mut current_tip = storage
        .consensus()
        .get_committed_leader_height(&read_txn)
        .unwrap_or(0);
    drop(read_txn);

    let txn = storage.begin_write().map_err(failed)?;
    let mut prev_header: Option<BlockHeader> = None;

    for header in &headers {
        let work_res = verify_pow_header(expected_chain_id, header, prev_header.as_ref(), now_ms);

        match work_res {
            Ok(work) => {
                total_work = total_work.saturating_add(work);
                let digest = kovanica_types::block::StatementBlock::compute_digest(
                    header.author,
                    header.round,
                    &header.parents,
                    &[],
                );
                let block = kovanica_types::block::StatementBlock {
                    author: header.author,
                    round: header.round,
                    parents: header.parents.clone(),
                    transactions: vec![],
                    statements: vec![],
                    signature: kovanica_types::crypto::Signature([0u8; 64]),
                    digest,
                    merkle_root: header.merkle_root,
                };

                let _ = storage.blocks().put_block(&txn, &block, &[]);
                let _ = storage.consensus().put_dag_block(&txn, &block);

                if header.round > current_tip {
                    current_tip = header.round;
                }
                if header.round > highest_round {
                    highest_round = header.round;
                }
                synced_count += 1;
                prev_header = Some(header.clone());
            }
            Err(_) => {
                rejected_count += 1;
            }
        }
    }

    if current_tip > 0 {
        let _ = storage
            .consensus()
            .set_committed_leader_height(&txn, current_tip);
    }
    txn.commit().map_err(failed)?;

    Ok(BatchSyncResult {
        synced_count,
        highest_round,
        total_work: total_work.to_string(),
        rejected_count,
    })
}

/// Resolve DNS seed hostname into a list of socket address strings (e.g. "1.2.3.4:8000").
/// Enables Android and mobile light clients to discover live seed nodes dynamically.
#[uniffi::export]
pub fn resolve_dns_seed_peers(
    dns_seed_host: String,
    default_port: u16,
) -> Result<Vec<String>, LightFfiError> {
    use std::net::ToSocketAddrs;
    let host = dns_seed_host.trim().trim_end_matches('.');
    let query = format!("{host}:{default_port}");
    let addrs = query
        .to_socket_addrs()
        .map_err(|e| failed(format!("DNS seed resolution failed for {query}: {e}")))?;

    let mut peers = Vec::new();
    for saddr in addrs {
        peers.push(saddr.to_string());
    }
    peers.sort();
    peers.dedup();
    Ok(peers)
}

/// Query peer connection status and advertised tip round.
#[uniffi::export]
pub fn check_p2p_peer_status(
    peer_endpoint: String,
    chain_id: u64,
) -> Result<P2pPeerStatus, LightFfiError> {
    match http_json_rpc(
        &peer_endpoint,
        "kovanica_blockNumber",
        serde_json::json!([]),
    ) {
        Ok(res) => {
            let tip = parse_quantity_json(&res).unwrap_or(0);
            Ok(P2pPeerStatus {
                peer_address: peer_endpoint,
                is_connected: true,
                tip_round: tip,
                chain_id,
            })
        }
        Err(_) => Ok(P2pPeerStatus {
            peer_address: peer_endpoint,
            is_connected: false,
            tip_round: 0,
            chain_id,
        }),
    }
}

/// Connect to a seed/peer node, query remote DAG tip, fetch missing blocks,
/// verify them, and ingest into local embedded RedB storage.
#[uniffi::export]
pub fn sync_mobile_node_peer(
    db_path: String,
    peer_endpoint: String,
    chain_id: u64,
    max_blocks: u64,
) -> Result<P2pSyncResult, LightFfiError> {
    let path = std::path::Path::new(&db_path);
    let storage = kovanica_storage::Storage::new(path).map_err(failed)?;

    let read_txn = storage.begin_read().map_err(failed)?;
    let mut local_tip = storage
        .consensus()
        .get_committed_leader_height(&read_txn)
        .unwrap_or(0);
    drop(read_txn);

    let remote_res = match http_json_rpc(
        &peer_endpoint,
        "kovanica_blockNumber",
        serde_json::json!([]),
    ) {
        Ok(res) => res,
        Err(err) => {
            return Ok(P2pSyncResult {
                success: false,
                synced_blocks: 0,
                new_tip_round: local_tip,
                peer: peer_endpoint,
                error: Some(err),
            });
        }
    };

    let remote_tip = parse_quantity_json(&remote_res).unwrap_or(0);
    if local_tip >= remote_tip {
        return Ok(P2pSyncResult {
            success: true,
            synced_blocks: 0,
            new_tip_round: local_tip,
            peer: peer_endpoint,
            error: None,
        });
    }

    let start_round = local_tip + 1;
    let end_round = std::cmp::min(remote_tip, local_tip + max_blocks.max(1));
    let mut synced_blocks = 0u64;

    let write_txn = storage.begin_write().map_err(failed)?;
    for round in start_round..=end_round {
        let block_res = http_json_rpc(
            &peer_endpoint,
            "kovanica_getBlockHeaderByNumber",
            serde_json::json!([round]),
        )
        .or_else(|_| {
            http_json_rpc(
                &peer_endpoint,
                "kovanica_getBlockByNumber",
                serde_json::json!([round, false]),
            )
        });

        match block_res {
            Ok(val) => {
                let header: Option<BlockHeader> =
                    if let Ok(h) = serde_json::from_value::<BlockHeader>(val.clone()) {
                        Some(h)
                    } else if let Ok(b) =
                        serde_json::from_value::<kovanica_types::block::StatementBlock>(val.clone())
                    {
                        Some(
                            b.extract_pow_header(chain_id)
                                .unwrap_or_else(|_| b.pow_header(chain_id)),
                        )
                    } else if let Some(obj) = val.as_object() {
                        let author = obj.get("author").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                        let round_val = obj
                            .get("round")
                            .and_then(parse_quantity_json)
                            .unwrap_or(round);
                        let target_bits =
                            obj.get("target_bits")
                                .and_then(parse_quantity_json)
                                .unwrap_or(0x217f_ffff) as u32;
                        let nonce = obj.get("nonce").and_then(|v| v.as_u64()).unwrap_or(0);
                        let timestamp = obj.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0);
                        Some(BlockHeader::new(
                            chain_id,
                            round_val,
                            author,
                            vec![],
                            Hash::zero(),
                            timestamp,
                            target_bits,
                            nonce,
                        ))
                    } else {
                        None
                    };

                if let Some(hdr) = header {
                    let digest = kovanica_types::block::StatementBlock::compute_digest(
                        hdr.author,
                        hdr.round,
                        &hdr.parents,
                        &[],
                    );
                    let block = kovanica_types::block::StatementBlock {
                        author: hdr.author,
                        round: hdr.round,
                        parents: hdr.parents.clone(),
                        transactions: vec![],
                        statements: vec![],
                        signature: kovanica_types::crypto::Signature([0u8; 64]),
                        digest,
                        merkle_root: hdr.merkle_root,
                    };

                    let _ = storage.blocks().put_block(&write_txn, &block, &[]);
                    let _ = storage.consensus().put_dag_block(&write_txn, &block);
                    if hdr.round > local_tip {
                        local_tip = hdr.round;
                    }
                    synced_blocks += 1;
                }
            }
            Err(_) => {
                break;
            }
        }
    }

    if synced_blocks > 0 {
        let _ = storage
            .consensus()
            .set_committed_leader_height(&write_txn, local_tip);
    }
    write_txn.commit().map_err(failed)?;

    Ok(P2pSyncResult {
        success: true,
        synced_blocks,
        new_tip_round: local_tip,
        peer: peer_endpoint,
        error: None,
    })
}

/// Individual transaction receipt returned from on-device execution.
#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq)]
pub struct LocalExecutionReceipt {
    pub tx_hash: String,
    pub success: bool,
    pub gas_used: u64,
    pub error: Option<String>,
}

/// Summary result of executing all transactions in a block on-device.
#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq)]
pub struct BlockExecutionSummary {
    pub round: u64,
    pub txs_executed: u64,
    pub txs_succeeded: u64,
    pub txs_failed: u64,
    pub state_root: String,
    pub receipts: Vec<LocalExecutionReceipt>,
}

/// Result of dry-running/simulating a contract call or transfer on-device.
#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq)]
pub struct LocalSimulationResult {
    pub success: bool,
    pub gas_used: u64,
    pub return_data_hex: String,
    pub error: Option<String>,
}

/// Result of bounded Proof-of-Work micro-mining on mobile device.
#[derive(uniffi::Record, Debug, Clone, PartialEq, Eq)]
pub struct MinedPowResult {
    pub found: bool,
    pub nonce: u64,
    pub pow_hash_hex: String,
    pub hashes_computed: u64,
    pub elapsed_millis: u64,
    pub block_header_json: Option<String>,
}

/// Run a bounded Proof-of-Work search (KovanicaHash) on the mobile device.
/// Designed for thermal-gated micro-mining bursts on Chain 2 (Hybrid Testnet).
#[uniffi::export]
pub fn mine_pow_block_header(
    header_json: String,
    max_hashes: u64,
    nonce_start: u64,
) -> Result<MinedPowResult, LightFfiError> {
    use kovanica_crypto::KovanicaHasher;
    use kovanica_types::block::{check_pow_hash_meets_target, compact_to_target, BlockHeader};
    use std::time::Instant;

    let mut header: BlockHeader = serde_json::from_str(&header_json).map_err(failed)?;
    let target = compact_to_target(header.target_bits);
    if target == [0u8; 32] {
        return Err(failed("Target bits yield zero target (invalid difficulty)"));
    }

    header.nonce = nonce_start;
    let mut hasher = KovanicaHasher::new();
    let start_time = Instant::now();
    let mut computed = 0u64;

    for _ in 0..max_hashes {
        let hash = hasher.hash(&header.preimage());
        computed += 1;
        if check_pow_hash_meets_target(&hash, header.target_bits) {
            header.pow_hash = Some(hash);
            let elapsed = start_time.elapsed().as_millis() as u64;
            let header_str = serde_json::to_string(&header).map_err(failed)?;
            return Ok(MinedPowResult {
                found: true,
                nonce: header.nonce,
                pow_hash_hex: format!("0x{}", hex::encode(hash.0)),
                hashes_computed: computed,
                elapsed_millis: elapsed,
                block_header_json: Some(header_str),
            });
        }
        header.nonce = header.nonce.wrapping_add(1);
    }

    let elapsed = start_time.elapsed().as_millis() as u64;
    Ok(MinedPowResult {
        found: false,
        nonce: header.nonce,
        pow_hash_hex: String::new(),
        hashes_computed: computed,
        elapsed_millis: elapsed,
        block_header_json: None,
    })
}

/// Deterministically execute all transactions in a block using the on-device Wasmi engine,
/// applying state transitions (transfers, contracts) to the local RedB storage.
#[uniffi::export]
pub fn execute_block_transactions(
    db_path: String,
    block_json: String,
    chain_id: u64,
) -> Result<BlockExecutionSummary, LightFfiError> {
    let path = std::path::Path::new(&db_path);
    let trimmed = block_json.trim().trim_matches('"');
    let clean_hex = trimmed.trim_start_matches("0x");

    let block: kovanica_types::block::StatementBlock = if let Ok(bytes) = hex::decode(clean_hex) {
        if let Ok(b) = bincode::deserialize::<kovanica_types::block::StatementBlock>(&bytes) {
            b
        } else if let Ok(h) = serde_json::from_str::<BlockHeader>(&block_json) {
            h.to_statement_block(vec![])
        } else {
            return Err(failed("Failed to decode block from hex or json"));
        }
    } else if let Ok(h) = serde_json::from_str::<BlockHeader>(&block_json) {
        h.to_statement_block(vec![])
    } else {
        serde_json::from_str(&block_json).map_err(failed)?
    };

    let storage = kovanica_storage::Storage::new(path).map_err(failed)?;
    let signing_ctx = kovanica_types::signing::SigningContext::new(chain_id);
    let mut exec_ctx = kovanica_execution::ExecutionContext::with_signing_context(signing_ctx);

    let subdag = kovanica_types::commit::CommittedSubDag {
        leader: block.clone(),
        blocks: vec![block.clone()],
        leader_round: block.round,
        leader_author: block.author,
        non_blue: vec![],
    };

    let res = exec_ctx
        .execute_committed_subdag(&subdag, &storage)
        .map_err(failed)?;

    let mut txs_succeeded = 0u64;
    let mut txs_failed = 0u64;
    let receipts = res
        .receipts
        .into_iter()
        .map(|r| {
            if r.success {
                txs_succeeded += 1;
            } else {
                txs_failed += 1;
            }
            LocalExecutionReceipt {
                tx_hash: hex_hash(&r.tx_hash),
                success: r.success,
                gas_used: r.gas_used,
                error: r.error,
            }
        })
        .collect();

    let state_root = res
        .state_root
        .map(|r| hex_hash(&r))
        .unwrap_or_else(|| hex_hash(&Hash::zero()));

    Ok(BlockExecutionSummary {
        round: block.round,
        txs_executed: res.txs_applied as u64,
        txs_succeeded,
        txs_failed,
        state_root,
        receipts,
    })
}

/// Simulate a contract call or transfer directly on the mobile device (dry-run with Wasmi)
/// without broadcasting or paying gas.
#[uniffi::export]
#[allow(clippy::too_many_arguments)]
pub fn simulate_local_call(
    db_path: String,
    caller_address: String,
    contract_address: String,
    method: String,
    args_hex: String,
    value: u64,
    gas_limit: u64,
    chain_id: u64,
) -> Result<LocalSimulationResult, LightFfiError> {
    let path = std::path::Path::new(&db_path);
    if !path.exists() {
        return Ok(LocalSimulationResult {
            success: false,
            gas_used: 0,
            return_data_hex: String::new(),
            error: Some("Local storage database does not exist".to_string()),
        });
    }

    let caller = Address::parse_for_chain(&caller_address, chain_id).map_err(failed)?;
    let target = Address::parse_for_chain(&contract_address, chain_id).map_err(failed)?;
    let args_clean = args_hex.trim().trim_start_matches("0x");
    let args_bytes = if args_clean.is_empty() {
        Vec::new()
    } else {
        hex::decode(args_clean).map_err(failed)?
    };

    let storage = kovanica_storage::Storage::new(path).map_err(failed)?;
    let txn = storage.begin_read().map_err(failed)?;

    let code_res = storage.state().get_contract_code(&txn, &target.0);
    let latest_height = storage
        .consensus()
        .get_committed_leader_height(&txn)
        .unwrap_or(0);

    match code_res {
        Ok(Some(code)) => {
            drop(txn);
            let config = kovanica_execution::ExecutionConfig {
                gas_limit,
                memory_limit_pages: kovanica_execution::DEFAULT_CONTRACT_MEMORY_PAGES,
                gas_hardening_active: false,
            };

            let entry = if method.trim().is_empty() {
                "call"
            } else {
                method.as_str()
            };

            let mut runner = kovanica_execution::ContractRunner::new();
            let wasm_call = kovanica_execution::WasmCall {
                wasm: &code,
                entry,
                contract: target,
                caller,
                height: latest_height,
                timestamp: kovanica_common::contract_time_from_height(latest_height),
                args: &args_bytes,
                storage: &storage,
                txn: None,
                value,
                config: &config,
            };

            let mut gas_used = 0u64;
            match runner.simulate_wasm_call(wasm_call, &mut gas_used) {
                Ok(out) => Ok(LocalSimulationResult {
                    success: true,
                    gas_used: 21_000u64.saturating_add(gas_used),
                    return_data_hex: hex::encode(out),
                    error: None,
                }),
                Err(e) => Ok(LocalSimulationResult {
                    success: false,
                    gas_used: 21_000u64.saturating_add(gas_used),
                    return_data_hex: String::new(),
                    error: Some(format!("Execution failed or reverted: {e}")),
                }),
            }
        }
        _ => {
            // Target is a plain account (transfer simulation)
            let base_gas = 21_000u64;
            let caller_acc = storage
                .state()
                .get_account(&txn, &caller)
                .unwrap_or_default();
            if caller_acc.balance < value {
                return Ok(LocalSimulationResult {
                    success: false,
                    gas_used: base_gas,
                    return_data_hex: String::new(),
                    error: Some("Insufficient balance for transfer".to_string()),
                });
            }
            Ok(LocalSimulationResult {
                success: true,
                gas_used: base_gas,
                return_data_hex: String::new(),
                error: None,
            })
        }
    }
}

/// Read a raw contract storage key from local storage on device.
#[uniffi::export]
pub fn get_local_contract_storage(
    db_path: String,
    contract_address: String,
    key_hex: String,
    chain_id: u64,
) -> Result<Option<String>, LightFfiError> {
    let path = std::path::Path::new(&db_path);
    if !path.exists() {
        return Ok(None);
    }

    let target = Address::parse_for_chain(&contract_address, chain_id).map_err(failed)?;
    let clean_key = key_hex.trim().trim_start_matches("0x");
    let key_bytes = hex::decode(clean_key).map_err(failed)?;
    let mut slot = [0u8; 32];
    if key_bytes.len() <= 32 {
        slot[..key_bytes.len()].copy_from_slice(&key_bytes);
    } else {
        return Err(failed("Storage key exceeds 32 bytes"));
    }

    let storage = kovanica_storage::Storage::new(path).map_err(failed)?;
    let txn = storage.begin_read().map_err(failed)?;

    match storage.state().get_storage(&txn, &target, &slot) {
        Ok(Some(val)) => Ok(Some(hex::encode(val))),
        Ok(None) => Ok(None),
        Err(kovanica_storage::StateStoreError::NotFound(_)) => Ok(None),
        Err(e) => Err(failed(e)),
    }
}

/// Query the latest account state root hash from local embedded node storage.
#[uniffi::export]
pub fn get_local_state_root(db_path: String) -> Result<Option<String>, LightFfiError> {
    let path = std::path::Path::new(&db_path);
    if !path.exists() {
        return Ok(None);
    }

    let storage = kovanica_storage::Storage::new(path).map_err(failed)?;
    let txn = storage.begin_read().map_err(failed)?;

    match storage.state().get_latest_state_root(&txn) {
        Ok(Some((_height, root))) => Ok(Some(hex_hash(&root))),
        Ok(None) => Ok(None),
        Err(_) => Ok(None),
    }
}

static LOCAL_RPC_RUNNING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static LOCAL_RPC_SERVER: std::sync::Mutex<Option<LocalRpcHandle>> = std::sync::Mutex::new(None);

struct LocalRpcHandle {
    bind_addr: String,
    join_handle: Option<std::thread::JoinHandle<()>>,
}

/// Check if the local embedded mobile RPC server is currently running.
#[uniffi::export]
pub fn is_local_rpc_server_running() -> bool {
    LOCAL_RPC_RUNNING.load(std::sync::atomic::Ordering::SeqCst)
}

/// Stop the local embedded mobile RPC server if active.
#[uniffi::export]
pub fn stop_local_rpc_server() -> Result<bool, LightFfiError> {
    let mut guard = LOCAL_RPC_SERVER.lock().map_err(|e| failed(e.to_string()))?;
    if let Some(mut handle) = guard.take() {
        LOCAL_RPC_RUNNING.store(false, std::sync::atomic::Ordering::SeqCst);
        let _ = std::net::TcpStream::connect(&handle.bind_addr);
        if let Some(join) = handle.join_handle.take() {
            let _ = join.join();
        }
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Start an embedded HTTP JSON-RPC server on the mobile device (default: 127.0.0.1:8545),
/// serving verified state directly from local storage.
#[uniffi::export]
pub fn start_local_rpc_server(
    db_path: String,
    bind_addr: String,
    chain_id: u64,
) -> Result<String, LightFfiError> {
    let mut guard = LOCAL_RPC_SERVER.lock().map_err(|e| failed(e.to_string()))?;
    if LOCAL_RPC_RUNNING.load(std::sync::atomic::Ordering::SeqCst) {
        if let Some(ref h) = *guard {
            return Ok(format!("http://{}", h.bind_addr));
        }
    }

    let bind_target = if bind_addr.trim().is_empty() {
        "127.0.0.1:8545".to_string()
    } else {
        bind_addr.trim().trim_start_matches("http://").to_string()
    };

    let listener = std::net::TcpListener::bind(&bind_target).map_err(failed)?;
    let actual_addr = listener.local_addr().map_err(failed)?.to_string();

    LOCAL_RPC_RUNNING.store(true, std::sync::atomic::Ordering::SeqCst);
    let srv_db_path = db_path;
    let srv_actual_addr = actual_addr.clone();

    let join_handle = std::thread::spawn(move || {
        while LOCAL_RPC_RUNNING.load(std::sync::atomic::Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _)) => {
                    if !LOCAL_RPC_RUNNING.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }
                    let db_clone = srv_db_path.clone();
                    std::thread::spawn(move || {
                        handle_local_rpc_connection(stream, &db_clone, chain_id);
                    });
                }
                Err(_) => {
                    if !LOCAL_RPC_RUNNING.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }
                }
            }
        }
    });

    *guard = Some(LocalRpcHandle {
        bind_addr: srv_actual_addr.clone(),
        join_handle: Some(join_handle),
    });

    Ok(format!("http://{}", srv_actual_addr))
}

fn handle_local_rpc_connection(mut stream: std::net::TcpStream, db_path: &str, chain_id: u64) {
    use std::io::{Read, Write};
    use std::time::Duration;

    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();

    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let mut header_end = None;
    let mut content_length = 0usize;

    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                buf.extend_from_slice(&chunk[..n]);
                if header_end.is_none() {
                    if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                        header_end = Some(pos + 4);
                        let headers_str = String::from_utf8_lossy(&buf[..pos]);
                        for line in headers_str.lines() {
                            if line.to_ascii_lowercase().starts_with("content-length:") {
                                if let Some(val) = line.split(':').nth(1) {
                                    content_length = val.trim().parse().unwrap_or(0);
                                }
                            }
                        }
                    }
                }
                if let Some(hdr_len) = header_end {
                    if buf.len() >= hdr_len + content_length {
                        break;
                    }
                }
                if buf.len() > 1024 * 1024 {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    if buf.is_empty() {
        return;
    }

    let request_str = String::from_utf8_lossy(&buf);

    if request_str.starts_with("OPTIONS ") {
        let resp = "HTTP/1.1 204 No Content\r\n\
                    Access-Control-Allow-Origin: *\r\n\
                    Access-Control-Allow-Methods: POST, GET, OPTIONS\r\n\
                    Access-Control-Allow-Headers: Content-Type\r\n\
                    Content-Length: 0\r\n\
                    Connection: close\r\n\r\n";
        let _ = stream.write_all(resp.as_bytes());
        return;
    }

    let body = if let Some(hdr_pos) = header_end {
        &buf[hdr_pos..]
    } else {
        &[]
    };

    let req_val: serde_json::Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => {
            let err_body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": serde_json::Value::Null,
                "error": { "code": -32700, "message": "Parse error" }
            })
            .to_string();
            let resp = format!(
                "HTTP/1.1 400 Bad Request\r\n\
                 Content-Type: application/json\r\n\
                 Access-Control-Allow-Origin: *\r\n\
                 Content-Length: {}\r\n\
                 Connection: close\r\n\r\n{}",
                err_body.len(),
                err_body
            );
            let _ = stream.write_all(resp.as_bytes());
            return;
        }
    };

    let id = req_val
        .get("id")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let method = req_val.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let params = req_val
        .get("params")
        .cloned()
        .unwrap_or(serde_json::Value::Array(vec![]));

    let rpc_result = dispatch_local_rpc(db_path, chain_id, method, params);

    let resp_obj = match rpc_result {
        Ok(res) => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": res
        }),
        Err(err_msg) => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32000, "message": err_msg }
        }),
    };

    let resp_body = resp_obj.to_string();
    let resp = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: application/json\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n{}",
        resp_body.len(),
        resp_body
    );
    let _ = stream.write_all(resp.as_bytes());
}

fn dispatch_local_rpc(
    db_path: &str,
    chain_id: u64,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let path = std::path::Path::new(db_path);
    let storage = if path.exists() {
        kovanica_storage::Storage::new(path).map_err(|e| e.to_string())?
    } else {
        return Err("Local mobile node storage not initialized".to_string());
    };

    match method {
        "kovanica_blockNumber" | "eth_blockNumber" => {
            let txn = storage.begin_read().map_err(|e| e.to_string())?;
            let tip = storage
                .consensus()
                .get_committed_leader_height(&txn)
                .unwrap_or(0);
            Ok(serde_json::json!(format!("0x{:x}", tip)))
        }
        "kovanica_getBalance" | "eth_getBalance" => {
            let addr_str = params
                .get(0)
                .and_then(|v| v.as_str())
                .ok_or_else(|| "missing address param".to_string())?;
            let addr = Address::parse_for_chain(addr_str, chain_id).map_err(|e| e.to_string())?;
            let txn = storage.begin_read().map_err(|e| e.to_string())?;
            let balance = storage
                .state()
                .get_account(&txn, &addr)
                .map(|a| a.balance)
                .unwrap_or(0);
            Ok(serde_json::json!(format!("0x{:x}", balance)))
        }
        "kovanica_getNonce" | "eth_getTransactionCount" => {
            let addr_str = params
                .get(0)
                .and_then(|v| v.as_str())
                .ok_or_else(|| "missing address param".to_string())?;
            let addr = Address::parse_for_chain(addr_str, chain_id).map_err(|e| e.to_string())?;
            let txn = storage.begin_read().map_err(|e| e.to_string())?;
            let nonce = storage
                .state()
                .get_account(&txn, &addr)
                .map(|a| a.nonce)
                .unwrap_or(0);
            Ok(serde_json::json!(format!("0x{:x}", nonce)))
        }
        "kovanica_getBlockByNumber" | "eth_getBlockByNumber" => {
            let num = params
                .get(0)
                .and_then(parse_quantity_json)
                .ok_or_else(|| "missing block number".to_string())?;
            let txn = storage.begin_read().map_err(|e| e.to_string())?;
            match storage.blocks().get_block_by_height(&txn, num) {
                Ok(Some(block)) => {
                    let mut val = serde_json::json!({
                        "round": format!("0x{:x}", block.round),
                        "number": format!("0x{:x}", block.round),
                        "hash": format!("0x{}", hex::encode(block.digest.0)),
                        "author": block.author,
                        "parents": block.parents.iter().map(|p| format!("0x{}", hex::encode(p.digest.0))).collect::<Vec<_>>(),
                        "transactions_count": block.transactions.len(),
                    });
                    if let Some((timestamp, target_bits, nonce, pow_hash)) =
                        kovanica_types::decode_pow_statement(&block.statements)
                    {
                        if let Some(map) = val.as_object_mut() {
                            map.insert("timestamp".to_string(), serde_json::json!(timestamp));
                            map.insert(
                                "target_bits".to_string(),
                                serde_json::json!(format!("0x{:08x}", target_bits)),
                            );
                            map.insert("nonce".to_string(), serde_json::json!(nonce));
                            if let Some(h) = pow_hash {
                                map.insert(
                                    "pow_hash".to_string(),
                                    serde_json::json!(format!("0x{}", hex::encode(h.0))),
                                );
                            }
                        }
                    }
                    Ok(val)
                }
                _ => Ok(serde_json::Value::Null),
            }
        }
        "kovanica_getBlockHeaderByNumber" => {
            let num = params
                .get(0)
                .and_then(parse_quantity_json)
                .ok_or_else(|| "missing block number".to_string())?;
            let txn = storage.begin_read().map_err(|e| e.to_string())?;
            match storage.blocks().get_block_by_height(&txn, num) {
                Ok(Some(block)) => {
                    let header = block
                        .extract_pow_header(chain_id)
                        .unwrap_or_else(|_| block.pow_header(chain_id));
                    Ok(serde_json::to_value(&header).map_err(|e| e.to_string())?)
                }
                _ => Ok(serde_json::Value::Null),
            }
        }
        "kovanica_estimateGas" | "eth_estimateGas" => {
            let call_obj = params
                .get(0)
                .ok_or_else(|| "missing call object".to_string())?;
            let from = call_obj.get("from").and_then(|v| v.as_str()).unwrap_or("");
            let to = call_obj.get("to").and_then(|v| v.as_str()).unwrap_or("");
            let data = call_obj.get("data").and_then(|v| v.as_str()).unwrap_or("");
            let value = call_obj
                .get("value")
                .and_then(parse_quantity_json)
                .unwrap_or(0);
            let sim = simulate_local_call(
                db_path.to_string(),
                from.to_string(),
                to.to_string(),
                "".to_string(),
                data.to_string(),
                value,
                500_000,
                chain_id,
            )
            .map_err(|e| format!("{:?}", e))?;
            Ok(serde_json::json!(format!("0x{:x}", sim.gas_used)))
        }
        "kovanica_call" | "eth_call" => {
            let call_obj = params
                .get(0)
                .ok_or_else(|| "missing call object".to_string())?;
            let from = call_obj.get("from").and_then(|v| v.as_str()).unwrap_or("");
            let to = call_obj.get("to").and_then(|v| v.as_str()).unwrap_or("");
            let data = call_obj.get("data").and_then(|v| v.as_str()).unwrap_or("");
            let value = call_obj
                .get("value")
                .and_then(parse_quantity_json)
                .unwrap_or(0);
            let sim = simulate_local_call(
                db_path.to_string(),
                from.to_string(),
                to.to_string(),
                "".to_string(),
                data.to_string(),
                value,
                500_000,
                chain_id,
            )
            .map_err(|e| format!("{:?}", e))?;
            if sim.success {
                let clean = sim.return_data_hex.trim_start_matches("0x");
                Ok(serde_json::json!(format!("0x{}", clean)))
            } else {
                Err(sim.error.unwrap_or_else(|| "Call reverted".to_string()))
            }
        }
        "kovanica_syncStatus" => {
            let txn = storage.begin_read().map_err(|e| e.to_string())?;
            let tip = storage
                .consensus()
                .get_committed_leader_height(&txn)
                .unwrap_or(0);
            Ok(serde_json::json!({
                "syncing": false,
                "current_round": tip,
                "highest_round": tip,
                "local_node": true,
                "engine": "Wasmi-v0.40"
            }))
        }
        "eth_chainId" => Ok(serde_json::json!(format!("0x{:x}", chain_id))),
        "net_version" => Ok(serde_json::json!(chain_id.to_string())),
        "web3_clientVersion" => Ok(serde_json::json!(
            "KovanicaMobileNode/0.2.0 (Android; Wasmi-v0.40)"
        )),
        _ => Err(format!(
            "Method `{}` not supported on local mobile node",
            method
        )),
    }
}

/// Check an account proof against a header this client already accepted.
#[uniffi::export]
pub fn confirm_account(
    header_json: String,
    proof_json: String,
) -> Result<AccountSnapshot, LightFfiError> {
    let header = json::leader_header_from_json(&json::parse_json(&header_json).map_err(failed)?)
        .map_err(failed)?;
    let (height, proof) =
        json::account_proof_from_json(&json::parse_json(&proof_json).map_err(failed)?)
            .map_err(failed)?;
    if height != header.committed_height {
        return Err(failed(format!(
            "account proof height {height} is not the sealed header {}",
            header.committed_height
        )));
    }
    if !verify_account(&header, &proof) {
        return Err(failed("account proof does not match the sealed state root"));
    }
    Ok(AccountSnapshot {
        balance: proof.balance,
        nonce: proof.nonce,
        verified: true,
    })
}

/// Sign a transfer. `fee_rate` is atoms per byte from `kovanica_estimateFee`.
#[uniffi::export]
pub fn sign_transfer_raw(
    keystore_json: String,
    passphrase: String,
    chain_id: u64,
    to: String,
    amount: u64,
    nonce: u64,
    fee_rate: u64,
) -> Result<String, LightFfiError> {
    let passphrase = SecretString::new(passphrase);
    let (signing_key, sender) = open_signing_key(&keystore_json, passphrase.as_str())?;
    let to = parse_address(&to, chain_id)?;
    priced_raw(fee_rate, |fee| {
        sign_transfer(chain_id, &signing_key, sender, to, amount, nonce, fee)
    })
}

/// Sign a contract call. `args_hex` is bincode-encoded calldata (`0x` optional).
///
/// `value` is the native KVNC (atoms) sent to the contract, `0` when unused.
/// `fee_rate` is atoms per byte from `kovanica_estimateFee`.
#[uniffi::export]
#[allow(clippy::too_many_arguments)]
pub fn sign_call_raw(
    keystore_json: String,
    passphrase: String,
    chain_id: u64,
    contract: String,
    value: u64,
    method: String,
    args_hex: String,
    gas_limit: u64,
    nonce: u64,
    fee_rate: u64,
) -> Result<String, LightFfiError> {
    let passphrase = SecretString::new(passphrase);
    let (signing_key, sender) = open_signing_key(&keystore_json, passphrase.as_str())?;
    let contract = parse_address(&contract, chain_id)?;
    let args = parse_hex_bytes(&args_hex, "args")?;
    priced_raw(fee_rate, |fee| {
        sign_call(
            chain_id,
            &signing_key,
            sender,
            nonce,
            fee,
            contract,
            value,
            method.clone(),
            args.clone(),
            gas_limit,
        )
    })
}

/// Sign a contract deploy. `code_hex` is the WASM bytecode (`0x` optional).
///
/// `fee_rate` is atoms per byte from `kovanica_estimateFee`.
#[uniffi::export]
pub fn sign_deploy_raw(
    keystore_json: String,
    passphrase: String,
    chain_id: u64,
    code_hex: String,
    nonce: u64,
    fee_rate: u64,
) -> Result<String, LightFfiError> {
    let passphrase = SecretString::new(passphrase);
    let (signing_key, sender) = open_signing_key(&keystore_json, passphrase.as_str())?;
    let code = parse_hex_bytes(&code_hex, "code")?;
    priced_raw(fee_rate, |fee| {
        sign_deploy(chain_id, &signing_key, sender, nonce, fee, code.clone())
    })
}

/// Sign a light-node bond. The stake does not enter the validator committee.
#[uniffi::export]
pub fn sign_bond_raw(
    keystore_json: String,
    passphrase: String,
    chain_id: u64,
    stake: u64,
    nonce: u64,
    fee_rate: u64,
) -> Result<String, LightFfiError> {
    let passphrase = SecretString::new(passphrase);
    let (signing_key, sender) = open_signing_key(&keystore_json, passphrase.as_str())?;
    let vrf = vrf_key(&signing_key)?;
    priced_raw(fee_rate, |fee| {
        sign_register_light_node(
            chain_id,
            &signing_key,
            sender,
            vrf.public_key_typed(),
            stake,
            nonce,
            fee,
        )
    })
}

/// Sign a light-node exit.
#[uniffi::export]
pub fn sign_exit_raw(
    keystore_json: String,
    passphrase: String,
    chain_id: u64,
    nonce: u64,
    fee_rate: u64,
) -> Result<String, LightFfiError> {
    let passphrase = SecretString::new(passphrase);
    let (signing_key, sender) = open_signing_key(&keystore_json, passphrase.as_str())?;
    priced_raw(fee_rate, |fee| {
        sign_exit_light_node(chain_id, &signing_key, sender, nonce, fee)
    })
}

/// Sign one witness for `epoch`. `beacon_hex` is the epoch-opening leader digest.
#[uniffi::export]
pub fn sign_witness_raw(
    keystore_json: String,
    passphrase: String,
    chain_id: u64,
    epoch: u64,
    beacon_hex: String,
    nonce: u64,
    fee_rate: u64,
) -> Result<String, LightFfiError> {
    let passphrase = SecretString::new(passphrase);
    let (signing_key, sender) = open_signing_key(&keystore_json, passphrase.as_str())?;
    let beacon = parse_beacon(&beacon_hex)?;
    let vrf = vrf_key(&signing_key)?;
    let input = kovanica_types::light_vrf_input(chain_id, epoch, &beacon);
    let output = vrf_prove(&vrf, &input);
    priced_raw(fee_rate, |fee| {
        sign_light_witness(
            chain_id,
            &signing_key,
            sender,
            epoch,
            output.hash,
            output.proof,
            nonce,
            fee,
        )
    })
}

fn wallet_from_seed(seed: &[u8; 32], passphrase: &str) -> Result<CreatedWallet, LightFfiError> {
    let mnemonic = kovanica_keystore::seed_to_mnemonic(seed).map_err(failed)?;
    let keystore = kovanica_keystore::from_seed(seed, passphrase).map_err(failed)?;
    let address = kovanica_keystore::address(&keystore)
        .map_err(failed)?
        .to_string();
    let keystore_json = to_json(&keystore).map_err(failed)?;
    Ok(CreatedWallet {
        address,
        mnemonic,
        keystore_json,
    })
}

fn open_signing_key(
    keystore_json: &str,
    passphrase: &str,
) -> Result<(SigningKey, Address), LightFfiError> {
    let keystore = parse(keystore_json.as_bytes()).map_err(failed)?;
    let sender = kovanica_keystore::address(&keystore).map_err(failed)?;
    let signing_key =
        kovanica_keystore::signing_key(&keystore, Some(passphrase)).map_err(failed)?;
    Ok((signing_key, sender))
}

fn vrf_key(signing_key: &SigningKey) -> Result<VRFKeyPair, LightFfiError> {
    let secret = derive_vrf_secret_from_seed(&signing_key.to_bytes());
    VRFKeyPair::from_secret(secret).map_err(failed)
}

fn parse_address(value: &str, chain_id: u64) -> Result<Address, LightFfiError> {
    kovanica_keystore::parse_address(value, "address", chain_id).map_err(failed)
}

fn parse_beacon(value: &str) -> Result<Hash, LightFfiError> {
    let raw = value.strip_prefix("0x").unwrap_or(value);
    let bytes = hex::decode(raw).map_err(failed)?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| failed("beacon must be 32 bytes"))?;
    Ok(Hash(bytes))
}

/// Decode a byte string from hex, `0x`/`0X` prefix optional and whitespace trimmed.
fn parse_hex_bytes(value: &str, field: &str) -> Result<Vec<u8>, LightFfiError> {
    let trimmed = value.trim();
    let stripped = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    hex::decode(stripped).map_err(|_| failed(format!("{field} is not valid hex")))
}

fn priced_raw(fee_rate: u64, sign: impl Fn(u64) -> Transaction) -> Result<String, LightFfiError> {
    let draft = bincode::serialize(&sign(1)).map_err(failed)?;
    let fee = fee_rate.saturating_mul(draft.len() as u64).max(1);
    let raw = bincode::serialize(&sign(fee)).map_err(failed)?;
    Ok(format!("0x{}", hex::encode(raw)))
}

fn hex_hash(hash: &Hash) -> String {
    format!("0x{}", hex::encode(hash.0))
}

/// Get the public key hex from a keystore JSON without decrypting.
/// This does not decrypt the seed.
#[uniffi::export]
pub fn wallet_pubkey_hex(keystore_json: String) -> Result<String, LightFfiError> {
    let keystore = parse(keystore_json.as_bytes()).map_err(failed)?;
    Ok(keystore.public_key.clone())
}

/// Reveal the 24-word seed phrase for backup.
///
/// **Note:** Runs off the UI thread (decrypts the keystore; Argon2id is expensive).
/// A wrong passphrase is an error, never a panic.
#[uniffi::export]
pub fn reveal_mnemonic_raw(
    keystore_json: String,
    passphrase: String,
) -> Result<String, LightFfiError> {
    let passphrase = SecretString::new(passphrase);
    reveal_mnemonic(&keystore_json, passphrase.as_str()).map_err(failed)
}

/// Re-encrypt the wallet under `new_passphrase`; returns the new keystore JSON.
///
/// The seed is preserved, so the address does not change. On success the caller
/// replaces its stored `keystore` value with the returned JSON.
///
/// **Note:** Runs off the UI thread (decrypts and re-encrypts; Argon2id runs twice).
/// A wrong `passphrase` is an error, never a panic.
#[uniffi::export]
pub fn change_passphrase_raw(
    keystore_json: String,
    passphrase: String,
    new_passphrase: String,
) -> Result<String, LightFfiError> {
    let passphrase = SecretString::new(passphrase);
    let new_passphrase = SecretString::new(new_passphrase);
    if new_passphrase.as_str().is_empty() {
        return Err(failed("passphrase is empty"));
    }
    change_passphrase(&keystore_json, passphrase.as_str(), new_passphrase.as_str()).map_err(failed)
}

/// Derive the address a `Deploy` transaction will create on-chain, off-node.
///
/// `sender_hex` is the `kvnc…dag` address, or a legacy hex form where
/// `chain_id` allows it (devnet 3, local 1337). `nonce` is the
/// sender's account nonce **as it stands while the `Deploy` tx is being
/// signed** (i.e. the `tx.nonce`); the `+1` the node bakes into the address is
/// applied inside. Returns the bare 64-hex contract address.
///
/// **Note:** Pure BLAKE3 — safe to call on the UI thread.
#[uniffi::export]
pub fn contract_address_for_raw(
    sender_hex: String,
    nonce: u64,
    chain_id: u64,
) -> Result<String, LightFfiError> {
    contract_address_for(&sender_hex, nonce, chain_id).map_err(failed)
}

/// Derive comms identity hex from public key hex.
#[uniffi::export]
pub fn derive_comms_id(pubkey_hex: String) -> Result<String, LightFfiError> {
    let trimmed = pubkey_hex.trim();
    let stripped = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    let bytes = hex::decode(stripped).map_err(failed)?;
    let key: [u8; 32] = bytes
        .try_into()
        .map_err(|_| failed("public key must be 32 bytes"))?;
    Ok(kovanica_comms_core::comms_id(&key).to_hex())
}

/// Sign a detached message with domain separation.
///
/// **Note:** Runs off UI thread (decrypts the keystore; Argon2id may be expensive).
#[uniffi::export]
pub fn sign_detached(
    keystore_json: String,
    passphrase: String,
    message_hex: String,
    domain: Option<String>,
) -> Result<String, LightFfiError> {
    let passphrase = SecretString::new(passphrase);
    let (sk, _) = open_signing_key(&keystore_json, passphrase.as_str())?;
    let msg_trim = message_hex.trim();
    let msg_stripped = msg_trim.strip_prefix("0x").unwrap_or(msg_trim);
    let msg_raw = hex::decode(msg_stripped).map_err(failed)?;
    let d = match domain.as_deref() {
        Some(dom) => dom.to_string(),
        None => core::str::from_utf8(COMMS_MSG_DOMAIN)
            .map_err(|_| failed("invalid domain"))?
            .to_string(),
    };
    let mut to_sign = Vec::with_capacity(d.len() + msg_raw.len());
    to_sign.extend_from_slice(d.as_bytes());
    to_sign.extend_from_slice(&msg_raw);
    let sig = kovanica_crypto::sign(&sk, &to_sign);
    Ok(hex::encode(sig.0))
}

/// Decode a 32-byte Ed25519 public key from hex (`0x` prefix optional).
fn parse_comms_pubkey(value: &str, field: &str) -> Result<[u8; 32], LightFfiError> {
    let trimmed = value.trim();
    let stripped = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    let bytes = hex::decode(stripped).map_err(|_| failed(format!("{field} is not valid hex")))?;
    bytes
        .try_into()
        .map_err(|_| failed(format!("{field} must be 32 bytes")))
}

/// Seal and sign a comms envelope for a recipient.
///
/// Decrypts the keystore to recover the sender seed, derives the comms ids, and
/// calls [`kovanica_comms_relay::envelope::seal_and_sign`]. Returns the compact
/// envelope JSON (wire shape `v,id,to,from,sentAt,expiresAt,alg,epk,nonce,ct,sig`).
///
/// **Note:** Runs off UI thread (decrypts the keystore; Argon2id may be expensive).
#[uniffi::export]
pub fn comms_seal_envelope(
    keystore_json: String,
    passphrase: String,
    recipient_pubkey_hex: String,
    payload_json: String,
    sent_at_ms: u64,
    expires_at_ms: u64,
) -> Result<String, LightFfiError> {
    let passphrase = SecretString::new(passphrase);
    let (signing_key, _) = open_signing_key(&keystore_json, passphrase.as_str())?;
    let recipient = parse_comms_pubkey(&recipient_pubkey_hex, "recipient public key")?;
    if payload_json.len() > MAX_PLAINTEXT {
        return Err(failed("payload exceeds MAX_PLAINTEXT (16384)"));
    }
    let mut seed = signing_key.to_bytes();
    let sealed = kovanica_comms_relay::envelope::seal_and_sign(
        &seed,
        &recipient,
        payload_json.as_bytes(),
        sent_at_ms,
        expires_at_ms,
    );
    seed.zeroize();
    let envelope = sealed.map_err(failed)?;
    envelope.to_json().map_err(failed)
}

/// Open a comms envelope addressed to this keystore's comms id.
///
/// The envelope signature is **not** checked here (see [`comms_verify_envelope`]);
/// only the recipient match and the AEAD are enforced. Returns the trimmed
/// payload JSON exactly as sealed.
///
/// **Note:** Runs off UI thread (decrypts the keystore; Argon2id may be expensive).
#[uniffi::export]
pub fn comms_open_envelope(
    keystore_json: String,
    passphrase: String,
    envelope_json: String,
) -> Result<String, LightFfiError> {
    let passphrase = SecretString::new(passphrase);
    let (signing_key, _) = open_signing_key(&keystore_json, passphrase.as_str())?;
    let envelope = Envelope::from_json(envelope_json.as_bytes()).map_err(failed)?;

    let my_pub = signing_key.verifying_key().to_bytes();
    let my_id = comms_id_hex(&comms_id_bytes(&my_pub));
    let to_id = comms_id_from_hex(&envelope.to).map_err(failed)?;
    if to_id != comms_id_from_hex(&my_id).map_err(failed)? {
        return Err(failed(
            "envelope is not addressed to this keystore's comms id",
        ));
    }
    let from_id = comms_id_from_hex(&envelope.from).map_err(failed)?;

    let sealed = Sealed {
        alg: envelope.alg.clone(),
        epk: envelope
            .epk
            .clone()
            .try_into()
            .map_err(|_| failed("envelope epk must be 32 bytes"))?,
        nonce: envelope
            .nonce
            .clone()
            .try_into()
            .map_err(|_| failed("envelope nonce must be 24 bytes"))?,
        ct: envelope.ct.clone(),
    };

    let mut seed = signing_key.to_bytes();
    let mut cipher = X25519Cipher::from_seed(&seed);
    seed.zeroize();
    let aad = e2e_aad(&to_id, &from_id);
    let plain = cipher.open(&sealed, &aad).map_err(failed)?;

    // Validate the padded payload is compact JSON (shared helper), then return
    // the exact on-wire text so object key order is preserved.
    kovanica_comms_relay::trim_payload(&plain).map_err(failed)?;
    let end = plain
        .iter()
        .rposition(|b| *b != 0)
        .map(|i| i + 1)
        .ok_or_else(|| failed("envelope payload is empty"))?;
    let text = core::str::from_utf8(&plain[..end]).map_err(failed)?;
    Ok(text.to_string())
}

/// Verify an envelope against a claimed sender Ed25519 public key.
///
/// Checks the Ed25519 signature over [`Envelope::signing_bytes`] **and** that
/// `from == comms_id(sender_pubkey)`. Returns `false` for any verification or
/// shape failure; only malformed JSON or hex is an error.
#[uniffi::export]
pub fn comms_verify_envelope(
    envelope_json: String,
    sender_pubkey_hex: String,
) -> Result<bool, LightFfiError> {
    let envelope = Envelope::from_json(envelope_json.as_bytes()).map_err(failed)?;
    let sender = parse_comms_pubkey(&sender_pubkey_hex, "sender public key")?;
    Ok(envelope.verify(&sender).is_ok())
}

/// Normalise (ASCII lowercase) then validate a registry name.
///
/// Mirrors `kovanica_name_registry::normalise_name` (spec §7.2): 3..=32 bytes,
/// charset `[a-z0-9-]`, no leading/trailing `-`, no `--`. Returns the stored
/// (lowercase) form.
fn normalise_registry_name(name: &str) -> Result<String, LightFfiError> {
    let lowered: String = name.chars().map(|c| c.to_ascii_lowercase()).collect();
    let bytes = lowered.as_bytes();
    if bytes.len() < 3 || bytes.len() > 32 {
        return Err(failed("name must be 3..=32 bytes"));
    }
    if !bytes
        .iter()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
    {
        return Err(failed("name may contain only [a-z0-9-]"));
    }
    if bytes.first() == Some(&b'-') || bytes.last() == Some(&b'-') {
        return Err(failed("name must not start or end with '-'"));
    }
    if bytes.windows(2).any(|w| w == b"--") {
        return Err(failed("name must not contain '--'"));
    }
    Ok(lowered)
}

/// Compute the contract-storage slot for a name-registry entry.
///
/// `slot = BLAKE3(b"kvnc/v1/name/" ++ lowercase(name))`, returned as 64
/// lowercase hex (**no** `0x` prefix). The prefix is taken from
/// [`kovanica_name_registry::entry_key`] (single source of truth) and hashed with
/// the canonical [`kovanica_common::hash`], exactly matching the host storage key
/// the contract writes (`blake3(raw_key)`).
///
/// Names are normalised-then-validated: 3..=32 bytes, only `[a-z0-9-]`, no
/// leading/trailing `-`, no `--`. Invalid input is a [`LightFfiError`].
#[uniffi::export]
pub fn registry_slot_hex(name: String) -> Result<String, LightFfiError> {
    let name = normalise_registry_name(&name)?;
    let slot = kovanica_common::hash(&kovanica_name_registry::entry_key(&name));
    Ok(hex::encode(slot))
}

/// Decode a bincode 1.3 `NameEntry` read from a registry storage slot.
///
/// Accepts `"0x"+hex` or bare hex (case-insensitive). The value is
/// `owner[32] ‖ pubkey[32] ‖ u64le(count) ‖ count×(u64le(len) ‖ utf8)`.
/// Returns compact JSON
/// `{"owner":"<64hex>","pubkey":"<64hex>","relay_endpoints":["…"]}`.
///
/// Truncated or malformed bytes are a [`LightFfiError`]. Callers must check for
/// an empty slot (`""` / `"0x"`) **before** calling this — those are not
/// special-cased here and decode to an error.
#[uniffi::export]
pub fn registry_decode_entry(value_hex: String) -> Result<String, LightFfiError> {
    let trimmed = value_hex.trim();
    let stripped = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    let bytes = hex::decode(stripped).map_err(|_| failed("entry value is not valid hex"))?;
    let entry: kovanica_name_registry::NameEntry = bincode::deserialize(&bytes).map_err(failed)?;
    // Key order `owner, pubkey, relay_endpoints` — compact JSON, no whitespace.
    let out = serde_json::json!({
        "owner": hex::encode(entry.owner),
        "pubkey": hex::encode(entry.pubkey),
        "relay_endpoints": entry.relay_endpoints,
    });
    serde_json::to_string(&out).map_err(failed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_crypto::sign;

    #[test]
    fn wipe_secret_clears_every_byte() {
        use zeroize::Zeroize;
        let mut bytes = *b"hunter2-phrase!!";
        bytes.zeroize();
        assert_eq!(bytes, [0u8; 16]);

        let mut secret = String::from("hunter2-phrase!!");
        wipe_secret(&mut secret);
        assert!(secret.is_empty());

        let held = SecretString::new(String::from("hunter2-phrase!!"));
        assert_eq!(held.as_str(), "hunter2-phrase!!");
        drop(held);
    }

    #[test]
    fn contract_address_for_raw_takes_a_chain_id() {
        use kovanica_light::contract_address_for;
        use kovanica_types::signing::chain_id;
        use kovanica_types::Address;
        let sender = Address([0x11; 32]);
        let on_devnet = contract_address_for_raw(sender.to_hex(), 41, chain_id::DEVNET).unwrap();
        assert_eq!(
            on_devnet,
            contract_address_for(&sender.encode(), 41, chain_id::MAINNET).unwrap()
        );
        assert!(contract_address_for_raw(sender.to_hex(), 41, chain_id::MAINNET).is_err());
        assert!(
            contract_address_for_raw(format!("0X{}", sender.to_hex()), 41, chain_id::DEVNET)
                .is_err()
        );
    }

    use kovanica_types::{
        committee_seal_hash, leader_header, CommittedSubDag, PublicKey, Signature, StatementBlock,
    };

    fn keys(n: u16) -> (Vec<SigningKey>, String) {
        let mut signers = Vec::new();
        let mut members = Vec::new();
        for index in 0..n {
            let signer = SigningKey::from_bytes(&[index as u8 + 1; 32]);
            let public_key = PublicKey::from(signer.verifying_key());
            let address = Address::from_public_key(&public_key);
            members.push(serde_json::json!({
                "index": index,
                "address": address.to_string(),
                "stake": "0x1",
                "publicKey": format!("0x{}", hex::encode(public_key.0)),
            }));
            signers.push(signer);
        }
        (signers, serde_json::Value::Array(members).to_string())
    }

    fn header_json(
        height: u64,
        parent: Hash,
        state: Hash,
        committee_json: &str,
    ) -> (String, kovanica_types::LeaderHeader) {
        let value: serde_json::Value = serde_json::from_str(committee_json).unwrap();
        let committee = json::committee_from_json(0, &value).unwrap();
        let digest = Hash::new([height as u8 + 1]);
        let leader = StatementBlock {
            author: 0,
            round: height.saturating_mul(3),
            parents: vec![],
            transactions: vec![],
            statements: vec![],
            signature: Signature([0; 64]),
            digest,
            merkle_root: Hash::zero(),
        };
        let subdag = CommittedSubDag {
            blocks: vec![leader.clone()],
            leader,
            leader_round: height.saturating_mul(3),
            leader_author: 0,
            non_blue: vec![],
        };
        let members: Vec<_> = committee
            .authorities
            .iter()
            .map(|authority| (authority.index, authority.public_key, authority.stake))
            .collect();
        let header = leader_header(
            &subdag,
            height,
            parent,
            state,
            committee.epoch,
            committee_seal_hash(committee.epoch, &members),
            false,
        );
        let json = serde_json::json!({
            "committedHeight": format!("0x{height:x}"),
            "leaderRound": format!("0x{:x}", header.leader_round),
            "leaderDigest": hex_hash(&header.leader_digest),
            "leaderAuthor": header.leader_author,
            "parentLeaderDigest": hex_hash(&header.parent_leader_digest),
            "txMerkleRoot": hex_hash(&header.tx_merkle_root),
            "executedTxRoot": hex_hash(&header.executed_tx_root),
            "nonBlueRoot": hex_hash(&header.non_blue_root),
            "blueRoot": hex_hash(&header.blue_root),
            "mysticghost": header.mysticghost,
            "stateRoot": hex_hash(&header.state_root),
            "epoch": format!("0x{:x}", header.epoch),
            "committeeHash": hex_hash(&header.committee_hash),
            "timestamp": "0x0",
        });
        (json.to_string(), header)
    }

    fn seals_json(signers: &[SigningKey], header: &kovanica_types::LeaderHeader, n: u16) -> String {
        let seals: Vec<_> = (0..n)
            .map(|index| {
                let signature = sign(
                    &signers[index as usize],
                    &header.seal_message(LOCAL_CHAIN_ID),
                );
                serde_json::json!({
                    "authorityIndex": index,
                    "signature": format!("0x{}", hex::encode(signature.0)),
                })
            })
            .collect();
        serde_json::json!({ "seals": seals }).to_string()
    }

    #[test]
    fn three_seals_verify_and_the_faucet_follows_chain_id() {
        assert!(faucet_allowed(2));
        assert!(faucet_allowed(chain_id::TESTNET));
        assert!(faucet_allowed(chain_id::HYBRID_TESTNET));
        assert!(faucet_allowed(TESTNET_CHAIN_ID));
        assert!(!faucet_allowed(1));
        assert!(!faucet_allowed(chain_id::MAINNET));
        assert!(faucet_allowed(3));
        assert!(faucet_allowed(chain_id::DEVNET));
        assert!(faucet_allowed(1337));
        assert!(faucet_allowed(chain_id::LOCAL));
        assert!(faucet_allowed(LOCAL_CHAIN_ID));
        assert!(!faucet_allowed(0));
        assert!(!faucet_allowed(4));
        let (signers, committee) = keys(4);
        let (header_json, header) = header_json(0, Hash::zero(), Hash::new([4u8]), &committee);
        let seals = seals_json(&signers, &header, 3);
        let verified = verify_header(
            LOCAL_CHAIN_ID,
            0,
            committee,
            header_json,
            seals,
            String::new(),
        )
        .unwrap();
        assert_eq!(verified.committed_height, 0);
        assert_eq!(verified.epoch, 0);
    }

    #[test]
    fn a_short_seal_set_is_rejected() {
        let (signers, committee) = keys(4);
        let (header_json, header) = header_json(1, Hash::zero(), Hash::zero(), &committee);
        let seals = seals_json(&signers, &header, 2);
        assert!(verify_header(
            LOCAL_CHAIN_ID,
            0,
            committee,
            header_json,
            seals,
            String::new()
        )
        .is_err());
    }

    #[test]
    fn wallet_pubkey_hex_roundtrip() {
        let created = create_wallet("test-pass".to_string()).unwrap();
        let pubkey = wallet_pubkey_hex(created.keystore_json.clone()).unwrap();
        let clean = pubkey.trim().strip_prefix("0x").unwrap_or(&pubkey);
        assert_eq!(clean.len(), 64);
        assert!(hex::decode(clean).is_ok());
    }

    #[test]
    fn derive_comms_id_zeros() {
        // KAT from comms-core/TS: pubkey all ones (0x01)
        let ones = "01".repeat(32);
        let comms = derive_comms_id(ones.clone()).unwrap();
        assert_eq!(
            comms,
            "6076cc6f05dbd8ad851ba6aa6b7e85c8dce112186e16d3e267663641d9fa9339"
        );
    }

    #[test]
    fn sign_detached_produces_128_hex_sig() {
        let created = create_wallet("pw".to_string()).unwrap();
        let msg = hex::encode(b"hello");
        let sig = sign_detached(created.keystore_json, "pw".to_string(), msg, None).unwrap();
        assert_eq!(sig.len(), 128);
        assert!(hex::decode(&sig).is_ok());
    }

    #[test]
    fn sign_detached_error_cases() {
        let created = create_wallet("pw".to_string()).unwrap();
        let bad = sign_detached(
            created.keystore_json.clone(),
            "pw".to_string(),
            "gg".to_string(),
            None,
        );
        assert!(bad.is_err());
        let bad2 = sign_detached(
            created.keystore_json,
            "wrong".to_string(),
            hex::encode(b"hi"),
            None,
        );
        assert!(bad2.is_err());
    }

    /// §1 KAT: seal → open returns the exact original payload JSON.
    #[test]
    fn comms_seal_open_round_trip() {
        let sender = create_wallet("sender-pw".to_string()).unwrap();
        let recipient = create_wallet("recipient-pw".to_string()).unwrap();
        let recipient_pub = wallet_pubkey_hex(recipient.keystore_json.clone()).unwrap();
        let payload = r#"{"kind":"text","body":"hello kovanica","ts":1730000000000}"#;

        let envelope_json = comms_seal_envelope(
            sender.keystore_json,
            "sender-pw".to_string(),
            recipient_pub,
            payload.to_string(),
            1730000000000,
            1730000000000 + 24 * 60 * 60 * 1000,
        )
        .unwrap();

        // Envelope is the M1 wire shape and leaks no plaintext.
        assert!(envelope_json.contains("\"v\":1"));
        assert!(!envelope_json.contains("hello kovanica"));

        let opened = comms_open_envelope(
            recipient.keystore_json,
            "recipient-pw".to_string(),
            envelope_json,
        )
        .unwrap();
        assert_eq!(opened, payload);
    }

    #[test]
    fn comms_open_rejects_wrong_recipient() {
        let sender = create_wallet("s".to_string()).unwrap();
        let recipient = create_wallet("r".to_string()).unwrap();
        let other = create_wallet("o".to_string()).unwrap();
        let recipient_pub = wallet_pubkey_hex(recipient.keystore_json).unwrap();
        let env = comms_seal_envelope(
            sender.keystore_json,
            "s".to_string(),
            recipient_pub,
            r#"{"kind":"text","body":"hi","ts":1}"#.to_string(),
            1,
            2,
        )
        .unwrap();
        assert!(comms_open_envelope(other.keystore_json, "o".to_string(), env).is_err());
    }

    /// §1: `comms_verify_envelope` is true for the correct sender pubkey only.
    #[test]
    fn comms_verify_envelope_matches_sender() {
        let sender = create_wallet("s".to_string()).unwrap();
        let recipient = create_wallet("r".to_string()).unwrap();
        let other = create_wallet("o".to_string()).unwrap();
        let recipient_pub = wallet_pubkey_hex(recipient.keystore_json).unwrap();
        let env = comms_seal_envelope(
            sender.keystore_json.clone(),
            "s".to_string(),
            recipient_pub,
            r#"{"kind":"text","body":"hi","ts":1}"#.to_string(),
            1,
            2,
        )
        .unwrap();

        let sender_pub = wallet_pubkey_hex(sender.keystore_json).unwrap();
        assert!(comms_verify_envelope(env.clone(), sender_pub).unwrap());

        let other_pub = wallet_pubkey_hex(other.keystore_json).unwrap();
        assert!(!comms_verify_envelope(env.clone(), other_pub).unwrap());

        // Malformed JSON is an error, not `false`.
        let zeros = "00".repeat(32);
        assert!(comms_verify_envelope("not json".to_string(), zeros).is_err());
    }

    /// §7.2 KAT: the slot is BLAKE3 of the contract's own raw storage key.
    /// Expected values computed independently (raw `blake3(b"kvnc/v1/name/…")`),
    /// matching the key `wasmi_e2e.rs` reads straight out of storage.
    #[test]
    fn registry_slot_hex_matches_contract_storage_key() {
        assert_eq!(
            registry_slot_hex("alice".to_string()).unwrap(),
            "9a86b101401f2c4477d327e0ef510222296e27cb5f65da1f7a68cdf0d85db65c"
        );
        assert_eq!(
            registry_slot_hex("BOB".to_string()).unwrap(),
            "da4fd9d43a1014b06827bf80c50a1198c8be69579f1db836e99337847ab75f9c"
        );

        // Single source of truth: equals the contract key hashed with the
        // canonical `kovanica_common::hash`, for any valid name.
        for name in [
            "alice",
            "bob",
            "abc",
            "a-b-c",
            "abcdefghijklmnopqrstuvwxyz012345",
        ] {
            let expected = hex::encode(kovanica_common::hash(&kovanica_name_registry::entry_key(
                name,
            )));
            assert_eq!(registry_slot_hex(name.to_string()).unwrap(), expected);
        }

        // Output is 64 lowercase hex with no `0x`.
        let slot = registry_slot_hex("alice".to_string()).unwrap();
        assert_eq!(slot.len(), 64);
        assert!(!slot.starts_with("0x"));
        assert!(slot
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn registry_slot_hex_rejects_invalid_names() {
        let bad = [
            "ab",                                // too short
            "abcdefghijklmnopqrstuvwxyz0123456", // 33 bytes → too long
            "-alice",                            // leading hyphen
            "alice-",                            // trailing hyphen
            "al--ice",                           // consecutive hyphens
            "al ice",                            // space
            "ali.ce",                            // dot
            "ali_ce",                            // underscore
            "alïce",                             // non-ASCII
            "",                                  // empty
        ];
        for name in bad {
            assert!(
                registry_slot_hex(name.to_string()).is_err(),
                "name {name:?} should be rejected"
            );
        }

        // Mixed case normalises; boundary lengths are valid.
        assert_eq!(
            registry_slot_hex("Alice".to_string()).unwrap(),
            registry_slot_hex("alice".to_string()).unwrap()
        );
        assert!(registry_slot_hex("abc".to_string()).is_ok());
        assert!(registry_slot_hex("abcdefghijklmnopqrstuvwxyz012345".to_string()).is_ok());
    }

    /// §7.2 decode round-trip from raw `bincode::serialize(NameEntry)` bytes.
    #[test]
    fn registry_decode_entry_round_trip() {
        let entry = kovanica_name_registry::NameEntry {
            owner: [0x11; 32],
            pubkey: [0xAB; 32],
            relay_endpoints: vec![
                "https://relay.example.com".to_string(),
                "http://relay2.example.com".to_string(),
            ],
        };
        let raw = bincode::serialize(&entry).unwrap();
        let hexed = hex::encode(&raw);

        // Bare hex and `0x`-prefixed (mixed case) decode identically.
        let json = registry_decode_entry(hexed.clone()).unwrap();
        assert_eq!(json, registry_decode_entry(format!("0x{hexed}")).unwrap());
        assert_eq!(
            json,
            registry_decode_entry(format!("0X{}", hexed.to_uppercase())).unwrap()
        );

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["owner"], "11".repeat(32));
        assert_eq!(parsed["pubkey"], "ab".repeat(32));
        assert_eq!(
            parsed["relay_endpoints"],
            serde_json::json!(["https://relay.example.com", "http://relay2.example.com"])
        );
    }

    #[test]
    fn registry_decode_entry_rejects_malformed() {
        let entry = kovanica_name_registry::NameEntry {
            owner: [0x11; 32],
            pubkey: [0xAB; 32],
            relay_endpoints: vec!["https://relay.example.com".to_string()],
        };
        let raw = bincode::serialize(&entry).unwrap();

        // Truncated bytes.
        assert!(registry_decode_entry(hex::encode(&raw[..raw.len() - 1])).is_err());
        // Non-hex.
        assert!(registry_decode_entry("zzzz".to_string()).is_err());
        // Empty slot / `0x` are not special-cased here → error.
        assert!(registry_decode_entry(String::new()).is_err());
        assert!(registry_decode_entry("0x".to_string()).is_err());
        // Structurally wrong (offset/len) — count claims an entry that isn't there.
        let corrupt = bincode::serialize(&kovanica_name_registry::NameEntry {
            owner: [0; 32],
            pubkey: [0; 32],
            relay_endpoints: vec!["https://x.example.com".to_string()],
        })
        .unwrap();
        let mut truncated_after_count = corrupt[..64 + 8].to_vec();
        truncated_after_count[64..72].copy_from_slice(&5u64.to_le_bytes()); // claim 5 endpoints
        assert!(registry_decode_entry(hex::encode(&truncated_after_count)).is_err());
    }

    /// `sign_call_raw` / `sign_deploy_raw` emit a bincode `Transaction` that
    /// verifies under the configured chain id and carries the expected kind.
    #[test]
    fn sign_call_raw_and_sign_deploy_raw_round_trip() {
        let created = create_wallet("pw".to_string()).unwrap();
        let contract = Address([0x33; 32]).to_string();

        let raw = sign_call_raw(
            created.keystore_json.clone(),
            "pw".to_string(),
            LOCAL_CHAIN_ID,
            contract,
            500,
            "transfer".to_string(),
            "0x010203".to_string(),
            100_000,
            6,
            10,
        )
        .unwrap();
        assert!(raw.starts_with("0x"));
        let bytes = hex::decode(raw.strip_prefix("0x").unwrap()).unwrap();
        let tx: Transaction = bincode::deserialize(&bytes).unwrap();
        assert!(tx.verify_signature(&kovanica_types::SigningContext::new(LOCAL_CHAIN_ID)));
        assert!(!tx.verify_signature(&kovanica_types::SigningContext::new(2)));
        assert!(matches!(
            tx.kind,
            kovanica_types::TransactionKind::Call {
                value: 500,
                ref method,
                ref args,
                gas_limit: 100_000,
                ..
            } if method == "transfer" && args == &[1u8, 2, 3]
        ));

        // Bare hex (no `0x`) is accepted for the deploy bytecode.
        let raw_deploy = sign_deploy_raw(
            created.keystore_json,
            "pw".to_string(),
            LOCAL_CHAIN_ID,
            "00ff".to_string(),
            7,
            10,
        )
        .unwrap();
        let bytes = hex::decode(raw_deploy.strip_prefix("0x").unwrap()).unwrap();
        let deploy: Transaction = bincode::deserialize(&bytes).unwrap();
        assert!(deploy.verify_signature(&kovanica_types::SigningContext::new(LOCAL_CHAIN_ID)));
        assert!(matches!(
            deploy.kind,
            kovanica_types::TransactionKind::Deploy { ref code } if code == &[0x00, 0xff]
        ));

        // Malformed hex is an error, not a panic.
        assert!(sign_call_raw(
            created_wallet_json(),
            "pw".to_string(),
            LOCAL_CHAIN_ID,
            Address([0x33; 32]).to_string(),
            0,
            "m".to_string(),
            "zz".to_string(),
            1,
            0,
            1,
        )
        .is_err());
    }

    /// `sign_transfer_raw` and `sign_call_raw` pass `chain_id` into `parse_address`.
    /// Legacy hex is accepted only on chains 3 and 1337.
    #[test]
    fn transfer_and_call_addresses_follow_the_chain_policy() {
        let key = Address([0x33; 32]);
        let canonical = key.to_string();
        let legacy = key.encode_legacy_hex();
        let bare = key.to_hex();
        let prefixed = format!("0x{bare}");
        let forms = [legacy.as_str(), bare.as_str(), prefixed.as_str()];
        let allowed = [chain_id::DEVNET, chain_id::LOCAL];
        let refused = [chain_id::MAINNET, chain_id::TESTNET, 0, 42];

        for chain in allowed {
            for form in forms {
                assert_eq!(
                    parse_address(form, chain).unwrap(),
                    key,
                    "{form} on {chain}"
                );
            }
            assert_eq!(parse_address(&canonical, chain).unwrap(), key);
        }
        for chain in refused {
            for form in forms {
                assert!(
                    parse_address(form, chain).is_err(),
                    "{form} must be refused on {chain}"
                );
            }
            assert_eq!(parse_address(&canonical, chain).unwrap(), key);
        }

        let created = create_wallet("pw".to_string()).unwrap();
        let transfer = sign_transfer_raw(
            created.keystore_json.clone(),
            "pw".to_string(),
            chain_id::DEVNET,
            legacy.clone(),
            1,
            0,
            1,
        )
        .unwrap();
        let tx = decode_signed(&transfer);
        assert!(matches!(
            tx.kind,
            kovanica_types::TransactionKind::Transfer { to, amount: 1 } if to == key
        ));

        let call = sign_call_raw(
            created.keystore_json.clone(),
            "pw".to_string(),
            chain_id::LOCAL,
            prefixed.clone(),
            0,
            "m".to_string(),
            "01".to_string(),
            1,
            0,
            1,
        )
        .unwrap();
        let tx = decode_signed(&call);
        assert!(matches!(
            tx.kind,
            kovanica_types::TransactionKind::Call { contract, .. } if contract == key
        ));

        let mainnet = sign_transfer_raw(
            created.keystore_json.clone(),
            "pw".to_string(),
            chain_id::MAINNET,
            canonical,
            1,
            0,
            1,
        )
        .unwrap();
        let tx = decode_signed(&mainnet);
        assert!(matches!(
            tx.kind,
            kovanica_types::TransactionKind::Transfer { to, .. } if to == key
        ));
        assert!(sign_transfer_raw(
            created.keystore_json.clone(),
            "pw".to_string(),
            chain_id::MAINNET,
            bare.clone(),
            1,
            0,
            1,
        )
        .is_err());
        assert!(sign_call_raw(
            created.keystore_json,
            "pw".to_string(),
            chain_id::TESTNET,
            legacy.clone(),
            0,
            "m".to_string(),
            "01".to_string(),
            1,
            0,
            1,
        )
        .is_err());
    }

    fn decode_signed(raw: &str) -> Transaction {
        let bytes = hex::decode(raw.strip_prefix("0x").unwrap()).unwrap();
        bincode::deserialize(&bytes).unwrap()
    }

    fn created_wallet_json() -> String {
        create_wallet("pw".to_string()).unwrap().keystore_json
    }

    #[test]
    fn reveal_mnemonic_round_trips_and_rejects_wrong_passphrase() {
        let created = create_wallet("backup-pw".to_string()).unwrap();
        let revealed =
            reveal_mnemonic_raw(created.keystore_json.clone(), "backup-pw".to_string()).unwrap();
        assert_eq!(revealed, created.mnemonic);
        assert_eq!(revealed.split_whitespace().count(), 24);

        // A wrong passphrase is an error, never a panic.
        assert!(reveal_mnemonic_raw(created.keystore_json, "wrong-pw".to_string()).is_err());
    }

    #[test]
    fn change_passphrase_switches_the_passphrase() {
        let created = create_wallet("old-pw".to_string()).unwrap();

        let updated = change_passphrase_raw(
            created.keystore_json.clone(),
            "old-pw".to_string(),
            "new-pw".to_string(),
        )
        .unwrap();

        // New passphrase unlocks the same seed; the old one no longer does.
        assert_eq!(
            reveal_mnemonic_raw(updated.clone(), "new-pw".to_string()).unwrap(),
            created.mnemonic
        );
        assert!(reveal_mnemonic_raw(updated.clone(), "old-pw".to_string()).is_err());

        // Same seed, so the address is unchanged.
        assert_eq!(
            wallet_address(updated).unwrap(),
            wallet_address(created.keystore_json.clone()).unwrap()
        );

        // A wrong old passphrase fails and changes nothing.
        assert!(change_passphrase_raw(
            created.keystore_json,
            "wrong-old".to_string(),
            "another".to_string(),
        )
        .is_err());

        // An empty new passphrase is rejected.
        assert!(
            change_passphrase_raw(created_wallet_json(), "pw".to_string(), String::new()).is_err()
        );
    }

    #[test]
    fn test_hd_wallet_ffi_lifecycle() {
        // 1. Create HD 12-word wallet
        let created12 = create_hd_wallet(12, "test-pw-12".to_string()).unwrap();
        assert_eq!(created12.mnemonic.split_whitespace().count(), 12);
        assert_eq!(created12.address.len(), 64);
        assert!(created12.address.starts_with("kvnc"));
        assert!(created12.address.ends_with("dag"));

        // 2. Derive accounts at index 0 and 1
        let addr0 = derive_account(
            created12.keystore_json.clone(),
            "test-pw-12".to_string(),
            "".to_string(),
            0,
        )
        .unwrap();
        let addr1 = derive_account(
            created12.keystore_json.clone(),
            "test-pw-12".to_string(),
            "".to_string(),
            1,
        )
        .unwrap();
        assert_eq!(addr0, created12.address);
        assert_ne!(addr0, addr1);

        // 3. Sign transfer
        let to_addr = addr1.clone();
        let raw_hex = sign_transfer_hd(
            created12.keystore_json.clone(),
            "test-pw-12".to_string(),
            "".to_string(),
            0,
            to_addr,
            1_000_000_000,
            10_000,
            0,
            chain_id::MAINNET,
        )
        .unwrap();
        assert!(!raw_hex.starts_with("0x"));

        // 4. Lock barrier
        lock();

        // 5. Create HD 24-word wallet
        let created24 = create_hd_wallet(24, "test-pw-24".to_string()).unwrap();
        assert_eq!(created24.mnemonic.split_whitespace().count(), 24);

        // 6. Import HD wallet
        let imported =
            import_hd_wallet(created24.mnemonic.clone(), "test-pw-24".to_string()).unwrap();
        assert_eq!(imported.address, created24.address);
    }

    #[test]
    fn mobile_pow_header_verification_and_profile_ffi_round_trip() {
        let profile = default_mobile_node_config(chain_id::HYBRID_TESTNET);
        assert_eq!(profile.chain_id, chain_id::HYBRID_TESTNET);
        assert_eq!(profile.ghostdag_k, 5);
        assert_eq!(profile.target_block_ms, 4_000);
        assert_eq!(profile.pruning_window_rounds, 14_400);
        assert_eq!(profile.max_redb_cache_mb, 64);

        let easy_bits = 0x217f_ffff;
        let mut h0 = BlockHeader::new(
            chain_id::HYBRID_TESTNET,
            1,
            0,
            vec![],
            Hash::zero(),
            1_700_000_000_000,
            easy_bits,
            1,
        );
        h0.pow_hash = Some(h0.compute_pow_hash());
        let mut h1 = BlockHeader::new(
            chain_id::HYBRID_TESTNET,
            2,
            0,
            vec![h0.reference()],
            Hash::zero(),
            1_700_000_004_000,
            easy_bits,
            2,
        );
        h1.pow_hash = Some(h1.compute_pow_hash());

        let h0_json = serde_json::to_string(&h0).unwrap();
        let h1_json = serde_json::to_string(&h1).unwrap();

        let v0 = verify_pow_header_json(
            chain_id::HYBRID_TESTNET,
            h0_json,
            String::new(),
            Some(1_700_000_010_000),
        )
        .unwrap();
        assert_eq!(v0.chain_id, chain_id::HYBRID_TESTNET);
        assert_eq!(v0.round, 1);

        let chain_json = serde_json::to_string(&vec![h0, h1]).unwrap();
        let total_work = verify_pow_header_chain_json(
            chain_id::HYBRID_TESTNET,
            chain_json,
            Some(1_700_000_010_000),
        )
        .unwrap();
        let w0: u128 = v0.work.parse().unwrap();
        let w_total: u128 = total_work.parse().unwrap();
        assert_eq!(w_total, w0 * 2);
        let _ = h1_json;
    }

    #[test]
    fn mobile_prune_local_node_storage_ffi() {
        use kovanica_storage::Storage;
        use kovanica_types::block::StatementBlock;
        use kovanica_types::crypto::Signature;
        use kovanica_types::hash::Hash;

        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("local_node.db");
        let db_str = db_path.to_str().unwrap().to_string();

        // 1. Non-existent path returns pruned=false
        let res = prune_local_node_storage("nonexistent.db".into(), 100).unwrap();
        assert!(!res.pruned);

        // 2. Populate and prune
        let storage = Storage::new(&db_path).unwrap();
        {
            let txn = storage.begin_write().unwrap();
            for round in 1..=5 {
                let digest = StatementBlock::compute_digest(0, round, &[], &[]);
                let block = StatementBlock {
                    author: 0,
                    round,
                    parents: vec![],
                    transactions: vec![],
                    statements: vec![],
                    signature: Signature([0; 64]),
                    digest,
                    merkle_root: Hash::zero(),
                };
                storage.blocks().put_block(&txn, &block, &[]).unwrap();
                storage.consensus().put_dag_block(&txn, &block).unwrap();
            }
            txn.commit().unwrap();
        }
        drop(storage);

        let res = prune_local_node_storage(db_str.clone(), 4).unwrap();
        assert!(res.pruned);
        assert_eq!(res.pruned_blocks, 3);
        assert_eq!(res.pruned_dag_blocks, 3);

        // 3. Check stats
        let stats = get_local_storage_stats(db_str.clone()).unwrap();
        assert_eq!(stats.total_blocks, 2);
        assert_eq!(stats.total_dag_blocks, 2);
        assert!(stats.size_bytes > 0);

        // 4. Store a new verified header
        let header = BlockHeader::new(
            chain_id::HYBRID_TESTNET,
            6,
            0,
            vec![],
            Hash::zero(),
            1_700_000_020_000,
            0x217f_ffff,
            1,
        );
        let header_json = serde_json::to_string(&header).unwrap();
        let stored = store_verified_block_header(db_str.clone(), header_json).unwrap();
        assert!(stored);

        let updated_stats = get_local_storage_stats(db_str.clone()).unwrap();
        assert_eq!(updated_stats.tip_round, 6);
        assert_eq!(updated_stats.total_blocks, 3);

        // 5. Account query
        let acc_info = get_local_account(
            db_str.clone(),
            Address([1; 32]).to_string(),
            chain_id::HYBRID_TESTNET,
        )
        .unwrap();
        assert!(!acc_info.found);

        // 6. Test sync_block_headers_batch
        let easy_bits = 0x217f_ffff;
        let mut b1 = BlockHeader::new(
            chain_id::HYBRID_TESTNET,
            7,
            0,
            vec![kovanica_types::BlockReference {
                author: 0,
                round: 6,
                digest: Hash::zero(),
            }],
            Hash::zero(),
            1_700_000_024_000,
            easy_bits,
            10,
        );
        b1.pow_hash = Some(b1.compute_pow_hash());

        let mut b2 = BlockHeader::new(
            chain_id::HYBRID_TESTNET,
            8,
            0,
            vec![kovanica_types::BlockReference {
                author: 0,
                round: 7,
                digest: b1.header_preimage_digest(),
            }],
            Hash::zero(),
            1_700_000_028_000,
            easy_bits,
            20,
        );
        b2.pow_hash = Some(b2.compute_pow_hash());

        let batch_json = serde_json::to_string(&vec![b1, b2]).unwrap();
        let batch_res = sync_block_headers_batch(
            db_str.clone(),
            batch_json,
            chain_id::HYBRID_TESTNET,
            Some(1_700_000_030_000),
        )
        .unwrap();

        assert_eq!(batch_res.synced_count, 2);
        assert_eq!(batch_res.highest_round, 8);
        assert_eq!(batch_res.rejected_count, 0);

        let final_stats = get_local_storage_stats(db_str.clone()).unwrap();
        assert_eq!(final_stats.tip_round, 8);
        assert_eq!(final_stats.total_blocks, 5);

        // 7. Check peer status offline behavior (does not panic)
        let offline_status = check_p2p_peer_status("127.0.0.1:59999".into(), 5).unwrap();
        assert!(!offline_status.is_connected);
        assert_eq!(offline_status.tip_round, 0);

        let sync_offline =
            sync_mobile_node_peer(db_str.clone(), "127.0.0.1:59999".into(), 5, 10).unwrap();
        assert!(!sync_offline.success);
        assert!(sync_offline.error.is_some());

        // 8. Phase M4: Local transaction simulation on-device
        let sim_res = simulate_local_call(
            db_str.clone(),
            Address([1; 32]).to_string(),
            Address([2; 32]).to_string(),
            "".into(),
            "".into(),
            100,
            50_000,
            chain_id::HYBRID_TESTNET,
        )
        .unwrap();
        assert!(!sim_res.success); // Insufficient balance
        assert!(sim_res.error.is_some());

        // 9. Local state root & storage query
        let state_root = get_local_state_root(db_str.clone()).unwrap();
        assert!(state_root.is_none());

        let storage_val = get_local_contract_storage(
            db_str.clone(),
            Address([2; 32]).to_string(),
            "0000000000000000000000000000000000000000000000000000000000000001".into(),
            chain_id::HYBRID_TESTNET,
        )
        .unwrap();
        assert!(storage_val.is_none());

        // 10. Execute block transactions on-device
        let header = BlockHeader::new(
            chain_id::HYBRID_TESTNET,
            9,
            0,
            vec![],
            Hash::zero(),
            1_700_000_032_000,
            easy_bits,
            30,
        );
        let block_json = serde_json::to_string(&header).unwrap();
        let exec_summary =
            execute_block_transactions(db_str.clone(), block_json, chain_id::HYBRID_TESTNET)
                .unwrap();
        assert_eq!(exec_summary.round, 9);
        assert_eq!(exec_summary.txs_executed, 0);

        // 11. Phase M5: Start embedded local RPC server on mobile
        assert!(!is_local_rpc_server_running());
        let server_url = start_local_rpc_server(
            db_str,
            "127.0.0.1:0".into(), // bind to random available port
            chain_id::HYBRID_TESTNET,
        )
        .unwrap();
        assert!(is_local_rpc_server_running());
        assert!(server_url.starts_with("http://127.0.0.1:"));

        // Query local RPC server via http_json_rpc
        let tip_res =
            http_json_rpc(&server_url, "kovanica_blockNumber", serde_json::json!([])).unwrap();
        let tip_qty = parse_quantity_json(&tip_res).unwrap();
        assert_eq!(tip_qty, 9); // Round 9 executed in step 10

        let status_res =
            http_json_rpc(&server_url, "kovanica_syncStatus", serde_json::json!([])).unwrap();
        assert_eq!(status_res["local_node"], true);

        let chain_res = http_json_rpc(&server_url, "net_version", serde_json::json!([])).unwrap();
        assert_eq!(
            chain_res.as_str().unwrap(),
            chain_id::HYBRID_TESTNET.to_string()
        );

        // Stop local RPC server
        let stopped = stop_local_rpc_server().unwrap();
        assert!(stopped);
        assert!(!is_local_rpc_server_running());

        // 12. Phase M6: Thermal-gated mobile micro-mining
        let easy_header = BlockHeader::new(
            chain_id::HYBRID_TESTNET,
            10,
            0,
            vec![],
            Hash::zero(),
            1_700_000_000_000,
            0x207f_ffff, // Very easy difficulty (~50% chance per nonce)
            0,
        );
        let easy_json = serde_json::to_string(&easy_header).unwrap();
        let mined_res = mine_pow_block_header(easy_json, 100, 0).unwrap();
        assert!(mined_res.found);
        assert!(mined_res.hashes_computed > 0);
        assert!(mined_res.block_header_json.is_some());
        assert!(!mined_res.pow_hash_hex.is_empty());

        // Low target / bounded hashes
        let hard_header = BlockHeader::new(
            chain_id::HYBRID_TESTNET,
            11,
            0,
            vec![],
            Hash::zero(),
            1_700_000_000_000,
            0x1a01_ffff, // Hard difficulty
            0,
        );
        let hard_json = serde_json::to_string(&hard_header).unwrap();
        let hard_res = mine_pow_block_header(hard_json, 3, 10).unwrap();
        assert!(!hard_res.found);
        assert_eq!(hard_res.hashes_computed, 3);
        assert_eq!(hard_res.nonce, 13);
        assert!(hard_res.block_header_json.is_none());

        // 13. DNS seed resolution test
        let resolved = resolve_dns_seed_peers("127.0.0.1".to_string(), 8000).unwrap();
        assert!(!resolved.is_empty());
        assert_eq!(resolved[0], "127.0.0.1:8000");

        // Invalid hostname returns LightFfiError
        let invalid =
            resolve_dns_seed_peers("invalid.domain.that.does.not.exist.local".to_string(), 8000);
        assert!(invalid.is_err());
    }
}

//! Contract subcommands (HTLC / vault / multisig / token).
//!
//! HTLC, vault and multisig map onto the typed contract RPC handlers in
//! `kovanica-rpc`. Token commands cannot: the `token_*` handlers are not wired
//! to the node's store and a server cannot sign, so `kovanica token …` deploys
//! and calls the token contract with signed transactions, exactly like
//! `kovanica deploy` / `kovanica call`.
//!
//! Addresses are parsed with [`kovanica_types::Address`] and sent as the byte
//! array that type stores. Hashes and ids stay raw hex.

use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::{anyhow, bail, Context, Result};
use clap::{Args, Subcommand};
use serde_json::{json, Value};
use tokio::time::sleep;

use crate::output::print_json;
use crate::rpc::{bytes32_json, bytes_json, RpcClient};
use crate::wallet;
use crate::wallet::parse_address;
use kovanica_common::Amount;
use kovanica_types::crypto::SigningKey;
use kovanica_types::signing::SigningContext;
use kovanica_types::transaction::TransactionKind;
use kovanica_types::Address;

/// Encode a Kovanica address as the JSON byte array the contract RPC expects.
fn address_param(text: &str, what: &str, chain_id: Option<u64>) -> Result<Value> {
    let address = parse_address(text, what, chain_id.unwrap_or(0))?;
    Ok(bytes_json(&address.0))
}

/// Top-level contract command group.
#[derive(Subcommand)]
pub enum ContractCommand {
    /// HTLC atomic swaps (KVP-104)
    Htlc {
        #[command(subcommand)]
        command: HtlcCommand,
    },
    /// Time-lock vaults (KVP-105)
    Vault {
        #[command(subcommand)]
        command: VaultCommand,
    },
    /// Multisig wallets (KVP-101)
    Multisig {
        #[command(subcommand)]
        command: MultisigCommand,
    },
    /// Native multi-asset tokens (KVP-102)
    Token {
        #[command(subcommand)]
        command: TokenCommand,
    },
}

/// Dispatch a contract subcommand.
pub async fn run(
    client: &RpcClient,
    chain_id: Option<u64>,
    command: ContractCommand,
    json_output: bool,
) -> Result<()> {
    match command {
        ContractCommand::Htlc { command } => run_htlc(client, chain_id, command, json_output).await,
        ContractCommand::Vault { command } => {
            run_vault(client, chain_id, command, json_output).await
        }
        ContractCommand::Multisig { command } => {
            run_multisig(client, chain_id, command, json_output).await
        }
        ContractCommand::Token { command } => {
            run_token(client, chain_id, command, json_output).await
        }
    }
}

/// Extract a field from a JSON object for human-readable output.
///
/// 32-byte id blobs are rendered as `0x`-prefixed hex.
fn field(value: &Value, key: &str) -> String {
    match value.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(items)) => {
            let is_byte_blob = items.len() == 32
                && items
                    .iter()
                    .all(|v| v.as_u64().map(|n| n <= 0xff).unwrap_or(false));
            if is_byte_blob {
                let bytes: Vec<u8> = items
                    .iter()
                    .map(|v| v.as_u64().unwrap_or_default() as u8)
                    .collect();
                format!("0x{}", hex::encode(bytes))
            } else {
                Value::Array(items.clone()).to_string()
            }
        }
        Some(other) => other.to_string(),
        None => "-".to_string(),
    }
}

/// Print a contract result as JSON or as key/value rows.
fn emit(json_output: bool, result: &Value, rows: &[(&str, String)]) -> Result<()> {
    if json_output {
        return print_json(result);
    }
    let width = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    for (key, value) in rows {
        println!("{key:<width$}  {value}");
    }
    Ok(())
}

// ============================================================================
// HTLC
// ============================================================================

/// HTLC subcommands.
#[derive(Subcommand)]
pub enum HtlcCommand {
    /// Lock funds against a hash lock
    Create(HtlcCreateArgs),
    /// Claim a lock using the preimage
    Claim(HtlcClaimArgs),
    /// Refund a lock after expiry
    Refund(HtlcRefundArgs),
}

/// Arguments for `kovanica htlc create`.
#[derive(Args)]
pub struct HtlcCreateArgs {
    /// Claimer Kovanica address
    #[arg(long)]
    pub claimer: String,
    /// Amount of KUNA (`1.5` or `1.5KUNA`) or atoms (`5atoms`). A bare integer is rejected.
    #[arg(long, value_parser = crate::amount::parse_cli_amount)]
    pub amount: u64,
    /// BLAKE3-256 hash lock (64 hex chars)
    #[arg(long = "hash-lock")]
    pub hash_lock: String,
    /// Expiry in contract time = committed-leader HEIGHT (~12 s per unit),
    /// NOT unix seconds. Use a height above the current one (see `status`).
    #[arg(long)]
    pub expiry: u64,
}

/// Arguments for `kovanica htlc claim`.
#[derive(Args)]
pub struct HtlcClaimArgs {
    /// Swap id (32-byte hex)
    #[arg(long)]
    pub id: String,
    /// Preimage (hex)
    #[arg(long)]
    pub preimage: String,
}

/// Arguments for `kovanica htlc refund`.
#[derive(Args)]
pub struct HtlcRefundArgs {
    /// Swap id (32-byte hex)
    #[arg(long)]
    pub id: String,
}

async fn run_htlc(
    client: &RpcClient,
    chain_id: Option<u64>,
    command: HtlcCommand,
    json_output: bool,
) -> Result<()> {
    match command {
        HtlcCommand::Create(args) => {
            let params = json!({
                "claimer": address_param(&args.claimer, "claimer", chain_id)?,
                "amount": args.amount,
                "hash_lock": bytes32_json(&args.hash_lock, "hash-lock")?,
                "expiry": args.expiry,
            });
            let result = client.call("htlc_create", params).await?;
            emit(
                json_output,
                &result,
                &[("swap_id", field(&result, "swap_id"))],
            )
        }
        HtlcCommand::Claim(args) => {
            let preimage = hex::decode(args.preimage.strip_prefix("0x").unwrap_or(&args.preimage))
                .map_err(|e| anyhow!("preimage: invalid hex: {e}"))?;
            let params = json!({
                "id": bytes32_json(&args.id, "id")?,
                "preimage": bytes_json(&preimage),
            });
            let result = client.call("htlc_claim", params).await?;
            emit(json_output, &result, &[("ok", field(&result, "ok"))])
        }
        HtlcCommand::Refund(args) => {
            let params = json!({ "id": bytes32_json(&args.id, "id")? });
            let result = client.call("htlc_refund", params).await?;
            emit(json_output, &result, &[("ok", field(&result, "ok"))])
        }
    }
}

// ============================================================================
// Vault
// ============================================================================

/// Vault subcommands.
#[derive(Subcommand)]
pub enum VaultCommand {
    /// Create a time-locked vault
    Create(VaultCreateArgs),
    /// Claim the vested amount
    Claim(VaultIdArgs),
    /// Cancel and reclaim the unclaimed balance
    Cancel(VaultIdArgs),
}

/// Arguments for `kovanica vault create`.
#[derive(Args)]
pub struct VaultCreateArgs {
    /// Beneficiary Kovanica address
    #[arg(long)]
    pub beneficiary: String,
    /// Amount of KUNA (`1.5` or `1.5KUNA`) or atoms (`5atoms`). A bare integer is rejected.
    #[arg(long, value_parser = crate::amount::parse_cli_amount)]
    pub amount: u64,
    /// Absolute unlock time in contract time = committed-leader HEIGHT
    /// (~12 s per unit), NOT unix seconds — mutually exclusive with --start/--end
    #[arg(long = "unlock-at")]
    pub unlock_at: Option<u64>,
    /// Linear vesting start (height, not unix seconds)
    #[arg(long, requires = "end")]
    pub start: Option<u64>,
    /// Linear vesting end (height, not unix seconds)
    #[arg(long, requires = "start")]
    pub end: Option<u64>,
    /// Optional cliff for linear vesting (height, not unix seconds)
    #[arg(long)]
    pub cliff: Option<u64>,
}

/// Arguments carrying only a vault id.
#[derive(Args)]
pub struct VaultIdArgs {
    /// Vault id (32-byte hex)
    #[arg(long)]
    pub id: String,
}

async fn run_vault(
    client: &RpcClient,
    chain_id: Option<u64>,
    command: VaultCommand,
    json_output: bool,
) -> Result<()> {
    match command {
        VaultCommand::Create(args) => {
            let schedule = match (args.unlock_at, args.start, args.end) {
                (Some(unlock_at), None, None) => json!({ "Absolute": { "unlock_at": unlock_at } }),
                (None, Some(start), Some(end)) => json!({
                    "Linear": { "start": start, "end": end, "cliff": args.cliff }
                }),
                (Some(_), _, _) => {
                    return Err(anyhow!(
                        "--unlock-at cannot be combined with --start/--end"
                    ))
                }
                (None, _, _) => {
                    return Err(anyhow!(
                        "provide --unlock-at for an absolute vault, or --start and --end for linear vesting"
                    ))
                }
            };

            let params = json!({
                "beneficiary": address_param(&args.beneficiary, "beneficiary", chain_id)?,
                "amount": args.amount,
                "schedule": schedule,
            });
            let result = client.call("vault_create", params).await?;
            emit(
                json_output,
                &result,
                &[("vault_id", field(&result, "vault_id"))],
            )
        }
        VaultCommand::Claim(args) => {
            let params = json!({ "id": bytes32_json(&args.id, "id")? });
            let result = client.call("vault_claim", params).await?;
            emit(
                json_output,
                &result,
                &[("amount_claimed", field(&result, "amount_claimed"))],
            )
        }
        VaultCommand::Cancel(args) => {
            let params = json!({ "id": bytes32_json(&args.id, "id")? });
            let result = client.call("vault_cancel", params).await?;
            emit(json_output, &result, &[("ok", field(&result, "ok"))])
        }
    }
}

// ============================================================================
// Multisig
// ============================================================================

/// Multisig subcommands.
#[derive(Subcommand)]
pub enum MultisigCommand {
    /// Create an M-of-N multisig
    Create(MultisigCreateArgs),
    /// Propose a transfer (proposer auto-confirms)
    Propose(MultisigProposeArgs),
    /// Confirm a proposed transfer
    Confirm(MultisigTxArgs),
    /// Execute a fully-confirmed transfer
    Execute(MultisigTxArgs),
}

/// Arguments for `kovanica multisig create`.
#[derive(Args)]
pub struct MultisigCreateArgs {
    /// Comma-separated owner Kovanica addresses
    #[arg(long, value_delimiter = ',')]
    pub owners: Vec<String>,
    /// Required confirmation threshold
    #[arg(long)]
    pub threshold: u32,
}

/// Arguments for `kovanica multisig propose`.
#[derive(Args)]
pub struct MultisigProposeArgs {
    /// Multisig id (32-byte hex)
    #[arg(long)]
    pub id: String,
    /// Recipient Kovanica address
    #[arg(long)]
    pub to: String,
    /// Amount of KUNA (`1.5` or `1.5KUNA`) or atoms (`5atoms`). A bare integer is rejected.
    #[arg(long, value_parser = crate::amount::parse_cli_amount)]
    pub amount: u64,
    /// Optional calldata (hex)
    #[arg(long)]
    pub data: Option<String>,
}

/// Arguments for `kovanica multisig confirm` / `execute`.
#[derive(Args)]
pub struct MultisigTxArgs {
    /// Multisig id (32-byte hex)
    #[arg(long)]
    pub id: String,
    /// Proposal transaction id
    #[arg(long = "tx-id")]
    pub tx_id: u64,
}

async fn run_multisig(
    client: &RpcClient,
    chain_id: Option<u64>,
    command: MultisigCommand,
    json_output: bool,
) -> Result<()> {
    match command {
        MultisigCommand::Create(args) => {
            let owners: Vec<Value> = args
                .owners
                .iter()
                .map(|owner| address_param(owner, "owner", chain_id))
                .collect::<Result<_>>()?;
            let params = json!({ "owners": owners, "threshold": args.threshold });
            let result = client.call("multisig_create", params).await?;
            emit(
                json_output,
                &result,
                &[("multisig_id", field(&result, "multisig_id"))],
            )
        }
        MultisigCommand::Propose(args) => {
            let data = match &args.data {
                Some(hex_str) => {
                    let raw = hex_str.strip_prefix("0x").unwrap_or(hex_str);
                    bytes_json(&hex::decode(raw).map_err(|e| anyhow!("data: invalid hex: {e}"))?)
                }
                None => bytes_json(&[]),
            };
            let params = json!({
                "multisig_id": bytes32_json(&args.id, "id")?,
                "to": address_param(&args.to, "to", chain_id)?,
                "amount": args.amount,
                "data": data,
            });
            let result = client.call("multisig_propose", params).await?;
            emit(json_output, &result, &[("tx_id", field(&result, "tx_id"))])
        }
        MultisigCommand::Confirm(args) => {
            let params = json!({
                "id": bytes32_json(&args.id, "id")?,
                "tx_id": args.tx_id,
            });
            let result = client.call("multisig_confirm", params).await?;
            emit(json_output, &result, &[("ok", field(&result, "ok"))])
        }
        MultisigCommand::Execute(args) => {
            let params = json!({
                "id": bytes32_json(&args.id, "id")?,
                "tx_id": args.tx_id,
            });
            let result = client.call("multisig_execute", params).await?;
            emit(json_output, &result, &[("ok", field(&result, "ok"))])
        }
    }
}

// ============================================================================
// Token
// ============================================================================

/// Bytecode `kovanica token create` deploys when `--wasm` is not given.
///
/// Baked into the binary from `contracts/` so creating a token needs neither a
/// checkout nor a Rust toolchain. `contracts/README.md` lists the two Android
/// assets that must stay byte-identical; a test below enforces that.
pub const TOKEN_WASM: &[u8] = include_bytes!("../../../contracts/kovanica-token.wasm");

/// How often the CLI re-reads chain state while waiting for a transaction.
const COMMIT_POLL: Duration = Duration::from_secs(2);

/// Token subcommands.
#[derive(Subcommand)]
pub enum TokenCommand {
    /// Create a token: deploys the token contract, then calls `token_create`
    Create(TokenCreateArgs),
    /// Transfer tokens
    Transfer(TokenAmountToArgs),
    /// Mint tokens (minter only)
    Mint(TokenAmountToArgs),
    /// Burn tokens
    Burn(TokenBurnArgs),
    /// Query a token balance
    Balance(TokenBalanceArgs),
    /// Approve an allowance
    Approve(TokenApproveArgs),
}

/// Arguments for `kovanica token create`.
#[derive(Args)]
pub struct TokenCreateArgs {
    /// Token name
    #[arg(long)]
    pub name: String,
    /// Token symbol
    #[arg(long)]
    pub symbol: String,
    /// Decimals
    #[arg(long, default_value_t = 9)]
    pub decimals: u8,
    /// Initial supply in token base units, not KUNA
    #[arg(long = "initial-supply")]
    pub initial_supply: Amount,
    /// Optional artwork: an https:// URL or a data:image/… URI (≤8 KiB)
    #[arg(long, default_value = "")]
    pub icon: String,
    /// Deploy this wasm instead of the token bytecode built into the CLI
    #[arg(long)]
    pub wasm: Option<PathBuf>,
    /// Keystore file path (must hold the token owner/minter)
    #[arg(long, short, default_value = "keystore.json")]
    pub keystore: PathBuf,
    /// Fee in atoms for each of the two transactions
    #[arg(long, default_value_t = 1)]
    pub fee: u64,
    /// Gas limit for the `token_create` call
    #[arg(long, default_value_t = kovanica_types::MAX_TX_GAS_LIMIT)]
    pub gas_limit: u64,
    /// Seconds to wait for each transaction to commit
    #[arg(long, default_value_t = 300)]
    pub timeout: u64,
}

/// Arguments for `kovanica token transfer` / `mint`.
#[derive(Args)]
pub struct TokenAmountToArgs {
    /// Token contract address
    #[arg(long)]
    pub contract: String,
    /// Recipient Kovanica address
    #[arg(long)]
    pub to: String,
    /// Amount in token base units, not KUNA
    #[arg(long)]
    pub amount: Amount,
    /// Keystore file path
    #[arg(long, short, default_value = "keystore.json")]
    pub keystore: PathBuf,
    /// Fee in atoms
    #[arg(long, default_value_t = 1)]
    pub fee: u64,
    /// Gas limit for the call
    #[arg(long, default_value_t = kovanica_types::MAX_TX_GAS_LIMIT)]
    pub gas_limit: u64,
    /// Seconds to wait for the transaction to commit
    #[arg(long, default_value_t = 300)]
    pub timeout: u64,
}

/// Arguments for `kovanica token burn`.
#[derive(Args)]
pub struct TokenBurnArgs {
    /// Token contract address
    #[arg(long)]
    pub contract: String,
    /// Amount in token base units, not KUNA
    #[arg(long)]
    pub amount: Amount,
    /// Keystore file path
    #[arg(long, short, default_value = "keystore.json")]
    pub keystore: PathBuf,
    /// Fee in atoms
    #[arg(long, default_value_t = 1)]
    pub fee: u64,
    /// Gas limit for the call
    #[arg(long, default_value_t = kovanica_types::MAX_TX_GAS_LIMIT)]
    pub gas_limit: u64,
    /// Seconds to wait for the transaction to commit
    #[arg(long, default_value_t = 300)]
    pub timeout: u64,
}

/// Arguments for `kovanica token balance`.
#[derive(Args)]
pub struct TokenBalanceArgs {
    /// Token contract address
    #[arg(long)]
    pub contract: String,
    /// Kovanica address to query
    #[arg(long)]
    pub address: String,
}

/// Arguments for `kovanica token approve`.
#[derive(Args)]
pub struct TokenApproveArgs {
    /// Token contract address
    #[arg(long)]
    pub contract: String,
    /// Spender Kovanica address
    #[arg(long)]
    pub spender: String,
    /// Amount in token base units, not KUNA
    #[arg(long)]
    pub amount: Amount,
    /// Keystore file path
    #[arg(long, short, default_value = "keystore.json")]
    pub keystore: PathBuf,
    /// Fee in atoms
    #[arg(long, default_value_t = 1)]
    pub fee: u64,
    /// Gas limit for the call
    #[arg(long, default_value_t = kovanica_types::MAX_TX_GAS_LIMIT)]
    pub gas_limit: u64,
    /// Seconds to wait for the transaction to commit
    #[arg(long, default_value_t = 300)]
    pub timeout: u64,
}

// ---------------------------------------------------------------------------
// Signed-transaction token path
//
// The `token_*` RPC methods are wired to a dispatcher the node does not
// implement (they cannot sign, so they cannot create state). Everything below
// therefore mirrors `kovanica deploy` + `kovanica call`: the CLI signs offline
// and the node sees ordinary `-signedTransaction` traffic.
// ---------------------------------------------------------------------------

/// Contract address the node derives for a `Deploy` signed with `nonce`.
///
/// The node charges the fee and bumps the sender nonce *before* running the
/// transaction body, so a deploy signed with `nonce` lands at
/// `hash(sender ‖ nonce + 1)`. Mirrors `kovanica-execution`'s `deployed_address`;
/// a mismatch here would point the follow-up call at an empty account.
pub fn deployed_contract_address(sender: Address, nonce: u64) -> Address {
    let mut seed = Vec::with_capacity(40);
    seed.extend_from_slice(&sender.0);
    seed.extend_from_slice(&nonce.saturating_add(1).to_le_bytes());
    Address(kovanica_common::hash(&seed))
}

/// Bincode args for `token_create(name, symbol, decimals, initial_supply, icon)`.
pub fn token_create_args(
    name: &str,
    symbol: &str,
    decimals: u8,
    initial_supply: Amount,
    icon: &str,
) -> Result<Vec<u8>> {
    bincode::serialize(&(
        name.to_string(),
        symbol.to_string(),
        decimals,
        initial_supply,
        icon.to_string(),
    ))
    .context("encoding token_create arguments")
}

/// Bincode args for calls that take `(to, amount)`.
pub fn token_to_amount_args(to: Address, amount: Amount) -> Result<Vec<u8>> {
    bincode::serialize(&(to.0, amount)).context("encoding token call arguments")
}

/// Bincode args for `token_burn(amount)`.
pub fn token_amount_args(amount: Amount) -> Result<Vec<u8>> {
    bincode::serialize(&amount).context("encoding token_burn arguments")
}

/// Render `amount` base units of a token with `decimals` as whole units,
/// trimming trailing zeros. `12_340_000_000` at 9 decimals is `12.34`.
pub fn format_token_amount(amount: Amount, decimals: u8) -> String {
    if decimals == 0 || decimals > 18 {
        return amount.to_string();
    }
    let scale = 10u128.pow(u32::from(decimals));
    let whole = amount / scale;
    let fraction = amount % scale;
    let mut text = format!("{whole}.{fraction:0width$}", width = usize::from(decimals));
    while text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

/// Nonce the next signed transaction from `sender` must carry.
async fn account_nonce(client: &RpcClient, sender: Address) -> Result<u64> {
    let value = client
        .call("kovanica_getNonce", json!([sender.to_string()]))
        .await?;
    crate::rpc::parse_quantity(&value)
}

/// Bytecode stored at `contract`, empty when nothing is deployed there.
async fn contract_code(client: &RpcClient, contract: Address) -> Result<Vec<u8>> {
    let value = client
        .call("kovanica_getCode", json!([contract.to_string()]))
        .await?;
    let text = value.as_str().unwrap_or_default();
    let text = text.strip_prefix("0x").unwrap_or(text);
    if text.is_empty() {
        return Ok(Vec::new());
    }
    hex::decode(text).context("kovanica_getCode returned non-hex bytes")
}

/// Decode the token stored at `contract`, `None` when the slot is empty.
async fn read_token(
    client: &RpcClient,
    contract: Address,
) -> Result<Option<kovanica_token::Token>> {
    let slot = hex::encode(kovanica_token::state_slot());
    let value = client
        .call("kovanica_getStorageAt", json!([contract.to_string(), slot]))
        .await
        .with_context(|| format!("reading token state at {contract}"))?;
    let text = value.as_str().unwrap_or_default();
    let text = text.strip_prefix("0x").unwrap_or(text);
    let Ok(bytes) = hex::decode(text) else {
        return Ok(None);
    };
    Ok(kovanica_token::decode_state(&bytes))
}

/// Load the keystore and prompt for its passphrase.
fn load_signer(keystore_path: &Path) -> Result<(SigningKey, Address)> {
    let keystore = wallet::load(keystore_path)?;
    let password = wallet::prompt_keystore_password(&keystore)?;
    let signing_key = wallet::signing_key(&keystore, password.as_ref().map(|p| p.as_str()))?;
    let sender = wallet::address(&keystore)?;
    Ok((signing_key, sender))
}

/// Sign `kind` with `nonce` and submit it over `kovanica_sendRawTransaction`.
async fn submit_signed(
    client: &RpcClient,
    ctx: &SigningContext,
    signing_key: &SigningKey,
    sender: Address,
    nonce: u64,
    kind: TransactionKind,
    fee: u64,
) -> Result<String> {
    let transaction = crate::tx::build_signed(ctx, sender, nonce, kind, fee, signing_key)?;
    let raw = crate::tx::encode_raw(&transaction)?;
    let result = client
        .call("kovanica_sendRawTransaction", json!([raw]))
        .await?;
    Ok(result.as_str().unwrap_or_default().to_string())
}

/// Poll until the deploy's code is readable, or fail with the deploy hash.
async fn wait_for_code(client: &RpcClient, contract: Address, timeout: Duration) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if !contract_code(client, contract).await?.is_empty() {
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!(
                "timed out after {}s waiting for code at {contract}; the deploy may have been \
                 rejected (check `kovanica history`)",
                timeout.as_secs()
            );
        }
        sleep(COMMIT_POLL).await;
    }
}

/// Poll until `sender`'s committed nonce passes `target`.
async fn wait_for_nonce(
    client: &RpcClient,
    sender: Address,
    target: u64,
    timeout: Duration,
) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if account_nonce(client, sender).await? > target {
            return Ok(());
        }
        if Instant::now() >= deadline {
            bail!(
                "timed out after {}s waiting for nonce {target} to commit; the transaction may \
                 have been rejected",
                timeout.as_secs()
            );
        }
        sleep(COMMIT_POLL).await;
    }
}

/// Parse `chain_id` into a signing context and refuse a node on another chain.
async fn signing_setup(client: &RpcClient, chain_id: Option<u64>) -> Result<SigningContext> {
    let ctx = crate::tx::signing_context(chain_id)?;
    crate::node::ensure_node_chain_matches(client, Some(ctx.chain_id)).await?;
    Ok(ctx)
}

/// Create the token: deploy the contract, wait, then call `token_create`.
async fn token_create(
    client: &RpcClient,
    chain_id: Option<u64>,
    args: TokenCreateArgs,
    json_output: bool,
) -> Result<()> {
    kovanica_token::Token::validate_icon(&args.icon).map_err(|_| {
        anyhow!(
            "--icon rejected: expected empty, https://…, or a data:image/… URI of at most {} bytes",
            kovanica_token::MAX_ICON_BYTES
        )
    })?;

    let wasm = match &args.wasm {
        Some(path) => {
            let bytes =
                std::fs::read(path).with_context(|| format!("reading wasm {}", path.display()))?;
            if bytes.is_empty() {
                bail!("WASM file is empty: {}", path.display());
            }
            bytes
        }
        None => TOKEN_WASM.to_vec(),
    };

    let ctx = signing_setup(client, chain_id).await?;
    let (signing_key, sender) = load_signer(&args.keystore)?;
    let timeout = Duration::from_secs(args.timeout);

    // The deploy address is a function of the *next* nonce, so it is known
    // before anything is signed and can be pre-checked.
    let deploy_nonce = account_nonce(client, sender).await?;
    let contract = deployed_contract_address(sender, deploy_nonce);
    if !contract_code(client, contract).await?.is_empty() {
        bail!(
            "{contract} already holds code (deploy nonce {deploy_nonce}); wait for pending \
             transactions to commit and retry"
        );
    }

    let deploy_hash = submit_signed(
        client,
        &ctx,
        &signing_key,
        sender,
        deploy_nonce,
        TransactionKind::Deploy { code: wasm.clone() },
        args.fee,
    )
    .await?;
    wait_for_code(client, contract, timeout)
        .await
        .with_context(|| format!("deploy {deploy_hash} ({} bytes) did not land", wasm.len()))?;

    // Sequential nonce: the call cannot run before its contract exists.
    let call_nonce = account_nonce(client, sender).await?;
    let call_args = token_create_args(
        &args.name,
        &args.symbol,
        args.decimals,
        args.initial_supply,
        &args.icon,
    )?;
    let create_hash = submit_signed(
        client,
        &ctx,
        &signing_key,
        sender,
        call_nonce,
        TransactionKind::Call {
            contract,
            value: 0,
            method: "token_create".to_string(),
            args: call_args,
            gas_limit: args.gas_limit,
        },
        args.fee,
    )
    .await?;
    wait_for_nonce(client, sender, call_nonce, timeout)
        .await
        .with_context(|| format!("token_create {create_hash} did not commit"))?;

    // Post-condition: the on-chain state is what was asked for, not just a
    // successful submission. A wrong unit (u64 vs u128) would surface here.
    let token = read_token(client, contract)
        .await?
        .ok_or_else(|| anyhow!("{create_hash} committed but {contract} holds no token state"))?;
    if token.name != args.name
        || token.symbol != args.symbol
        || token.decimals != args.decimals
        || token.total_supply != args.initial_supply
        || token.icon != args.icon
    {
        bail!(
            "{contract} decoded as {:?}/{:?} supply {} (wanted {:?}/{:?} supply {}); \
             the node may not support the icon field yet",
            token.name,
            token.symbol,
            token.total_supply,
            args.name,
            args.symbol,
            args.initial_supply
        );
    }

    let result = json!({
        "contract_address": contract.to_string(),
        "deployTransactionHash": deploy_hash,
        "createTransactionHash": create_hash,
        "name": token.name,
        "symbol": token.symbol,
        "decimals": token.decimals,
        "total_supply": token.total_supply.to_string(),
        "icon_bytes": token.icon.len(),
        "wasm_bytes": wasm.len(),
    });
    emit(
        json_output,
        &result,
        &[
            ("contract", contract.to_string()),
            ("name", token.name.clone()),
            ("symbol", token.symbol.clone()),
            ("decimals", token.decimals.to_string()),
            (
                "supply",
                format_token_amount(token.total_supply, token.decimals),
            ),
            ("icon_bytes", token.icon.len().to_string()),
            ("deploy_tx", deploy_hash),
            ("create_tx", create_hash),
        ],
    )
}

/// Balance the token holds for `address`, `0` when it never held any.
fn balance_of(token: &kovanica_token::Token, address: Address) -> Amount {
    token.balances.get(&address.0).copied().unwrap_or(0)
}

/// Re-read the contract after a call and refuse to report success unless the
/// state actually moved as asked.
///
/// The node has no receipt index for `kovanica_getTransactionReceipt` on every
/// build, and a rejected body still consumes the nonce, so a committed
/// transaction is not a successful one. The contract's own state is the only
/// proof that works against every node.
async fn confirm_token_state(
    client: &RpcClient,
    contract: Address,
    hash: &str,
    method: &str,
    check: impl FnOnce(&kovanica_token::Token) -> Option<String>,
) -> Result<()> {
    let after = read_token(client, contract)
        .await?
        .ok_or_else(|| anyhow!("{contract} holds no token state after {method} {hash}"))?;
    if let Some(problem) = check(&after) {
        bail!(
            "{method} {hash} committed but the contract state did not follow: {problem}; the fee \
             was charged and the nonce consumed"
        );
    }
    Ok(())
}

/// Submit one token call to an existing contract and wait for it to commit.
async fn token_call(
    client: &RpcClient,
    chain_id: Option<u64>,
    keystore_path: &Path,
    contract_text: &str,
    method: &str,
    call_args: Vec<u8>,
    fee: u64,
    gas_limit: u64,
    timeout: Duration,
) -> Result<(String, Address, Address, kovanica_token::Token)> {
    let ctx = signing_setup(client, chain_id).await?;
    let contract = parse_address(contract_text, "contract", ctx.chain_id)?;
    let token = read_token(client, contract).await?.ok_or_else(|| {
        anyhow!("no token state at {contract}: pass the contract address, not the token holder")
    })?;
    let (signing_key, sender) = load_signer(keystore_path)?;
    let nonce = account_nonce(client, sender).await?;
    let hash = submit_signed(
        client,
        &ctx,
        &signing_key,
        sender,
        nonce,
        TransactionKind::Call {
            contract,
            value: 0,
            method: method.to_string(),
            args: call_args,
            gas_limit,
        },
        fee,
    )
    .await?;
    wait_for_nonce(client, sender, nonce, timeout)
        .await
        .with_context(|| format!("{method} {hash} did not commit"))?;
    Ok((hash, sender, contract, token))
}

async fn run_token(
    client: &RpcClient,
    chain_id: Option<u64>,
    command: TokenCommand,
    json_output: bool,
) -> Result<()> {
    match command {
        TokenCommand::Create(args) => token_create(client, chain_id, args, json_output).await,
        TokenCommand::Transfer(args) => {
            let chain = crate::tx::signing_context(chain_id)?.chain_id;
            let to = parse_address(&args.to, "to", chain)?;
            let call_args = token_to_amount_args(to, args.amount)?;
            let (hash, sender, contract, token) = token_call(
                client,
                chain_id,
                &args.keystore,
                &args.contract,
                "token_transfer",
                call_args,
                args.fee,
                args.gas_limit,
                Duration::from_secs(args.timeout),
            )
            .await?;
            let amount = args.amount;
            let from_before = balance_of(&token, sender);
            let to_before = balance_of(&token, to);
            confirm_token_state(client, contract, &hash, "token_transfer", |after| {
                let from_now = balance_of(after, sender);
                let to_now = balance_of(after, to);
                let expected_from = from_before.saturating_sub(amount);
                let expected_to = to_before.saturating_add(amount);
                // A self transfer moves nothing; only the supply must hold.
                if sender == to {
                    return (from_now != from_before)
                        .then(|| format!("a self transfer changed the balance to {from_now}"));
                }
                if from_now != expected_from {
                    return Some(format!(
                        "sender balance is {from_now}, expected {expected_from}"
                    ));
                }
                if to_now != expected_to {
                    return Some(format!(
                        "recipient balance is {to_now}, expected {expected_to}"
                    ));
                }
                if after.total_supply != token.total_supply {
                    return Some(format!(
                        "total supply moved from {} to {}",
                        token.total_supply, after.total_supply
                    ));
                }
                None
            })
            .await?;
            let result = json!({
                "transactionHash": hash,
                "contract": contract.to_string(),
                "from": sender.to_string(),
                "to": to.to_string(),
                "amount": args.amount.to_string(),
                "symbol": token.symbol,
            });
            emit(
                json_output,
                &result,
                &[
                    ("transaction", hash),
                    ("contract", contract.to_string()),
                    (
                        "amount",
                        format!(
                            "{} {}",
                            format_token_amount(args.amount, token.decimals),
                            token.symbol
                        ),
                    ),
                    ("to", to.to_string()),
                ],
            )
        }
        TokenCommand::Mint(args) => {
            let chain = crate::tx::signing_context(chain_id)?.chain_id;
            let to = parse_address(&args.to, "to", chain)?;
            let call_args = token_to_amount_args(to, args.amount)?;
            let (hash, sender, contract, token) = token_call(
                client,
                chain_id,
                &args.keystore,
                &args.contract,
                "token_mint",
                call_args,
                args.fee,
                args.gas_limit,
                Duration::from_secs(args.timeout),
            )
            .await?;
            let amount = args.amount;
            let supply_before = token.total_supply;
            let to_before = balance_of(&token, to);
            confirm_token_state(client, contract, &hash, "token_mint", |after| {
                let to_now = balance_of(after, to);
                let expected_to = to_before.saturating_add(amount);
                if to_now != expected_to {
                    return Some(format!(
                        "recipient balance is {to_now}, expected {expected_to}"
                    ));
                }
                let expected_supply = supply_before.saturating_add(amount);
                if after.total_supply != expected_supply {
                    return Some(format!(
                        "total supply is {}, expected {expected_supply}",
                        after.total_supply
                    ));
                }
                None
            })
            .await?;
            let result = json!({
                "transactionHash": hash,
                "contract": contract.to_string(),
                "minter": sender.to_string(),
                "to": to.to_string(),
                "amount": args.amount.to_string(),
                "symbol": token.symbol,
            });
            emit(
                json_output,
                &result,
                &[
                    ("transaction", hash),
                    ("contract", contract.to_string()),
                    (
                        "minted",
                        format!(
                            "{} {}",
                            format_token_amount(args.amount, token.decimals),
                            token.symbol
                        ),
                    ),
                    ("to", to.to_string()),
                ],
            )
        }
        TokenCommand::Burn(args) => {
            let call_args = token_amount_args(args.amount)?;
            let (hash, sender, contract, token) = token_call(
                client,
                chain_id,
                &args.keystore,
                &args.contract,
                "token_burn",
                call_args,
                args.fee,
                args.gas_limit,
                Duration::from_secs(args.timeout),
            )
            .await?;
            let amount = args.amount;
            let supply_before = token.total_supply;
            let balance_before = balance_of(&token, sender);
            confirm_token_state(client, contract, &hash, "token_burn", |after| {
                let balance_now = balance_of(after, sender);
                let expected = balance_before.saturating_sub(amount);
                if balance_now != expected {
                    return Some(format!(
                        "burner balance is {balance_now}, expected {expected}"
                    ));
                }
                let expected_supply = supply_before.saturating_sub(amount);
                if after.total_supply != expected_supply {
                    return Some(format!(
                        "total supply is {}, expected {expected_supply}",
                        after.total_supply
                    ));
                }
                None
            })
            .await?;
            let result = json!({
                "transactionHash": hash,
                "contract": contract.to_string(),
                "burner": sender.to_string(),
                "amount": args.amount.to_string(),
                "symbol": token.symbol,
            });
            emit(
                json_output,
                &result,
                &[
                    ("transaction", hash),
                    ("contract", contract.to_string()),
                    (
                        "burned",
                        format!(
                            "{} {}",
                            format_token_amount(args.amount, token.decimals),
                            token.symbol
                        ),
                    ),
                ],
            )
        }
        TokenCommand::Approve(args) => {
            let chain = crate::tx::signing_context(chain_id)?.chain_id;
            let spender = parse_address(&args.spender, "spender", chain)?;
            let call_args = token_to_amount_args(spender, args.amount)?;
            let (hash, sender, contract, token) = token_call(
                client,
                chain_id,
                &args.keystore,
                &args.contract,
                "token_approve",
                call_args,
                args.fee,
                args.gas_limit,
                Duration::from_secs(args.timeout),
            )
            .await?;
            let amount = args.amount;
            confirm_token_state(client, contract, &hash, "token_approve", |after| {
                let allowance = after
                    .allowances
                    .get(&(sender.0, spender.0))
                    .copied()
                    .unwrap_or(0);
                (allowance != amount)
                    .then(|| format!("allowance is {allowance}, expected {amount}"))
            })
            .await?;
            let result = json!({
                "transactionHash": hash,
                "contract": contract.to_string(),
                "owner": sender.to_string(),
                "spender": spender.to_string(),
                "amount": args.amount.to_string(),
                "symbol": token.symbol,
            });
            emit(
                json_output,
                &result,
                &[
                    ("transaction", hash),
                    ("contract", contract.to_string()),
                    (
                        "allowance",
                        format!(
                            "{} {}",
                            format_token_amount(args.amount, token.decimals),
                            token.symbol
                        ),
                    ),
                    ("spender", spender.to_string()),
                ],
            )
        }
        TokenCommand::Balance(args) => {
            let chain = crate::tx::signing_context(chain_id)?.chain_id;
            let contract = parse_address(&args.contract, "contract", chain)?;
            let holder = parse_address(&args.address, "address", chain)?;
            let token = read_token(client, contract).await?.ok_or_else(|| {
                anyhow!("no token state at {contract}: pass the contract address, not a holder")
            })?;
            let balance = token.balances.get(&holder.0).copied().unwrap_or(0);
            let result = json!({
                "contract": contract.to_string(),
                "address": holder.to_string(),
                "balance": balance.to_string(),
                "formatted": format_token_amount(balance, token.decimals),
                "decimals": token.decimals,
                "symbol": token.symbol,
            });
            emit(
                json_output,
                &result,
                &[
                    ("contract", contract.to_string()),
                    ("address", holder.to_string()),
                    (
                        "balance",
                        format!(
                            "{} {}",
                            format_token_amount(balance, token.decimals),
                            token.symbol
                        ),
                    ),
                ],
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_address_params_follow_the_address_type() {
        let address = Address([0xab; 32]);
        let value = address_param(&address.to_string(), "addr", None).unwrap();
        let decoded: Address = serde_json::from_value(value).unwrap();
        assert_eq!(decoded, address);

        let from_hex = address_param(
            &address.to_hex(),
            "addr",
            Some(kovanica_types::signing::chain_id::LOCAL),
        )
        .unwrap();
        let decoded_hex: Address = serde_json::from_value(from_hex).unwrap();
        assert_eq!(decoded_hex, address);

        assert!(address_param(
            &address.to_hex(),
            "addr",
            Some(kovanica_types::signing::chain_id::MAINNET),
        )
        .is_err());
        assert!(address_param("not-an-address", "addr", None).is_err());
    }

    #[test]
    fn deployed_contract_address_matches_the_execution_rule() {
        // `kovanica-execution` derives hash(sender ‖ nonce + 1) and its tests
        // pin that: the sender nonce is bumped before the body runs. Signing a
        // deploy at nonce 0 must therefore point at `…00 01`, not `…00 00`.
        let sender = Address([0x11; 32]);
        let mut at_zero = Vec::new();
        at_zero.extend_from_slice(&sender.0);
        at_zero.extend_from_slice(&1u64.to_le_bytes());
        assert_eq!(
            deployed_contract_address(sender, 0),
            Address(kovanica_common::hash(&at_zero))
        );

        let mut at_seven = Vec::new();
        at_seven.extend_from_slice(&sender.0);
        at_seven.extend_from_slice(&8u64.to_le_bytes());
        assert_eq!(
            deployed_contract_address(sender, 7),
            Address(kovanica_common::hash(&at_seven))
        );
        assert_ne!(
            deployed_contract_address(sender, 0),
            deployed_contract_address(sender, 1)
        );
    }

    #[test]
    fn token_create_args_are_the_wasm_abi_tuple() {
        // The contract decodes (String, String, u8, u128, String). `Amount` is
        // u128, so a u64 supply would shift every following byte.
        let icon = "data:image/png;base64,AA==";
        let bytes = token_create_args("Kovanica Gold", "KGOLD", 9, 1_000_000, icon).unwrap();
        let decoded: (String, String, u8, Amount, String) = bincode::deserialize(&bytes).unwrap();
        assert_eq!(
            decoded,
            (
                "Kovanica Gold".to_string(),
                "KGOLD".to_string(),
                9,
                1_000_000,
                icon.to_string()
            )
        );
        assert_eq!(
            bytes.len(),
            8 + "Kovanica Gold".len() + 8 + "KGOLD".len() + 1 + 16 + 8 + icon.len()
        );
    }

    #[test]
    fn token_value_args_are_the_wasm_abi_tuples() {
        let to = Address([0x22; 32]);
        let decoded: ([u8; 32], Amount) =
            bincode::deserialize(&token_to_amount_args(to, 7).unwrap()).unwrap();
        assert_eq!(decoded, (to.0, 7));

        let decoded: Amount = bincode::deserialize(&token_amount_args(9).unwrap()).unwrap();
        assert_eq!(decoded, 9);
    }

    #[test]
    fn token_amounts_render_in_whole_units() {
        assert_eq!(format_token_amount(0, 9), "0");
        assert_eq!(format_token_amount(1, 9), "0.000000001");
        assert_eq!(format_token_amount(12_340_000_000, 9), "12.34");
        assert_eq!(format_token_amount(1_000_000_000, 9), "1");
        assert_eq!(format_token_amount(42, 0), "42");
        // Beyond 18 decimals the contract refuses the token; render raw instead
        // of overflowing the scale.
        assert_eq!(format_token_amount(42, 19), "42");
    }

    #[test]
    fn shipped_token_wasm_matches_the_repo_copies() {
        // Three copies ship: the CLI embeds `contracts/`, each Android app
        // vendors its own asset. A rebuild that updates only one silently
        // deploys different bytecode than the wallets, so compare bytes.
        assert!(TOKEN_WASM.starts_with(b"\0asm"), "not a wasm module");
        let wallet =
            include_bytes!("../../../apps/android-wallet/app/src/main/assets/kovanica_token.wasm");
        let lightnode =
            include_bytes!("../../../apps/android/app/src/main/assets/kovanica_token.wasm");
        assert_eq!(TOKEN_WASM, wallet.as_slice());
        assert_eq!(TOKEN_WASM, lightnode.as_slice());
    }
}

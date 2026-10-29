//! RPC method definitions and handlers for Kovanica contract entry points.

use crate::{execute_contract_call, RpcError, RpcState};
use kovanica_common::{Address, Amount, Hash, Height, Timestamp};
use kovanica_vault::VestingSchedule;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ============================================================================
// HTLC Methods
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct HtlcCreateParams {
    pub claimer: Address,
    pub amount: Amount,
    pub hash_lock: Hash,
    pub expiry: Timestamp,
}

#[derive(Debug, Serialize)]
pub struct HtlcCreateResult {
    pub swap_id: Hash,
}

#[derive(Debug, Deserialize)]
pub struct HtlcClaimParams {
    pub id: Hash,
    pub preimage: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub struct HtlcClaimResult {
    pub ok: bool,
}

#[derive(Debug, Deserialize)]
pub struct HtlcRefundParams {
    pub id: Hash,
}

#[derive(Debug, Serialize)]
pub struct HtlcRefundResult {
    pub ok: bool,
}

pub async fn handle_htlc_create(params: Value, _state: RpcState) -> Result<Value, RpcError> {
    let p: HtlcCreateParams = serde_json::from_value(params)
        .map_err(|e| RpcError::InvalidParams(format!("htlc_create: {}", e)))?;

    // Contract RPCs are not wired to the node yet (NotImplemented); these
    // placeholders are harmless and never reach execution.
    let contract: Address = [0u8; 32];
    let caller: Address = [0u8; 32];
    let height: Height = 0;
    let timestamp: Timestamp = 0;

    let args = bincode::serialize(&(p.claimer, p.amount, p.hash_lock, p.expiry))
        .map_err(|e| RpcError::InternalError(format!("encode args: {}", e)))?;

    let result_bytes =
        execute_contract_call("htlc_create", contract, caller, height, timestamp, &args).await?;

    let swap_id: Hash = bincode::deserialize(&result_bytes)
        .map_err(|e| RpcError::InternalError(format!("decode result: {}", e)))?;

    Ok(json!({ "swap_id": swap_id }))
}

pub async fn handle_htlc_claim(params: Value, _state: RpcState) -> Result<Value, RpcError> {
    let p: HtlcClaimParams = serde_json::from_value(params)
        .map_err(|e| RpcError::InvalidParams(format!("htlc_claim: {}", e)))?;

    let contract: Address = [0u8; 32];
    let caller: Address = [0u8; 32];
    let height: Height = 0;
    let timestamp: Timestamp = 0;

    let args = bincode::serialize(&(p.id, p.preimage))
        .map_err(|e| RpcError::InternalError(format!("encode args: {}", e)))?;

    execute_contract_call("htlc_claim", contract, caller, height, timestamp, &args).await?;

    Ok(json!({ "ok": true }))
}

pub async fn handle_htlc_refund(params: Value, _state: RpcState) -> Result<Value, RpcError> {
    let p: HtlcRefundParams = serde_json::from_value(params)
        .map_err(|e| RpcError::InvalidParams(format!("htlc_refund: {}", e)))?;

    let contract: Address = [0u8; 32];
    let caller: Address = [0u8; 32];
    let height: Height = 0;
    let timestamp: Timestamp = 0;

    let args = bincode::serialize(&p.id)
        .map_err(|e| RpcError::InternalError(format!("encode args: {}", e)))?;

    execute_contract_call("htlc_refund", contract, caller, height, timestamp, &args).await?;

    Ok(json!({ "ok": true }))
}

// ============================================================================
// Vault Methods
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct VaultCreateParams {
    pub beneficiary: Address,
    pub amount: Amount,
    pub schedule: VestingSchedule,
}

#[derive(Debug, Serialize)]
pub struct VaultCreateResult {
    pub vault_id: Hash,
}

#[derive(Debug, Deserialize)]
pub struct VaultClaimParams {
    pub id: Hash,
}

#[derive(Debug, Serialize)]
pub struct VaultClaimResult {
    pub amount_claimed: Amount,
}

#[derive(Debug, Deserialize)]
pub struct VaultCancelParams {
    pub id: Hash,
}

#[derive(Debug, Serialize)]
pub struct VaultCancelResult {
    pub ok: bool,
}

pub async fn handle_vault_create(params: Value, _state: RpcState) -> Result<Value, RpcError> {
    let p: VaultCreateParams = serde_json::from_value(params)
        .map_err(|e| RpcError::InvalidParams(format!("vault_create: {}", e)))?;

    let contract: Address = [0u8; 32];
    let caller: Address = [0u8; 32];
    let height: Height = 0;
    let timestamp: Timestamp = 0;

    let args = bincode::serialize(&(p.beneficiary, p.amount, p.schedule))
        .map_err(|e| RpcError::InternalError(format!("encode args: {}", e)))?;

    let result_bytes =
        execute_contract_call("vault_create", contract, caller, height, timestamp, &args).await?;

    let vault_id: Hash = bincode::deserialize(&result_bytes)
        .map_err(|e| RpcError::InternalError(format!("decode result: {}", e)))?;

    Ok(json!({ "vault_id": vault_id }))
}

pub async fn handle_vault_claim(params: Value, _state: RpcState) -> Result<Value, RpcError> {
    let p: VaultClaimParams = serde_json::from_value(params)
        .map_err(|e| RpcError::InvalidParams(format!("vault_claim: {}", e)))?;

    let contract: Address = [0u8; 32];
    let caller: Address = [0u8; 32];
    let height: Height = 0;
    let timestamp: Timestamp = 0;

    let args = bincode::serialize(&p.id)
        .map_err(|e| RpcError::InternalError(format!("encode args: {}", e)))?;

    let result_bytes =
        execute_contract_call("vault_claim", contract, caller, height, timestamp, &args).await?;

    let amount_claimed: Amount = bincode::deserialize(&result_bytes)
        .map_err(|e| RpcError::InternalError(format!("decode result: {}", e)))?;

    Ok(json!({ "amount_claimed": amount_claimed }))
}

pub async fn handle_vault_cancel(params: Value, _state: RpcState) -> Result<Value, RpcError> {
    let p: VaultCancelParams = serde_json::from_value(params)
        .map_err(|e| RpcError::InvalidParams(format!("vault_cancel: {}", e)))?;

    let contract: Address = [0u8; 32];
    let caller: Address = [0u8; 32];
    let height: Height = 0;
    let timestamp: Timestamp = 0;

    let args = bincode::serialize(&p.id)
        .map_err(|e| RpcError::InternalError(format!("encode args: {}", e)))?;

    execute_contract_call("vault_cancel", contract, caller, height, timestamp, &args).await?;

    Ok(json!({ "ok": true }))
}

// ============================================================================
// Multisig Methods
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct MultisigCreateParams {
    pub owners: Vec<Address>,
    pub threshold: u32,
}

#[derive(Debug, Serialize)]
pub struct MultisigCreateResult {
    pub multisig_id: Hash,
}

#[derive(Debug, Deserialize)]
pub struct MultisigProposeParams {
    pub multisig_id: Hash,
    pub to: Address,
    pub amount: Amount,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize)]
pub struct MultisigProposeResult {
    pub tx_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct MultisigConfirmParams {
    pub id: Hash,
    pub tx_id: u64,
}

#[derive(Debug, Serialize)]
pub struct MultisigConfirmResult {
    pub ok: bool,
}

#[derive(Debug, Deserialize)]
pub struct MultisigExecuteParams {
    pub id: Hash,
    pub tx_id: u64,
}

#[derive(Debug, Serialize)]
pub struct MultisigExecuteResult {
    pub ok: bool,
}

pub async fn handle_multisig_create(params: Value, _state: RpcState) -> Result<Value, RpcError> {
    let p: MultisigCreateParams = serde_json::from_value(params)
        .map_err(|e| RpcError::InvalidParams(format!("multisig_create: {}", e)))?;

    let contract: Address = [0u8; 32];
    let caller: Address = [0u8; 32];
    let height: Height = 0;
    let timestamp: Timestamp = 0;

    let args = bincode::serialize(&(p.owners, p.threshold))
        .map_err(|e| RpcError::InternalError(format!("encode args: {}", e)))?;

    let result_bytes = execute_contract_call(
        "multisig_create",
        contract,
        caller,
        height,
        timestamp,
        &args,
    )
    .await?;

    let multisig_id: Hash = bincode::deserialize(&result_bytes)
        .map_err(|e| RpcError::InternalError(format!("decode result: {}", e)))?;

    Ok(json!({ "multisig_id": multisig_id }))
}

pub async fn handle_multisig_propose(params: Value, _state: RpcState) -> Result<Value, RpcError> {
    let p: MultisigProposeParams = serde_json::from_value(params)
        .map_err(|e| RpcError::InvalidParams(format!("multisig_propose: {}", e)))?;

    let contract: Address = [0u8; 32];
    let caller: Address = [0u8; 32];
    let height: Height = 0;
    let timestamp: Timestamp = 0;

    let args = bincode::serialize(&(p.multisig_id, p.to, p.amount, p.data))
        .map_err(|e| RpcError::InternalError(format!("encode args: {}", e)))?;

    let result_bytes = execute_contract_call(
        "multisig_propose",
        contract,
        caller,
        height,
        timestamp,
        &args,
    )
    .await?;

    let tx_id: u64 = bincode::deserialize(&result_bytes)
        .map_err(|e| RpcError::InternalError(format!("decode result: {}", e)))?;

    Ok(json!({ "tx_id": tx_id }))
}

pub async fn handle_multisig_confirm(params: Value, _state: RpcState) -> Result<Value, RpcError> {
    let p: MultisigConfirmParams = serde_json::from_value(params)
        .map_err(|e| RpcError::InvalidParams(format!("multisig_confirm: {}", e)))?;

    let contract: Address = [0u8; 32];
    let caller: Address = [0u8; 32];
    let height: Height = 0;
    let timestamp: Timestamp = 0;

    let args = bincode::serialize(&(p.id, p.tx_id))
        .map_err(|e| RpcError::InternalError(format!("encode args: {}", e)))?;

    execute_contract_call(
        "multisig_confirm",
        contract,
        caller,
        height,
        timestamp,
        &args,
    )
    .await?;

    Ok(json!({ "ok": true }))
}

pub async fn handle_multisig_execute(params: Value, _state: RpcState) -> Result<Value, RpcError> {
    let p: MultisigExecuteParams = serde_json::from_value(params)
        .map_err(|e| RpcError::InvalidParams(format!("multisig_execute: {}", e)))?;

    let contract: Address = [0u8; 32];
    let caller: Address = [0u8; 32];
    let height: Height = 0;
    let timestamp: Timestamp = 0;

    let args = bincode::serialize(&(p.id, p.tx_id))
        .map_err(|e| RpcError::InternalError(format!("encode args: {}", e)))?;

    execute_contract_call(
        "multisig_execute",
        contract,
        caller,
        height,
        timestamp,
        &args,
    )
    .await?;

    Ok(json!({ "ok": true }))
}

// ============================================================================
// Token Methods
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct TokenCreateParams {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_supply: Amount,
    /// Optional artwork (`https://…` or `data:image/…`), empty when absent.
    #[serde(default)]
    pub icon: String,
}

#[derive(Debug, Serialize)]
pub struct TokenCreateResult {
    pub contract_address: Address,
}

#[derive(Debug, Deserialize)]
pub struct TokenTransferParams {
    pub to: Address,
    pub amount: Amount,
}

#[derive(Debug, Serialize)]
pub struct TokenTransferResult {
    pub ok: bool,
}

#[derive(Debug, Deserialize)]
pub struct TokenMintParams {
    pub to: Address,
    pub amount: Amount,
}

#[derive(Debug, Serialize)]
pub struct TokenMintResult {
    pub ok: bool,
}

#[derive(Debug, Deserialize)]
pub struct TokenBurnParams {
    pub amount: Amount,
}

#[derive(Debug, Serialize)]
pub struct TokenBurnResult {
    pub ok: bool,
}

#[derive(Debug, Deserialize)]
pub struct TokenBalanceParams {
    pub address: Address,
}

#[derive(Debug, Serialize)]
pub struct TokenBalanceResult {
    pub amount: Amount,
}

/// Token mutation over RPC is impossible: the server holds no keys, and token
/// state lives in the contract's storage, written only by an executed
/// transaction. Answer honestly instead of faking a zero contract address.
fn token_needs_signature(method: &str, cli: &str) -> RpcError {
    RpcError::NotImplemented(format!(
        "{method}: token state changes only through signed transactions, and this RPC server \
         holds no keys; use `{cli}` (or sign a Call and submit it with \
         kovanica_sendRawTransaction)"
    ))
}

pub async fn handle_token_create(params: Value, _state: RpcState) -> Result<Value, RpcError> {
    let _p: TokenCreateParams = serde_json::from_value(params)
        .map_err(|e| RpcError::InvalidParams(format!("token_create: {}", e)))?;

    Err(token_needs_signature(
        "token_create",
        "kovanica token create",
    ))
}

pub async fn handle_token_transfer(params: Value, _state: RpcState) -> Result<Value, RpcError> {
    let _p: TokenTransferParams = serde_json::from_value(params)
        .map_err(|e| RpcError::InvalidParams(format!("token_transfer: {}", e)))?;

    Err(token_needs_signature(
        "token_transfer",
        "kovanica token transfer",
    ))
}

pub async fn handle_token_mint(params: Value, _state: RpcState) -> Result<Value, RpcError> {
    let _p: TokenMintParams = serde_json::from_value(params)
        .map_err(|e| RpcError::InvalidParams(format!("token_mint: {}", e)))?;

    Err(token_needs_signature("token_mint", "kovanica token mint"))
}

pub async fn handle_token_burn(params: Value, _state: RpcState) -> Result<Value, RpcError> {
    let _p: TokenBurnParams = serde_json::from_value(params)
        .map_err(|e| RpcError::InvalidParams(format!("token_burn: {}", e)))?;

    Err(token_needs_signature("token_burn", "kovanica token burn"))
}

pub async fn handle_token_balance(params: Value, _state: RpcState) -> Result<Value, RpcError> {
    let _p: TokenBalanceParams = serde_json::from_value(params)
        .map_err(|e| RpcError::InvalidParams(format!("token_balance: {}", e)))?;

    // A read, but scoped by contract address rather than by holder: the shape
    // this handler advertises cannot answer it. The CLI reads the contract's
    // state slot directly, which needs no node upgrade.
    Err(RpcError::NotImplemented(
        "token_balance: balances are per token contract; read the contract's state slot with \
         kovanica_getStorageAt or run `kovanica token balance --contract`"
            .into(),
    ))
}

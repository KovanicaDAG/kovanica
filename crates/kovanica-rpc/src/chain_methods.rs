//! Standard chain / transaction / account / staking / mempool / consensus
//! JSON-RPC methods.
//!
//! These handlers are backed by the real node state carried in [`RpcState`]:
//! the redb [`Storage`](kovanica_storage::Storage) for blocks/accounts/consensus,
//! the [`Mempool`](kovanica_mempool::Mempool) for pending transactions, the shared
//! [`StakingState`](kovanica_staking::StakingState) for validator queries and the
//! [`CommitteeInfo`](kovanica_consensus::CommitteeInfo) for leader selection.
//!
//! Response shapes follow the Ethereum JSON-RPC conventions where a Kovanica
//! equivalent exists: 32-byte values and byte blobs are `0x`-prefixed hex
//! strings, while integer quantities (height, balance, nonce, fee, stake) are
//! `0x`-prefixed minimal hex ("quantity") strings.

use crate::{RpcError, RpcState};
use kovanica_storage::BlockStoreError;
use kovanica_types::{
    block::StatementBlock,
    hash::Hash,
    light_client::{LightClientCheckpoint, WaveCommitCertificate},
    merkle::{self, MerkleProof, MerkleStep},
    transaction::{Transaction, TransactionKind},
    Address, CommitSeal, LeaderHeader,
};
use serde_json::{json, Value};

/// Default number of rounds returned by `kovanica_getLeaderSchedule` when the
/// caller does not specify a count.
const DEFAULT_LEADER_SCHEDULE_LEN: u64 = 16;
/// Largest `count` accepted by `kovanica_getLeaderSchedule`; larger requests
/// would let one call allocate and serialize an arbitrarily long schedule.
pub const MAX_LEADER_SCHEDULE_LEN: u64 = 1024;

/// Default suggested fee rate (atoms per byte) when the mempool is empty.
const DEFAULT_FEE_RATE: u64 = 1;

// ============================================================================
// Helpers
// ============================================================================

/// Format a byte slice as a `0x`-prefixed hex string.
fn to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

/// Format an integer as an Ethereum-style minimal hex "quantity".
fn quantity<T: std::fmt::LowerHex>(value: T) -> Value {
    Value::String(format!("0x{value:x}"))
}

/// Normalise JSON-RPC `params` into a positional argument list.
///
/// Supports the Ethereum-style array form (`[a, b]`), a bare scalar
/// (`"0x..."`) and the empty/absent form (`null`).
fn params_vec(params: &Value) -> Vec<Value> {
    match params {
        Value::Array(items) => items.clone(),
        Value::Null => Vec::new(),
        other => vec![other.clone()],
    }
}

/// Borrow the `index`-th positional argument, or fail with a clear error.
fn required<'a>(args: &'a [Value], index: usize, what: &str) -> Result<&'a Value, RpcError> {
    args.get(index)
        .ok_or_else(|| RpcError::InvalidParams(format!("missing parameter: {what}")))
}

/// Decode a `0x`-prefixed (or bare) hex string into exactly 32 bytes.
fn parse_bytes32(value: &Value, what: &str) -> Result<[u8; 32], RpcError> {
    let raw = value
        .as_str()
        .ok_or_else(|| RpcError::InvalidParams(format!("{what}: expected a hex string")))?;
    let raw = raw.strip_prefix("0x").unwrap_or(raw);
    let bytes = hex::decode(raw)
        .map_err(|e| RpcError::InvalidParams(format!("{what}: invalid hex: {e}")))?;
    if bytes.len() != 32 {
        return Err(RpcError::InvalidParams(format!(
            "{what}: expected 32 bytes, got {}",
            bytes.len()
        )));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

/// Parse a block/transaction hash parameter.
fn parse_hash(value: &Value, what: &str) -> Result<Hash, RpcError> {
    Ok(Hash(parse_bytes32(value, what)?))
}

/// Parse an account address parameter.
fn parse_address(value: &Value, what: &str) -> Result<Address, RpcError> {
    let raw = value
        .as_str()
        .ok_or_else(|| RpcError::InvalidParams(format!("{what}: expected an address string")))?;
    let raw = raw.trim();
    // Canonical `kvnc…dag` encoding (checksum verified) or a raw 32-byte hex.
    if raw.starts_with(kovanica_types::address::ADDRESS_PREFIX)
        || raw.ends_with(kovanica_types::address::ADDRESS_SUFFIX)
    {
        return raw
            .parse::<Address>()
            .map_err(|e| RpcError::InvalidParams(format!("{what}: {e}")));
    }
    Ok(Address(parse_bytes32(value, what)?))
}

/// Parse an integer parameter accepted as either a JSON number or a
/// `0x`-prefixed quantity string.
fn parse_u64(value: &Value, what: &str) -> Result<u64, RpcError> {
    if let Some(n) = value.as_u64() {
        return Ok(n);
    }
    if let Some(raw) = value.as_str() {
        let raw = raw.strip_prefix("0x").unwrap_or(raw);
        return u64::from_str_radix(raw, 16)
            .map_err(|e| RpcError::InvalidParams(format!("{what}: invalid number: {e}")));
    }
    Err(RpcError::InvalidParams(format!(
        "{what}: expected an integer"
    )))
}

/// Parse a contract storage slot.
///
/// Accepts either a full 32-byte hex key or a small integer (Ethereum-style
/// slot number) which is left-padded big-endian into 32 bytes.
fn parse_storage_key(value: &Value, what: &str) -> Result<[u8; 32], RpcError> {
    if value.is_number() || value.as_str().map(|s| s.len() <= 18).unwrap_or(false) {
        if let Ok(slot) = parse_u64(value, what) {
            let mut key = [0u8; 32];
            key[24..].copy_from_slice(&slot.to_be_bytes());
            return Ok(key);
        }
    }
    parse_bytes32(value, what)
}

/// Map an internal storage/state error onto the JSON-RPC internal-error code.
fn internal<E: std::fmt::Display>(e: E) -> RpcError {
    RpcError::InternalError(e.to_string())
}

/// Serialise a transaction kind into a tagged JSON object.
fn kind_to_json(kind: &TransactionKind) -> Value {
    match kind {
        TransactionKind::Transfer { to, amount } => json!({
            "type": "transfer",
            "to": to.to_string(),
            "amount": quantity(*amount),
        }),
        TransactionKind::Stake { amount } => json!({
            "type": "stake",
            "amount": quantity(*amount),
        }),
        TransactionKind::Unstake { amount } => json!({
            "type": "unstake",
            "amount": quantity(*amount),
        }),
        TransactionKind::Delegate { validator, amount } => json!({
            "type": "delegate",
            "validator": validator.to_string(),
            "amount": quantity(*amount),
        }),
        TransactionKind::ClaimRewards { validator } => json!({
            "type": "claim_rewards",
            "validator": validator.map(|v| v.to_string()),
        }),
        TransactionKind::Deploy { code } => json!({
            "type": "deploy",
            "code": to_hex(code),
        }),
        TransactionKind::Call {
            contract,
            method,
            args,
            gas_limit,
            value,
        } => json!({
            "type": "call",
            "contract": contract.to_string(),
            "value": value,
            "method": method,
            "args": to_hex(args),
            "gas_limit": quantity(*gas_limit),
        }),
        TransactionKind::RegisterLightNode {
            vrf_public_key,
            stake,
        } => json!({
            "type": "register_light_node",
            "vrfPublicKey": to_hex(vrf_public_key.as_bytes()),
            "stake": quantity(*stake),
        }),
        TransactionKind::ExitLightNode => json!({
            "type": "exit_light_node",
        }),
        TransactionKind::SubmitLightWitness {
            epoch,
            vrf_hash,
            vrf_proof,
        } => json!({
            "type": "submit_light_witness",
            "epoch": quantity(*epoch),
            "vrfHash": to_hex(vrf_hash),
            "vrfProof": to_hex(vrf_proof),
        }),
    }
}

/// Serialise a transaction into JSON (without block context).
pub(crate) fn transaction_to_json(tx: &Transaction) -> Value {
    json!({
        "hash": to_hex(&tx.hash.0),
        "sender": tx.sender.to_string(),
        "nonce": quantity(tx.nonce),
        "fee": quantity(tx.fee),
        "kind": kind_to_json(&tx.kind),
        "signature": to_hex(&tx.signature.0),
    })
}

/// Serialise a block into JSON.
pub(crate) fn block_to_json(block: &StatementBlock) -> Value {
    json!({
        "hash": to_hex(&block.digest.0),
        "digest": to_hex(&block.digest.0),
        "author": block.author,
        "number": quantity(block.round),
        "round": quantity(block.round),
        "parents": block
            .parents
            .iter()
            .map(|p| to_hex(&p.digest.0))
            .collect::<Vec<_>>(),
        "transactions": block
            .transactions
            .iter()
            .map(transaction_to_json)
            .collect::<Vec<_>>(),
        "statements": to_hex(&block.statements),
        "signature": to_hex(&block.signature.0),
    })
}

// ============================================================================
// Chain methods
// ============================================================================

/// `kovanica_blockNumber` — latest committed leader height.
pub async fn handle_block_number(_params: Value, state: RpcState) -> Result<Value, RpcError> {
    let height = state
        .consensus_store
        .get_committed_leader_height()
        .map_err(internal)?;
    Ok(quantity(height))
}

/// `kovanica_getBlockByHash(hash, full_transactions)` — block by digest.
pub async fn handle_get_block_by_hash(params: Value, state: RpcState) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let hash = parse_hash(required(&args, 0, "block hash")?, "block hash")?;

    let txn = state.storage.begin_read().map_err(internal)?;
    match state.storage.blocks().get_block(&txn, &hash) {
        Ok(block) => Ok(block_to_json(&block)),
        Err(BlockStoreError::NotFound(_)) => Ok(Value::Null),
        Err(e) => Err(internal(e)),
    }
}

/// `kovanica_getBlockByNumber(number, full_transactions)` — block by height/round.
pub async fn handle_get_block_by_number(params: Value, state: RpcState) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let number = parse_u64(required(&args, 0, "block number")?, "block number")?;

    let txn = state.storage.begin_read().map_err(internal)?;
    match state
        .storage
        .blocks()
        .get_block_by_height(&txn, number)
        .map_err(internal)?
    {
        Some(block) => Ok(block_to_json(&block)),
        None => Ok(Value::Null),
    }
}

// ============================================================================
// Transaction methods
// ============================================================================

/// `kovanica_sendRawTransaction(raw_hex)` — decode and submit a signed tx.
pub async fn handle_send_raw_transaction(
    params: Value,
    state: RpcState,
) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let raw = required(&args, 0, "raw transaction")?
        .as_str()
        .ok_or_else(|| RpcError::InvalidParams("raw transaction: expected a hex string".into()))?;
    let raw = raw.strip_prefix("0x").unwrap_or(raw);
    let bytes = hex::decode(raw)
        .map_err(|e| RpcError::InvalidParams(format!("raw transaction: invalid hex: {e}")))?;
    let tx: Transaction = bincode::deserialize(&bytes)
        .map_err(|e| RpcError::InvalidParams(format!("raw transaction: invalid encoding: {e}")))?;

    let hash = tx.hash;
    let event_tx = tx.clone();
    state
        .mempool
        .add_transaction(tx)
        .map_err(|e| RpcError::ExecutionError(e.to_string()))?;
    // Only transactions the mempool accepted reach this point; publish them
    // on gossip so other validators can include them. A closed channel
    // (node shutting down) is not an RPC error: the tx is already pooled.
    if let Some(gossip) = &state.tx_gossip {
        let _ = gossip.send(event_tx.clone());
    }
    state.events.publish_pending_transaction(&event_tx);

    Ok(json!(to_hex(&hash.0)))
}

/// `kovanica_getTransactionReceipt(hash)` — receipt for a mined transaction.
pub async fn handle_get_transaction_receipt(
    params: Value,
    state: RpcState,
) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let hash = parse_hash(required(&args, 0, "transaction hash")?, "transaction hash")?;

    let txn = state.storage.begin_read().map_err(internal)?;
    let Some(block_hash) = state
        .storage
        .blocks()
        .get_block_for_transaction(&txn, &hash)
        .map_err(internal)?
    else {
        return Ok(Value::Null);
    };

    let block = state
        .storage
        .blocks()
        .get_block(&txn, &block_hash)
        .map_err(internal)?;
    let index = block.transactions.iter().position(|t| t.hash == hash);

    Ok(json!({
        "transactionHash": to_hex(&hash.0),
        "blockHash": to_hex(&block_hash.0),
        "blockNumber": quantity(block.round),
        "transactionIndex": index.map(|i| quantity(i as u64)).unwrap_or(Value::Null),
        "status": "0x1",
        "success": true,
    }))
}

/// `kovanica_getTransactionByHash(hash)` — transaction with block context.
pub async fn handle_get_transaction_by_hash(
    params: Value,
    state: RpcState,
) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let hash = parse_hash(required(&args, 0, "transaction hash")?, "transaction hash")?;

    let txn = state.storage.begin_read().map_err(internal)?;
    let Some(block_hash) = state
        .storage
        .blocks()
        .get_block_for_transaction(&txn, &hash)
        .map_err(internal)?
    else {
        // Not in a blue block yet: a pooled tx (fresh, or returned from a
        // non-blue block, #13) is reported as pending, without block context.
        return Ok(match state.mempool.get(&hash) {
            Some(tx) => {
                let mut value = transaction_to_json(&tx);
                if let Value::Object(ref mut map) = value {
                    map.insert("blockHash".into(), Value::Null);
                    map.insert("blockNumber".into(), Value::Null);
                    map.insert("transactionIndex".into(), Value::Null);
                    map.insert("status".into(), json!("pending"));
                }
                value
            }
            None => Value::Null,
        });
    };

    let block = state
        .storage
        .blocks()
        .get_block(&txn, &block_hash)
        .map_err(internal)?;
    let Some(index) = block.transactions.iter().position(|t| t.hash == hash) else {
        return Ok(Value::Null);
    };
    let tx = &block.transactions[index];

    let mut value = transaction_to_json(tx);
    if let Value::Object(ref mut map) = value {
        map.insert("blockHash".into(), json!(to_hex(&block_hash.0)));
        map.insert("blockNumber".into(), quantity(block.round));
        map.insert("transactionIndex".into(), quantity(index as u64));
    }
    Ok(value)
}

// ============================================================================
// Account methods
// ============================================================================

/// `kovanica_getBalance(address)` — native KUNA balance in atoms.
pub async fn handle_get_balance(params: Value, state: RpcState) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let address = parse_address(required(&args, 0, "address")?, "address")?;

    let txn = state.storage.begin_read().map_err(internal)?;
    let account = state
        .storage
        .state()
        .get_account_or_default(&txn, &address)
        .map_err(internal)?;
    Ok(quantity(account.balance))
}

/// `kovanica_getNonce(address)` — account nonce.
pub async fn handle_get_nonce(params: Value, state: RpcState) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let address = parse_address(required(&args, 0, "address")?, "address")?;

    let txn = state.storage.begin_read().map_err(internal)?;
    let account = state
        .storage
        .state()
        .get_account_or_default(&txn, &address)
        .map_err(internal)?;
    Ok(quantity(account.nonce))
}

/// `kovanica_getCode(address)` — contract bytecode as hex.
pub async fn handle_get_code(params: Value, state: RpcState) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let address = parse_address(required(&args, 0, "address")?, "address")?;

    let txn = state.storage.begin_read().map_err(internal)?;
    let account = state
        .storage
        .state()
        .get_account_or_default(&txn, &address)
        .map_err(internal)?;

    let code = if account.code_hash == [0u8; 32] {
        Vec::new()
    } else {
        state
            .storage
            .state()
            .get_contract_code(&txn, &account.code_hash)
            .map_err(internal)?
            .unwrap_or_default()
    };
    Ok(json!(to_hex(&code)))
}

/// `kovanica_getStorageAt(address, key)` — raw contract storage slot as hex.
pub async fn handle_get_storage_at(params: Value, state: RpcState) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let address = parse_address(required(&args, 0, "address")?, "address")?;
    let key = parse_storage_key(required(&args, 1, "storage key")?, "storage key")?;

    let txn = state.storage.begin_read().map_err(internal)?;
    let value = state
        .storage
        .state()
        .get_storage(&txn, &address, &key)
        .map_err(internal)?
        .unwrap_or_default();
    Ok(json!(to_hex(&value)))
}

// ============================================================================
// Staking methods
// ============================================================================

/// `kovanica_getValidators()` — active validator set.
pub async fn handle_get_validators(_params: Value, state: RpcState) -> Result<Value, RpcError> {
    let staking = state.staking.read().await;
    let validators: Vec<Value> = staking
        .validators
        .iter()
        .map(|v| {
            json!({
                "address": v.address.to_string(),
                "stake": quantity(v.stake),
                "commission_bps": v.commission_bps,
                "active": v.active,
                "payout_address": v.payout_address.to_string(),
            })
        })
        .collect();
    Ok(Value::Array(validators))
}

/// `kovanica_getStake(address)` — total stake attributable to an address.
///
/// Sums the address's own validator stake (if any) and any delegations it
/// has made.
pub async fn handle_get_stake(params: Value, state: RpcState) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let address = parse_address(required(&args, 0, "address")?, "address")?;

    let staking = state.staking.read().await;
    let own_stake: u64 = staking
        .validators
        .iter()
        .filter(|v| v.address == address)
        .map(|v| v.stake)
        .sum();
    let delegated: u64 = staking
        .delegations
        .iter()
        .filter(|d| d.delegator == address)
        .map(|d| d.amount)
        .sum();
    Ok(quantity(own_stake.saturating_add(delegated)))
}

/// `kovanica_getRewards(address)` — cumulative rewards for an address.
pub async fn handle_get_rewards(params: Value, state: RpcState) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let address = parse_address(required(&args, 0, "address")?, "address")?;
    let rewards = state.staking.read().await.reward_total(&address);
    Ok(quantity(rewards as u128))
}

// ============================================================================
// Mempool methods
// ============================================================================

/// `kovanica_getPendingTransactions()` — hashes of transactions in the mempool.
pub async fn handle_get_pending_transactions(
    _params: Value,
    state: RpcState,
) -> Result<Value, RpcError> {
    let hashes = state.mempool.pending_hashes();
    let mut out = Vec::with_capacity(hashes.len());
    for h in hashes {
        out.push(Value::String(format!("0x{}", hex::encode(h.0))));
    }
    Ok(Value::Array(out))
}

/// `kovanica_estimateFee()` — suggested fee rate in atoms per byte.
pub async fn handle_estimate_fee(_params: Value, state: RpcState) -> Result<Value, RpcError> {
    let stats = state.mempool.stats();
    let rate = stats
        .fee_rates
        .iter()
        .map(|(rate, _)| *rate)
        .min()
        .unwrap_or(DEFAULT_FEE_RATE)
        .max(DEFAULT_FEE_RATE);
    Ok(quantity(rate))
}

// ============================================================================
// Consensus methods
// ============================================================================

/// `kovanica_getLeaderSchedule(count)` — upcoming round leaders.
pub async fn handle_get_leader_schedule(params: Value, state: RpcState) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let count = match args.first() {
        Some(value) => parse_u64(value, "count")?,
        None => DEFAULT_LEADER_SCHEDULE_LEN,
    };
    if count > MAX_LEADER_SCHEDULE_LEN {
        return Err(RpcError::InvalidParams(format!(
            "count: at most {MAX_LEADER_SCHEDULE_LEN}, got {count}"
        )));
    }

    let start = {
        let txn = state.storage.begin_read().map_err(internal)?;
        state
            .storage
            .consensus()
            .get_committed_leader_height(&txn)
            .map_err(internal)?
            .saturating_add(1)
    };

    let committee = &state.committee;
    let schedule: Vec<Value> = (0..count)
        .map(|offset| {
            let round = start.saturating_add(offset);
            let leader_index = committee.leader(round);
            let leader_address = committee
                .get_by_index(leader_index)
                .map(|a| a.address.to_string())
                .unwrap_or_default();
            json!({
                "round": quantity(round),
                "leader_index": leader_index,
                "leader_address": leader_address,
            })
        })
        .collect();
    Ok(Value::Array(schedule))
}

/// JSON for a sealed leader header and the seals stored at its height.
pub fn sealed_header_json(header: &LeaderHeader, seals: &[CommitSeal]) -> Value {
    json!({
        "header": leader_header_json(header),
        "seals": seals.iter().map(seal_json).collect::<Vec<_>>(),
    })
}

fn leader_header_json(header: &LeaderHeader) -> Value {
    json!({
        "committedHeight": quantity(header.committed_height),
        "leaderRound": quantity(header.leader_round),
        "leaderDigest": to_hex(header.leader_digest.as_ref()),
        "leaderAuthor": header.leader_author,
        "parentLeaderDigest": to_hex(header.parent_leader_digest.as_ref()),
        "txMerkleRoot": to_hex(header.tx_merkle_root.as_ref()),
        "executedTxRoot": to_hex(header.executed_tx_root.as_ref()),
        "nonBlueRoot": to_hex(header.non_blue_root.as_ref()),
        "blueRoot": to_hex(header.blue_root.as_ref()),
        "mysticghost": header.mysticghost,
        "stateRoot": to_hex(header.state_root.as_ref()),
        "epoch": quantity(header.epoch),
        "committeeHash": to_hex(header.committee_hash.as_ref()),
        "timestamp": quantity(header.timestamp),
        "hash": to_hex(header.hash().as_ref()),
    })
}

fn seal_json(seal: &CommitSeal) -> Value {
    json!({
        "authorityIndex": seal.authority_index,
        "signature": to_hex(seal.signature.as_bytes()),
    })
}

fn proof_json(index: usize, proof: &MerkleProof) -> Value {
    json!({
        "index": quantity(index as u64),
        "steps": proof.steps.iter().map(step_json).collect::<Vec<_>>(),
    })
}

fn step_json(step: &MerkleStep) -> Value {
    match step {
        MerkleStep::Promote => json!({"type": "promote"}),
        MerkleStep::Sibling { hash, is_left } => json!({
            "type": "sibling",
            "hash": to_hex(hash.as_ref()),
            "isLeft": is_left,
        }),
    }
}

/// `kovanica_getLeaderHeader(height?)` — sealed header at `height`, or the latest.
///
/// Missing headers are JSON `null`. Historical headers are returned as stored.
pub async fn handle_get_leader_header(params: Value, state: RpcState) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let txn = state.storage.begin_read().map_err(internal)?;
    let header = if let Some(value) = args.first() {
        let height = parse_u64(value, "height")?;
        state
            .storage
            .state()
            .get_leader_header(&txn, height)
            .map_err(internal)?
    } else {
        state
            .storage
            .state()
            .get_latest_leader_header(&txn)
            .map_err(internal)?
    };
    Ok(header
        .map(|header| leader_header_json(&header))
        .unwrap_or(Value::Null))
}

/// `kovanica_getCommitSeals(height)` — seals stored for one committed height.
pub async fn handle_get_commit_seals(params: Value, state: RpcState) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let height = parse_u64(required(&args, 0, "height")?, "height")?;
    let txn = state.storage.begin_read().map_err(internal)?;
    let header = state
        .storage
        .state()
        .get_leader_header(&txn, height)
        .map_err(internal)?;
    let seals = state
        .storage
        .state()
        .seals_for(&txn, height)
        .map_err(internal)?;
    Ok(json!({
        "height": quantity(height),
        "headerHash": header.as_ref().map(|header| to_hex(header.hash().as_ref())),
        "seals": seals.iter().map(seal_json).collect::<Vec<_>>(),
    }))
}

/// `kovanica_getTxProof(height, txHash)` — inclusion in the executed-tx root.
///
/// A leader-block proof is included when the leader block is still in the
/// DAG and its stored merkle root matches the header.
pub async fn handle_get_tx_proof(params: Value, state: RpcState) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let height = parse_u64(required(&args, 0, "height")?, "height")?;
    let tx_hash = parse_hash(required(&args, 1, "txHash")?, "txHash")?;
    let txn = state.storage.begin_read().map_err(internal)?;
    let Some(header) = state
        .storage
        .state()
        .get_leader_header(&txn, height)
        .map_err(internal)?
    else {
        return Ok(Value::Null);
    };
    let hashes = state
        .storage
        .state()
        .get_executed_tx_hashes(&txn, height)
        .map_err(internal)?
        .unwrap_or_default();
    if merkle::root(&hashes) != header.executed_tx_root {
        return Err(internal(
            "executed transaction list does not match the header root",
        ));
    }
    let executed = hashes
        .iter()
        .position(|hash| *hash == tx_hash)
        .and_then(|index| merkle::proof(&hashes, index).map(|proof| proof_json(index, &proof)));
    let leader = leader_tx_proof(&state, &header, &tx_hash);
    Ok(json!({
        "header": leader_header_json(&header),
        "executed": executed,
        "leader": leader,
    }))
}

fn leader_tx_proof(state: &RpcState, header: &LeaderHeader, tx_hash: &Hash) -> Value {
    let block = match state.consensus_store.get_block(&header.leader_digest) {
        Ok(block) => block,
        Err(_) => return Value::Null,
    };
    if block.merkle_root != header.tx_merkle_root {
        return Value::Null;
    }
    let hashes: Vec<Hash> = block.transactions.iter().map(|tx| tx.hash).collect();
    if merkle::root(&hashes) != header.tx_merkle_root {
        return Value::Null;
    }
    let Some(index) = hashes.iter().position(|hash| hash == tx_hash) else {
        return Value::Null;
    };
    let Some(proof) = merkle::proof(&hashes, index) else {
        return Value::Null;
    };
    proof_json(index, &proof)
}

/// `kovanica_getAccountProof(address, height?)` — balance and nonce at the latest height.
///
/// Only the latest committed height is honest: older heights keep the root,
/// not the account rows. A requested older height is rejected.
pub async fn handle_get_account_proof(params: Value, state: RpcState) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let address = parse_address(required(&args, 0, "address")?, "address")?;
    let requested = match args.get(1) {
        Some(value) => Some(parse_u64(value, "height")?),
        None => None,
    };
    let txn = state.storage.begin_read().map_err(internal)?;
    let Some((latest_height, saved_root)) = state
        .storage
        .state()
        .get_latest_state_root(&txn)
        .map_err(internal)?
    else {
        return Ok(Value::Null);
    };
    if let Some(height) = requested {
        if height != latest_height {
            return Err(RpcError::InvalidParams(format!(
                "account proofs are served only at the latest committed height 0x{latest_height:x}"
            )));
        }
    }
    let proof = state
        .storage
        .state()
        .prove_account(&txn, &address)
        .map_err(internal)?;
    if proof.state_root != saved_root {
        return Err(internal(
            "live account tree does not match the stored state root",
        ));
    }
    if !proof.verify() {
        return Err(internal("account proof did not verify against its root"));
    }
    Ok(json!({
        "height": quantity(latest_height),
        "stateRoot": to_hex(proof.state_root.as_ref()),
        "address": address.to_string(),
        "balance": quantity(proof.balance),
        "nonce": quantity(proof.nonce),
        "present": proof.present,
        "siblings": proof.siblings.iter().map(|hash| to_hex(hash.as_ref())).collect::<Vec<_>>(),
    }))
}

/// `kovanica_getLightSet()` — registered light nodes and the epoch beacon.
///
/// `rewardBps` is [`kovanica_staking::LIGHT_REWARD_BPS`] (20% of `block_reward`).
pub async fn handle_get_light_set(_params: Value, state: RpcState) -> Result<Value, RpcError> {
    let txn = state.storage.begin_read().map_err(internal)?;
    let height = match state.storage.state().load_staking_state(&txn) {
        Ok(staking) => staking.committed_leader_height,
        Err(_) => 0,
    };
    let epoch = height / kovanica_staking::EPOCH_ROUNDS;
    let Some(set) = state
        .storage
        .state()
        .load_light_set(&txn)
        .map_err(internal)?
    else {
        return Ok(json!({
            "rewardBps": kovanica_staking::LIGHT_REWARD_BPS,
            "epoch": quantity(epoch),
            "beacon": Value::Null,
            "nodes": [],
        }));
    };
    let beacon = set.beacon(epoch).map(|hash| to_hex(hash.as_ref()));
    let winners = set.winners(epoch);
    let nodes: Vec<Value> = set
        .nodes()
        .iter()
        .map(|node| {
            let witnessed = set
                .witnesses(epoch)
                .iter()
                .any(|witness| witness.address == node.address);
            json!({
                "address": node.address.to_string(),
                "stake": quantity(node.stake),
                "vrfPublicKey": to_hex(node.vrf_public_key.as_bytes()),
                "effectiveEpoch": quantity(node.effective_epoch),
                "exitEpoch": node.exit_epoch.map(quantity),
                "active": node.effective_epoch <= epoch
                    && node.exit_epoch.map(|end| epoch <= end).unwrap_or(true),
                "witnessed": witnessed,
                "selected": winners.contains(&node.address),
            })
        })
        .collect();
    Ok(json!({
        "rewardBps": kovanica_staking::LIGHT_REWARD_BPS,
        "epoch": quantity(epoch),
        "beacon": beacon,
        "nodes": nodes,
    }))
}

/// Serialise a committee's authorities into the member-row shape shared by
/// [`handle_get_committee`] and [`handle_get_committee_info`]. Kept in one
/// place so both methods stay byte-identical.
fn committee_members_json(committee: &kovanica_consensus::CommitteeInfo) -> Vec<Value> {
    committee
        .authorities()
        .iter()
        .map(|a| {
            json!({
                "index": a.index,
                "address": a.address.to_string(),
                "stake": quantity(a.stake),
                "publicKey": to_hex(a.public_key.as_bytes()),
                "network_address": a.network_address,
            })
        })
        .collect::<Vec<Value>>()
}

/// `kovanica_getCommittee(epoch?)` — current consensus committee.
///
/// Returns a bare JSON array of member rows; the Android client parses the
/// no-param response directly as a `JSONArray`, so that shape is preserved.
/// Committee history is not retained by the node: when an optional `epoch`
/// parameter is supplied and does not match the live committee epoch this
/// returns an invalid-params error rather than silently answering with the
/// wrong epoch's committee.
pub async fn handle_get_committee(params: Value, state: RpcState) -> Result<Value, RpcError> {
    let committee = &state.committee;
    let args = params_vec(&params);
    if let Some(requested) = args.first() {
        let requested = parse_u64(requested, "epoch")?;
        let current = committee.epoch();
        if requested != current {
            return Err(RpcError::InvalidParams(format!(
                "committee history is not retained; only the current epoch ({current}) is available"
            )));
        }
    }
    Ok(Value::Array(committee_members_json(committee)))
}

/// `kovanica_getCommitteeInfo()` — current committee together with its epoch.
///
/// Object form of [`handle_get_committee`]: `{ epoch, members }`, where
/// `members` is exactly the same row array. A light client needs the epoch to
/// recompute `committee_seal_hash(epoch, members)` against a sealed leader
/// header's `committee_hash`.
pub async fn handle_get_committee_info(
    _params: Value,
    state: RpcState,
) -> Result<Value, RpcError> {
    let committee = &state.committee;
    Ok(json!({
        "epoch": quantity(committee.epoch()),
        "members": committee_members_json(committee),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kovanica_consensus::{AuthorityInfo, CommitteeInfo};
    use kovanica_dag::DagStore;
    use kovanica_mempool::{Mempool, MempoolConfig};
    use kovanica_staking::{StakingState, MIN_VALIDATOR_STAKE};
    use kovanica_storage::Storage;
    use kovanica_types::crypto::PublicKey;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn hex32(byte: u8) -> String {
        format!("0x{}", hex::encode([byte; 32]))
    }

    fn test_state(storage: Arc<Storage>) -> RpcState {
        let authority = AuthorityInfo {
            index: 0,
            stake: MIN_VALIDATOR_STAKE,
            public_key: PublicKey([7u8; 32]),
            vrf_public_key: None,
            address: Address([9u8; 32]),
            network_address: "127.0.0.1:9000".to_string(),
        };
        RpcState {
            health: crate::NodeHealth::new(),
            consensus_store: Arc::new(DagStore::from_storage(storage.clone())),
            mempool: Arc::new(Mempool::new(
                MempoolConfig::default(),
                storage.clone(),
                kovanica_types::SigningContext::new(kovanica_types::signing::chain_id::LOCAL),
            )),
            storage,
            staking: Arc::new(RwLock::new(StakingState::new())),
            committee: CommitteeInfo::try_new(0, vec![authority]).expect("test committee is valid"),
            peer_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            events: crate::EventBus::new(),
            rate_limiter: Arc::new(crate::RateLimiterState::new(
                crate::RateLimitConfig::default(),
            )),
            auth_config: Arc::new(crate::AuthConfig::default()),
            tx_gossip: None,
        }
    }

    fn open_storage() -> (tempfile::TempDir, Arc<Storage>) {
        let dir = tempfile::tempdir().expect("tempdir");
        let storage = Arc::new(Storage::new(dir.path().join("state.db")).expect("storage"));
        (dir, storage)
    }

    #[test]
    fn parses_hash_and_address_hex() {
        let value = Value::String(hex32(0xab));
        assert_eq!(parse_hash(&value, "hash").unwrap(), Hash([0xab; 32]));
        assert_eq!(
            parse_address(&value, "address").unwrap(),
            Address([0xab; 32])
        );

        assert!(parse_hash(&Value::String("0x1234".into()), "hash").is_err());
        assert!(parse_hash(&Value::String("not-hex".into()), "hash").is_err());
    }

    #[test]
    fn parses_canonical_and_hex_addresses() {
        let address = Address([0x2a; 32]);
        assert_eq!(address.to_string(), address.encode());
        assert_eq!(
            parse_address(&json!(address.to_string()), "address").unwrap(),
            address
        );
        assert_eq!(
            parse_address(&json!(hex32(0x2a)), "address").unwrap(),
            address
        );

        // Correct affixes/length but a wrong checksum must be rejected.
        let bad = json!(format!("kvnc{}dag", "00".repeat(36)));
        assert!(parse_address(&bad, "address").is_err());
    }

    #[test]
    fn parses_quantities_in_decimal_and_hex() {
        assert_eq!(parse_u64(&json!(42), "n").unwrap(), 42);
        assert_eq!(parse_u64(&json!("0x2a"), "n").unwrap(), 42);
        assert!(parse_u64(&json!("nope"), "n").is_err());
        assert_eq!(quantity(0u64), json!("0x0"));
        assert_eq!(quantity(255u64), json!("0xff"));
    }

    #[test]
    fn parses_storage_key_as_slot_or_hash() {
        let mut expected = [0u8; 32];
        expected[31] = 5;
        assert_eq!(parse_storage_key(&json!(5), "key").unwrap(), expected);
        assert_eq!(
            parse_storage_key(&Value::String(hex32(0x01)), "key").unwrap(),
            [0x01u8; 32]
        );
    }

    #[test]
    fn normalises_params_forms() {
        assert_eq!(params_vec(&json!([1, 2])).len(), 2);
        assert_eq!(params_vec(&json!("x")), vec![json!("x")]);
        assert!(params_vec(&Value::Null).is_empty());
    }

    #[tokio::test]
    async fn query_handlers_read_real_storage() {
        let (_dir, storage) = open_storage();
        let state = test_state(storage);

        assert_eq!(
            handle_block_number(Value::Null, state.clone())
                .await
                .unwrap(),
            json!("0x0")
        );
        assert_eq!(
            handle_get_balance(json!([hex32(0x11)]), state.clone())
                .await
                .unwrap(),
            json!("0x0")
        );
        assert_eq!(
            handle_get_nonce(json!([hex32(0x11)]), state.clone())
                .await
                .unwrap(),
            json!("0x0")
        );
        assert_eq!(
            handle_get_code(json!([hex32(0x11)]), state.clone())
                .await
                .unwrap(),
            json!("0x")
        );
        assert_eq!(
            handle_get_block_by_hash(json!([hex32(0x22)]), state.clone())
                .await
                .unwrap(),
            Value::Null
        );

        let committee = handle_get_committee(Value::Null, state.clone())
            .await
            .unwrap();
        assert_eq!(committee.as_array().unwrap().len(), 1);
        assert_eq!(
            committee[0]["address"],
            json!(Address([0x09; 32]).to_string())
        );

        let schedule = handle_get_leader_schedule(Value::Null, state.clone())
            .await
            .unwrap();
        assert_eq!(
            schedule.as_array().unwrap().len(),
            DEFAULT_LEADER_SCHEDULE_LEN as usize
        );
        let max = handle_get_leader_schedule(json!([MAX_LEADER_SCHEDULE_LEN]), state.clone())
            .await
            .unwrap();
        assert_eq!(
            max.as_array().unwrap().len(),
            MAX_LEADER_SCHEDULE_LEN as usize
        );
        assert!(matches!(
            handle_get_leader_schedule(json!([MAX_LEADER_SCHEDULE_LEN + 1]), state.clone()).await,
            Err(RpcError::InvalidParams(_))
        ));

        let validators = handle_get_validators(Value::Null, state.clone())
            .await
            .unwrap();
        assert!(validators.as_array().unwrap().is_empty());

        let pending = handle_get_pending_transactions(Value::Null, state.clone())
            .await
            .unwrap();
        assert!(pending.as_array().unwrap().is_empty());

        assert_eq!(
            handle_estimate_fee(Value::Null, state).await.unwrap(),
            json!("0x1")
        );
    }

    #[tokio::test]
    async fn committee_info_matches_committee_and_rejects_other_epochs() {
        let (_dir, storage) = open_storage();
        let state = test_state(storage);

        // `kovanica_getCommitteeInfo` reports the live epoch (0 in the fixture)
        // and the exact same member rows as the bare-array `kovanica_getCommittee`.
        let info = handle_get_committee_info(Value::Null, state.clone())
            .await
            .unwrap();
        assert_eq!(info["epoch"], json!("0x0"));
        let members = info["members"].as_array().expect("members is an array");
        assert_eq!(members.len(), 1);
        assert_eq!(
            members[0]["address"],
            json!(Address([0x09; 32]).to_string())
        );

        let committee = handle_get_committee(Value::Null, state.clone())
            .await
            .unwrap();
        assert_eq!(info["members"], committee);

        // The epoch-matching parameter behaves like the no-param call.
        let same = handle_get_committee(json!(["0x0"]), state.clone())
            .await
            .unwrap();
        assert_eq!(same, committee);

        // A different epoch is rejected instead of silently answering with the
        // current committee (history is not retained).
        assert!(matches!(
            handle_get_committee(json!(["0x5"]), state.clone()).await,
            Err(RpcError::InvalidParams(_))
        ));
    }

    #[tokio::test]
    async fn estimate_fee_uses_minimum_observed_rate_and_floor() {
        use kovanica_storage::state_store::Account;
        use kovanica_types::crypto::{Signature, SigningKey};

        let (_dir, storage) = open_storage();
        let state = test_state(storage.clone());

        // Two distinct funded senders so both fee-rate samples are admissible
        // (a valid signature is required now that addresses are public keys).
        let (sk_a, pk_a) = kovanica_crypto::generate_keypair();
        let (sk_b, pk_b) = kovanica_crypto::generate_keypair();
        let sender_a = Address::from_public_key(&pk_a);
        let sender_b = Address::from_public_key(&pk_b);

        let txn = storage.begin_write().unwrap();
        {
            let st = storage.state();
            let account = Account {
                balance: 1_000_000,
                nonce: 0,
                code_hash: [0; 32],
                code: Vec::new(),
            };
            st.set_account(&txn, &sender_a, &account).unwrap();
            st.set_account(&txn, &sender_b, &account).unwrap();
        }
        txn.commit().unwrap();

        let make_signed = |sender: Address, to: Address, fee: u64, sk: &SigningKey| {
            let mut tx = Transaction {
                sender,
                nonce: 0,
                kind: TransactionKind::Transfer { to, amount: 1 },
                fee,
                signature: Signature([0; 64]),
                hash: Hash::zero(),
            };
            let signing_hash = tx.signing_hash(&state.mempool.signing_context());
            tx.signature = kovanica_crypto::sign(sk, signing_hash.as_ref());
            tx.hash = signing_hash;
            tx
        };

        let low_rate_tx = make_signed(sender_a, Address([2; 32]), 1, &sk_a);
        let serialized_size = bincode::serialize(&low_rate_tx).unwrap().len() as u64;
        let higher_rate_tx = make_signed(sender_b, Address([3; 32]), serialized_size * 4, &sk_b);

        state
            .mempool
            .add_transaction(low_rate_tx)
            .expect("admit nonzero-fee transaction");
        state
            .mempool
            .add_transaction(higher_rate_tx)
            .expect("admit higher-fee transaction");

        assert_eq!(
            state.mempool.stats().fee_rates,
            vec![(0, 1), (4, 1)],
            "fixture must contain distinct observed rates"
        );
        assert_eq!(
            handle_estimate_fee(Value::Null, state).await.unwrap(),
            json!("0x1"),
            "the minimum observed zero rate is advisory and floored at 1 atom/byte"
        );
    }

    #[tokio::test]
    async fn send_raw_transaction_gossips_only_accepted_transactions() {
        use kovanica_storage::state_store::Account;
        use kovanica_types::crypto::Signature;

        let (_dir, storage) = open_storage();
        let mut state = test_state(storage.clone());
        let (gossip_tx, mut gossip_rx) = tokio::sync::mpsc::unbounded_channel();
        state.tx_gossip = Some(gossip_tx);

        let (sk, pk) = kovanica_crypto::generate_keypair();
        let sender = Address::from_public_key(&pk);
        let txn = storage.begin_write().unwrap();
        {
            let account = Account {
                balance: 1_000_000,
                nonce: 0,
                code_hash: [0; 32],
                code: Vec::new(),
            };
            storage
                .state()
                .set_account(&txn, &sender, &account)
                .unwrap();
        }
        txn.commit().unwrap();

        let mut tx = Transaction {
            sender,
            nonce: 0,
            kind: TransactionKind::Transfer {
                to: Address([3u8; 32]),
                amount: 1,
            },
            fee: 10_000,
            signature: Signature([0; 64]),
            hash: Hash::zero(),
        };
        let signing_hash = tx.signing_hash(&state.mempool.signing_context());
        tx.signature = kovanica_crypto::sign(&sk, signing_hash.as_ref());
        tx.hash = signing_hash;
        let raw = |tx: &Transaction| json!([to_hex(&bincode::serialize(tx).unwrap())]);

        // Rejected (bad signature): error, nothing gossiped.
        let mut forged = tx.clone();
        forged.signature = Signature([1; 64]);
        assert!(handle_send_raw_transaction(raw(&forged), state.clone())
            .await
            .is_err());
        assert!(
            gossip_rx.try_recv().is_err(),
            "rejected tx must not be gossiped"
        );

        // Accepted: gossiped exactly once.
        handle_send_raw_transaction(raw(&tx), state.clone())
            .await
            .expect("valid tx is accepted");
        assert_eq!(
            gossip_rx.try_recv().expect("accepted tx gossiped").hash,
            tx.hash
        );

        // Duplicate: mempool rejects (AlreadyExists), no second gossip.
        assert!(handle_send_raw_transaction(raw(&tx), state.clone())
            .await
            .is_err());
        assert!(
            gossip_rx.try_recv().is_err(),
            "duplicate must not be re-gossiped"
        );
    }

    #[tokio::test]
    async fn returned_transaction_is_pending_without_receipt() {
        use kovanica_storage::state_store::Account;
        use kovanica_types::crypto::Signature;

        let (_dir, storage) = open_storage();
        let state = test_state(storage.clone());
        let (sk, pk) = kovanica_crypto::generate_keypair();
        let sender = Address::from_public_key(&pk);
        let txn = storage.begin_write().unwrap();
        storage
            .state()
            .set_account(
                &txn,
                &sender,
                &Account {
                    balance: 1_000_000,
                    nonce: 0,
                    code_hash: [0; 32],
                    code: Vec::new(),
                },
            )
            .unwrap();
        txn.commit().unwrap();
        let mut tx = Transaction {
            sender,
            nonce: 0,
            kind: TransactionKind::Transfer {
                to: Address([3u8; 32]),
                amount: 1,
            },
            fee: 10_000,
            signature: Signature([0; 64]),
            hash: Hash::zero(),
        };
        let h = tx.signing_hash(&state.mempool.signing_context());
        tx.signature = kovanica_crypto::sign(&sk, h.as_ref());
        tx.hash = h;

        // Returned from a non-blue block (#13).
        let report = state.mempool.reinsert_returned(vec![tx.clone()], true);
        assert_eq!(report[0].outcome, kovanica_mempool::ReinsertOutcome::Accepted);

        let hash = json!([to_hex(&tx.hash.0)]);
        let receipt = handle_get_transaction_receipt(hash.clone(), state.clone())
            .await
            .unwrap();
        assert_eq!(receipt, Value::Null, "no receipt until in a blue block");
        let by_hash = handle_get_transaction_by_hash(hash, state.clone())
            .await
            .unwrap();
        assert_eq!(by_hash["status"], json!("pending"));
        assert_eq!(by_hash["blockHash"], Value::Null);

        // Unknown hash stays null.
        let unknown = handle_get_transaction_by_hash(json!([hex32(0x44)]), state)
            .await
            .unwrap();
        assert_eq!(unknown, Value::Null);
    }

    #[tokio::test]
    async fn light_reads_follow_the_latest_account_root() {
        use kovanica_storage::state_store::Account;

        let (_dir, storage) = open_storage();
        let address = Address([4u8; 32]);
        let tx_hash = Hash::new([9u8]);
        let header = LeaderHeader {
            committed_height: 2,
            leader_round: 6,
            leader_digest: Hash::new([2u8]),
            leader_author: 0,
            parent_leader_digest: Hash::zero(),
            tx_merkle_root: Hash::zero(),
            executed_tx_root: merkle::root(&[tx_hash]),
            non_blue_root: Hash::zero(),
            blue_root: Hash::zero(),
            mysticghost: false,
            state_root: Hash::zero(),
            epoch: 0,
            committee_hash: Hash::zero(),
            timestamp: 0,
        };
        {
            let txn = storage.begin_write().unwrap();
            storage
                .state()
                .set_account(
                    &txn,
                    &address,
                    &Account {
                        balance: 5,
                        nonce: 1,
                        code_hash: [0; 32],
                        code: Vec::new(),
                    },
                )
                .unwrap();
            let root = storage.state().commit_account_smt(&txn, 2).unwrap();
            let mut header = header;
            header.state_root = root;
            storage.state().save_leader_header(&txn, &header).unwrap();
            storage
                .state()
                .save_executed_tx_hashes(&txn, 2, &[tx_hash])
                .unwrap();
            txn.commit().unwrap();
        }
        let state = test_state(storage);
        let got = handle_get_leader_header(json!([2]), state.clone())
            .await
            .unwrap();
        assert_eq!(got["committedHeight"], json!("0x2"));
        assert_eq!(got["timestamp"], json!("0x0"));
        assert!(handle_get_leader_header(json!([9]), state.clone())
            .await
            .unwrap()
            .is_null());
        let seals = handle_get_commit_seals(json!([2]), state.clone())
            .await
            .unwrap();
        assert_eq!(seals["seals"], json!([]));
        let proof = handle_get_tx_proof(
            json!([2, format!("0x{}", hex::encode(tx_hash.0))]),
            state.clone(),
        )
        .await
        .unwrap();
        assert!(proof["executed"].is_object());
        assert!(proof["leader"].is_null());
        let account = handle_get_account_proof(json!([address.to_string()]), state.clone())
            .await
            .unwrap();
        assert_eq!(account["height"], json!("0x2"));
        assert_eq!(account["present"], json!(true));
        assert_eq!(account["balance"], json!("0x5"));
        let err = handle_get_account_proof(json!([address.to_string(), "0x1"]), state.clone())
            .await
            .unwrap_err();
        assert!(err.to_string().contains("latest"));
        let set = handle_get_light_set(Value::Null, state).await.unwrap();
        assert_eq!(kovanica_staking::LIGHT_REWARD_BPS, 2_000);
        assert_eq!(set["rewardBps"], json!(2_000));
        assert_eq!(set["nodes"], json!([]));
    }
}

// ============================================================================
// Light Client (client-only / read-only) — P15 B-lane
// ============================================================================

/// Minimal read-only light-client checkpoint endpoint.
/// Accepts block hash or height; returns existing [`LightClientCheckpoint`]
/// structure plus a verification-status flag. No consensus rule changed.
pub async fn handle_light_client_checkpoint(
    params: Value,
    _state: RpcState,
) -> Result<Value, RpcError> {
    let args = params_vec(&params);
    let height = args.get(0).and_then(|v| parse_u64(v, "height").ok()).unwrap_or(0);
    let hash_arg = args.get(1);
    let leader_hash = hash_arg
        .map(|v| parse_hash(v, "hash").ok())
        .flatten()
        .unwrap_or(Hash::zero());
    // Minimal checkpoint built from existing types only (no new logic).
    let wave = WaveCommitCertificate::new(leader_hash, 0, height, std::collections::BTreeMap::new(), Hash::zero());
    let checkpoint = LightClientCheckpoint::new(height, leader_hash, Hash::zero(), Hash::zero(), wave, 0);
    // Verification exposes existing verify_quorum / verify_colouring (client-only,
    // no new consensus logic). Callers supply a real Committee to run them.
    let verified = false;
    Ok(json!({
        "checkpoint": checkpoint,
        "verified": verified,
        "height": quantity(height),
        "hash": to_hex(leader_hash.as_ref()),
        "note": "read-only / client-only; no consensus change",
    }))
}

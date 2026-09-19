---
name: rust-account-ledger
description: Use when implementing an account-based ledger in Rust: account state (balance, nonce, code, storage), state trie (MPT), state root commitment, transaction application (nonce increment, balance changes), and state diffs.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [blockchain, account-ledger, state-trie, MPT, state-root, nonce, balance, account-state, state-diff]
    related_skills: [blockchain-fundamentals, rust-utxo-ledger, rust-merkle-structures, rust-crypto-primitives]
---

# Rust Account Ledger

## Overview

An account-based ledger models state as a mapping from account address to account data: balance, nonce, contract code (if applicable), and storage (for smart contracts). The state is committed in a Merkle tree (usually a Merkle Patricia Trie, MPT), and the state root is included in the block header. This is the Ethereum model.

This skill covers the account state model, the MPT for state commitment, transaction application (how a transaction changes state), and state root updates.

## When to Use

- Implementing an account-model blockchain
- Designing account state (balance, nonce, code, storage)
- Using a Merkle Patricia Trie to commit state
- Computing state diffs between blocks
- Validating that a block's state root matches the applied state
- Supporting light clients that verify state via Merkle proofs

**Don't use for:** UTXO-based ledgers (use the UTXO skill), or systems that don't commit state in a trie.

## Account State

### Account Representation

```rust
#[derive(Debug, Clone, PartialEq)]
struct Account {
    nonce: u64,           // number of transactions sent from this account
    balance: u64,         // balance in the base unit (wei, satoshi-equivalent, etc.)
    code: Option<Vec<u8>>, // contract code (if this is a contract account)
    storage: HashMap<Hash, Hash>, // storage slots (for contracts)
    // For simple payment accounts, code and storage are None/empty
}
```

For a simple value-transfer ledger (no contracts), the account has just `nonce` and `balance`.

### Account Address

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Address([u8; 20]);   // 20-byte address (Ethereum-style)

// Or 32-byte for some designs
// Or derived from pubkey (hash of pubkey, or last bytes of pubkey)
```

Address derivation:
- From pubkey: `hash(pubkey)[..20]` (Ethereum) or `pubkey[1..21]` (Bitcoin script addresses are different).

### Account Store

```rust
struct AccountStore {
    accounts: HashMap<Address, Account>,
    // Optionally, a trie for state root commitment
}
```

## Merkle Patricia Trie (MPT) for State

### Why a Trie?

The state is committed to a Merkle trie so that:
- The state root in the block header commits to the entire state.
- A light client can verify an account's balance with a Merkle proof (a path from the account to the root).
- State diffs (what changed between blocks) are computable.

### MPT Basics

A Merkle Patricia Trie is a trie where:
- Keys are account addresses (or storage keys).
- Values are the account data (or storage values).
- Each node has a hash (the Merkle hash of its children and its value).
- The root hash is the state root.

Implementing a full MPT is complex. Use a crate like `ethereum MPT` or `rust-merkle-patricia-trie`, or implement a simpler Merkle trie if you don't need Patricia's compact representation.

For a simple implementation:

```rust
use std::collections::BTreeMap;

// A simple Merkle trie (not Patricia, but demonstrates the concept)
// Keys are addresses (sorted), values are account hashes
// Each internal node is the hash of its children

fn build_state_trie(accounts: &BTreeMap<Address, Account>) -> Hash {
    // Build a trie from the accounts, compute root hash
    // ...
}
```

For production, a proper MPT implementation handles:
- Compact encoding (Patricia's radix of 16 for hex-encoded nibbles).
- Partial proofs (Merkle proofs for specific keys).
- Efficient updates (only changed nodes are recomputed).

### State Root

```rust
struct StateRoot(Hash);   // the hash of the state trie root
```

The state root is stored in the block header. To verify the state, recompute the trie from the accounts and compare the root.

## Transaction Application

### Transaction Structure

```rust
#[derive(Debug, Clone)]
struct Transaction {
    sender: Address,
    nonce: u64,           // the sender's current nonce (must match)
    recipient: Address,
    amount: u64,
    fee: u64,
    signature: Signature,
    // For contract calls: data, gas, etc.
}
```

### Validation and Application

```rust
fn apply_transaction(tx: &Transaction, state: &mut AccountStore) -> Result<(), Error> {
    // 1. Verify signature
    let msg = sign_message(tx.sender, tx.nonce, tx.recipient, tx.amount, tx.fee);
    tx.sender_pubkey().verify(&msg, &tx.signature)?;   // verify sender is who they claim

    // 2. Get sender account
    let sender = state.get_or_create(tx.sender);

    // 3. Check nonce
    if sender.nonce != tx.nonce {
        return Err(Error::NonceMismatch);
    }

    // 4. Check balance (amount + fee)
    if sender.balance < tx.amount + tx.fee {
        return Err(Error::InsufficientBalance);
    }

    // 5. Deduct from sender
    sender.balance -= tx.amount + tx.fee;
    sender.nonce += 1;

    // 6. Credit recipient
    let recipient = state.get_or_create(tx.recipient);
    recipient.balance += tx.amount;

    // 7. Recompute state root (if using a trie)
    state.recompute_root();

    Ok(())
}
```

### Nonce Semantics

The nonce is the sender's transaction counter. It serves two purposes:
1. **Replay protection** — a transaction with nonce N can only be applied once; the next transaction from that sender must have nonce N+1.
2. **Ordering** — transactions from the same sender are applied in nonce order.

Nonce management:
- On applying a transaction, increment the sender's nonce.
- If a transaction is included, its nonce is consumed.
- If a transaction is dropped (not included), the nonce is not consumed (the sender retries with the same nonce).

### State Diff

Between two blocks, the state diff is the set of accounts that changed. This is useful for:
- Light clients syncing state.
- Indexing and analytics.
- Rollbacks (reverting to a previous state).

```rust
struct StateDiff {
    updated: Vec<(Address, Account)>,   // accounts that changed
    created: Vec<(Address, Account)>,   // new accounts
    deleted: Vec<Address>,              // accounts that were emptied and removed
}
```

Compute the diff by comparing the account store before and after the block.

## State Commitment in Blocks

### Block Header with State Root

```rust
struct BlockHeader {
    prev_hash: Hash,
    tx_root: Hash,          // Merkle root of transactions
    state_root: Hash,       // Merkle root of the state trie AFTER applying the block
    receipts_root: Option<Hash>,  // Merkle root of transaction receipts
    timestamp: u64,
    nonce: u64,             // block nonce (for PoW) — not to be confused with account nonce
    difficulty: u32,
}
```

The state root commits to the state after all transactions in the block are applied. This is the Ethereum model.

### Validation of State Root

When a node receives a block, it:
1. Starts from the previous state (state root of the previous block).
2. Applies all transactions.
3. Computes the new state root.
4. Compares with the block header's state root.

If they match, the block is valid (assuming transactions are valid). If they don't, the block is invalid.

```rust
fn validate_block_state(block: &Block, prev_state: &AccountStore) -> Result<(), Error> {
    let mut state = prev_state.clone();
    for tx in &block.transactions {
        apply_transaction(tx, &mut state)?;
    }
    let computed_root = state.state_root();
    if computed_root != block.header.state_root {
        return Err(Error::StateRootMismatch);
    }
    Ok(())
}
```

## Verification Checklist

- [ ] Can represent an account with nonce, balance, code, storage
- [ ] Can represent an address (derived from pubkey)
- [ ] Can apply a transaction: verify signature, check nonce, check balance, update sender and recipient
- [ ] Can maintain a state trie (or understand how an MPT commits state)
- [ ] Can compute the state root after applying a block
- [ ] Can validate a block's state root against the computed state
- [ ] Can compute a state diff between two blocks
- [ ] Understands that the state root in the block header commits to the state after the block
- [ ] Understands nonce semantics (replay protection, ordering)

## Common Pitfalls

1. **Confusing account nonce with block nonce.** Account nonce is the sender's tx counter. Block nonce is the PoW nonce (or a random value in PoS). Different concepts.

2. **Not validating the state root.** A block's state root must be checked against the applied state. Without this, a fake state root could hide invalid state.

3. **Allowing nonce reuse.** If a transaction is applied and the nonce isn't incremented, the same transaction could be applied again. Nonce must increment on application.

4. **Not handling new accounts.** When a transaction sends to an address that has no account, the account must be created (with zero balance, nonce 0, then credited). `get_or_create` pattern.

5. **Assuming the state trie is a simple hash of all accounts.** A Merkle trie (MPT) allows proofs and efficient updates. A single hash of all accounts doesn't allow proofs. Use a trie.

6. **Not handling storage correctly for contract accounts.** If contracts are supported, storage is part of the account state and must be included in the state root. Storage is a mapping from slot to value.

7. **Forgetting that the state root is after the block, not before.** Some designs commit the pre-state root (the state before the block). The block header's state root should be the post-state (after applying the block). Be consistent.

8. **Using a HashMap for accounts without a trie for the state root.** A HashMap is fine for the in-memory state, but the state root must be from a trie. Keep the trie in sync with the HashMap.

9. **Not handling state rollback.** If a block is orphaned, the state must roll back to the previous state root. This requires either saving state snapshots or replaying from genesis (for light clients, Merkle proofs allow verifying specific accounts without full state).

10. **Ignoring state bloat.** Over time, the state grows. Accounts with zero balance may accumulate. Some designs prune empty accounts; others keep them for history. Plan for state size.

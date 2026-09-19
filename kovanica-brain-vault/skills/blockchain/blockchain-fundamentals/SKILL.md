---
name: blockchain-fundamentals
description: Use when understanding blockchain basics: blocks, chains, hashing, consensus, UTXO vs account models, Merkle trees, and how distributed ledgers work at a high level before writing any code.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [blockchain, fundamentals, blocks, hashes, consensus, UTXO, account-model, Merkle]
    related_skills: [rust-consensus-pow, rust-utxo-ledger, rust-crypto-primitives]
---

# Blockchain Fundamentals

## Overview

A blockchain is a replicated, append-only, ordered log of transactions grouped into blocks, secured by cryptography and consensus. Different designs make different trade-offs: Bitcoin-style PoW with UTXOs, Ethereum-style PoS with accounts and a global state trie, or DAG-based systems like Kaspa's GHOSTDAG.

This skill covers the conceptual building blocks. Code-level details live in the implementation-specific skills (PoW, UTXO, GHOSTDAG, etc.).

## When to Use

- Before implementing any blockchain component
- Understanding what "consensus" means and why it's needed
- Choosing between UTXO and account models
- Learning what a Merkle tree is and why blocks use them
- Understanding the role of hashing in chaining blocks and committing state

**Don't use for:** specific implementation details (use the relevant implementation skill), or legal/financial advice about tokens.

## Core Concepts

### 1. Blocks

A block is a batch of transactions, plus metadata:

```
Block:
  - version
  - previous block hash (links blocks into a chain)
  - timestamp
  - consensus data (nonce for PoW, validator signature for PoS, etc.)
  - transactions (list)
  - Merkle root of transactions
  - (optional) state root, receipts root
```

The block's own hash is typically `hash(block_header)` — a cryptographic hash of the header fields. This hash is what previous blocks reference.

### 2. Chaining

Each block references the hash of the previous block. This creates an ordered chain where:
- Changing any block's content changes its hash.
- That changes the next block's "previous hash" field, so the next block's hash changes too.
- The chain is immutable under the assumption that recomputing the consensus work (PoW) or obtaining validator signatures (PoS) is infeasible.

### 3. Cryptographic Hash Functions

A hash function `H(data) -> digest` must be:
- **Deterministic** — same input → same output.
- **One-way** — infeasible to find `data` from `digest`.
- **Collision-resistant** — infeasible to find two inputs with the same hash.
- **Avalanche** — small input change → large output change.

Common choices: SHA-256 (Bitcoin), SHA-3 / Keccak (Ethereum), BLAKE2b/BLAKE3 (some newer systems).

### 4. Consensus

Consensus is the mechanism by which all replicas agree on the same canonical chain/state. Without consensus, replicas diverge and there's no single source of truth.

Two broad families:

**Proof of Work (PoW):**
- Miners expend computational work (finding a nonce that makes the block hash below a target).
- The chain with the most work (or greatest cumulative difficulty) wins.
- Byzantine fault tolerant against attackers with less hash power than honest nodes.
- Energy-intensive; probabilistic finality.

**Proof of Stake (PoS):**
- Validators lock up stake (currency) and take turns proposing/attesting to blocks.
- Slashing conditions punish misbehavior (double-signing, downtime).
- Finality can be faster and deterministic (e.g., Tendermint) or probabilistic (e.g., Ethereum's Gasper).
- "Nothing at stake" problem is addressed by slashing.

**Hybrid and DAG approaches:**
- GHOSTDAG (Kaspa): blocks are in a DAG, not a chain; a k-cluster coloring algorithm selects which blocks are "blue" (canonical) and which are "red" (excluded).
- Hashgraph, Narwhal+Bullshark, etc.: different data structures and consensus protocols.

### 5. UTXO Model vs Account Model

**UTXO (Unspent Transaction Output):**
- State is a set of unspent outputs, each with an amount and a locking script/owner.
- A transaction consumes some UTXOs (inputs) and produces new UTXOs (outputs).
- No account balance — balance is the sum of UTXOs owned by a key.
- Naturally supports parallel validation (inputs are independent if from different owners).
- Bitcoin uses this model.
- History is the set of all transactions; present state is the UTXO set.

**Account Model:**
- State is a mapping from account address to balance, nonce, code (if contract), and storage.
- A transaction is from an account, deducts balance, increments nonce.
- Global state is a Merkle trie (Ethereum's MPT); state root is committed in blocks.
- Harder to parallelize (nonces must be sequential per account).
- Easier for smart contracts (stateful, mutable accounts).

**Trade-offs:**
- UTXO: clearer audit trail, easier privacy (no account history), harder for complex state.
- Account: simpler for contracts and stateful apps, easier to reason about balances, harder to parallelize.

### 6. Merkle Trees

A Merkle tree commits to a set of items with a single root hash:

```
Leaves: H(Leaf1), H(Leaf2), H(Leaf3), H(Leaf4)
Level 1: H(Leaf1+Leaf2), H(Leaf3+Leaf4)
Root: H(L1Left + L1Right)
```

A Merkle proof lets you prove that a specific leaf is in the tree, without revealing the whole tree — transmit only the leaf and the sibling hashes up to the root.

Uses in blockchains:
- **Transaction Merkle tree** — commit to the set of txs in a block; light clients can verify a tx is in a block with a Merkle proof.
- **State Merkle trie** (account model) — commit to the global state; state root in block header lets light clients verify account balances.
- **Receipts Merkle tree** — commit to tx execution results.

### 7. Transactions

A transaction is a signed message that changes state:

```
Transaction:
  - version
  - inputs (UTXO: outpoint + unlocking script) OR sender + nonce (account)
  - outputs (UTXO: amount + locking script) OR recipient + amount (account)
  - fee
  - signature(s)
```

Validation checks:
- Signatures are valid.
- Inputs haven't been spent before (no double-spend).
- Inputs exist (for UTXO).
- Account nonce matches (for account model).
- Fee is sufficient.
- (For smart contracts) execution doesn't violate rules.

### 8. Block Time, Throughput, Finality

- **Block time** — how often a new block is produced (Bitcoin: ~10 min, Ethereum: ~12 sec, Kaspa: ~1 sec).
- **Throughput** — transactions per second, limited by block size and block time.
- **Finality** — when a transaction is "settled." PoW is probabilistic (more confirmations = safer). PoS can have deterministic finality (once a block is finalized, it can't be reverted without slashing).

### 9. Forks

A fork happens when two blocks extend the same parent (PoW) or when validators disagree. Consensus rules resolve forks:
- PoW: the chain with more work wins; the other branch is orphaned.
- PoS: the protocol's fork choice rule (e.g., latest finalized, heaviest attestation) determines canonical.
- GHOSTDAG: the blue set is canonical; red blocks are excluded but still referenced.

## Verification Checklist

- [ ] Can explain the difference between a block and a transaction
- [ ] Understands that the "previous block hash" chains blocks together
- [ ] Understands what a cryptographic hash function provides (one-way, collision-resistant, deterministic)
- [ ] Can explain PoW vs PoS vs DAG-based consensus at a high level
- [ ] Can explain UTXO vs account model and the trade-offs
- [ ] Understands Merkle trees and Merkle proofs (why they're used, not just how they work)
- [ ] Understands the concepts of block time, throughput, and finality
- [ ] Understands what a fork is and how consensus resolves it

## Common Pitfalls

1. **Confusing the transaction with the block.** Transactions are the state-changing operations; blocks are batches of transactions plus consensus metadata.

2. **Thinking a hash proves content.** A hash proves that you know content that hashes to that value — not what the content is. You need the preimage.

3. **Assuming "blockchain" means Bitcoin-style chain.** A blockchain is a design pattern; different systems make different choices (UTXO vs account, PoW vs PoS, chain vs DAG).

4. **Confusing finality with confirmation.** A block being "confirmed" means it's in the chain; finality means it can't be reverted. PoW has probabilistic finality; some PoS systems have deterministic finality.

5. **Thinking UTXOs are "coins."** UTXOs are outputs — claims on value. A wallet's balance is the sum of UTXOs it can unlock.

6. **Over-simplifying Merkle trees.** Merkle trees aren't just "hash everything together" — they're a specific tree structure that allows proofs of membership. Understanding the proof construction matters for light clients.

7. **Ignoring the consensus layer when designing a "blockchain."** A chain of blocks without consensus is just a log — it doesn't give you agreement. Consensus is the hard part.

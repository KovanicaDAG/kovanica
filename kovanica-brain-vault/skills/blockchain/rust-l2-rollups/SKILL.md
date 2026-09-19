---
name: rust-l2-rollups
description: Use when building or understanding L2 rollups in Rust: ZK rollups vs optimistic rollups, data availability, state diffs, fraud proofs (optimistic), validity proofs (ZK), sequencer design, and rollup-to-L1 messaging.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [rust, blockchain, l2, rollup, zk-rollup, optimistic-rollup, data-availability, fraud-proof, validity-proof, sequencer]
    related_skills: [rust-zk-proofs, rust-block-propagation, rust-consensus-pos, blockchain-fundamentals]
---

# Rust L2 Rollups

## Overview

A rollup is an L2 scaling solution that executes transactions off-chain (on L2) and posts the results (state diffs or transaction data) to L1 for security and data availability. There are two main types:

- **ZK rollups:** post a validity proof (ZK proof) that the state transition is correct. L1 verifies the proof.
- **Optimistic rollups:** post the state diff and assume it's correct (optimistic). L1 can challenge it via a fraud proof within a window.

This skill covers the architecture and components of rollups. Building a production rollup is a large undertaking — this skill is for understanding the design and the Rust components involved.

## When to Use

- Designing a ZK or optimistic rollup
- Building a sequencer (L2 transaction ordering)
- Building a bridge between L1 and L2
- Understanding data availability (DA) for rollups (L1 as DA, dedicated DA layer, etc.)
- Implementing fraud proofs or validity proofs for a rollup
- Understanding rollup-to-L1 messaging (withdrawals, cross-chain calls)

**Don't use for:** L1 consensus design, or general L2 applications that don't involve rollup infrastructure. Use the relevant L2 chain's SDK for app development on an existing rollup.

## Rollup Architecture

```
┌─────────────┐    ┌─────────────┐    ┌──────────────────┐
│   Users     │───▶│  Sequencer  │───▶│   L2 State Machine│
│ (L2 txs)    │    │ (order txs) │    │  (execute txs)    │
└─────────────┘    └─────────────┘    └──────────────────┘
                                                   │
                                                   ▼
                                         ┌──────────────────┐
                                         │   Batcher /      │
                                         │   Prover         │
                                         │ (post to L1)     │
                                         └──────────────────┘
                                                   │
                                                   ▼
                                         ┌──────────────────┐
                                         │   L1 Contract    │
                                         │ (rollup registry)│
                                         └──────────────────┘
```

**Components:**
- **Sequencer:** orders L2 transactions, produces L2 blocks.
- **L2 state machine:** executes transactions, updates state.
- **Batcher (ZK):** collects state diffs, produces a ZK proof, posts to L1.
- **Prover (ZK):** generates the ZK proof that the state transition is correct.
- **Fraud prover (optimistic):** generates a fraud proof if a state diff is incorrect.
- **L1 contract:** verifies ZK proofs or handles fraud proof disputes, stores the L2 state root.

## ZK Rollup (Validity Rollup)

A ZK rollup posts a validity proof to L1. The proof attests that the L2 state transition (from state root A to state root B) is correct given the posted transactions.

### Data Posted to L1

- **State diff:** the new state root (or the full state diff, depending on design).
- **Transaction data (calldata):** the L2 transactions (or a compressed representation) for data availability.
- **Validity proof:** a ZK proof that the transactions, when applied to state root A, produce state root B.

### L1 Verification

The L1 contract verifies the ZK proof. If the proof is valid, the state root is updated. Users can trust that the L2 state is correct because the proof is mathematically binding.

```rust
// L1 verifier contract (conceptual, Solidity or Wasm)
fn verify_rollup_update(
    prev_state_root: Hash,
    new_state_root: Hash,
    tx_data: Vec<u8>,
    proof: Vec<u8>,
) -> bool {
    // Verify the ZK proof
    let valid = zk_verifier.verify(prev_state_root, new_state_root, tx_data, proof);
    if valid {
        state_root = new_state_root;
    }
    valid
}
```

### ZK Rollup Pros and Cons

| Pros | Cons |
|---|---|
| Strong security (validity proof) | Expensive proof generation |
| Fast finality on L1 (once proof verified) | Proof generation time (minutes to hours) |
| No fraud window | Complex ZK circuit design |

## Optimistic Rollup

An optimistic rollup posts the state diff to L1 and assumes it's correct. There's a challenge window (e.g., 7 days) during which anyone can submit a fraud proof. If a fraud proof is valid, the incorrect state update is reverted and the dishonest party is slashed.

### Data Posted to L1

- **State diff:** the new state root (or state diff).
- **Transaction data:** L2 transactions for data availability (so anyone can re-execute and verify).
- **No proof initially:** the proof is only needed if there's a dispute.

### Fraud Proof

A fraud proof shows that a specific state transition is incorrect. The fraud prover (a watchtower or anyone) submits a proof that re-executes a disputed portion of the transition and shows the correct result differs from the posted result.

```rust
// Fraud proof (conceptual)
fn submit_fraud_proof(
    disputed_block: Block,
    expected_state_root: Hash,
    actual_state_root: Hash,
) -> Result<(), FraudError> {
    // Re-execute the disputed block on L1 (or in a verifier contract)
    let computed_root = execute_block(disputed_block);
    if computed_root != actual_state_root {
        // Fraud proven — revert the state update, slash the sequencer
        revert_state_update();
        slash(sequencer);
        Ok(())
    } else {
        Err(FraudError::NoFraud)
    }
}
```

**Fraud proof design:** the fraud proof must be small enough to verify on L1 (gas-constrained). This is why many optimistic rollups use "interactive fraud proofs" — bisect the dispute down to a single step that's cheap to verify.

### Optimistic Rollup Pros and Cons

| Pros | Cons |
|---|---|
| Cheaper to produce (no ZK proof) | Long withdrawal window (fraud window) |
| Simpler prover (no ZK circuit) | Fraud proof complexity (interactive proofs) |
| Faster block production | Security depends on at least one honest watcher |

## Data Availability (DA)

Data availability is the guarantee that the data needed to verify the L2 (transactions, state diffs) is available to anyone. Without DA, an L2 can't be verified, and users can't withdraw.

### L1 as DA (calldata / blob data)

Post transaction data to L1 (as calldata or blob data). L1 provides DA because the data is on L1 and anyone can download it.

- **Pro:** strong DA, inherits L1 security.
- **Con:** expensive (L1 calldata is costly), especially for high-throughput rollups.

### Dedicated DA Layer

Use a separate DA layer (e.g., Celestia, EigenDA, Avail) that's designed for high-throughput data availability.

- **Pro:** cheaper, higher throughput.
- **Con:** different security assumptions (the DA layer's security), bridging complexity.

### DA in Rust

For a rollup, the DA layer is an interface: the batcher posts data, and the verifier (on L1 or off-chain) checks that the data is available.

```rust
pub trait DataAvailabilityLayer {
    fn post_data(&self, data: &[u8]) -> Result<DataCommitment, Error>;
    fn verify_availability(&self, commitment: &DataCommitment) -> Result<(), Error>;
}

// L1-based DA
pub struct L1Da {
    l1_client: L1Client,
}

impl DataAvailabilityLayer for L1Da {
    fn post_data(&self, data: &[u8]) -> Result<DataCommitment, Error> {
        // post as calldata or blob to L1
        let tx = self.l1_client.send_calldata(data)?;
        Ok(DataCommitment::from_tx_hash(tx.hash()))
    }

    fn verify_availability(&self, commitment: &DataCommitment) -> Result<(), Error> {
        // check the L1 tx is included in a block
        self.l1_client.verify_tx_included(commitment.tx_hash)?;
        Ok(())
    }
}
```

## Sequencer Design

The sequencer orders L2 transactions and produces L2 blocks. It's the "block producer" for L2.

### Centralized Sequencer

A single sequencer (running by the rollup operator). Fast, simple, but centralized (single point of failure, censorship risk).

### Decentralized Sequencer

Multiple sequencers, with a consensus mechanism (PoS, etc.) to order transactions. More decentralized, more complex.

### Sequencer in Rust

```rust
pub struct Sequencer {
    mempool: Mempool,
    state: L2State,
    consensus: SequencerConsensus,   // if decentralized
}

impl Sequencer {
    pub async fn produce_block(&mut self) -> Result<Block, Error> {
        let txs = self.mempool.select_for_block(MAX_BLOCK_GAS)?;
        let block = Block {
            parent: self.state.current_root,
            txs,
            timestamp: current_time(),
        };
        let new_state = self.state.apply_block(&block)?;
        self.state = new_state;
        Ok(block)
    }
}
```

## State Diff and Commitment

A rollup posts state diffs to L1 (or a DA layer). The state diff is the change in state from one block to the next, or from one batch to the next.

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateDiff {
    pub prev_root: Hash,
    pub new_root: Hash,
    pub updates: Vec<StateUpdate>,   // key-value updates, or account deltas
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateUpdate {
    pub key: Hash,      // account address, storage slot, etc.
    pub value: Vec<u8>, // new value (or deletion marker)
}
```

**State root:** the Merkle root of the state tree after applying the state diff. The L1 contract stores the state root (or the rollup contract does).

## L2-to-L1 Messaging (Withdrawals, Cross-Chain Calls)

Users on L2 can send messages to L1 (withdrawals, token transfers, cross-chain calls). The message is proven by the rollup (either via the state root being on L1 for ZK rollups, or via a fraud-proof window passing for optimistic rollups).

```rust
pub struct L2Message {
    pub sender: Address,
    pub recipient: Address,
    pub value: u64,
    pub data: Vec<u8>,
    pub l2_block_height: u64,
    pub proof: MessageProof,   // proof that the message was included in an L2 block
}
```

**For ZK rollups:** the proof is that the L2 block (with the message) is part of a state transition that was verified on L1.

**For optimistic rollups:** the proof is that the L2 block was posted to L1 and the challenge window has passed (no fraud proof submitted).

```rust
pub fn verify_l2_message(message: &L2Message, l1_state: &L1State) -> bool {
    match l1_state.rollup_type {
        RollupType::ZK => {
            // Verify that the L2 block hash is in a verified state root on L1
            let state_root = l1_state.get_verified_state_root(message.l2_block_height);
            let proof = verify_merkle_proof(message.l2_block_hash, &state_root)?;
            proof.is_ok()
        }
        RollupType::Optimistic => {
            // Check that the L2 block was posted and the challenge window has passed
            let posted = l1_state.is_block_posted(message.l2_block_height);
            let window_passed = l1_state.is_window_passed(message.l2_block_height);
            posted && window_passed
        }
    }
}
```

## Rollup Bridge

A bridge connects L1 and L2, allowing asset transfers and message passing.

```rust
pub struct Bridge {
    l1: L1Client,
    l2: L2Client,
    rollup_contract: RollupContract,
}

impl Bridge {
    pub async fn deposit(&self, amount: u64, from: Address, to: Address) -> Result<(), Error> {
        // Lock tokens on L1, mint/transfer on L2
        self.l1.lock_tokens(amount, from).await?;
        self.l2.mint_tokens(amount, to).await?;
        Ok(())
    }

    pub async fn withdraw(&self, amount: u64, from: Address, to: Address) -> Result<(), Error> {
        // Burn on L2, unlock on L1 (after proof / challenge window)
        self.l2.burn_tokens(amount, from).await?;
        let message = L2Message { ... };
        // Wait for proof / challenge window
        self.l1.unlock_tokens(amount, to, &message).await?;
        Ok(())
    }
}
```

## Verification Checklist

- [ ] Can explain the difference between ZK rollups (validity proof) and optimistic rollups (fraud proof)
- [ ] Can describe what data a rollup posts to L1 (state diff, tx data, proof)
- [ ] Can explain data availability and why it matters for rollups
- [ ] Can describe the role of the sequencer in L2
- [ ] Can describe the withdrawal process for ZK vs optimistic rollups
- [ ] Can describe the fraud proof concept (optimistic) and the validity proof concept (ZK)
- [ ] Can design a simple state diff structure
- [ ] Understands the tradeoff between L1 as DA and a dedicated DA layer
- [ ] Understands that the L1 contract verifies ZK proofs (or handles fraud disputes)

## Common Pitfalls

1. **Confusing ZK rollups and optimistic rollups.** They have different security models, different withdrawal times, and different proof mechanisms. Don't mix them up.

2. **Ignoring data availability.** Without DA, the rollup can't be verified. Design for DA explicitly (L1 calldata, blob, or dedicated DA layer).

3. **Assuming ZK proofs are cheap to generate.** ZK proof generation is computationally expensive. Plan for proving time and prover infrastructure.

4. **Not understanding the fraud window for optimistic rollups.** Withdrawals take longer (fraud window). Users need to know this.

5. **Building a centralized sequencer without a decentralization plan.** A centralized sequencer is a single point of failure and censorship risk. Plan for decentralization or accept the tradeoff.

6. **Not designing L2-to-L1 messaging carefully.** Messages from L2 to L1 need a proof mechanism (ZK proof or challenge window). Design this explicitly.

7. **Over-simplifying the state diff.** A state diff must capture all changes to the state. If it misses something, the L1 state root is wrong.

8. **Not considering the L1 verification cost.** Verifying a ZK proof on L1 costs gas. Optimistic rollups cost gas for fraud proof verification (if a dispute happens). Plan for L1 costs.

9. **Assuming the sequencer is honest.** A malicious sequencer can censor txs or order them unfavorably. Design for sequencer trust assumptions.

10. **Not testing the full flow (deposit → L2 tx → withdraw).** The bridge flow is the main user-facing feature. Test it end-to-end.

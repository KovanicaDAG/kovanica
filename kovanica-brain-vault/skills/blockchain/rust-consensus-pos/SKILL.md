---
name: rust-consensus-pos
description: Use when implementing Proof of Stake consensus in Rust: validator sets, stake-weighted voting, proposal/attestation rounds, slashing conditions, Tendermint-style BFT, and fork choice rules.
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [blockchain, PoS, proof-of-stake, BFT, Tendermint, validator, slashing, attestation, fork-choice]
    related_skills: [blockchain-fundamentals, rust-consensus-pow, rust-crypto-primitives]
---

# Rust Proof of Stake Consensus

## Overview

Proof of Stake (PoS) consensus selects block proposers and validators based on the amount of stake (locked currency) they hold. Validators vote on blocks; the protocol specifies how votes are aggregated, how finality is reached, and how misbehavior is punished (slashing). Two major BFT-style families: Tendermint/CometBFT (round-based, deterministic finality) and Ethereum's Gasper (LMD-GHOST + FFG, probabilistic finality with checkpoints).

This skill covers the core concepts and implementation patterns for BFT-style PoS. For specific implementations (Substrate's GRANDPA/BABE, Cosmos SDK, Ethereum), see the platform-specific skills.

## When to Use

- Designing a PoS-based blockchain
- Implementing validator selection (stake-weighted or round-robin)
- Implementing a BFT consensus round (propose, prevote, precommit)
- Defining slashing conditions (equivocation, downtime)
- Managing validator set changes (stake changes, joins/leaves)
- Fork choice and finality logic

**Don't use for:** PoW consensus, or DAG-based consensus. Don't use for simple "longest chain" PoS without BFT finalization.

## Validator Set

### Validator Representation

```rust
#[derive(Debug, Clone)]
struct Validator {
    pub_id: PublicKey,       // identity of the validator
    stake: u64,              // amount at stake
    commission: u8,          // optional: commission rate for delegators
    status: ValidatorStatus, // active, jailed, unbonding
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ValidatorStatus {
    Active,
    Unbonding,
    Jailed,   // slashed or misbehaving, temporarily excluded
}
```

The validator set is the set of active validators eligible to propose and vote. It's usually a sorted/ordered list (by pub_id or by stake) for deterministic selection.

### Validator Selection

Proposer selection is deterministic and stake-weighted:

```rust
// Round-robin among validators, weighted by stake
fn select_proposer(validators: &[Validator], round: u64) -> Option<usize> {
    let total_stake: u64 = validators.iter().map(|v| v.stake).sum();
    if total_stake == 0 { return None; }

    let seed = hash(&[round, some_seed]);   // deterministic pseudo-random
    let picked = (seed as u64) % total_stake;

    let mut cumulative = 0u64;
    for (i, v) in validators.iter().enumerate() {
        cumulative += v.stake;
        if picked < cumulative {
            return Some(i);
        }
    }
    None
}
```

This is a simplified stake-weighted roulette. Real protocols may use sortition (Algorand-style), round-robin with stake ordering (Tendermint), or VRF-based selection.

### Validator Set Changes

Validator sets change over time (stake changes, validators join/leave). Two approaches:
- **Snapshot-based:** the validator set is fixed for an epoch; changes take effect at epoch boundaries.
- **Gradual:** validators enter/leave gradually (e.g., after an unbonding period).

```rust
struct ValidatorSet {
    active: Vec<Validator>,
    unbonding: Vec<Validator>,   // leaving, stake still at risk for a period
    epoch: u64,
}
```

For BFT consensus, the validator set must be known and deterministic at each round. Changes at epoch boundaries are simpler than mid-round changes.

## BFT Consensus Round (Tendermint-style)

### Round Structure

A round has several steps:
1. **Propose** — a validator is selected to propose a block.
2. **Prevote** — validators vote on whether they accept the proposal.
3. **Precommit** — if enough prevotes (2/3+ of stake), validators precommit.
4. **Commit** — if enough precommits, the block is committed.

If the round fails (no proposal, no 2/3+ prevotes), it moves to the next round with a new proposer.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RoundStep {
    Propose,
    Prevote,
    Precommit,
    Commit,
    RoundChange,   // move to next round
}

struct ConsensusState {
    round: u64,
    step: RoundStep,
    height: u64,
    proposer: Option<ValidatorIndex>,
    // votes, proposals, etc.
}
```

### Votes

```rust
#[derive(Debug, Clone)]
struct Prevote {
    validator: PublicKey,
    height: u64,
    round: u64,
    block_hash: Option<Hash>,   // vote for a specific block, or nil (vote nil)
    signature: Signature,
}

#[derive(Debug, Clone)]
struct Precommit {
    validator: PublicKey,
    height: u64,
    round: u64,
    block_hash: Option<Hash>,
    signature: Signature,
}
```

Votes are signed by the validator. The protocol aggregates votes and checks for 2/3+ stake threshold.

### Locking and Haunting

To prevent conflicting commits, validators "lock" on a block when they precommit. In subsequent rounds, they must prevote for the locked block (or unlock under certain conditions). This is the "locking" mechanism in Tendermint.

```rust
struct ValidatorLock {
    locked_block: Option<Hash>,
    locked_round: u64,
}
```

Rules:
- After precommitting a block, lock on it.
- In future rounds, prevote for the locked block unless a higher round's precommit is seen.
- Unlock if a timeout or a newer lock is seen.

The locking rules prevent nothing-at-stake and ensure safety.

## Fork Choice and Finality

### Tendermint Finality

In Tendermint, a block is finalized when it's committed (2/3+ precommits). There's no fork after finality — the committed block is canonical. If a validator signs two conflicting precommits at the same height/round, that's equivocation and is slashable.

### Fork Choice Rule (when forks exist)

If forks exist (e.g., in a less strict BFT or a longest-chain PoS), the fork choice rule determines canonical:

- **Longest chain** (most blocks, or most weight).
- **Heaviest attestation** (Ethereum's LMD-GHOST): the chain with the most stake-weighted attestations in the latest epoch.
- **Last finalized block as anchor:** in FFG, the last finalized checkpoint is the anchor; forks below it are irrelevant.

```rust
fn fork_choice(forks: &[Chain] ) -> &Chain {
    // Example: heaviest weight (most stake-weighted blocks)
    forks.iter().max_by_key(|chain| weight(chain)).unwrap()
}
```

## Slashing Conditions

Slashing punishes validators for provable misbehavior. Common conditions:

- **Equivocation (double-signing):** signing two different votes (prevotes or precommits) at the same height and round. This is a direct safety violation.
- **Downtime:** being offline for too long (missed proposals/votes). Less severe — usually a minor slash or jail.
- **Light client attack:** voting for an invalid block or a block that doesn't extend the correct chain. Protocol-specific.

```rust
#[derive(Debug)]
struct SlashEvent {
    validator: PublicKey,
    offence: Offence,
    evidence: Vec<u8>,   // signed votes proving the offence
}

#[derive(Debug)]
enum Offence {
    Equivocation(Height, Round),   // signed two votes at same height/round
    Downtime { missed_slots: u64, window: u64 },
}
```

Slashing typically:
- Burns a portion of the validator's stake (e.g., 1% for downtime, more for equivocation).
- Jails the validator (removes from active set for a period).
- May trigger unbonding of delegators' stake.

Evidence of equivocation: two signed votes from the same validator at the same height/round with different block hashes. This is publicly verifiable.

## Implementation Patterns

### Vote Aggregation

```rust
struct VoteSet {
    height: u64,
    round: u64,
    prevotes: HashMap<Hash, Vec<Prevote>>,   // block_hash -> votes
    precommits: HashMap<Hash, Vec<Precommit>>,
    total_prevotes: u64,   // number of prevotes (for threshold check)
    total_precommits: u64,
    stake_prevotes: HashMap<Hash, u64>,   // stake per block_hash
    stake_precommits: HashMap<Hash, u64>,
}
```

Aggregate by block_hash and check if any block_hash has 2/3+ stake.

### Timeout and Round Progression

```rust
// If no proposal within timeout_propose, move to prevote (or round change)
// If no 2/3+ prevote within timeout_prevote, move to next round
fn advance_round(state: &mut ConsensusState, timeout: Duration) {
    if state.step == RoundStep::Propose && time_since_propose() > timeout {
        state.round += 1;
        state.step = RoundStep::Propose;
        state.proposer = select_proposer(validators, state.round);
    }
}
```

### Message Propagation

Proposals, prevotes, and precommits are broadcast to all validators. Use P2P networking (gossip) or a point-to-point broadcast. For small validator sets, broadcast to all; for large sets, use a gossipsub or a dedicated pub/sub.

## Verification Checklist

- [ ] Can define a validator set with stake and status
- [ ] Can implement stake-weighted proposer selection (deterministic per round)
- [ ] Can define the round structure (propose, prevote, precommit, commit)
- [ ] Can verify a prevote/precommit signature and check it's from a validator
- [ ] Can aggregate votes and check for 2/3+ stake threshold
- [ ] Can implement locking rules to prevent conflicting commits
- [ ] Can define slashing conditions (equivocation, downtime) and verify evidence
- [ ] Can implement a fork choice rule (heaviest chain, heaviest attestation, or last finalized)
- [ ] Understands the difference between finality (Tendermint commit) and probabilistic confirmation

## Common Pitfalls

1. **Assuming the validator set is static.** Validators join/leave, stake changes. The consensus must handle set changes correctly (epoch-based snapshots are simplest).

2. **Not handling round timeouts.** Rounds can stall if a proposer is offline or a vote doesn't reach 2/3+. Implement timeouts and round progression.

3. **Not locking on precommits.** Without locking, a validator can precommit conflicting blocks in different rounds, violating safety. Locking is essential.

4. **Slashing without provable evidence.** Slashing must be based on verifiable evidence (signed votes), not just accusations. Equivocation is provable; downtime requires a window of missed votes.

5. **Not distinguishing between "prevote nil" and "prevote for a block."** A nil prevote means "I don't support any block yet." A prevote for a block means "I support this block." Both are valid; the aggregation must account for nil votes separately.

6. **Assuming 2/3+ of validators = 2/3+ of stake.** The threshold is stake-weighted, not validator-count-weighted. A few large validators can dominate.

7. **Fork choice that doesn't account for finality.** If a block is finalized (Tendermint-commit), it's canonical regardless of chain length. Fork choice must respect finality.

8. **Validator keys on the same machine as the node keys.** For security, validator signing keys should be isolated (HSM, remote signer). If a validator key is compromised, the attacker can sign conflicting votes.

9. **Ignoring network partitions.** In a partition, one side may not reach 2/3+. The protocol must handle this gracefully (timeout, round change, no commit).

10. **Not versioning the consensus protocol.** Validator software may be upgraded. The consensus messages and rules must be versioned so that old and new validators can coexist (or the set is upgraded atomically at epoch boundaries).

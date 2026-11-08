# Kovanica Consensus Fork Handling Policy & Equivocation Specification

This document formalizes the fork resolution rules, equivocation proof construction, slashing mechanics, and finality invariants for the Kovanica consensus engine.

---

## 1. Executive Summary & Protocol Parameters

Kovanica employs high-throughput, uncertified DAG consensus based on the Mysticeti wave architecture, combined with an on-chain Proof-of-Stake (PoS) validator committee. Under normal operation, scheduled leaders produce statement blocks in round-based waves, and validators vote on proposals to reach commit quorum.

Because DAG proposals are uncertified prior to commit, an adversarial or malfunctioning leader may produce multiple conflicting blocks for the same round (known as **equivocation** or **double-signing**). Kovanica implements a strict, multi-layered fork handling policy:
1. **Pre-Decision Tie-Breaking (`use_min_digest_fork`)**: When competing blocks for the same undecided round/leader are observed, nodes deterministically converge on the block with the lexicographically smaller digest (`min(digest(B1), digest(B2))`).
2. **Equivocation Evidence & Slashing**: Validators observing equivocation generate and sign an `EquivocationProof` (`EquivocationEvidence`) carrying both conflicting block headers. Submission of valid proof slashes the offender by 5% (`SLASH_PCT_BPS = 500`), immediately removes the validator from the committee, and sentences them to jail for 7,200 leader heights.
3. **Absolute Commit Finality (Zero Reorg)**: Once a leader block is certified in a wave commit (`mark_decided_and_commit_leader`), it is immutable and finalized. The reorganization depth bound past commit certification is strictly **zero**.

### 1.1 Canonical Protocol Parameters

| Parameter | Value | Reference / Code Binding |
|---|---|---|
| **Currency Ticker** | **KUNA** | 9 decimals ($1\text{ KUNA} = 10^9\text{ atoms}$) |
| **Address Format** | **PRO-21 base32** | 64 chars (`kvnc` + 57 base32 chars + `dag`) |
| **Wave Length** | **3 rounds** | `WAVE_LENGTH = 3` (Round 0 = Leader, Round 1 = Votes, Round 2 = Cert/Anchor) |
| **Commit Quorum** | **$2f + 1$ stake** | `CommitteeInfo::quorum_threshold` |
| **Validity Threshold** | **$f + 1$ stake** | `CommitteeInfo::validity_threshold` |
| **Union-Quorum Floor** | **$\text{votes} > \text{total} - \text{quorum}$** | PRO-40 / PRO-41 union commit rule |
| **Min-Digest Fork Flag** | **`true` (default)** | `ConsensusConfig::use_min_digest_fork` |
| **Slashing Penalty** | **500 bps (5%)** | `SLASH_PCT_BPS = 500` (`kovanica_params`) |
| **Slashing Target** | **Self + Delegations + Unbonding** | Slashes active stake and pending unbonding queue entries |
| **Jail Duration** | **7,200 leader heights** | $\approx 24\text{ hours}$ (3 epochs of 2,400 heights) |
| **Evidence Max Age** | **30,000 rounds** | `EVIDENCE_MAX_AGE_ROUNDS = 30_000` ($\approx 33\text{ hours}$) |
| **Evidence Deduplication** | **Keyed Domain Hash** | `Hash::new_keyed(DOMAIN_EVIDENCE, author ‖ round_le)` |

---

## 2. DAG Fork Resolution & Mysticeti Commit Rule

### 2.1 Wave Architecture

The Kovanica DAG structure groups consensus rounds into 3-round waves (`WAVE_LENGTH = 3`):
- **Round $3k$ (Leader Slot)**: The deterministically scheduled leader (via round-robin or stake-weighted VRF) proposes a `StatementBlock` containing user transactions and references to causal parent blocks.
- **Round $3k + 1$ (Voting Round)**: Validators ingest and validate the leader block, then broadcast cryptographically signed `Vote` messages binding their authority key to `(round, leader_hash)`.
- **Round $3k + 2$ (Certification / Wave Anchor)**: Validators reference preceding voting blocks, aggregating quorum and preparing causal anchors for subsequent waves.

### 2.2 Direct Commit Rule

A leader block at round $R = 3k$ is **directly committed** if:
1. It receives valid votes from a quorum of validators representing at least $2f + 1$ stake in the committee (`BaseCommitter::try_direct_decide`).
2. Each vote's signature strictly verifies under the voter's public key registered in `CommitteeInfo`.
3. The votes match the leader block's digest (`vote.leader_hash == leader_block.digest`).

### 2.3 Union-Quorum Commit Rule (PRO-40 / PRO-41)

When network latency or round timeouts split the validator set around the skip deadline $T_{\text{skip}}$, a slot may collect both votes and timeout attestations. The committer evaluates the union-quorum rule (`try_direct_decide_with_timeouts`):
- A slot is decided as a **Commit** when:
  $$\text{votes} + \text{timeouts} \ge \text{quorum\_threshold}$$
  **AND**
  $$\text{votes} > \text{total\_stake} - \text{quorum\_threshold}$$
- This rule guarantees that non-voting validators cannot assemble a competing `SkipCert` ($< \text{quorum\_threshold}$ remaining stake), safely resolving the split into a deterministic commit without fork divergence.

### 2.4 Indirect Commit Rule

A leader block at round $R$ that obtained validity threshold ($f + 1$) but did not achieve direct commit in its immediate wave can be committed **indirectly** when a future leader block at round $R + 3m$ achieves direct commit and includes round $R$'s leader in its causal history (`get_ancestors`).

---

## 3. Min-Digest Fork Resolution (`use_min_digest_fork`)

### 3.1 Motivation & Threat Model

In an uncertified DAG, a Byzantine validator acting as the scheduled leader for round $R$ can equivocate by creating two distinct blocks $B_1$ and $B_2$ where:
$$\text{round}(B_1) = \text{round}(B_2) = R, \quad \text{author}(B_1) = \text{author}(B_2) = L, \quad \text{digest}(B_1) \ne \text{digest}(B_2)$$

If validators independently accept whichever block they observe first ("first-valid-wins"), network partitions or gossip latency can cause half the network to vote for $B_1$ and the other half for $B_2$. This splits vote stake, stalls commit progress, and requires timeout skip cycles.

### 3.2 Flag Semantics & Deterministic Tie-Breaking

To eliminate arbitrary arrival-order divergence, `ConsensusConfig::use_min_digest_fork` defaults to **`true`** in production.

When `use_min_digest_fork = true`, the consensus engine enforces a strict tie-breaking invariant:
- If a leader block $B_{\text{incoming}}$ arrives and the DAG store already holds an existing leader block $B_{\text{existing}}$ for the same `(author, round)`:
  - If $\text{digest}(B_{\text{incoming}}).0 < \text{digest}(B_{\text{existing}}).0$ (lexicographically smaller byte array):
    - Provided the round is still `Undecided` (`!committer.is_decided(round)`),
    - And the DAG store accepts the replacement (`dag_store.replace_block(&existing, &incoming).is_ok()`),
    - The engine atomically swaps the leader candidate: drops $B_{\text{existing}}$, installs $B_{\text{incoming}}$, updates the committer's leader slot, and rebuffers matching votes.
  - If $\text{digest}(B_{\text{incoming}}).0 \ge \text{digest}(B_{\text{existing}}).0$:
    - The incoming candidate is rejected, the existing candidate is retained, and equivocation is reported for slashing.

### 3.3 Storage Atomicity Invariants (`DagStore::replace_block`)

`DagStore::replace_block` in `crates/kovanica-dag` executes inside a single read-write storage transaction (`storage.begin_write()`) and enforces the following conditions:
1. **Identity Invariant**: `existing.author == incoming.author` and `existing.round == incoming.round`.
2. **Undecided Round Invariant**: The round must not exist in `tables::DECIDED_ROUNDS`. If the round has already been decided, the swap is refused immediately with `DagStoreError::NotFound("round is already decided")`.
3. **No Dependent Children Invariant**: The existing block digest must have **zero children** in the DAG (`get_children(&txn, &existing.digest)?.is_empty()`). If subsequent blocks have already referenced $B_{\text{existing}}$ as a causal parent, replacement is strictly refused.
4. **Digest Exclusion**: The incoming block digest must not already be present in `tables::DAG_BLOCKS`.
5. **Atomic Mutation**: The storage transaction deletes $B_{\text{existing}}$ and writes $B_{\text{incoming}}$ before committing. If any check fails, the transaction aborts and storage state remains completely unchanged.

### 3.4 Convergence Invariant

For any set of nodes $\mathcal{N}$ receiving competing blocks $\{B_1, B_2, \dots, B_k\}$ in arbitrary permutations:
$$\forall N_i \in \mathcal{N}, \quad \text{selected\_block}(N_i) = \arg\min_{B \in \{B_1, \dots, B_k\}} \text{digest}(B)$$
All nodes deterministically converge on the identical canonical leader block, preventing split votes and consensus stalls.

---

## 4. Equivocation Proof Structure & Verification

### 4.1 Proof Structure

Equivocation proof is encapsulated by `SignedBlockHeader` and `EquivocationEvidence` (aliased as `EquivocationProof`):

```rust
pub struct SignedBlockHeader {
    /// Author address (derived from the signing public key)
    pub author: Address,
    /// Consensus round
    pub round: Round,
    /// Block header digest
    pub digest: Hash,
    /// Ed25519 signature over block_signature_data(chain_id, digest)
    pub signature: Signature,
}

pub struct EquivocationEvidence {
    pub header_a: SignedBlockHeader,
    pub header_b: SignedBlockHeader,
}
pub type EquivocationProof = EquivocationEvidence;
```

### 4.2 Verification Rules (`offender()`)

An `EquivocationProof` is valid if and only if all of the following conditions pass:
1. **Author Equality**: `header_a.author == header_b.author`.
2. **Round Equality**: `header_a.round == header_b.round`.
3. **Digest Discrepancy**: `header_a.digest != header_b.digest` (identical digests do not constitute equivocation).
4. **Strict Cryptographic Signatures**: Both `header_a.signature` and `header_b.signature` must be valid strict Ed25519 signatures covering their respective digests under the author's public key (`verify_strict`). Signatures must match the active network chain ID binding (`KUNA-BLOCK-v1` domain separation tag).
5. **Freshness Window**: $\text{current\_round} - \text{round} \le \text{EVIDENCE\_MAX\_AGE\_ROUNDS}$ (30,000 rounds $\approx 33$ hours). Evidence older than 30,000 rounds is rejected to prevent unbonding queue replay exploits.

---

## 5. Evidence Gossip & Slashing Mechanics

### 5.1 Evidence Transaction Generation

When `ConsensusEngine::process_block` detects an equivocation (on the reject branch):
1. The engine checks if slashing is active (`slashing_active_from(round)`).
2. It constructs an `EquivocationEvidence` from the two conflicting blocks and queries the local validator's account nonce.
3. It packages the proof into a `TransactionKind::Evidence { evidence }` transaction.
4. The observing validator signs the transaction with its local key and submits it to the mempool.
5. Evidence transactions submitted by active validators are **fee-free** (`requires_fee = false`) and gossip to all network peers.

### 5.2 Slashing Execution & State Changes

When an evidence transaction is executed in a committed block (`kovanica_execution::execute_evidence`):
1. **Deduplication Check**: The execution engine computes the deterministic evidence identifier:
   $$\text{evidence\_id} = \text{Hash::new\_keyed}(\text{DOMAIN\_EVIDENCE}, \text{author} \parallel \text{round}_{\text{LE}})$$
   If `evidence_id` exists in `tables::PROCESSED_EVIDENCE`, the transaction is dropped as a duplicate. Otherwise, the ID is recorded immediately.
2. **Offender Validation**: The offender must be an active validator in `StakingState`.
3. **Stake Penalty (5% Slash)**:
   - Exactly $5\%$ (`SLASH_PCT_BPS = 500` / 10,000) is deducted from the validator's self-stake.
   - Exactly $5\%$ is deducted from all active delegators delegated to this validator.
   - Exactly $5\%$ is deducted from all pending unbonding entries associated with this validator in `unbonding_queue`.
   - All slashed KUNA funds are credited directly to the protocol treasury address (`TREASURY`).
4. **Immediate Deactivation & Jailing**:
   - The validator is immediately deactivated and removed from the active committee.
   - The validator is placed in jail until:
     $$\text{release\_height} = \text{committed\_leader\_height} + 7,200$$
   - Jailed validators cannot produce blocks, cast votes, or earn block rewards during the jail penalty period.

---

## 6. Reorganization Depth Bounds & Finality Guarantees

### 6.1 Absolute Commit Finality (Reorg Depth = 0)

In traditional Nakamoto consensus, reorganizations of depth $k$ are possible if an alternate proof-of-work chain overtakes the canonical chain.

In Kovanica Mysticeti consensus, **certified commits are irrevocable**:
- Once a leader block receives quorum ($2f + 1$) and is committed via `try_commit_and_mark_durable`, it is durably marked in `tables::DECIDED_ROUNDS`.
- The reorganization depth bound past commit certification is **0 (zero)**.
- A block certified in a wave commit cannot be displaced, reorganized, or rolled back under any circumstances.

### 6.2 Multi-Layered Invariant Defense

Four distinct architectural layers guarantee reorg depth bound = 0:

```
[Layer 1: Engine Guard]
  if self.committer.is_decided(block.round) => REJECT ALTERNATE BLOCK
                 │
                 ▼
[Layer 2: Committer Guard]
  decided_leaders: BTreeMap<Round, LeaderInfo> => IMMUTABLE ONCE DECIDED
                 │
                 ▼
[Layer 3: Storage Table Guard]
  tables::DECIDED_ROUNDS.contains(round) => REFUSE DagStore::replace_block
                 │
                 ▼
[Layer 4: DAG Topology Guard]
  get_children(existing.digest).len() > 0 => REFUSE REPLACEMENT
```

1. **Engine Layer**: `ConsensusEngine::process_block` explicitly checks `!self.committer.is_decided(block.round)` before even attempting a replacement. If the round has been decided, it branches directly to equivocation rejection and proof generation.
2. **Committer Layer**: `UniversalCommitter` records decisions in `decided_leaders`. The commit cursor (`last_decided_round`) is monotonic and only moves forward. `update_leader` refuses to alter decided leaders.
3. **Storage Layer**: `DagStore::replace_block` queries `tables::DECIDED_ROUNDS`. If a decided entry exists, the storage write transaction aborts with an error.
4. **DAG Topology Layer**: Once subsequent blocks reference the leader block as a parent, `replace_block` refuses to delete or swap it.

### 6.3 Finality Under Partition & Adversarial Attack

Even in the presence of up to $f$ Byzantine validators actively equivocating or broadcasting competing branches, an honest quorum of $2f + 1$ validators ensures that two conflicting blocks can never both achieve quorum:
$$(2f + 1) + (2f + 1) = 4f + 2 = 3f + f + 2 > 3f + 1$$
Because any two quorums intersect at least at $f + 1$ validators, at least one honest validator exists in the intersection. Since honest validators never sign two conflicting votes for the same round, two conflicting blocks cannot both be certified.

Certified blocks are finalized in $\le 2$ network round-trips ($\approx 4-8\text{ seconds}$), providing deterministic, zero-reorg settlement for all Kovanica transactions.

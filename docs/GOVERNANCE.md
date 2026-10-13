# Phase 25 — Governance (post-mainnet, kvnc)

> **Status:** design finalized **2026-10-13** (kvnc team; no external dependency).
> This document is the authoritative design. Implementation is gated on
> RFC-006 activation + stable staking (Phase 18) and lands **after mainnet**, so
> every rule below is written to be deterministic and auditable before a single
> line of code is added.

## 0. Scope & layering

| Layer | Verdict | Why |
|---|---|---|
| Consensus (DAG, wave, k=3) | **immutable** | not governable — no on-chain vote can change it |
| Tokenomics (cap, era, decay, reward, maturity, fee split) | **immutable** | locked by AGENTS.md; governance must never reach these |
| Ledger rules (state transition, gas, sighash) | **governed via upgrade signaling only** | hard-fork, operator-coordinated |
| Operational params (round duration, peer caps, RPC limits, treasury spend) | **governed on-chain** | ledger-safe, deterministic |
| Client-only surfaces (dashboards, notifications) | **not governed** | off-chain |

**Hard invariant:** a proposal that targets a tokenomics or consensus constant is
**invalid at submission time** and must be rejected before it is ever counted.

## 1. Proposal object

```
GovernanceProposal {
    id:          ProposalId,      // (epoch: u64, counter: u32) -> "prop-{epoch}-{counter}"
    kind:        ProposalKind,    // ParamChange | TreasurySpend | UpgradeSignal
    payload:     ProposalPayload, // kind-dependent, deterministic encoding
    proposer:    Address,         // must be an active validator at submission
    submit_epoch: u64,            // = epoch at commit
    deadline_epoch: u64,          // = submit_epoch + VOTING_EPOCHS
    snapshot:    StakeSnapshot,   // voting power frozen at submit_epoch
    tally:       Tally,           // filled during voting
    state:       ProposalState,   // Pending -> Passed | Rejected | Expired | Executed
}
```

- **Uniqueness:** `(epoch, counter)` is assigned deterministically from committed
  state; two proposals for the same `(epoch, counter)` are impossible because the
  counter increments monotonically on commit.
- **Submitter:** must be an active validator (in the committee at `submit_epoch`),
  with a bond of `MIN_VALIDATOR_STAKE`. Prevents spam from non-stake accounts.
- **Deposit:** a fixed, non-refundable governance fee (`PROPOSAL_FEE`) is burned
  on submit — same 75/25 burn/producer split as a normal fee, to keep the supply
  accounting uniform. No refundable anti-spam deposit in v1 (keeps the ledger
  simple and deterministic).

## 2. Vote

```
GovernanceVote {
    proposal: ProposalId,
    voter:    Address,
    power:    u64,               // = snapshot[bonded stake of voter]  (see §4)
    signature: Signature,        // domain tag: KUNA/vote/gov/v1
}
```

- Votes are **signed messages** (not transactions) gossiped on the `votes` topic
  under a dedicated governance sub-protocol and included by the producer; they
  are **idempotent**: a second vote by the same `(proposal, voter)` is ignored
  (first-seen wins, deterministic by commit order).
- A voter may **change** its vote exactly once while `state == Pending` by
  submitting a `GovernanceVoteRevote`; the latest committed vote (by DAG order)
  wins. This is the only mutation of an otherwise append-only tally.
- Voting power never changes mid-proposal: it is frozen from the snapshot.
- Deadline is measured in **epochs**, never wall-clock.

## 3. Tally & thresholds

| Kind | Quorum (share of total snapshot stake) | Approval (share of **cast** stake) |
|---|---|---|
| ParamChange | ≥ 1/2 (`2/3` recommended) | > 1/2 |
| TreasurySpend | ≥ 2/3 | ≥ 2/3 |
| UpgradeSignal | ≥ 2/3 | ≥ 2/3 |

- `quorum` is measured against `snapshot.total_active_stake`; abstentions (no vote)
  count toward the denominator, exactly like Tendermint-style gov.
- `approval` is measured against `cast` stake (yes + no), not total.
- All arithmetic is integer-only with saturating add; no floats anywhere in the
  tally path (determinism).

## 4. Voting snapshot

- At `submit_epoch`, freeze `StakeSnapshot { total_active_stake, per_validator,
  per_delegator }` computed from committed `StakingState`.
- Validators vote with their own bond **plus** delegated stake
  (`self_stake + delegated_to`), matching `StakingState::reward_share` semantics.
- **v1 simplification:** a delegator's power is exercised by its validator
  (operator votes on behalf of its delegators). Direct delegator votes are a
  **v2** feature; they require an extra vote-routing layer and are intentionally
  out of scope here.
- Snapshot is a value type persisted with the proposal; it is recomputable from
  the block at `submit_epoch` and therefore auditable.

## 5. Execution

- A proposal is executed **only at an epoch boundary** (`StakingState::rotate_epoch`),
  in ascending `ProposalId` order, in the same deterministic block that closes
  the epoch. No other trigger exists.
- `ParamChange`: writes the new value into the runtime parameter table
  (`RuntimeParams`). Only keys in the **whitelist** are accepted:

  | key | type | bounds |
  |---|---|---|
  | `round_duration_ms` | u64 | 250 ..= 10_000 |
  | `max_peers` | u32 | 1 ..= 128 |
  | `rpc_rate_limit_per_min` | u32 | 0 ..= 100_000 (0 = off) |
  | `faucet_rate_per_hour` | u32 | 0 ..= 100 |

  A `ParamChange` whose `new_value` is out of bounds is `Rejected` at tally time,
  not at execution time (fail early, never half-apply).
- `TreasurySpend`: calls the treasury vault release path (RFC-005). The payload
  fixes `to: Address`, `amount: u64`, `claim_height: u64`. Execution re-checks
  `amount <= treasury.available()`; if the vault is short the proposal is
  `Rejected` (never partially pays).
- `UpgradeSignal`: emits a `ChainUpgradeScheduled { version_hash, activation_epoch }`
  event. It does **not** mutate state; operators act on it. Activation requires
  a second, later proposal after the binary is distributed (two-phase).
- Failed execution never rolls back the epoch: the proposal is marked `Rejected`
  and the epoch proceeds.

## 6. Upgrade signaling (hard fork coordination)

```
UpgradeSignalPayload {
    version_hash:   Hash,   // blake3 of the canonical release manifest
    activation_epoch: u64,  // >= execute_epoch + UPGRADE_LEAD_EPOCHS (e.g. 4)
}
```

- Phase A: `UpgradeSignal` passes → event emitted; nodes keep running the old
  binary and log a **hard warning**.
- Phase B: after `activation_epoch`, a node whose `version_hash` mismatches the
  latest **passed** signal refuses to start. This is an explicit operator
  hard-fork gate, not automatic code swap.
- `UPGRADE_LEAD_EPOCHS` gives operators a full era-quarter to upgrade.

## 7. Determinism & safety checklist

- No wall-clock, no floats, no `HashMap` iteration in the tally/execution path.
- `ProposalId` ordering is total and stable across nodes.
- Votes are replayed against committed state; signature domain
  `KUNA/vote/gov/v1` prevents cross-context replay with consensus votes.
- Governance is **non-consensus-critical in v1**: it changes a param table and
  emits events; it never rewrites balances outside the explicit treasury path.
- Every new `TransactionKind` used to submit/vote is activation-gated on
  RFC-006 + a genesis/epoch height, so a pre-mainnet chain rejects it.

## 8. Implementation plan (needs approval before coding)

| Step | Deliverable | Layer | Gate |
|---|---|---|---|
| 25.0 | This spec | docs | ✅ done |
| 25.1 | `GovernanceProposal`/`Vote` types + deterministic encoding | types (ledger-safe) | spec approved |
| 25.2 | `GovernanceState` in `kvnc-staking` (proposals, snapshot, tally, execute) + unit tests | ledger-safe | 25.1 |
| 25.3 | `TransactionKind::ProposalSubmit` / `Vote` wired through execution + gas | ledger-safe | 25.2 |
| 25.4 | RPC `kvnc_submitProposal` / `kvnc_voteProposal` / `kvnc_getProposal` | client surface | 25.3 |
| 25.5 | CLI `governance propose|vote|list` | client surface | 25.4 |
| 25.6 | Param table (`RuntimeParams`) consumed by node config at epoch boundary | node | 25.3 |

**Explicitly deferred (v2):** direct delegator votes, on-chain text discussions,
slashing for governance abuse, automatic binary swap.

## 9. Immutable / out of scope (never governable)

- `MAX_SUPPLY = 90 200 000 KUNA`, `DECIMALS = 9`
- Founder premine `200 000 KUNA`, genesis validator allocation `1 000 000 KUNA`,
  treasury `8 000 000 KUNA` (8×1M linear)
- `INITIAL_BLOCK_REWARD = 2,5 KUNA`, `ERA_LENGTH = 8 100 000`, decay `×3/4`,
  `MINING_SUBSIDY_BUDGET = 81 000 000`
- `MIN_VALIDATOR_STAKE = 50 000 KUNA`, validators `15–21`
- `COINBASE_MATURITY = 100`, fee split `75 % burn / 25 % producer`
- Consensus: Mysticeti-style uncertified DAG, `wave = 3`, `k = 3`

## References

- `crates/kvnc-staking/src/lib.rs` — `StakingState`, `rotate_epoch`, treasury vaults
- `docs/TOKENOMICS.md`, `docs/A8.2-DELEGATION-FULL.md`
- `docs/SIGNATURE_FORMAT.md` — domain-tag convention

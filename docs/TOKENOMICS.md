# KVNC Tokenomics Specification

Canonical constants: `crates/kovanica-params`.

**Version:** 0.3.0  
**Status:** Approved by Toni (2026-11-06)

---

## 1. Supply Overview

Kovanica defines two genesis supply allocations depending on the network's consensus model:
1. **No-Stake Hybrid Networks (NoStake82M):** Mainnet (Chain 1) and Testnet (Chain 2) both run Hybrid PoW + GHOSTDAG with no stake in the consensus path. The 1,000,000 KVNC genesis validator allocation is retired and returned to capped mining emissions, setting the mining subsidy budget to 82,000,000 KVNC.
2. **Legacy (with validator stake, `Legacy81M`):** The former mining budget with a 1,000,000 KVNC genesis validator split survives only for existing databases and the dev/local chains (3, 1337). No new network uses it.

| Allocation                  | Legacy (with stake) | New NO-Stake (82M) | % of Total         | Notes                                      |
|-----------------------------|---------------------|--------------------|--------------------|--------------------------------------------|
| Mining subsidy              | 81 000 000 KVNC     | 82 000 000 KVNC    | 89.80 % / 90.91 %  | Emitted via block rewards                  |
| Treasury                    | 8 000 000 KVNC      | 8 000 000 KVNC     | 8.87 %             | 1 M / year × 8 years, linear vesting       |
| Genesis validator allocation| 1 000 000 KVNC      | 0 KVNC             | 1.11 % / 0 %       | Omitted on new NO-stake networks           |
| Founder premine             | 200 000 KVNC        | 200 000 KVNC       | 0.22 %             | Unlocked at genesis                        |
| **Total supply**            | **90 200 000 KVNC** | **90 200 000 KVNC**| 100 %              | Hard cap                                   |

Decimals: **9** (1 KVNC = 1 000 000 000 base units).

For legacy networks, the genesis validator allocation is minted **once** at genesis: 1 000 000 KVNC are
split evenly across the genesis validator set. Each validator's share backs its
`MIN_VALIDATOR_STAKE` (50 000 KVNC) self-stake; the remainder is liquid balance.
See §7.2 for the deterministic split rule.

For the NO-stake hybrid networks (`NoStake82M`, chains 1 and 2), zero tokens are minted for validators at genesis (`GENESIS_VALIDATOR_ALLOCATION_NO_STAKE = 0`).
All 82,000,000 KVNC are emitted solely through block rewards according to the 82M subsidy schedule. The genesis bootstrap seeds hold no stake and receive no allocation.

---

## 2. Block Reward Schedule

### 2.1 Legacy policy (existing networks)

- **Initial reward:** 2.5 KVNC  
- **Decay:** × ¾ every **8 100 000** committed leader blocks (one era)  
- Reward is paid **only** to the author of a **committed leader block** in the DAG.
- Round length is **4 s**; with a wave of 3, one leader commits every **12 s**
  (`LEADER_INTERVAL_SECS = 12`). One era is therefore ≈ **3.08 years**.
- The geometric series sums to `s₀ × era × 1/(1 − ¾) = 2.5 × 8 100 000 × 4 = 81 M`.

### 2.2 Approved NoStake82M Policy (NO-stake hybrid networks, Toni 2026-11-06)

- **Scope & Gating:** Required on both NO-stake hybrid chains — Mainnet (Chain 1) and Testnet (Chain 2). The node refuses to start a hybrid chain with any other emission policy (fail closed). Non-hybrid dev chains (3, 1337) may still use `Legacy81M`.
- **Initial reward:** 2.5 KVNC (`2_500_000_000` base units).
- **Era length:** **8 200 000** rewarded blocks (`SUBSIDY_ERA_BLOCKS_NO_STAKE_82M`).
  The first rewarded block is defined as index 0: indices `0..8_199_999` use the initial reward, index `8_200_000` starts era 1.
- **Decay:** × ¾ every 8,200,000 rewarded blocks, evaluated using exact integer arithmetic:
  `reward = reward * 3 / 4` (no floating point).
- **Terminal Remainder Payout:**
  Integer division rounding causes scheduled block rewards to reach zero at era 73 (rewarded block index 598,600,000).
  At that exact block, total scheduled issuance across eras 0..72 equals `81_999_999_147_200_000` base units.
  The exact remaining mining budget of **852 800 000 base units** (0.8528 KVNC) is paid out ONCE to the next eligible rewarded block.
  Thereafter, block rewards mint exactly zero.
  - Scheduled issuance (eras 0..72): `81_999_999_147_200_000` base units
  - Terminal remainder (era 73, once): `852_800_000` base units
  - Hard mining cap: `82_000_000_000_000_000` base units = **82 000 000 KVNC** exactly.
- **Sub-DAG Multi-Block Counting:**
  In GHOSTDAG hybrid PoW, each committed sub-DAG can contain multiple blocks. Every block in a committed sub-DAG is rewarded in canonical consensus topological order. The persistent rewarded block counter (`rewarded_block_count`) increments per block, not per leader height.
- **Activation & Safe Migration:**
  - Persisted in storage tables `EMISSION_POLICY` and `MINING_REWARD_STATE`.
  - Configured via `genesis_validators.toml` (`[genesis] policy = "no-stake-82m"`), CLI (`--emission-policy no-stake-82m`), or environment variable (`KOVANICA_EMISSION_POLICY=no-stake-82m`).
  - Existing databases without an explicit policy table default to `Legacy81M`. On a hybrid chain (1, 2) that default fails closed at startup, so the policy must be set explicitly; on dev chains (3, 1337) the legacy default is kept, preventing any accidental reinterpretation of existing balances.

---

## 3. Reward Distribution (DAG binding)

In the Mysticeti-style DAG:

1. Consensus produces a `CommittedSubDag` containing a **leader block**.
2. The execution layer calls `StakingState::on_leader_committed(leader_author)`.
3. The current `committed_leader_height` is used to look up `block_reward(height)`.
4. The reward is credited to the **payout address** of the validator that authored the leader.
5. `committed_leader_height` is incremented and treasury vesting is advanced.

This guarantees that only **finalized, ordered leader blocks** mint new tokens.

Light-node share (`LIGHT_REWARD_BPS = 2 000`, 20 %): it is escrowed per
**light-set epoch** and settled once per epoch, i.e. every **2 400 committed
leaders ≈ 8 h** (`EPOCH_LEADER_HEIGHTS`, `epoch_for_height`; Toni 2026-10-23).
The beacon for an epoch is taken at its first height. A light node that
registers becomes active in the next light-set epoch (at most ≈ 8 h later).

---

## 4. Treasury Vesting

- Total: **8 000 000 KVNC**
- Schedule: **1 000 000 KVNC per year** for 8 years
- Conversion: `BLOCKS_PER_YEAR = 2 628 000` (≈ 1 committed leader / 12 s)
- Vesting is advanced automatically on every committed leader.
- The treasury address can `claim()` only the already-vested portion.

```text
year 0          year 1          year 2          …          year 8
|---------------|---------------|---------------|-----------|
0 KVNC          1 M             2 M             …          8 M (capped)
```

---

## 5. Staking Parameters

| Parameter                | Value                  |
|--------------------------|------------------------|
| Active validators        | 15 – 21                |
| Min stake to join        | 50 000 KVNC            |
| Unbonding period         | 100 000 committed leader blocks (~13.9 days at 12 s/leader) |
| Commission               | set by validator (bps) |
| Commission change cooldown | 7 200 committed leader blocks (≈ 1 day = 3 epochs of 8 h) |
| Slash jail               | 7 200 committed leader blocks (≈ 1 day = 3 epochs of 8 h, nominal) |

Rewards go to the validator’s configured `payout_address` (defaults to the staking address). Future versions may support auto-compounding or commission splits with delegators.

---

## 6. Circulating Supply Formula

```
circulating ≈ FOUNDER_PREMINE
            + GENESIS_VALIDATOR_ALLOCATION
            + total_mining_issued
            + treasury.vested
```

Capped at `TOTAL_SUPPLY`. For the NO-stake hybrid chains (1, 2), `GENESIS_VALIDATOR_ALLOCATION = 0`, so genesis circulating supply is 200 000 KVNC (founder premine only). For legacy chains (3, 1337), genesis circulating supply is 1 200 000 KVNC (premine + genesis allocation). Treasury vests over time.

---

## 7. Implementation Locations

| Component                    | Crate / Module                          |
|-----------------------------|-----------------------------------------|
| Constants & emission math   | `kovanica-params`, `kovanica-staking`   |
| Types & Policy Definitions  | `kovanica-types::emission`              |
| Storage & Persistence       | `kovanica-storage::StateStore`          |
| Execution & Sub-DAG Rewards | `kovanica-execution::ExecutionContext`  |
| Node Config & Genesis Init  | `kovanica-node`                         |
| Consensus & CommittedSubDag | `kovanica-consensus`                    |

### 7.1 Canonical constants (`crates/kovanica-params/src/lib.rs`)

| Constant | Canonical value |
|----------|-----------------|
| `DECIMALS` / `ONE_KUNA` | 9 decimals, 1 KVNC = 1 000 000 000 base units |
| `TOTAL_SUPPLY` | 90 200 000 KVNC (hard cap) |
| `FOUNDER_PREMINE` | 200 000 KVNC |
| `GENESIS_VALIDATOR_ALLOCATION` | 1 000 000 KVNC (legacy) |
| `GENESIS_VALIDATOR_ALLOCATION_NO_STAKE` | 0 KVNC (new NO-stake) |
| `TREASURY_TOTAL` / `TREASURY_ANNUAL` / `TREASURY_YEARS` | 8 000 000 KVNC / 1 000 000 KVNC per year / 8 years (linear) |
| `MINING_SUBSIDY_BUDGET` | 81 000 000 KVNC (legacy) |
| `MINING_SUBSIDY_BUDGET_NO_STAKE_82M` | 82 000 000 KVNC (new NO-stake) |
| `INITIAL_BLOCK_REWARD` (s₀) | 2.5 KVNC per block |
| `SUBSIDY_ERA_BLOCKS` | 8 100 000 blocks per era (legacy) |
| `SUBSIDY_ERA_BLOCKS_NO_STAKE_82M` | 8 200 000 blocks per era (new NO-stake) |
| `DECAY_NUM` / `DECAY_DEN` | × ¾ per era |
| `LEADER_INTERVAL_SECS` | 12 (4 s round × wave 3) |
| `BLOCKS_PER_YEAR` | 2 628 000 (treasury vesting conversion only) |
| `EPOCH_ROUNDS` | epoch = 8 h = 7200 DAG rounds = 2400 leader heights |
| `MIN_VALIDATOR_STAKE` | 50 000 KVNC |
| `MIN_ACTIVE_VALIDATORS` / `MAX_ACTIVE_VALIDATORS` | 15 / 21 |
| `UNBONDING_ROUNDS` | 100 000 committed leader blocks (~13.9 days at 12 s/leader) |

### 7.2 Genesis validator allocation split (Legacy Networks)

`genesis_validator_shares(n)` splits `GENESIS_VALIDATOR_ALLOCATION` across `n`
genesis validators as evenly as integer atoms allow:

- `base = ALLOCATION / n`, `remainder = ALLOCATION % n`
- the first `remainder` validators (sorted ascending by address) receive `base + 1`
- the shares sum to **exactly** `GENESIS_VALIDATOR_ALLOCATION`

Under the `NoStake82M` policy, this allocation is zero and no validator shares are created.

---

## 8. Future Governance Hooks

- Change `BLOCKS_PER_YEAR` (affects vesting speed)
- Adjust min stake / max validators (on legacy/staking chains)
- Activate residual fee-burn or fee-to-miners mechanisms once mining subsidy is exhausted

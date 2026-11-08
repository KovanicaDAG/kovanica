# Kovanica Staking & Delegation Guide

A comprehensive manual for token holders (delegators) and node operators (validators) participating in Kovanica Proof-of-Stake consensus, delegation, reward distribution, and slashing protection.

---

## 1. Introduction & Staking Mechanics

Kovanica uses high-throughput, uncertified DAG consensus (Mysticeti-style wave consensus) paired with an on-chain Proof-of-Stake (PoS) validator committee. Validators order DAG transactions, produce committed leader blocks, and secure the network. Token holders who do not run node infrastructure can delegate their stake to active validators to earn a share of block rewards.

### 1.1 Core Protocol Parameters

| Parameter | Value | Description |
|---|---|---|
| **Currency Ticker** | **KUNA** | Native token ticker (9 decimals: $1\text{ KUNA} = 10^9\text{ atoms}$) |
| **Address Format** | **PRO-21 base32** | 64 characters: `kvnc` + 57 base32 chars + `dag` |
| **Minimum Validator Stake** | **50,000 KUNA** | `MIN_VALIDATOR_STAKE` required to join the active committee |
| **Minimum Delegation** | **50,000 KUNA** | `MIN_DELEGATION` minimum stake per delegation transaction |
| **Epoch Duration** | **7,200 DAG rounds** | `EPOCH_ROUNDS` ($\approx 8\text{ hours}$ at 4 s/round = 2,400 leader heights) |
| **Unbonding Cooldown** | **100,000 rounds** | `UNBONDING_ROUNDS` committed leader blocks ($\approx 13.9\text{ days}$) |
| **Slashing Penalty** | **500 bps (5%)** | `SLASH_PCT_BPS` slashed on verifiable equivocation evidence |
| **Jail Duration** | **7,200 rounds** | `JAIL_ROUNDS` committed leader heights ($\approx 1\text{ day}$ = 3 epochs) |
| **Initial Block Reward** | **2.5 KUNA** | `INITIAL_BLOCK_REWARD` minted per committed leader block |

---

## 2. Role 1: Delegator Guide

Delegating allows any KUNA holder to participate in network security and earn rewards without managing validator infrastructure.

### 2.1 How to Choose a Validator

When choosing a validator to delegate your stake to, evaluate the following factors:

1. **Commission Rate (`commission_bps`)**:
   - The fee the validator retains from block rewards before distributing the remainder proportionally to delegators.
   - Example: 500 bps = 5% fee (delegators receive 95% of earned rewards).
2. **Uptime & Online Status (`last_online_height`)**:
   - Validators who consistently participate in DAG rounds and submit votes earn block rewards reliably.
   - Validators that stay offline risk missing leader blocks and eviction from the active set.
3. **Total Stake & Committee Position**:
   - A validator's total stake equals their **self-stake plus all delegated stake**.
   - Total stake determines leader election probability and voting weight.

### 2.2 Step-by-Step Delegator CLI Instructions

All commands are executed using the official `kovanica` CLI.

#### Step 1: Check Your Account Balance

Verify you hold sufficient KUNA for the delegation amount plus transaction fee buffer:

```bash
kovanica balance kvnc1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdag
```

Output:
```text
Address:  kvnc1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdag
Balance:  65,000.000000000 KUNA (65000000000000 atoms)
```

#### Step 2: Delegate Stake to a Validator

Delegate at least `50,000 KUNA` to your chosen validator address:

```bash
kovanica delegate \
  --validator kvnc1p23456789abcdefghijklmnopqrstuvwxyz0123456789abcdefghijkldag \
  --amount 50000KUNA \
  --keystore ~/.kovanica/keystore.json \
  --chain-id 1
```

> [!NOTE]
> `--amount` accepts units such as `50000KUNA` or `50000000000000atoms`. Bare integers without units are rejected to prevent unit ambiguity.

#### Step 3: Check Delegations and Staking Status

Inspect active self-stakes, delegations, and unclaimed rewards:

```bash
kovanica staking --address kvnc1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdag
```

Output display:
```text
STAKE DETAILS
Address:           kvnc1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdag
Own Stake:         0.000000000 KUNA
Delegated Stake:   50,000.000000000 KUNA
Unclaimed Rewards: 12.500000000 KUNA

DELEGATIONS
VALIDATOR                                                        AMOUNT
---------------------------------------------------------------  -------------------
kvnc1p23456789abcdefghijklmnopqrstuvwxyz0123456789abcdefghijkldag  50,000.000000000 KUNA
```

#### Step 4: Claim Accumulated Staking Rewards

Claim your earned rewards from all delegated validators:

```bash
kovanica claim-rewards \
  --keystore ~/.kovanica/keystore.json \
  --chain-id 1
```

To claim from a specific validator only:

```bash
kovanica claim-rewards \
  --validator kvnc1p23456789abcdefghijklmnopqrstuvwxyz0123456789abcdefghijkldag \
  --keystore ~/.kovanica/keystore.json \
  --chain-id 1
```

### 2.3 Unbonding Process & Timeline

- If you wish to withdraw your delegated stake, an unbonding cooldown period of **100,000 committed leader blocks** (`UNBONDING_ROUNDS`, $\approx 13.9$ days) applies.
- During unbonding, funds remain in the unbonding queue and do not earn rewards.
- Once `UNBONDING_ROUNDS` elapse, the bonded tokens are released back to your available balance.

---

## 3. Role 2: Validator Guide

Validators maintain node infrastructure, participate in DAG leader proposals and consensus voting, and secure the network.

### 3.1 Minimum Requirements & Node Setup

- **Hardware Target**: 2–4 CPU cores, 2–4 GB RAM, 50 GB NVMe SSD, low-latency network connection.
- **Minimum Self-Stake**: `MIN_VALIDATOR_STAKE = 50,000 KUNA`.
- **Ports**: P2P (8000), JSON-RPC (8545), WebSocket (8546), Metrics (9100).

#### Step 1: Generate Validator Keypair & Keystore

```bash
kovanica keygen --out validator-keystore.json
```

Derive the canonical address:
```bash
kovanica address --key-file validator-keystore.json
```

#### Step 2: Bond Validator Self-Stake

Bond at least `50,000 KUNA` to join the active committee:

```bash
kovanica stake \
  --amount 50000KUNA \
  --keystore validator-keystore.json \
  --chain-id 1
```

#### Step 3: Verify Committee Membership

Check node sync state and committee inclusion:

```bash
kovanica node sync
kovanica status
```

In the status table, verify that your validator address is listed in the `COMMITTEE` table with `STAKE >= 50,000.000000000 KUNA`.

### 3.2 Setting Validator Commission

Validators can set a commission fee between 0% and 100%, specified in basis points (`10000 bps = 100%`, `100 bps = 1%`).

```bash
kovanica set-commission \
  --validator kvnc1v890abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopdag \
  --commission-bps 500 \
  --keystore validator-keystore.json \
  --chain-id 1
```

> [!IMPORTANT]
> A commission change cooldown of **7,200 committed leader blocks** (`COMMISSION_CHANGE_COOLDOWN`, $\approx 1\text{ day}$) prevents erratic commission spikes and protects delegators.

### 3.3 Slashing Penalties & Equivocation Evidence

Kovanica enforces strict safety penalties through on-chain equivocation evidence (RFC-010):

1. **Equivocation Trigger**:
   - Producing conflicting block headers at the same round.
   - Dual-voting for conflicting leader blocks.
2. **Slashing Impact**:
   - **5% penalty** (`SLASH_PCT_BPS = 500`) deducted immediately from the validator's self-stake and delegated stake.
   - Pending unbonding entries are subject to the same 5% cut.
3. **Jail State**:
   - The slashed validator is immediately deactivated and placed in jail for **7,200 committed leader heights** (`JAIL_ROUNDS`, $\approx 1$ day = 3 epochs).
   - Once the jail duration expires, the validator is not automatically restored; an explicit stake transaction is required to reactivate.

---

## 4. FAQ & Troubleshooting

### Q1: Why did my delegation fail with "amount below minimum"?
Both validator self-stake and delegator stake enforce a hard minimum of **50,000 KUNA** (`MIN_VALIDATOR_STAKE = 50_000 KUNA`, `MIN_DELEGATION = 50_000 KUNA`). Any transaction attempting to stake or delegate fewer than 50,000 KUNA is rejected by the mempool and execution runtime.

### Q2: What address format should I use?
Always use canonical PRO-21 base32 addresses:
- Starts with `kvnc`
- Followed by exactly 57 lowercase base32 characters
- Ends with `dag`
- Total length: exactly 64 characters (e.g., `kvnc1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdag`).
Legacy hex addresses (`kvnc<72 hex>dag` or raw 32-byte hex) are strictly rejected on Mainnet (`chain_id = 1`) and Testnet (`chain_id = 2`).

### Q3: Why do I need a balance buffer for fees?
Transactions require a small fee paid in atoms (e.g. 1 to 100 atoms; default fee: 1 atom). If you delegate your entire balance without leaving atoms for fees, the transaction will fail with `InsufficientBalance`. Keep at least `0.01 KUNA` unbonded in your account for network fees.

### Q4: When do newly delegated stakes start earning rewards?
Committee rotation occurs at epoch boundaries every **7,200 DAG rounds** (`EPOCH_ROUNDS`, $\approx 8\text{ hours}$). Stakes delegated during the current epoch take effect in the next epoch committee snapshot.

### Q5: Can a delegator be slashed if their validator equivocates?
Yes. Staking is stake-weighted and consensus-aligned; if a validator commits a verifiable equivocation, the 5% slash applies to the validator's entire stake pool (self-stake and delegations). Choose reliable, well-maintained validators to minimize risk.

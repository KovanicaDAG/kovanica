# KVNC tokenomics (RFC-006)

Numbers match the RFC-006 reference implementation in `kovanica-state` and
`kovanica-node`. If code and this doc diverge, **code wins**.

**Network:** `kovanica-testnet` (mainnet profile filled but dormant).

> **Activation note:** Switching a live network to RFC-006 is a consensus fork
> and requires a **testnet reset**.

> **Consensus decision (ratified 2026-09-25): Kovanica is PoA-only.**
> Proof-of-Work is being **removed**, not merely disabled. Items marked
> `[TARGET]` are ratified but not yet implemented; `[CURRENT]` items describe
> shipped code. Policy lives in
> [`RFC-POA-Migration.md` §0](./RFC-POA-Migration.md).
>
> **Nothing in this document changes.** This is the load-bearing statement for
> the whole RFC: the PoA removal is **tokenomics-neutral**, and it is
> tokenomics-neutral for a specific reason, not by luck —
> - the emission curve is **height-indexed**, not work-indexed, so removing a
>   work target does not move a block's subsidy;
> - `cumulative_minted` is **capped in `apply_block`**, so MAX_SUPPLY holds
>   regardless of who is allowed to produce blocks.
>
> Every figure below therefore remains canonical and byte-identical after the
> transition: **MAX_SUPPLY 90.2M KVNC**, **s₀ 10 KVNC/block**, era **2 000 000
> blocks**, **α 3/4 per era**, **maturity 100 blocks**, fee **75% burned /
> 25% to producer**, 1 KVNC = **100_000_000** atoms, GHOSTDAG **k=3**. The
> RFC-006 curve totals (80M curve emission, 0.2M founder premine, 10M vested
> treasury) are equally unaffected.
>
> What *does* change is **block pace, not block totals**: authorities produce
> on a slot schedule (`KOVANICA_SLOT_DURATION`) rather than on mining luck, so
> the *rate at which* KVNC is emitted will differ. The *amount* per block, the
> cap, the maturity window and the fee split do not.
>
> `TODO/plans/RFC-008/` (hybrid PoW+PoS+VRF) is **SUPERSEDED**; the PoA
> transition supersedes it, and no emission parameter in it survives. Hybrid was
> subsequently **dropped entirely** (decided 2026-09-25, RFC-POA-Migration
> §0.7.1), so the stake registry retires with it. That is *also*
> tokenomics-neutral: bond/unbond moves value into and out of a frozen registry
> overlay, it does not mint, so removing it changes no supply figure.
>
> ⚠️ **Still true and worth restating:** under PoA there is **no** path — not
> permissionless, not stake-weighted — for a non-authority to produce a block
> and earn a subsidy (§0.5, §0.7.1). "Where does emission go" and "who is
> allowed to receive it" are now the same question.
>
> ⚠️ **One-way door:** removing PoW cannot be quietly reversed; restoring any
> permissionless admission path would itself be consensus-breaking
> (RFC-POA-Migration §0.1.1).

---

## Units

| Symbol | Meaning |
|--------|--------|
| **KVNC** | Native currency ticker |
| **atom** | Smallest unit |
| **Decimals** | 8 |
| **1 KVNC** | `100_000_000` atoms (`ATOM`) |

---

## Emission (smooth geometric curve)

| Parameter | Value |
|-----------|--------|
| Genesis subsidy \(s_0\) | **10 KVNC** per block |
| Era length \(E\) | **2_000_000** blocks |
| Decay \(\alpha\) | **3/4** per era (integer floor) |
| Curve total | **80_000_000 KVNC** |

```text
era = floor(height / 2_000_000)
s(era) = floor(s(era-1) * 3/4)   with s(0) = 10 KVNC
```

---

## Hard cap & distribution

| Component | Amount | Mechanism |
|-----------|--------|-----------|
| Founder premine | 0.2M KVNC | Genesis coinbase (P2PK) |
| Treasury | 10M KVNC | 10 × 1M RFC-005 vaults at genesis |
| Curve emission | 80M KVNC | Block subsidies |
| **MAX_SUPPLY** | **90.2M KVNC** | Enforced via `native_minted` |

Treasury tranche *k* (1..=10) unlocks at height `k * 31_536_000`.
Owner keys are **placeholders** (`TREASURY_SEED_BASE + k`) until ceremony.

---

## Coinbase maturity

| Parameter | Value |
|-----------|--------|
| `COINBASE_MATURITY` | **100** blocks |
| Error | `LedgerError::CoinbaseImmature` |

---

## Fees

| Parameter | Value |
|-----------|--------|
| Floor | `max(1, subsidy / 500_000)` atoms per byte |
| Producer share | **25%** (`fee / 4`) |
| Burned | **75%** |

---

## Supply metrics

| Metric | Definition |
|--------|------------|
| `total` / `native_minted` | cumulative minted |
| `circulating` | tip UTXO native total (approx.) |
| `burned` | cumulative 75% fee burn |
| `max_supply` | `MAX_SUPPLY` |

Exposed via `Ledger::supply()` and HTTP snapshot JSON fields.

---

## Consensus parameters (related)

| Parameter | Value | Status |
|-----------|--------|--------|
| GHOSTDAG **k** | 3 | `[CURRENT]` — unchanged by the PoA decision |
| PoW | real, opt-in | `[TARGET]`-for-removal (being deleted, not just off). Replaced by an authority set: `KOVANICA_CONSENSUS=poa` + `KOVANICA_AUTHORITIES` + `KOVANICA_AUTHORITY_THRESHOLD` + `KOVANICA_SLOT_DURATION`. See RFC-POA-Migration §0.2 |
| Difficulty / retarget | exists, `H*work < 2^256` | `[TARGET]`-for-removal — **there is no retune knob to mis-set**, so any pre-transition "don't retune the difficulty window" advice is void. Block pace becomes a fixed slot |
| Emission index | block **height** | `[CURRENT]` — *the reason tokenomics survives PoA removal* |

---

## References

- `crates/kovanica-state/src/ledger.rs`
- `crates/kovanica-node/src/explorer.rs` — `NetworkProfile`
- RFC-005 `VaultScript`

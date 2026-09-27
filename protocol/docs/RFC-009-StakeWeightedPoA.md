# RFC-009 — Stake-Weighted PoA: A Proposal, and a Contradiction That Must Be Resolved

This document does two things:

1. **Reports a contradiction** between the ratified canonical spec and the
   shipped code. The code has always contained a stake-weighted scheduling
   path; the spec deliberately removed stake weighting. One of the two is
   wrong, and that is a maintainer decision, not an authoring one.
2. **Proposes the stake-weighted schedule** in full normative detail, so that if
   the decision is to keep it, it is specified rather than accidental — and so
   that if the decision is to delete it, this document records what was there
   and why it was broken.

**Status:** Draft — **not canonical.** The canonical spec is
[`SW-PoA-SPV-CONSENSUS.md`](./SW-PoA-SPV-CONSENSUS.md) (KVP-201, ratified
2026-09-25). This RFC **contradicts** it and does not supersede it.
**KVP:** would be KVP-202 (Consensus) if ratified
**Consensus impact:** **consensus-breaking.** Both options in §2 change which
authority is admitted to produce a block, and the schedule is not committed
on chain, so divergence is silent.
**Reference implementation:** `crates/kovanica-dag/src/authority.rs`
(`build_schedule`, `active_authority`, `schedule_period`, `total_stake`,
`stake_merkle_root`, `stake_merkle_proof`)

> **Read §2 before anything else.** The framing of this document changed
> during drafting. An earlier version described the stake-weighted path as a
> feature that shipped ahead of its specification. That was wrong, and
> understating it would have made this look like a documentation chore rather
> than a fork risk.

---

## 1. The contradiction

`SW-PoA-SPV-CONSENSUS.md` is marked **Canonical (ratified 2026-09-25)** and
states, in three separate normative places, that scheduling is plain
round-robin:

> **§1 Authority Signatures are Binding** — "The scheduled authority is
> `authorities[slot % n]` over the canonical (sorted) key order"

> **One-Line Summary** — "no permissionless or stake-weighted admission of any
> kind"

> **SPV light client** — "2. Active authority = authorities[slot % n]"

Its canonical source, `RFC-POA-Migration.md`, is blunter and says it three more
times:

> "There is no permissionless path and no stake-weighted path."

> "§0.7.1 removes the stake-weighted one."

Stake weighting was therefore **removed on purpose**, alongside PoW and VRF,
under a "one-way door" rationale. The `HybridConfig` / `StakedVrf` surface was
deleted for exactly this reason.

Meanwhile `crates/kovanica-dag/src/authority.rs` has carried, since before the
PoA migration landed:

- `AuthoritySet.stakes: Option<Vec<u64>>`
- `new_with_stakes(..)` with stake validation
- an `active_authority` branch that schedules by stake when stakes are present
- `stake_merkle_root` / `stake_merkle_proof` and a `total_stake` in the SPV path

So the code **honours a stake vector that the spec says cannot exist**. The two
artefacts disagree, and the disagreement is not cosmetic: a `KVA1` Authority
UTXO carrying a stake vector would make nodes weight their schedule, which the
spec forbids.

### 1.1 Why this has not caused a live fork

Only because the path is unreachable in practice:

| Access route | Reachable? |
|---|---|
| `AuthoritySet::new` | **No** — always passes `stakes: None` |
| Environment variable | **No** — no such variable exists |
| CLI / RPC | **No** |
| `AuthoritySet::from_bytes` | **Yes** — on-chain `KVA1` Authority UTXO data |

The live testnet runs classic equal-weight PoA with genesis
`93efd2d7…`, configured through `KOVANICA_AUTHORITIES` + `authorities.conf`.
No stake vector exists anywhere on it, so no node has ever disagreed with
another about a slot.

**This is luck, not design.** A staked `KVA1` output is one `AuthorityUpdateTx`
away, and the failure would be silent: nodes agree on the Authority UTXO and on
its BLAKE3 hash, then disagree about who may produce. There is no version byte
and no format bump to catch it.

---

## 2. Options

### Option A — delete the staked path (conforms to the ratified spec)

`new_with_stakes` rejects any stake vector with a new
`AuthorityError::StakesNotPermitted`; `from_bytes` treats a stake-bearing
encoding as `MalformedEncoding`; `stakes()`, `total_stake()`'s stake branch,
`stake_merkle_root`, `stake_merkle_proof` and the SW-PoA metrics are deleted.

- **Consensus impact: none for existing chains.** Classic PoA scheduling is
  untouched and byte-identical. The change only makes an already-invalid input
  fail loudly.
- **Removes the silent-fork vector entirely.**
- Costs the weight data model. §5's Merkle proof work would be discarded.
- Honest about the current design intent: the spec says PoW, VRF and
  stake-weighting were all removed as a package.

### Option B — ratify stake weighting (amends the ratified spec)

Adopt §4 as the normative slot rule, and amend `SW-PoA-SPV-CONSENSUS.md` §1
and its One-Line Summary, plus the SPV step "Active authority =
`authorities[slot % n]`" in all three documents.

- **Consensus impact: consensus-breaking.** Block admission changes.
- Requires a **hard fork and a genesis reset**, on the same reasoning the PoA
  migration itself used (§0.6 of `RFC-POA-Migration.md`): the PoA genesis
  commits the authority set, so a staked set produces a different chain.
- Partially reverses a deliberate "one-way door". That is a legitimate thing to
  do, but it should be a conscious reversal with the rationale recorded, not a
  side effect of a feature that was already in the tree.
- Requires the activation discipline in §7. The absence of a committed
  schedule makes this materially harder to roll out safely than a normal fork.

### Recommendation

**Option A, now.** It is a small change, it conforms to the ratified decision,
it closes a silent-fork vector, and it costs nothing that any live chain uses.

Option B remains a reasonable feature — stake weighting is a real product
idea — but it should be a deliberate amendment with the fork and reset planned
up front, not a ratification of code that arrived unnoticed. If B is chosen,
keep §4: it is a sound schedule, and specifying it now is most of the work.

The rest of this document specifies §4 for that purpose. **If A is chosen,
§4 becomes historical record and the tests in §8 should be deleted with the
code they cover.**

---

## 3. Motivation: the defect, had it ever been reached

The first implementation of the staked branch was:

```rust
if let Some(stakes) = &self.stakes {
    let total: u64 = stakes.iter().sum();
    let target = (slot as u128 * total as u128 % total as u128) as u64;
    let mut acc = 0u64;
    for (i, &stake) in stakes.iter().enumerate() {
        acc += stake;
        if target < acc {
            return &self.authorities[i];
        }
    }
    &self.authorities[0]
}
```

`x mod x` is `0` for every `x`, so `target` was **always 0**. Construction
rejects zero stakes, so the first iteration always matched and the function
**always returned `authorities[0]`**.

Stakes `[70, 20, 10]` over 30 slots, against the proportional expectation of
`[21, 6, 3]`:

| | slot allocations |
|---|---|
| as shipped | `[30, 0, 0]` |
| intended | `[21, 6, 3]` |

The two unselected authorities are **permanently starved**. Under
`verify_slot_signature`, a block is checked against `active_authority(slot)`, so
any block they *did* produce would be rejected as
`DagError::InvalidAuthoritySignature`. At `t = 2` of `3` the chain **halts** —
not merely degraded, but unable to produce a block the others will accept,
because two of three authorities are permanently locked out.

**Why it survived.** No test constructed a staked set and compared slot
allocation against stake share. The defect was in a path that the spec says
should not exist, that config cannot reach, and that had no allocation
invariant. Two of the three conditions were accidents, and the third was luck.
The generalisable lesson is in §9.

This is fixed in `ae487de`, and the fix is specified in §4. It remains latent
only because of §1.1.

---

## 4. Specification (Option B)

### 4.1 Data model

A set is *staked* when it carries a stake vector. Canonical encoding
(KVP-201, unchanged):

```
threshold  u64 LE
count      u64 LE
pk_1 … pk_n    32 bytes each, ascending by 32-byte encoding
[stake_1 … stake_n]  u64 LE each, positionally matching pk_i
```

`hash` is BLAKE3 over exactly those bytes, and is what the on-chain `KVA1`
Authority UTXO commits. A set is staked iff its encoding exceeds
`16 + 32n` bytes; `from_bytes` infers this from length. Stakes are bound
positionally to their key and then sorted together, so `authorities()[i]`
always pairs with `stakes()[i]` **after** canonical sorting — never with the
i-th value the operator supplied.

Invariants enforced by `new_with_stakes`:

| Invariant | Error |
|---|---|
| `3 ≤ n ≤ 16` (`MIN_AUTHORITIES`/`MAX_AUTHORITIES`) | `InvalidAuthorityCount` |
| `2 ≤ t ≤ n` (`MIN_THRESHOLD`) | `InvalidThreshold` |
| keys pairwise distinct | `DuplicateAuthority` |
| every stake `> 0` | `InvalidAuthorityCount` |
| stake sum representable in `u64` | `StakeOverflow` |
| gcd-normalised period `≤ MAX_SCHEDULE_PERIOD` | `SchedulePeriodTooLarge` |

Zero stakes are rejected. That is what turned §3 from "wrong" into "chain
halting": a zero stake would have been the natural encoding for "authority
present but never scheduled", and forbidding it forces the weighting to be
total.

### 4.2 The schedule

With `stakes` the canonical vector, `g = gcd(stakes)`:

```
w_i    = stakes_i / g        (minimal integer weights)
period = Σ w_i                (one full weighting cycle, in slots)
```

`active_authority(s) = authorities[table[s mod period]]`.

Dividing by the gcd makes the schedule depend on stake **ratios only**.
`[70, 20, 10]`, `[7, 2, 1]` and `[70e8, 20e8, 10e8]` yield the identical
10-slot table. Operators may denominate stake in atoms, KVNC or anything else
without forking — which matters because RFC-006 fixes KVNC = 10^8 atoms and an
operator picking the wrong unit must not be consensus-relevant.

The table is **smooth weighted round-robin** (SWRR). With `cur[0..n] = 0` and
`total = period`, repeat `period` times:

```
1. for each i:  cur[i] += w_i
2. let j = argmax cur[i]        (ties → HIGHEST index, §4.3)
3. cur[j] -= total
4. append j to table
```

SWRR is the only cheap scheme giving both required properties:

- **Exactly proportional.** Over one period, authority `i` appears exactly
  `w_i` times. Not approximately.
- **Evenly spread.** The gap between consecutive selections of one authority
  stays within 2 slots of the ideal `period / w_i`.

Alternatives considered and rejected:

- *Contiguous cumulative intervals* (walk `slot mod total` against cumulative
  sums) is proportional but hands a 70 % authority **70 consecutive slots**. If
  it halts, the chain stalls for 70 slots. Unacceptable.
- *Stateless hashing / VRF selection* is well spread but not exactly
  proportional, and unpredictable to operators.
- *Per-slot SWRR replay* keeps the properties but is O(period) on the
  validation hot path, so it needs a precomputed table regardless.

### 4.3 The tie-break

Step 2 must resolve equal scores to the **highest** index. In Rust that is what
`Iterator::max_by_key` does — it returns the **last** maximum, not the first.
An earlier version of the code comment claimed the opposite; the behaviour was
deterministic and so not itself a bug, but a wrong normative statement beside
consensus code is a defect, and both the comment and the pinned test now state
the real rule.

The choice is arbitrary. It would be consensus-critical: two nodes with
different tie-breaks reject each other's valid blocks. The table is therefore
pinned by `sw_poa_schedule_table_is_pinned`.

### 4.4 Worked vectors

Normative, and directly testable — `build_schedule` is pure and takes stakes in
the order given.

| stakes | gcd | weights | period | table | counts |
|---|---|---|---|---|---|
| `[70, 20, 10]` | 10 | `[7, 2, 1]` | 10 | `[0, 1, 0, 0, 2, 0, 0, 1, 0, 0]` | `[7, 2, 1]` |
| `[1, 1, 1]` | 1 | `[1, 1, 1]` | 3 | `[2, 1, 0]` | `[1, 1, 1]` |
| `[60, 30, 10]` | 10 | `[6, 3, 1]` | 10 | `[0, 1, 0, 2, 0, 1, 0, 0, 1, 0]` | `[6, 3, 1]` |
| `[1, 2]` | 1 | `[1, 2]` | 3 | `[1, 0, 1]` | `[1, 2]` |
| `[5, 3, 2]` | 1 | `[5, 3, 2]` | 10 | `[0, 1, 2, 0, 1, 0, 0, 2, 1, 0]` | `[5, 3, 2]` |

`[1, 1, 1]` degenerates to round-robin in canonical order, as it must.
`[1, 2]` is the smallest case exercising the tie-break.

### 4.5 Classic PoA is unchanged

A set with no stakes keeps KVP-201's rule verbatim: `authorities[slot mod n]`,
guarded so the staked branch is unreachable. Asserted by
`classic_poa_keeps_plain_round_robin`.

### 4.6 Work bounds

`MAX_SCHEDULE_PERIOD = 65_536` slots. The table is at most 128 KiB (`Vec<u16>`)
and is built once per authority set.

Oversized sets are **rejected at construction, never truncated.** A silently
shortened period would change the slot→authority mapping while looking like a
successful load. The bound is reachable: co-prime stakes such as
`[65537, 65539]` have a smallest period of 131 076 and are refused. Without it,
an attacker able to mint a `KVA1` Authority UTXO could force arbitrarily large
allocations on every node, rebuilt on every authority-set update.

`total_stake()` uses a saturating fold. Construction rejects overflow, so the
saturation is defence-in-depth for a value no longer reachable through the
public constructors.

---

## 5. Stake proofs

`stake_merkle_root()` is BLAKE3 of a Merkle tree over leaves
`BLAKE3(pk_i ‖ stake_i.to_le_bytes())` in canonical order.
`stake_merkle_proof(pk)` returns one authority's path, letting a light client
learn a single weight without the whole vector.

This is a **commitment** mechanism, not proof of ownership: it shows "this
weight is in the set", not "this signer holds this stake". Binding weight to
signatures and to anything slashable is out of scope and belongs with the
slashing RFC.

---

## 6. Backwards compatibility

- **Classic PoA chains are byte-identical.** No set identity changes, no
  schedule serialisation is added; `canonical_bytes_of` and `to_bytes` are
  untouched.
- **No re-commitment is possible or required.** The `KVA1` commitment is BLAKE3
  of the canonical encoding, which excludes the schedule. A node on this code
  and a node on pre-`ae487de` code agree on set identity and disagree only on
  scheduling **for staked sets**. Were one to exist they would fork at the
  first staked slot — which is precisely why §7 forbids staked sets until every
  node is upgraded.
- **SPV.** `spv.rs` already compares `sw_poa_proof.stake_root` against
  `stake_merkle_root()` and `sw_poa_proof.total_stake` against `total_stake()`.
  Neither is affected by the schedule.
- **Ledger and economics.** Untouched. No UTXO, script, emission, maturity or
  fee-burn rule changes. RFC-006 constants are unaffected: MAX_SUPPLY 90.2M
  KVNC, s₀ 10 KVNC/block, era 2 000 000, α 3/4, maturity 100 blocks, fee split
  75 % burned / 25 % producer. *Who* receives the subsidy becomes a function of
  the schedule; for classic PoA it is unchanged.

---

## 7. Activation (required only for Option B)

**Staked authority sets must not appear on a public network until this is
ratified and every node implementation is upgraded.**

Because the schedule is not committed on chain (§6), a staked set is a
**silent** fork: nodes agree on the Authority UTXO and its hash, then disagree
about who may produce. There is no version byte to gate on. The only defence is
operational ordering.

1. Ratify. Fix the tie-break and `MAX_SCHEDULE_PERIOD` as permanent consensus
   constants, not implementation details.
2. Update **all** validators and the light-client fleet. Confirm via
   `kovanica_poa_schedule_period` that every node reads `0` (classic).
3. Only then permit a `KVA1` Authority UTXO with a stake vector.
4. Keep the path **unreachable from configuration** until step 3 completes. No
   `KOVANICA_AUTHORITY_STAKES`-style variable ships here: it turns an operator
   typo into a fork, and configuration is the least trustworthy input in the
   system — the exact failure mode KVP-201's canonical-ordering work exists to
   prevent.
5. Genesis reset per §0.6 of `RFC-POA-Migration.md`, since the PoA genesis
   commits the authority set.

Testnet resets are cheap (`docs/TESTNET-RESET-POLICY.md`), so a staked-set
experiment there is reasonable. Mainnet is not, and this is the only part of
this RFC that touches it.

---

## 8. Testing requirements

Applies to Option B. Under Option A these tests are deleted with the code.

| Test | Asserts |
|---|---|
| `sw_poa_schedule_table_is_pinned` | the exact §4.4 vectors, including the tie-break |
| `sw_poa_schedule_is_exactly_proportional` | counts over one period == normalised weights |
| `sw_poa_every_authority_gets_slots` | **direct regression for §3** — no authority starved |
| `sw_poa_schedule_normalises_stake_units` | a common stake factor does not change the table |
| `sw_poa_schedule_keeps_gaps_near_uniform` | gap within 2 of `period / weight` |
| `sw_poa_schedule_survives_serialisation_round_trip` | a config-built set and a `from_bytes` set schedule identically |
| `sw_poa_schedule_is_not_part_of_the_serialised_identity` | stake changes change the hash; the schedule does not appear in it |
| `sw_poa_rejects_an_oversized_schedule_period` | `[65537, 65539, 65533]` refused, not truncated |
| `sw_poa_rejects_a_stake_sum_overflow` | `[u64::MAX, 1, 1]` refused |
| `sw_poa_scheduled_owner_can_sign_its_slot` | end-to-end `verify_slot_signature` for a staked owner |

Two rules learned the hard way, recorded so they are not repeated:

- **Compare against `set.stakes()`, not the constructor literal.**
  `new_with_stakes` sorts keys *and* stakes together, so the canonical vector is
  a permutation of the input. The first version of this test asserted the wrong
  permutation and looked like a code bug.
- **"Never twice in a row" is not a property of SWRR.** A weighted schedule can
  give one authority consecutive slots. Assert the gap bound instead.

---

## 9. Process lessons

Three failures compounded here, and they are worth more than the schedule.

1. **A code path with no specification will be wrong.** §3's bug sat in a path
   no test exercised and no document described. There was no statement of the
   intended behaviour to violate, so nothing caught it.
2. **"Unreachable" was load-bearing and unverified.** The path was defended by
   three independent facts — no env var, no CLI, no config — all of which were
   true only by inspection, never by a test asserting the rejection. One
   `from_bytes` call away from production.
3. **A contradiction between spec and code is the finding.** The most valuable
   output of this exercise is not the schedule; it is §1. Two canonical
   documents in this repository disagree about whether stake weighting exists,
   and nobody noticed because a latent, unreachable, broken path looks exactly
   like dead code.

The generalisable check: **for every canonical invariant, assert that the
non-conforming case is rejected.** `sw_poa_every_authority_gets_slots` asserts
the schedule works; what was missing was a test asserting the *spec's* rule —
`authorities[slot % n]` — holds for every set a node can actually load. That
test is the one that would have surfaced §1, and it belongs in the tree
regardless of which option is chosen.

---

## 10. Observability

`crates/kovanica-node/src/metrics.rs` exposes
`kovanica_poa_sw_poa_enabled`, `kovanica_poa_schedule_period`,
`kovanica_poa_total_stake`, `kovanica_poa_authority_stake{authority=<hex>}`,
`kovanica_poa_authority_share_bps{authority=<hex>}` and
`kovanica_poa_slots_{scheduled,produced,missed}_total{authority=<hex>}`.

`record_poa_authority_set` returns before the per-authority loop when the set
carries no stakes, so **classic PoA emits no stake series** — a dashboard must
not infer weighting that is not in effect. This is also the cheapest available
detector for §1: `schedule_period` must read `0` on every node, and under
Option A it must read `0` forever.

---

## 11. Open questions

1. **Option A or B?** Blocking. Everything else here is conditional on it.
2. **Should the period be capped below 65 536?** The bound is a DoS guard, not
   a target. At the 3 000 ms slot duration, 65 536 slots is ~55 hours per full
   rotation; realistic weight vectors normalise to periods under 100.
3. **Should the schedule be committed on chain?** It would make staked-set
   divergence loud instead of silent, at the cost of a format bump. This
   document deliberately avoids that (§6, §7); a successor may reverse it.
4. **Stake-weighted threshold.** Should `t` become a fraction of total stake
   rather than a count of signers? Deferred; it interacts with slashing.
5. **Minimum stake / diversification.** Nothing stops one authority holding
   100 % of stake and therefore 100 % of slots, which collapses PoA to a single
   signer and defeats the threshold entirely. A cap, or a minimum number of
   distinct stakes, is probably required before this is safe on a public
   network. **Open and material.**

---

## 12. Changelog

| Version | Change |
|---|---|
| Draft 1 | Initial draft. Framed as a specification that shipped after its code. |
| Draft 2 | **Corrected.** Found `SW-PoA-SPV-CONSENSUS.md` (canonical, ratified 2026-09-25) and `RFC-POA-Migration.md` §0.7.1, which both state stake weighting was deliberately removed. Reframed as a contradiction report plus an explicit Option A / Option B decision, with A recommended. Downgraded all code comments from "normative" to "proposed". |

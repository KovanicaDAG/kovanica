# RFC-008 — Hybrid PoW/PoS Admission with VRF + Light-Client (SPV) Path

**Status:** Draft  
**Public name (proposed):** KVP-107 Hybrid Consensus  
**Depends on:** RFC-006 (tokenomics, MAX_SUPPLY, maturity), GHOSTDAG k=3, RFC-005 (vaults for stake)  
**Affects:** `kovanica-dag`, `kovanica-state`, `kovanica-node`, light-node / UniFFI, explorer, wallet  
**Consensus impact:** **Hard fork** (admission rules + header extension). Requires testnet reset or carefully gated activation height.

---

## 1. Motivation

Kovanica currently relies solely on Proof-of-Work for block admission and GHOSTDAG blue-work for chain selection. As subsidy decays (RFC-006 geometric curve), long-term security budget must be supplemented. Stage-3 vision (already noted in internal GHOSTDAG notes) is:

- Keep **GHOSTDAG blue-work** as the sole chain-selection signal.
- Add **VRF-based stake admission** so that staked blocks can be proposed without requiring expensive PoW.
- Enable **light-nodes / SPV-style clients** that verify the selected chain + eligibility proofs without a full DAG or full UTXO set.

Goals:

1. Hybrid security: PoW remains the ultimate work source; stake gates cheap admission.
2. Light-client friendliness: compact headers + VRF proofs + blue-score finality.
3. No inflation of blue weight by low-cost staked blocks (fixed nominal work).
4. Clean interaction with existing MAX_SUPPLY, coinbase maturity, and fee-burn rules.

Non-goals (this RFC):

- Replacing GHOSTDAG with pure PoS.
- Full account-model staking rewards distribution (can be later KVP).
- On-chain governance voting.

---

## 2. High-level design

### 2.1 Block types

| Type        | Admission requirement          | Work contribution to blue-score | Coinbase allowed |
|-------------|--------------------------------|---------------------------------|------------------|
| PoW block   | Valid PoW (current rules)      | Real PoW work                   | Yes              |
| Staked block| Valid VRF proof + sufficient stake | **Fixed nominal work** (constant) | Yes (same rules) |

Both types are full blocks in the DAG. GHOSTDAG ordering and blue/red classification remain unchanged. Only the *admission* gate and the *work value* assigned to a staked block differ.

### 2.2 Chain selection (unchanged principle)

```
selected_parent = GHOSTDAG(k=3) using blue-work
blue-work(staked_block) = FIXED_STAKE_WORK   // consensus constant
blue-work(pow_block)    = actual_pow_work
```

`FIXED_STAKE_WORK` must be chosen so that a realistic number of staked blocks cannot dominate a well-mined PoW tip. Exact value is a consensus parameter (see §6).

### 2.3 VRF sortition (eligibility)

For a given slot derived from blue-score:

```
slot          = blue_score_of_selected_parent / SLOT_LENGTH
seed          = H(selected_parent_header || epoch_randomness)
vrf_out       = VRF_prove(sk_validator, seed || slot)
eligible      = (vrf_out * total_active_stake) < (stake_of_validator * THRESHOLD)
```

- `VRF` is a verifiable random function (recommended: ECVRF-ED25519-SHA512-Elligator2 or equivalent that re-uses Ed25519 keys already in the protocol).
- `THRESHOLD` and `SLOT_LENGTH` are consensus constants.
- Only validators with locked stake (see §4) may produce a valid proof.

A staked block header **must** contain the VRF proof, the stake public key (or commitment), and the claimed slot. Full nodes reject the block if the proof does not verify or the validator is not eligible for that slot.

### 2.4 Light-node / SPV path

A light-node needs only:

1. Selected-parent header chain (or a compact proof of the current tip’s ancestry).
2. Blue-score / selected-chain depth.
3. VRF eligibility proofs for recent staked blocks (optional for pure PoW tips).
4. (Future) UTXO commitment or sparse-Merkle root if transaction inclusion proofs are required.

Finality signal for wallets: **blue-score advance** of N windows (same recommendation as current GHOSTDAG notes), not raw height.

---

## 3. Header extension (consensus-critical)

Current header fields remain. New optional / versioned fields (activated at fork height):

```rust
// Conceptual – exact serialization in technical design
struct BlockHeaderV2 {
    // … existing fields …
    version: u32,                    // bump for hybrid
    // Staked-block only (absent or zero for pure PoW)
    stake_pubkey: Option<[u8; 32]>,  // Ed25519
    vrf_proof: Option<VrfProof>,     // variable or fixed-size
    slot: Option<u64>,
    // Optional for light clients
    utxo_commitment: Option<[u8; 32]>, // future
}
```

Validation rules:

- If `vrf_proof` is present → treat as staked block → enforce eligibility + assign `FIXED_STAKE_WORK`.
- If absent → treat as PoW block → enforce current PoW difficulty / target.
- Mixed DAG is legal; GHOSTDAG runs on the resulting blue-work values.

---

## 4. Staking model (ledger-safe)

Stake is locked via existing **RFC-005 vault** machinery (or a thin staking script that re-uses CSV + time-lock semantics):

- Deposit: KVNC (or approved KVP-102 asset) into a staking vault controlled by the validator’s Ed25519 key.
- Activation delay: stake becomes active after `STAKE_ACTIVATION_BLOCKS`.
- Unbonding: `UNBONDING_PERIOD` (must be ≥ practical reorg depth under k=3).
- Slashed stake: burned or sent to treasury vault on proven misbehaviour (double-sign, invalid VRF, etc.).

Exact script templates and slash conditions are specified in the companion technical design. This RFC only requires that active stake weight is deterministically computable from the UTXO set (or a committed stake root) at the selected parent.

---

## 5. Interaction with RFC-006 tokenomics

- Coinbase rules, `native_minted`, `MAX_SUPPLY`, 100-block maturity, and 75/25 fee burn apply **identically** to both PoW and staked blocks.
- A staked block that would push `native_minted` over the hard cap is rejected exactly as a PoW block would be.
- Security budget note: as subsidy → 0, the combination of remaining fees + economic value of stake (and future staking rewards if introduced) becomes the dominant incentive.

---

## 6. Consensus parameters (initial proposal)

| Parameter              | Suggested starting value          | Notes |
|------------------------|-----------------------------------|-------|
| `SLOT_LENGTH`          | 10 blue-score units               | Tunable |
| `FIXED_STAKE_WORK`     | Equivalent to ~1–5 % of current average PoW work | Must not let pure-stake tips overtake honest PoW |
| `THRESHOLD`            | 1 / expected_validators_per_slot  | Classic sortition |
| `STAKE_ACTIVATION_BLOCKS` | 100–500                        | After deposit |
| `UNBONDING_PERIOD`     | ≥ 2–4× practical finality depth   | k=3 aware |
| `MIN_STAKE`            | TBD (economic)                    | Prevent dust validators |

All values are consensus constants; changing them after activation requires another fork.

---

## 7. Activation & migration

- New consensus version / fork height (or genesis reset on testnet).
- Pre-activation: only PoW blocks valid.
- Post-activation: both types accepted under the rules above.
- Testnet **must** reset or clear all prior state that assumed pure-PoW headers (same discipline as RFC-006).
- Light-node clients must be updated before or at activation; old light clients that ignore the new fields will be unsafe.

---

## 8. Security considerations

- **Work inflation attack**: mitigated by fixed nominal work for staked blocks.
- **Stake grinding / adaptive VRF**: use proper ECVRF; seed must be unpredictable until the parent is fixed.
- **Nothing-at-stake**: GHOSTDAG + slashable double-sign + unbonding period.
- **Long-range attacks on light clients**: rely on weak subjectivity / checkpointing from trusted sources or social consensus for deep history (standard PoS light-client issue). Document clearly.
- **Centralisation**: monitor stake distribution; consider progressive thresholds or delegation limits in later KVPs.

---

## 9. Implementation sketch (Rust crates)

- `kovanica-vrf` — pure VRF prove/verify + test vectors (no DAG dependency).
- `kovanica-dag` — header version, admission, work assignment, GHOSTDAG unchanged.
- `kovanica-state` — stake weight calculation, vault recognition, slash.
- `kovanica-node` — mining / staking loops, P2P propagation of both block types.
- Light / UniFFI — header sync + VRF verify path.

---

## 10. Open questions (to resolve before Final)

1. Exact VRF construction and serialization size.
2. Whether stake root is committed in every header or only periodically.
3. Initial `FIXED_STAKE_WORK` calibration methodology (testnet experiment).
4. Whether staking rewards (beyond residual subsidy) are in-scope for this RFC or a follow-up.
5. Delegation / nominator model (yes/no for v1).

---

## 11. References

- Internal: `ghostdag-notes.md` (Stage-3 hybrid admission vision)
- RFC-005 — Time-lock vault + CSV
- RFC-006 — Tokenomics, MAX_SUPPLY, maturity, fee burn
- Classic literature: Ouroboros / Praos VRF, GHOSTDAG papers, Bitcoin SPV, weak subjectivity

---

**Authors / editors:** Kovanica core (draft for review)  
**Next step:** Technical design document + test vectors before any consensus code lands.

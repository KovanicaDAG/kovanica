# Hybrid PoW & Mining with a Phone as a Light Node — Research Compilation

> Compiled 2026-08-25 from `/root/kovanica-protocol` (live code, authoritative) and
> `KovanicaDAG/` vault snapshots. Workspace version at time of research: **v0.2.0**
> (Stage 3 close-out). All claims verified against source files listed in §10.

---

## 1. TL;DR

- **"Hybrid PoW"** in Kovanica means **hybrid PoW + VRF-staked block admission**: Nakamoto proof-of-work remains the chain-selection work source ("what wins"), while bonded validators win block-production slots via stake-weighted VRF sortition ("who may add") — Algorand/Praos-style. It is **not** classic PoS, merged mining, or auxpow.
- **Phones do not PoW-mine.** The staked path is explicitly designed as *"the phone-friendly production path (sign one VRF over the tip input; no mining rig)"*. A phone produces blocks by signing one ECVRF over the tip input; PoW remains available as fallback.
- Both features **shipped** in v0.2.0: stake registry + hybrid admission (slices 1–2), mobile FFI `LightNode` with Kotlin/Swift bindings (slices 3–8), SPV light sync, Android/iOS packaging, CI drift guard.

---

## 2. Terminology

| Term | Meaning in this project |
|---|---|
| Hybrid admission | Per-block choice between two admission paths: PoW or staked-VRF |
| Staked block | Block carrying `StakedVrf { vrf_pk, proof, output }`, work pinned to nominal |
| Bond / Unbond | Freeze/unfreeze UTXOs via tx tag conventions (`KVB1` / `KVU1`) — no new tx types |
| Sortition | Stake-weighted lottery comparing `VrfOutput::as_u64()` against an eligibility threshold |
| Light node | Phone running `kovanica-ffi::LightNode` — full validating wallet over byte-blob sync |
| SPV mode | Watch-only sync via headers + compact block filters (`KVLS` v1 blobs) |

Verified zero-match terms across both repos: `hibrid` (typo), `dual PoW`, `PoS`, `proof of stake`, `merged mining`, `auxpow`, `smartphone`, `low-power`. The concept exists solely under the name **"hybrid (PoW + VRF-staked)"**.

---

## 3. Hybrid PoW — Design & Implementation

### 3.1 Design rationale (verbatim, KovanicaDAG/AGENTS.md)

> "keeps PoW as the chain-selection work source while letting a bonded validator win slots by stake-weighted sortition — **the phone-friendly production path (sign one VRF over the tip input; no mining rig)**. Kaspa-style mergeability is untouched; Algorand/Praos-style eligibility decides *who may add*, GHOSTDAG blue-work still decides *what wins*."

All admission lives in `Ledger`. Enabling hybrid clears the DAG core's own switches to avoid double standards:

```rust
Ledger::set_hybrid(HybridConfig)   // calls dag.set_proof_of_work(false),
                                   //      dag.clear_difficulty(),
                                   //      dag.disable_vrf()
```

### 3.2 Configuration — `crates/kovanica-state/src/ledger.rs:106-133`

```rust
pub struct HybridConfig {
    pub rate_num: u64,             // sortition rate numerator
    pub rate_den: u64,             // whole-supply win prob ≈ num/den per block
    pub stake_nominal_work: u128,  // exact work pinned to staked blocks (tiny →
                                   //   PoW dominates chain selection)
    pub retarget: Option<Retarget>,// PoW-path work pinning; None disables pin
}
// Default: rate_num=1, rate_den=1, stake_nominal_work=1,
//          retarget=Some(Retarget::default())
```

Staked blocks pin `work == stake_nominal_work` so cheaply-inflatable blue weight stays out of chain selection no matter how a winner grinds parent combinations.

### 3.3 Admission rules — `hybrid_admit` (ledger.rs:1152-1240)

Shared pre-check (both paths):
- **Timestamp monotonicity**: `block.timestamp_ms() < max(parent ts)` → `TimestampRegression`.

**Staked path** (`Some(StakedVrf)`):
1. Verify ECVRF proof over `Dag::vrf_input(parents)` recovers exactly `output` → else `BadStakeProof`.
2. Eligibility checked against the **selected parent's pre-state** registry — a bond inside the same block cannot vote for its own producer → else `NotEligible`.
3. Defensive pin: `work != stake_nominal_work` → `StakeWorkMismatch`.
4. Sibling-spam/grinding guard: at most ONE staked block per `(vrf_pk, selected_parent)` → else `DuplicateStakedBlock` (map pruned with finality).

**PoW path** (`None`):
- `pow::meets_target(id, work)` → else `PowTargetNotMet`;
- if `retarget` set: `work != dag.work_target_with(parents, rt)` → `WorkTargetMismatch`.

Typed error enum `LedgerInsertError`: `HybridDisabled`, `BadStakeProof{vrf_pk}`, `NotEligible{vrf_pk,threshold,output,stake,total}`, `StakeWorkMismatch`, `DuplicateStakedBlock{vrf_pk,selected_parent}`, `PowTargetNotMet{id,work}`, `WorkTargetMismatch{work,expected}`, `TimestampRegression`, `Finality{..}`, plus `Dag`/`State`/`Payload`.

Insert API split:
- `insert(parents, work, ts, nonce, txs)` — PoW template
- `insert_with_vrf(parents, ts, staked_vrf, txs)` — forces nominal work; errors `HybridDisabled` if off
- `insert_prepared_block` / `insert_raw_block` — identity-preserving receive/replay paths (legacy shim strips VRF when hybrid off)

### 3.4 Stake registry — `crates/kovanica-state/src/stake.rs`

Constants (stake.rs:41-55):

```rust
pub const UNBOND_MATURITY: u64 = 100;     // blue heights before unbond allowed
pub const BOND_PREFIX: &[u8] = b"KVB1";   // bond tag = KVB1 || vrf_pk(32)
pub const UNBOND_PREFIX: &[u8] = b"KVU1"; // whole-tag match, no pk suffix
```

Model (module doc, stake.rs:1-31): registry **overlay on the UTXO set** — no new tx types.
- **Bond**: signed tx tagged `KVB1||vrf_pk`, exactly one output paid back to the spender itself; that output becomes **frozen** (recorded in per-block `StakeState`, unspendable except via unbond).
- **Unbond**: `KVU1`-tagged tx spending only frozen outpoints owned/signed by the same key holder; unlocks after `UNBOND_MATURITY` (100 blue heights). FIFO ordering enforced at node layer.
- Regular spends of frozen outpoints → `StakeError::FrozenInput`; malformed bonds → `BondShape`; immature unbonds → `UnbondImmature`.

State: `Freeze { vrf_pk, bond_height, value }`; `StakeState { frozen: HashMap<OutPoint, Freeze>, locked: HashMap<[u8;32], u64> }` with `stake_of`, `total_stake`, `bond_count`. Deterministic sorted encode/decode (rejects duplicates/trailing bytes). Per-block `stakes: HashMap<BlockId, StakeState>` mirrors per-block UTXO sets; getter `stake_state(&block)`. Checkpoint format bumped to **v3** adding length-prefixed `StakeState` blob; hybrid replay via `read_snapshot_with_hybrid` / `read_checkpoint_with_hybrid`.

### 3.5 Sortition formula (stake.rs:296-330)

```rust
eligibility_threshold(stake, total, rate_num, rate_den) -> u64 {
    if total == 0 || stake == 0 { return 0 }
    share  = ((stake as u128) << 64) / total
    scaled = share.saturating_mul(rate_num) / max(rate_den, 1)
    min(scaled, u64::MAX)
}
is_eligible(output, ...) = output.as_u64() < threshold   // strict less-than, big-endian top 64 bits
```

Win probability ≈ `(stake / total_stake) × (rate_num / rate_den)` per block. Full-stake validator always wins; zero stake never eligible.

### 3.6 DAG-side primitives composed by the hybrid layer

| Primitive | File | Details |
|---|---|---|
| VRF | `crates/kovanica-dag/src/vrf.rs` | ECVRF over Ristretto255 (CFRG draft-07); proof 96 bytes `(Γ, c, s)`; `VrfOutput::as_u64()` = big-endian first 8 bytes; domain separation `VRF_HASH_TO_CURVE_v1` / `VRF_CHALLENGE_v1` / `VRF_OUTPUT_v1` |
| VRF input | `dag.rs:666-674` | `BLAKE3(b"KOVANICA_VRF_INPUT_v1" ‖ parent ids)` |
| PoW | `pow.rs` | `meets_target(id, work)`: `H * work < 2^256` limb arithmetic; `mine(template)` |
| Retarget | `difficulty.rs:38-96` | Default `Retarget { target_interval_ms: 1_000, window: 20, max_factor: 4, min_work: 1 }`; scales avg work by expected/actual timespan, clamped |

---

## 4. Mining with a Phone as a Light Node

### 4.1 How a phone "mines"

The phone runs the same hybrid admission rules as a full node but produces blocks through the **staked draw**:

1. `set_validator_seed(seed)` → derives VRF keypair
2. `enable_hybrid(rate_num, rate_den, nominal_work, retarget)`
3. `bond_stake(seed, amount)` → freezes coins into the registry (auto-splits an oversized coin via mined split+bond blocks)
4. Each heartbeat: sign `Dag::vrf_input(parents)` → if `output < threshold(stake)` the phone emits a staked block (**no mining rig, one signature**); if not eligible, falls back to ordinary PoW mining (or simply skips with `produce_block() -> Option<BlockInfo>`)

Node-level logic mirrors this: `produce_block()` / `produce_empty()` call `try_insert_staked` first and silently fall back to PoW on `NotEligible | DuplicateStakedBlock`.

### 4.2 The FFI crate — `crates/kovanica-ffi`

UniFFI **0.32**, scaffolding name `"kovanica"`. Crate doc: the phone is a *"light validating wallet"* — byte-blob sync over any transport, same hybrid admission rules, bonded VRF production by signing one hash. Crate types `["lib", "staticlib", "cdylib"]` (staticlib required by iOS App Store rules; cdylib for Android JNA). Generated Kotlin/Swift bindings are committed under `bindings/`.

`LightConfig` defaults (light_node.rs:154-178): `k=3, subsidy=1000, founder_amount=1000, founder_seed=1, finality_depth=u64::MAX, payload_pruning_depth=u64::MAX`.

### 4.3 Complete `LightNode` public API (light_node.rs:224-747)

| Group | Methods |
|---|---|
| Lifecycle | `new(LightConfig)` |
| Identity & hybrid | `set_validator_seed(Vec<u8>)`, `validator_public_key_hex()`, `set_miner_seed(u64)`, `enable_hybrid(rate_num, rate_den, nominal_work: U128Parts, retarget: bool)`, `hybrid_enabled()` |
| Bonding | `bond_stake(seed, amount) -> String`, `total_stake()`, `my_stake()`, `unbond(from_seed, amount) -> SendReceipt`, `pending_unbond_height() -> Option<u64>`, `chain_height()` |
| Production/transfers | `produce_block() -> Option<BlockInfo>`, `produce_empty_block() -> BlockInfo` (*the phone's steady-state heartbeat*), `send(from_seed, amount, to_seed)`, `send_from(signing_secret_hex, amount, to_address)` — secrets cross the bridge per call, never stored |
| Byte-blob sync | `export_blocks() -> Vec<u8>` (gossip wire format), `receive_blocks(blob) -> u32` |
| Queries/persistence | `balance_of_seed`, `balance_of_address`, `tips`, `selected_tip`, `block_count`, `block_by_id`, `save_snapshot`, `load_snapshot` (replays under active hybrid policy) |
| SPV | `block_filter(block_id_hex)`, `filter_matches(filter_blob, address)`, `filter_matches_any(blob, addresses)`, `history_of(address, max_blocks)`, `export_light_sync()`, `receive_light_sync(blob) -> u32`, `synced_height() -> Option<u64>`, `synced_filter_matches(block_id, address) -> Option<bool>`, `prove_tx(block_id_hex, tx_id_hex) -> Option<Vec<u8>>`, `verify_tx_proof(proof_blob, block_id_hex) -> bool` |

Value types: `U128Parts{high,low}` (+`decimal_string()`), `BlockKind::{Pow,Staked}`, `BlockInfo{id_hex,parents_hex,kind,work,timestamp_ms}`, `SendReceipt`, `TxDirection`, `HistoryEntry`, `LightNodeError{AlreadyInitialized, Hex, BadSeedLength, Invalid, InsufficientFunds, BadSecretLength, InsufficientStake, Node}`.

Rust uses snake_case (`produce_block`); camelCase forms (`produceBlock`, `bondStake`, …) exist only in generated Kotlin/Swift bindings.

### 4.4 SPV light-sync wire format (FFI-owned, versioned, big-endian)

- `LIGHT_SYNC_MAGIC = b"KVLS"`, `LIGHT_SYNC_VERSION = 1`, `FILTER_K = 8`
- Blob = magic(4) + version(1) + count(u32), then per block a fixed **160-byte header** (id, prev_hash, merkle_root, work u128, ts, nonce, blue_score, chain_blue_work, height) + filter `k(1)‖n(8)‖len(4)‖data`
- `receive_light_sync` verifies through the real `SpvClient::new(header, require_pow=false, None)` — linkage/timestamps/monotone blue work enforced; PoW relaxed because staked blocks carry only nominal work
- Inclusion proofs: `tx_id‖root‖path_len‖path…‖index‖count` (min 72 bytes; single-tx blocks prove as bare leaves, blob exactly 84 bytes)
- Filters are Golomb-Rice compact block filters for address watching

### 4.5 Android/iOS packaging

| Piece | Detail |
|---|---|
| `build-android.sh` | cargo-ndk `--platform 24`, ABIs `arm64-v8a x86_64` (mapping includes `armeabi-v7a` for later) → `.so` into `android/src/main/jniLibs/` |
| Gradle module `android/` | namespace `uniffi.kovanica`, minSdk 24, compiles committed `bindings/kotlin` via sourceSets, sole dep `net.java.dev.jna:jna:5.14.0@aar`, consumer R8 rules |
| `build-apple.sh` | slices `aarch64-apple-ios`, `aarch64-apple-darwin`, `x86_64-apple-darwin` → staticlibs → `target/kovanica.xcframework` + committed `bindings/swift/kovanicaFFI.h` + `.modulemap` |
| CI drift guard | `.github/workflows/bindings.yml` regenerates kotlin+swift into /tmp, `diff -r -x README.md` vs committed trees; fails on any difference; shellchecks both scripts |

### 4.6 Copy-paste examples (from protocol README.md:42-135)

Kotlin / Android:

```kotlin
val producer = LightNode(LightConfig())
producer.setValidatorSeed(...)
producer.enableHybrid(1uL, 1uL, nominalWork, false)
producer.bondStake(seed, 1uL, 500uL)
val block = producer.produceEmptyBlock()
// byte-blob sync convergence check
```

Swift / iOS:

```swift
try producer.enableHybrid(rateNum: 1, rateDen: 1, nominalWork: nominalWork, retarget: false)
```

Plus an "SPV mode for watch-only wallets" section: `export_light_sync()` / `receive_light_sync(blob)` (`KVLS` v1), filter queries, `prove_tx` / `verify_tx_proof` (merkle inclusion proofs rooted at a synced header).

---

## 5. Node Layer & Wire Format

- `Node::set_validator_seed([u8;32])` → `validator_sk` via `vrf_keypair_from_seed`; `enable_hybrid(config)`; queries `total_stake()`, `stake_of(vrf_pk)`, `outpoint_is_frozen(op)`, `pending_unbond_height(vrf_pk)`
- `unbond_with(kp, vrf_pk, amount, to)`: owner guard (`UnbondOwnerMismatch`), FIFO over matured coins only, `InsufficientStake{requested, available}`; fee-0 value-conserving `KVU1` tx sealed in a mined PoW block
- Production staked-first, PoW fallback (silent on not-eligible/duplicate)
- Wire format (net.rs:250-320): VRF flag byte after nonce — `0` none, `1` = pk32+proof96+output32 (`VRF_FLAG_STAKED`); min record 49 bytes
- Persistence: `load_with_hybrid(path, config)` so staked ids survive snapshot replay
- RPC: single read-only command `staking [vrf-pk-hex]` reporting `hybrid=`, `total_stake=`, optional `validator=` / `stake_of=`. **No RPC exists to bond or enable hybrid** — programmatic only.

---

## 6. Test Coverage (43+ tests touching hybrid/mobile behavior)

| Suite | Count | Highlights |
|---|---|---|
| `kovanica-state/tests/hybrid.rs` | 12 | `staked_insert_requires_hybrid_mode`, `full_stake_validator_always_wins`, `zero_stake_never_eligible`, `half_stake_threshold_is_half_the_output_space`, `duplicate_staked_sibling_rejected`, `pow_path_rejects_unmined_blocks_but_accepts_mined_ones`, `pow_path_work_is_pinned_to_the_retarget_policy`, `timestamp_regression_rejected_on_both_paths`, `bad_stake_proof_rejected`, snapshot/checkpoint roundtrips preserving staked ids |
| `kovanica-node/tests/hybrid_node.rs` | 3 | `staked_block_produced_gossiped_and_readmitted`, `unbonded_validator_falls_back_to_pow`, `staking_rpc_reports_state` |
| `kovanica-node/tests/unbond_node.rs` | 3 | maturity lifecycle, FIFO over matured coins only, foreign-signer rejection |
| `kovanica-ffi/tests/ffi.rs` | 17 | `bond_splits_then_freezes_and_staked_block_wins`, `sync_blob_between_two_nodes_converges`, `garbage_sync_blob_is_rejected_not_panicked_on`, `snapshot_roundtrip_preserves_staked_ids_and_keeps_producing`, `send_from_uses_imported_secret_without_storing_it`, `retarget_enabled_hybrid_pins_pow_and_syncs`, `light_sync_filters_and_proofs_end_to_end`, `history_over_ffi_matches_utxo_semantics`, … |
| `stake.rs` unit tests | 8 | incl. `eligibility_distribution_is_statistically_sound` (20k-trial ±5% distribution check) |

Related: `kovanica-dag/tests/{vrf,pow,difficulty}.rs`, `adversarial_spv.rs`, `spv_sync.rs`, `wallet_history.rs`.

---

## 7. Status & Roadmap Position

- Shipped as part of **Stage 3 close-out**, workspace bumped to **v0.2.0**: "VRF leader eligibility, hybrid PoW+staked admission, stake registry, P2P hardening, mempool v2, metrics/observability, DHT+DNS discovery, mobile FFI slices 1–8"
- Vault roadmap: `SPV wire protocol ✅ Complete`; `Light clients / SPV proofs ✅ DONE`
- Plan file `docs/plans/mobile-light-node.md`: slices 4–8 all marked ✅ LANDED ("Landed as built" deviation notes; e.g., no separate `spv_api.rs` — folded into `light_node.rs`)
- Hard-won lesson attached to hybrid: **identity-preserving block replay** — never rebuild a received block with fresh `Block::new`; once VRF fields exist re-encoding changes the id. Replay hybrid-era snapshots only through `_with_hybrid` readers.

### Stale-doc flags found during research
- `KovanicaDAG/CODE_INDEX.md` has **no entries** for `kovanica-ffi`, `light_node.rs`, `stake.rs`, or hybrid tests
- `KovanicaDAG/myObsidianVaultDAG.md` repo structure lists only 4 crates (missing `kovanica-ffi`)
- One AGENTS.md roadmap bullet still says "Mobile light-node slices 4–8 ◀ ACTIVE" although the slices above it are documented as landed — treat slice entries as authoritative

---

## 8. Explicitly NOT Implemented (verified gaps)

- **No epoch randomness beacon** — VRF input is still parent tips; grinding-resistant beacon documented as future work (stake.rs:12, AGENTS.md:497)
- **No `Node::bond_stake`** — bonding at node level constructs the tagged tx directly; convenience method exists only on FFI `LightNode`
- **No RPC commands** for bonding / validator-seed setup / enabling hybrid (read-only `staking` query only)
- **No CLI / explorer / web surface** for staking or hybrid (zero hits in `kovanica-cli`, `explorer.rs`, `web/src`)
- **No `set_fee_floor`** — deferred in Slice 7 ("no congestion signal yet")
- **No default `armeabi-v7a` Android builds** (mapping exists; defaults are arm64-v8a + x86_64)

---

## 9. Why This Design Suits Phones (synthesis)

1. **No rig needed**: block production = one ECVRF signature over the tip input; probability proportional to bonded stake, not burned electricity.
2. **Chain selection stays PoW-anchored**: staked blocks carry nominal work, so phones can never distort GHOSTDAG blue-weight ordering — they participate without needing to outrun miners.
3. **Light footprint**: byte-blob sync works over any transport; `KVLS` light-sync blobs (160 B/block header + filter) enable watch-only wallets; snapshot save/load for fast cold start.
4. **Safe custody model**: secrets cross the FFI bridge per call and are never stored; unbond requires maturity (100 blue heights) preventing instant stake withdrawal attacks.
5. **Grinding resistance**: eligibility read from selected-parent pre-state + one-staked-block-per-(pk,parent) guard + pinned nominal work close the obvious grinding/sibling-spam vectors.

---

## 10. Source Map (authoritative files)

**Protocol (`/root/kovanica-protocol`)**
- `crates/kovanica-state/src/stake.rs` — registry, tags, sortition math
- `crates/kovanica-state/src/ledger.rs` — `HybridConfig`, `StakedVrf`, `set_hybrid`, `hybrid_admit`, checkpoint v3
- `crates/kovanica-dag/src/{vrf,pow,difficulty,dag}.rs` — primitives
- `crates/kovanica-node/src/node.rs` — validator seed, hybrid enable, staked-first production, unbond
- `crates/kovanica-node/src/rpc.rs:132-155` — `staking` command
- `crates/kovanica-node/src/net.rs:250-320` — VRF wire flag
- `crates/kovanica-ffi/src/lib.rs`, `src/light_node.rs` — mobile API + KVLS format
- `crates/kovanica-ffi/build-android.sh`, `build-apple.sh`, `android/`, `bindings/{kotlin,swift}/`
- Tests: `kovanica-state/tests/hybrid.rs`, `kovanica-node/tests/{hybrid_node,unbond_node}.rs`, `kovanica-ffi/tests/ffi.rs`
- `docs/plans/mobile-light-node.md`, root `README.md:42-135`, `Cargo.toml` (workspace v0.2.0)

**Vault (`/root/Obsidian-Vault/KovanicaDAG/`)**
- `AGENTS.md:480-597, 610-656, 659-665, 683-718` — slice narrative, rationale, hard-won lessons
- `README.md:7-9, 19-21, 42-129` — protocol description + Kotlin/Swift guides
- `myObsidianVaultDAG.md:195-201`, `TODO.md:83`, `ROADMAP.md:65` — status markers

---

## 11. Research Method Notes

- Two exhaustive passes: vault-wide (including hidden dirs, canvas files, synced agent copies) and code-wide grep/AST search.
- MARKDOWN.god (agent brain) contains **zero** hybrid-PoW or mobile-FFI references — it predates this work (stale copy of the older agent brain).
- Old multi-repo snapshots under `KovanicaDAG/kovanica-*/` and `docs/vault/` mention these topics only via stale copies; live code was treated as authoritative per repo convention.

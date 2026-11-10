# ADR 0003: Hybrid PoW + GHOSTDAG Consensus

- **Status:** Accepted (Toni 2026-11-05)
- **Decision scope:** Product-of-record consensus architecture and migration path
- **Validation owner:** Toni / Architecture Gate
- **Supersedes / Amends:** Supersedes [ADR 0001: Paper-faithful Mysticeti-C with weighted stake thresholds](0001-mysticeti-c-weighted-consensus.md) for product consensus direction.

---

## 1. Context

The original architecture aimed for a Mysticeti-C uncertified DAG consensus engine with stake-weighted voting thresholds and VRF-based leader selection (as documented in ADR 0001 and PRO-7).

On 2026-11-05, project owner Toni locked a strategic change in product consensus direction:
1. **Consensus Model:** Kovanica shifts to a pure **Hybrid PoW + GHOSTDAG** consensus architecture.
2. **No Stake in Consensus:** There is **NO stake** in the consensus path (Toni clarified earlier notes; stake is not used for consensus voting, leader schedules, or block validity).
3. **Coin Ticker:** The native coin ticker is renamed back from `KUNA` to **`KVNC`** (9 decimals). This aligns the native coin ticker with the canonical address prefix (`kvnc...dag`) and the existing cryptographic prefixes.
4. **Target Timing & Parameters:**
   - `TARGET_BLOCK_MS = 4000` (4 seconds).
   - `GHOSTDAG_K = 5` (superseding an earlier same-day intermediate proposal of 8000 ms / k=3).
5. **Planned PoW Algorithm:** Named **KovanicaHash** (strictly NOT KunaHash). Design intent is a memory-hard proof-of-work combining BLAKE3 with a 2 MiB memory mix, requiring a formal audit prior to mainnet activation.
6. **Public Launch Milestone:** The public launch (previously targeted for 2026-11-25) is now gated on the successful passing of the **Hybrid Testnet M8** milestone.
7. **PRO-7 Freeze:** Because the Mysticeti VRF leader selection path is dead under hybrid PoW + GHOSTDAG, work on PRO-7 (design B full VRF set + pinned leader schedule) is **FROZEN**.

---

## 2. Decision

1. **Consensus Direction:**
   - Adopt **Hybrid PoW + GHOSTDAG** as the sole product consensus architecture.
   - Remove stake weighting, VRF leader rotation, and certificate timeouts from the consensus production path.

2. **Locked Hybrid Parameters:**
   - `TARGET_BLOCK_MS = 4000` ms (4 s).
   - `GHOSTDAG_K = 5`.
   - Planned PoW: `KovanicaHash` (spec stub and architectural intent only; implementation deferred to subsequent phases).
   - Native coin ticker: `KVNC` (9 decimals; base unit atoms `1 KVNC = 10^9`).

3. **Chain Plan:**
   - Introduce planned Chain ID `5` dedicated to the hybrid PoW testnet, gated behind a feature/flag `hybrid-pow`. (Documented intent only in P0; runtime wiring deferred to subsequent phases).

4. **Public Launch Gating:**
   - Public launch is formally gated on passing the Hybrid Testnet M8 milestone.

5. **Lifting the GHOSTDAG k=3 Hard-Lock (P0 Scope):**
   - The tip codebase previously hard-locked GHOSTDAG parameter `k` to `3` (`colouring.rs` `UnsupportedK` error, `ghostdag_scoped.rs` panic on `k != 3`, and `mysticghost.rs` default `k: 3`).
   - P0 unlocks `k = 5` in `colouring.rs`, `ghostdag_scoped.rs`, and updates `mysticghost.rs` defaults to use `GHOSTDAG_K = 5` (or accept both 3 and 5), without altering the underlying colouring, mergeset, or non-blue recycling algorithms.

---

## 3. Consequences & Non-Goals

### Consequences:
- **ADR 0001 Superseded:** ADR 0001 (Mysticeti-C weighted consensus) is superseded for product direction.
- **PRO-7 Frozen:** PRO-7 (Mysticeti VRF leader schedule pinning) is frozen. No further VRF leader schedule or committee timeout work will be merged for consensus.
- **Code Preservation for Follow-up Phases:** Algorithms for colouring, mergeset computation, non-blue block tracking, and the redb store are preserved as foundation for the GHOSTDAG engine in subsequent phases (P1–P8).
- **Hard-Lock Lifted:** GHOSTDAG colouring accepts `k = 5` as well as legacy `k = 3`.
- **Lint & Rules Inversion:** Automated documentation lint (`scripts/lint_docs.py`), `README.md`, and the (now archived) `docs/archive/AUDIT_REPORT_2026.md` are updated to permit and require `KVNC` as the product ticker instead of forbidding it.

### Explicit Non-Goals for P0 (Spec Phase):
- **No PoW Mining Loop:** No mining workers, nonce search, or difficulty adjustment algorithms are implemented in this PR.
- **No Header Rewrite:** Block header format remains untouched in P0; rewrite is scheduled for P1/P2.
- **No Hasher Crate:** No `KovanicaHash` or `KunaHash` crate or symbol is introduced. Only the name and design intent are recorded.
- **No GHOSTDAG Engine Swap:** The consensus engine loop, committer, and sync protocol remain intact in P0.
- **No Apps / Web Changes:** No changes to `apps/`, `packages/web`, or UI frontends beyond ticker documentation.
- **No Genesis Ceremony:** No genesis balance generation or ceremony script changes.
- **No Final Emission / Escrow Decisions:** Tokenomics open Page questions (escrow schedules, emission curves, validator staking crate transition) remain open and are not finalized in this PR.

---

## 4. References & Related Documents

- [ADR 0001: Paper-faithful Mysticeti-C with weighted stake thresholds](0001-mysticeti-c-weighted-consensus.md) (Superseded)
- `STATE.md`: Living source of truth tracking consensus priority, tip SHA, and launch milestones.
- `AGENTS.md`: Agent and contributor instructions reflecting hybrid PoW + GHOSTDAG product truth.
- Toni locks (2026-11-05): `TARGET_BLOCK_MS = 4000`, `GHOSTDAG_K = 5`, PoW name `KovanicaHash`, ticker `KVNC`, no stake for consensus.

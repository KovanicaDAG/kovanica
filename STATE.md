# STATE — living source of truth

**Tip:** `89394a2da7d9fb119072aacd1ff7997f0e3225c4` (feat/hybrid-p1-p2-pow-header bases here; merge of docs/hybrid-p0-spec #177 on 2026-11-05; prior living tip `559cc2014592e1c621da2df78d9671e7476972d1`).
Update this file when frozen params or open product gaps change. Point-in-time dumps live under `docs/archive/`; they are not SoT.

## Active Priority: Hybrid PoW + GHOSTDAG (P0–P8)

- **Consensus Direction (Toni 2026-11-05):** Hybrid PoW + GHOSTDAG. **NO stake for consensus**.
- **Priority Rank:** Hybrid PoW + GHOSTDAG phases P0–P8 rank strictly above PRO-6, PRO-14, and Gate leftover 20/36/7.
- **PRO-7:** **FROZEN** for hybrid (Mysticeti VRF leader path dead under hybrid PoW + GHOSTDAG).
- **Public Launch:** Public launch is delayed from unconditional 2026-11-25 until **Hybrid Testnet M8** passes.
- **PoW Hash:** Planned PoW hash name is **KovanicaHash** (strictly NOT KunaHash; blake3 + 2 MiB memory mix; audit required before mainnet).
- **Consensus Defaults:** `TARGET_BLOCK_MS = 4000`, `GHOSTDAG_K = 5`.
- **Product Ticker:** **KVNC** (renamed back from KUNA per Toni 2026-11-05).

## Chain 4 & Chain 5 (Testnet tracks)

- `chain_id = 4`. Network tag form: `<chain_id>-<16 hex genesis digest>` (see `docs/TESTNET-SEED-NODES.md` for topology). Internal testnet track until public launch (gated on hybrid testnet M8 pass).
- `chain_id = 5`. Planned Hybrid PoW + GHOSTDAG testnet track under feature/flag `hybrid-pow` (ADR 0003; `CHAIN_ID_HYBRID_TESTNET = 5`, network tag `hybrid-testnet`). Legacy hex allowed on chains 3, 4, 5, 1337.
- Launch gating: public launch delayed from unconditional 2026-11-25 until hybrid testnet M8 passes.
- Genesis for testnet: **5** validators (legacy). Mainnet genesis requires **≥15** (15–20 in ceremony).
- Ordering default: GHOSTDAG (lifted k=3 hard-lock in P0 to allow `GHOSTDAG_K = 5`).
- Topology, seeds, and ops runbooks: `docs/TESTNET-SEED-NODES.md`, `docs/TESTNET-RESET.md`, `docs/VALIDATOR-ONBOARDING.md`. Do not treat archived `sync-*.md` tables as live inventory.

## Frozen protocol params (code-true decisions)

| Topic | Value |
| --- | --- |
| Coin | KVNC, **9** decimals (`ONE_KVNC = 10^9`, legacy alias `ONE_KUNA`) |
| Consensus | Hybrid PoW + GHOSTDAG (NO stake in consensus path; `TARGET_BLOCK_MS = 4000`, `GHOSTDAG_K = 5`; PoW: KovanicaHash) |
| Cap / subsidy sketch | 90.2M total; mining budget 81M; treasury 8M; genesis validators 1M; founder premine 200k |
| Address | `kvnc` + 57 bech32-alphabet base32 + `dag` (**64** chars); 32 B Ed25519 ‖ 29-bit blake3 checksum. Legacy hex only on chains **3, 4, 5, 1337** |
| Epoch (legacy) | **7200** DAG rounds ≈ 8 h ≈ **2400** leader heights (legacy Mysticeti constant still in tree until hybrid flag) |
| Jail (legacy) | `JAIL_ROUNDS = 7200` committed leaders (legacy / non-consensus under hybrid) |
| Min validator stake (legacy) | `50_000` KVNC (`MIN_VALIDATOR_STAKE = 50_000 * ONE_KUNA`; legacy / non-consensus) |
| Min delegation (legacy) | `50_000` KVNC (`MIN_DELEGATION = 50_000 * ONE_KUNA`; legacy / non-consensus) |
| `funded_accounts.toml` | Allowed on every chain **except** mainnet (chain 1) |
| Hash / sig domains | `KUNA-*` / `KVNC-*` crypto context tags remain for protocol compatibility |

Gated hardening still inert on live chain until a genesis reset flips `*_ACTIVE_FROM_ROUND` to `0` (gas, contract, size limits, slashing).

## Open product gaps (not archived noise)

1. CI move into `KovanicaDAG/kovanica-protocol` (fmt/clippy/test + pinned gitleaks).
2. Ops drift (live fleet): VPS systemd units still `kvnc-*` / `KVNC_*`; DNS seed names for mainnet (`docs/SEED-DNS-AUDIT.md`).
3. Governance (Phase 25), BIP32 HD, external audit, mainnet launch runbook — see root `TASKLIST.md`.

**Done / frozen, do not re-open as missing:**
- PRO-7 (Mysticeti VRF leader set + pin): **FROZEN** for hybrid.
- Hybrid PoW + GHOSTDAG P1–P8 (PoW header primitives, preimages, target conversion, Chain ID 5 wiring, KovanicaHash BLAKE3+2MiB algorithm, header verification, CPU mining worker, DAA, P2P network anti-DoS ingress, GHOSTDAG k=5 committer, 2.5 KVNC reward minting, node miner loop, JSON-RPC getWork/submitWork/miningInfo/dagTips, CLI mining/dag commands, Prometheus metrics, multi-node simulation test suite). Milestone M8 ready.
- Contract SDK (`crates/kovanica-contract-sdk`) and Indexer (`crates/kovanica-indexer`).
- Escrow E0433 (`kovanica-escrow` wired into `kovanica-execution/src/contracts.rs`, merged in #163).
- PRO-20 / PRO-36 merged code (kept).
- Vote signature **v2** (`KUNA/vote/v2` + `genesis_hash` in 106 B preimage; 74 B v1 retired).
- Light-client checkpoint (`kovanica_lightClientCheckpoint` state-backed lookup in `kovanica-rpc`).
- PRO-40 late-leader rule: union-quorum commit with non-overlapping timeout stake above the skip-complement floor.

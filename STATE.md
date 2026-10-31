# STATE — living source of truth

**Tip (docs hygiene base):** `baf41511fb4b4c9beb7d84224b1bbf851e240bac` (`main`, 2026-10-31).
Update this file when frozen params or open product gaps change. Point-in-time dumps live under `docs/archive/`; they are not SoT.

## Chain 4 (public testnet track)

- `chain_id = 4`. Network tag form: `<chain_id>-<16 hex genesis digest>` (see `docs/TESTNET-SEED-NODES.md` for topology).
- Target launch window: public testnet **2026-11-25** as product **1.0.0** if apps are solid; mainnet target **2027-01-01** as **2.0.0**.
- Genesis for testnet: **5** validators (growing toward 15–21 active via staking). Mainnet genesis requires **≥15** (15–20 in ceremony; 21 via staking later).
- Ordering default on testnet: MysticGhost (GHOSTDAG). Round duration **4 s** (leader ~12 s). Consensus env overrides (`KOVANICA_ROUND_DURATION_MS`, `KOVANICA_MYSTICGHOST`, `KOVANICA_VRF_LEADER`) are hard-rejected on non-dev chains.
- Topology, seeds, and ops runbooks: `docs/TESTNET-SEED-NODES.md`, `docs/TESTNET-RESET.md`, `docs/VALIDATOR-ONBOARDING.md`. Do not treat archived `sync-*.md` tables as live inventory.

## Frozen protocol params (code-true decisions)

| Topic | Value |
| --- | --- |
| Coin | KUNA, **9** decimals (`ONE_KUNA = 10^9`) |
| Cap / subsidy sketch | 90.2M total; mining budget 81M; treasury 8M; genesis validators 1M; founder premine 200k |
| Address | `kvnc` + 57 bech32-alphabet base32 + `dag` (**64** chars); 32 B Ed25519 ‖ 29-bit blake3 checksum. Legacy hex only on chains **3, 4, 1337** |
| Epoch | **7200** DAG rounds ≈ 8 h ≈ **2400** leader heights (consensus + staking/light set) |
| Jail | `JAIL_ROUNDS = 7200` committed leaders ≈ 1 day (= 3 epochs) |
| Min validator stake | `50_000` KUNA |
| Min delegation | **decided 50_000 KUNA** (Toni 2026-10-31); constant still open in code (see TASKLIST) |
| `funded_accounts.toml` | Allowed on every chain **except** mainnet (chain 1) |
| Hash / sig domains | `KUNA-*` (vote still `KUNA/vote/v1`; v2 + genesis_hash still open) |

Gated hardening still inert on live chain until a genesis reset flips `*_ACTIVE_FROM_ROUND` to `0` (gas, contract, size limits, slashing).

## Open product gaps (not archived noise)

1. Wire `kovanica-escrow` into `kovanica-execution` (E0433 on `main` after #160; free-fn dispatch in `contracts.rs`).
2. `MIN_DELEGATION` constant = 50_000 KUNA in staking path.
3. Vote signature **v2** (`KUNA/vote/v2` + `genesis_hash` in preimage).
4. CI move into `KovanicaDAG/kovanica-protocol` (fmt/clippy/test + pinned gitleaks).
5. PRO-40 follow-up: deterministic late-leader / skip-cert vs union-quorum (see `docs/dev/NOTED-pro-40-late-vote.md`).
6. Ops drift: VPS units still `kvnc-*` / `KVNC_*`; physical `/root/Projects/kvnc` path rename; DNS seed names for mainnet.
7. Governance (Phase 25), BIP32 HD, external audit, mainnet launch runbook — see root `TASKLIST.md`.

**Done, do not re-open as missing:** Contract SDK (`crates/kovanica-contract-sdk`) and Indexer (`crates/kovanica-indexer`).

# TASKLIST — remaining work

Historical completed items moved to `docs/dev/NOTED*` and `docs/archive/`. This file lists only remaining work as of 2026-10-31.

Base tip: `d951da451c9982fc8924c4c7f2ec08e06f549d64` (main @ merge #165). Full prior checklist: `docs/archive/TASKLIST-pre-baf41511.md`.

## Protocol / consensus

- [x] **MIN_DELEGATION** — **50_000 KUNA** (`MIN_DELEGATION = 50_000 * ONE_KUNA`) wired in `kovanica-params`, enforced in `kovanica-staking::StakingState::delegate` and `kovanica-execution`.
- [x] **Vote signature v2** — `KUNA/vote/v2` + `genesis_hash` in preimage (106 B; retired 74 B v1) across `kovanica-types`, `kovanica-crypto`, `kovanica-consensus`, `kovanica-network`, and `kovanica-node`.
- [ ] **PRO-40 follow-up** — deterministic late leader past `T_skip`: skip-cert quorum or union-quorum with timeout stake; cluster green (`docs/dev/NOTED-pro-40-late-vote.md`).
- [~] **Fork handling** — `use_min_digest_fork` default **on** and reachable; only tests set `false`. Keep as partial until product wants a dedicated fork-policy doc/test pack.
- [~] **Light-client checkpoint** — `kovanica_lightClientCheckpoint` wired as minimal stub; not full state-backed lookup.

## Build / CI / ops

- [ ] **CI move** — fmt/clippy/test + pinned gitleaks into `KovanicaDAG/kovanica-protocol` when frozen code lives there.
- [x] **Escrow E0433** — `kovanica-escrow` wired into `kovanica-execution` (`contracts.rs`), merged in #163.
- [ ] **VPS unit rename (live cutover)** — deployed VPS units still `kvnc-*` exporting `KVNC_*`; repo scripts/docs use `KOVANICA_*` and `${KOVANICA_ROOT:-/root/Projects/kovanica}`. Coordinate live unit+script+binary cutover with Seed Ops.
- [x] **Checkout path (repo scripts/docs)** — replaced hard-coded `/root/Projects/kvnc` in seed scripts (`run-seed1.sh`, `testnet/run-seed*.sh`, `mainnet/run-seed*.sh`) and docs (`docs/TESTNET-SEED-NODES.md`, `docs/VALIDATOR-ONBOARDING.md`) with `${KOVANICA_ROOT:-/root/Projects/kovanica}` / `/root/Projects/kovanica`. Physical VPS directory rename remains part of live Seed Ops cutover.
- [ ] **DNS seeds** — 3+ kovanica-native DNS-only names for mainnet (`docs/SEED-DNS-AUDIT.md`).
- [ ] **Genesis file delete decision** — whether to remove root `testnet/genesis/genesis.json`, `testnet/validators.json`, `run-seed1.sh` (owner decision).
- [~] **Soak / load** — bench + soak scripts exist; 6h/24h Docker/CI adaptation and a real 24h run still open; property tests present but not comprehensive.
- [~] **Resource budgets** — soak cgroup limits exist; full CI enforcement pending.

## Product / apps / governance

- [ ] BIP32-style HD derivation (mnemonic import exists; no HD path).
- [ ] Kubernetes manifests (plan doc only).
- [ ] Key distribution ceremony.
- [ ] External security review (template: `docs/AUDIT_REPORT.template.md`).
- [ ] Mainnet launch + monitoring runbook.
- [ ] Governance: parameter proposals, stake-weighted voting, treasury spend, upgrade signaling (`propose`/`vote` still `not_implemented`).
- [~] Staking user-facing guide (archive/plan docs exist; no end-user guide).
- [~] Explorer docs; mainnet freeze checklist draft not fully verified on live testnet.
- [~] CLI node ops `sync`/`peers` missing; live_vote harness flaky/infra.

## Corrected false opens

- Contract SDK — **exists**: `crates/kovanica-contract-sdk`.
- Indexer — **exists**: `crates/kovanica-indexer` (structured indexer depth may still grow; crate is present).
- Escrow E0433 — **merged in #163**: `kovanica-escrow` wired into `kovanica-execution`.

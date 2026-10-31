# TASKLIST — remaining work

Historical completed items moved to `docs/dev/NOTED*` and `docs/archive/`. This file lists only remaining work as of 2026-10-31.

Base tip when trimmed: `baf41511fb4b4c9beb7d84224b1bbf851e240bac`. Full prior checklist: `docs/archive/TASKLIST-pre-baf41511.md`.

## Protocol / consensus

- [ ] **MIN_DELEGATION** — Toni decided **50_000 KUNA** (2026-10-31); wire constant + enforcement (still missing in code; only noted in TASKLIST/STATE).
- [ ] **Vote signature v2** — `KUNA/vote/v2` + `genesis_hash` in preimage (106 B; retire v1 74 B). Still `KUNA/vote/v1` in `kovanica-types`.
- [ ] **PRO-40 follow-up** — deterministic late leader past `T_skip`: skip-cert quorum or union-quorum with timeout stake; cluster green (`docs/dev/NOTED-pro-40-late-vote.md`).
- [~] **Fork handling** — `use_min_digest_fork` default **on** and reachable; only tests set `false`. Keep as partial until product wants a dedicated fork-policy doc/test pack.
- [~] **Light-client checkpoint** — `kovanica_lightClientCheckpoint` wired as minimal stub; not full state-backed lookup.

## Build / CI / ops

- [ ] **CI move** — fmt/clippy/test + pinned gitleaks into `KovanicaDAG/kovanica-protocol` when frozen code lives there.
- [ ] **Escrow E0433** — add `kovanica-escrow` path dep to `kovanica-execution` and call free fns from `contracts.rs` escrow arms (fails on `main` after #160).
- [ ] **VPS unit rename** — deployed units still `kvnc-*` exporting `KVNC_*`; coordinate unit+script+binary cutover to `KOVANICA_*`.
- [ ] **Checkout path** — `/root/Projects/kvnc` hard-coded in seed scripts/docs; rename with matching path updates.
- [ ] **DNS seeds** — 3+ kovanica-native names for mainnet (`docs/SEED-DNS-AUDIT.md`).
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

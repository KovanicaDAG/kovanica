# TASKLIST — remaining work

Historical completed items moved to `docs/dev/NOTED*` and `docs/archive/`. This file lists only remaining work as of 2026-10-31.

Base tip: `d951da451c9982fc8924c4c7f2ec08e06f549d64` (main @ merge #165). Full prior checklist: `docs/archive/TASKLIST-pre-baf41511.md`.

## Protocol / consensus

- [x] **MIN_DELEGATION** — **50_000 KUNA** (`MIN_DELEGATION = 50_000 * ONE_KUNA`) wired in `kovanica-params`, enforced in `kovanica-staking::StakingState::delegate` and `kovanica-execution`.
- [x] **Vote signature v2** — `KUNA/vote/v2` + `genesis_hash` in preimage (106 B; retired 74 B v1) across `kovanica-types`, `kovanica-crypto`, `kovanica-consensus`, `kovanica-network`, and `kovanica-node`.
- [x] **PRO-40 follow-up** — deterministic late leader past `T_skip`: skip-cert quorum or union-quorum with non-overlapping timeout stake above the skip-complement floor (`votes > total_stake - quorum_threshold`); cluster green (`docs/dev/NOTED-pro-40-late-vote.md`).
- [x] **Fork handling** — documented in docs/FORK-POLICY.md with adversarial test pack.
- [x] **Light-client checkpoint** — `kovanica_lightClientCheckpoint` backed by storage + consensus state (`LeaderHeader`, `CommitSeal`, votes, `WaveCommitCertificate`, `verify_quorum` / `verify_colouring`).

## Build / CI / ops

- [ ] **CI move** — fmt/clippy/test + pinned gitleaks into `KovanicaDAG/kovanica-protocol` when frozen code lives there.
- [x] **Escrow E0433** — `kovanica-escrow` wired into `kovanica-execution` (`contracts.rs`), merged in #163.
- [ ] **VPS unit rename (live cutover)** — deployed VPS units still `kvnc-*` exporting `KVNC_*`; repo scripts/docs use `KOVANICA_*` and `${KOVANICA_ROOT:-/root/Projects/kovanica}`. Coordinate live unit+script+binary cutover with Seed Ops.
- [x] **Checkout path (repo scripts/docs)** — replaced hard-coded `/root/Projects/kvnc` in seed scripts (`run-seed1.sh`, `testnet/run-seed*.sh`, `mainnet/run-seed*.sh`) and docs (`docs/TESTNET-SEED-NODES.md`, `docs/VALIDATOR-ONBOARDING.md`) with `${KOVANICA_ROOT:-/root/Projects/kovanica}` / `/root/Projects/kovanica`. Physical VPS directory rename remains part of live Seed Ops cutover.
- [x] **DNS seeds** — automated seed audit & reachability tool `scripts/check-dns-seeds.sh` verifies grey-cloud DNS-only resolution, anti-Cloudflare-proxying, and TCP 8000 P2P connectivity (`docs/SEED-DNS-AUDIT.md`).
- [x] **Genesis file delete decision** — legacy duplicates `testnet/genesis/genesis.json`, `testnet/validators.json`, and root `run-seed1.sh` removed; canonical configurations reside in `testnet/validators/` and `ops/`.
- [x] **Soak / load** — automated multi-node soak test harness and load generator under `ops/soak/` (`docker-compose.soak.yml`, `load_generator.py`, `run-soak.sh`); supports 10m smoke, 6h endurance, and 24h soak runs with throughput & latency reporting.
- [x] **Resource budgets** — soak cgroup resource limits (1.5 CPU, 2048 MB memory per container) enforced and asserted with zero OOM kills in `ops/soak/run-soak.sh`.

## Product / apps / governance

- [x] BIP32-style HD derivation (SLIP-0010 Ed25519; coin type 3007; opt-in via --hd/--account/--index).
- [x] Kubernetes manifests (`deploy/k8s/`, `deploy/helm/kovanica/`).
- [x] Key distribution ceremony (`docs/KEY-CEREMONY.md`).
- [ ] External security review (template: `docs/AUDIT_REPORT.template.md`).
- [x] Mainnet launch + monitoring runbook (`docs/MAINNET-RUNBOOK.md`).
- [x] Governance: parameter proposals, stake-weighted voting, treasury spend (`propose`/`vote` in `kovanica-cli`, state tracking in `kovanica-staking`).
- [x] Staking user-facing guide — [`docs/STAKING-GUIDE.md`](docs/STAKING-GUIDE.md) covers delegator & validator onboarding, CLI staking/delegation/unbonding/rewards, and FAQ.
- [x] Explorer docs ([`docs/EXPLORER.md`](docs/EXPLORER.md)) and mainnet freeze checklist ([`docs/MAINNET-FREEZE-CHECKLIST.md`](docs/MAINNET-FREEZE-CHECKLIST.md)).
- [x] CLI node ops `sync`/`peers` implemented via `kovanica_syncStatus`.

## Corrected false opens

- Contract SDK — **exists**: `crates/kovanica-contract-sdk`.
- Indexer — **exists**: `crates/kovanica-indexer` (structured indexer depth may still grow; crate is present).
- Escrow E0433 — **merged in #163**: `kovanica-escrow` wired into `kovanica-execution`.

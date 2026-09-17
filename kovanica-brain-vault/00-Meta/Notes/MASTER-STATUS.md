# Kovanica Protocol — Master Status

> **Consolidated from:** `_archive/TODO.md`, `_archive/AGENTS.md`, `_archive/kovanica-ledger/AGENTS.md`, `Notes/UPGRADE-PHASES.md`, `Notes/ROADMAP.md`, `Notes/DECISIONS.md`
> **Last sync:** 2026-09-06
> **Note:** `_archive/AGENTS.md` and `_archive/kovanica-ledger/AGENTS.md` are duplicates (CLAUDE.md snapshot); both preserved per user request.

---

## ✅ COMPLETED WORK

### Stage 0 — BlockDAG Testnet (Shipped)
*Source: AGENTS.md §Roadmap Stage 0, ROADMAP.md §Completed Stages*

- [x] DAG + GHOSTDAG consensus core (`kovanica-dag`)
- [x] UTXO ledger with Ed25519 (`kovanica-state`)
- [x] Runnable node with mempool, P2P, explorer (`kovanica-node`)
- [x] Real PoW (opt-in), difficulty retargeting, halving schedule
- [x] Incremental reachability oracle (Kaspa-style)
- [x] WebSocket explorer, TAP faucet, dual-stack P2P
- [x] Human addresses (`kvnc…dag`)
- [x] Framed bidirectional TCP sync (`pull_blocks_timeout` / `serve_exchange`)
- [x] Multi-input transfers (largest-first UTXO accumulation)
- [x] CI gate + dual-stack P2P (fmt, clippy, test before deploy)

### Stage 1 — Operations Hardening
*Source: AGENTS.md §Roadmap Stage 1, ROADMAP.md §Completed Stages*

- [x] Auto-deploy armed (`VPS_HOST`, `DEPLOY_ENABLED`, SSH :2222)
- [x] Seed ops runbook (`OPERATIONS.md`) — backup/restore, restart drill, post-deploy checks
- [x] Web proxy resolved (server-side upstream in kovanica-web)
- [x] Wallet shows `kvnc…dag` addresses (kovanica-web `bcef5f0`)

### Stage 2 — Scale & Persistence
*Source: AGENTS.md §Roadmap Stage 2, ROADMAP.md §Completed Stages*

- [x] Headers-first sync (tips/headers exchange, then fetch bodies by hash)
- [x] DAG-level payload pruning (`Block.payload = Option<Vec<u8>>`, `set_payload_pruning_depth`)
- [x] Finality checkpointing (UTXO set at finality depth, `Ledger::write_checkpoint`/`read_checkpoint`, checkpoint format v2)
- [x] Reachability interval-reindex amortisation (`CHILD_RESERVE = 1<<40`, `reachability_reindex_metrics()`)

### Stage 3 — Protocol Evolution
*Source: AGENTS.md §Roadmap Stage 3, ROADMAP.md §Completed Stages*

- [x] VRF for leader selection / randomness beacon (`kovanica-dag::vrf`, ECVRF over Ristretto255, snapshot v5)
- [x] P2P hardening: rate limits, duplicate suppression, peer scoring/banning (`kovanica-node::p2p_hardening`)
- [x] Mempool V2: orphan handling, fee-based eviction, capacity limits (`kovanica-node::mempool_v2`)
- [x] Stake registry (ledger layer): bond/unbond via tag conventions, frozen outputs, maturity gate, checkpoint v3

### Upgrade Phases 1–7 (All Merged)
*Source: UPGRADE-PHASES.md §Status*

| Phase | Scope | Key Deliverables | PR / Commit |
|-------|-------|------------------|-------------|
| 1 — Foundation & consensus infra | Mainnet profile (dormant), staked uplink, light_sync, rate limits, dead `mempool.rs` removed, cargo-audit CI | A1–A3, D1, H1–H2, I1, D2 | ✅ completed |
| 2 — Consensus evolution | Epoch randomness beacon (B1), DAG-level past-set pruning (B2), UTXO undo log (B3) | B1 #49, B2 #36/#51, B3 #50 | ✅ completed |
| 3 — Performance & scalability | Incremental on-disk store (C1), incremental sync + API pagination (C2) | merged `c85013a` #37 | ✅ completed |
| 4 — Mobile light-node (slices 9b–9d) | Wallet UX (E1), light sync + persistence (E2), staking uplink (E3) | merged `84b6516` #38 | ✅ completed |
| 5 — Wallet & security | Multisig node layer + FFI bindings (B4), web wallet custody (D4/F1) | `8a3bec6` #41 | ✅ completed |
| 6 — Operations & reliability | Seed backup automation (A8), soak tuning review (G1), 4th seed + tunnel alerting (G2), web deploy automation (G3), release pinning (G4) | `840e8f1` #40 | ✅ completed |
| 7 — P2 polish | Fee market & RBF (B5), BPS/k scaling (B6), fuzz/property tests + Criterion + P2P ban persistence (D3/D5), explorer detail views + API docs (F3/I3), Android background sync + Keystore (E4–E6) | #39, #43, #44, #45, #46, #48 | ✅ completed |

> **Phase 7 breakdown:** All PRs merged to `main`. Android unit tests for `Format.kt`, KVLS header parsing, and address derivation remain pending (no SDK/device locally).

### Mobile Light-Node Slices 4–8 (Landed 2026-08-25, workspace v0.2.0)
*Source: ROADMAP.md §Active/Next, UPGRADE-PHASES.md Phase 4*

- [x] Slice 4 — Custody & unbond over FFI (`send_from`, `unbond`, FIFO maturity)
- [x] Slice 5 — SPV/filter surface over FFI (`KVLS`v1 light-sync blobs, merkle proofs, Golomb-Rice filters)
- [x] Slice 6 — Mobile packaging & CI drift guard (`build-android.sh`, `build-apple.sh` xcframework, `bindings.yml`)
- [x] Slice 7 — Wallet UX layer (`history_of`, batched `filter_matches_any`)
- [x] Slice 8 — Docs & release (README light-node guide, hard-won lessons, v0.2.0 bump)

### Infra & Release (Shipped 2026-08-24)
*Source: ROADMAP.md §Infra & Release, TODO.md §Current Session*

- [x] Public mirror sync+release pipeline (`sync-public-node`): mirror → 4-target build → rolling `v0.1.0`; `kovanica-ffi` rides the mirror
- [x] `install.sh` prebuilt-first; `deploy-seed.sh` Amazon Linux/RHEL support
- [x] seed3 deployed (AWS EC2 eu-north-1, systemd `kovanica-seed3`, mining on, DNS `seed3.kovanica.online`)
- [x] Process manager unified on systemd (pm2 retired); atomic binary swap auto-deploy (#25, #26)
- [x] `kovanica-cli` publication decision: excluded from public mirror by design
- [x] Windows release assets for `install.ps1` (added x86_64 target to GitHub Actions)

---

## 🔴 ACTIVE NEXT

### Testnet Soak & Parameter Tuning
*Source: AGENTS.md §Post-Stage 3 item 4, ROADMAP.md §Active/Next, TODO.md §Next session*

**Status:** Infrastructure ready, baseline captured, 1–2 week data collection in progress

| Item | Status | Details |
|------|--------|---------|
| Multi-seed infra | ✅ | seed (Hostinger VPS) + **seed3** (AWS eu-north-1, live 2026-08-24) |
| Peers rollout | ✅ | `seed3.kovanica.online:9000` in default `KOVANICA_PEERS` |
| DNS-seed list fix | ✅ | `seed`/`seed2`/`seed3.kovanica.online` all resolve (A records) |
| Prometheus scraping | ✅ | Both seeds' `/metrics`; 15 alerts + 9 recording rules loaded |
| Baseline captured | ✅ | 2026-08-24 16:20 UTC: height 448/447, peers 2/2, mempool 0, orphans 0, blue_score≈height, no reorgs |
| **Tuning review** | 🔴 **PENDING** | After 1–2 weeks: `k`, finality depth, payload pruning depth, difficulty window |

**Current soak metrics to watch:**
- Orphan rate
- Propagation latency
- Fork rate
- Disk growth (both seeds expose `/metrics`)

---

## 📋 BACKLOG / DEFERRED

| Item | Status | Reason / Notes |
|------|--------|----------------|
| Light clients / SPV wire protocol | ✅ Code complete | Header chain, Merkle proofs, Golomb-Rice filters, `SpvClient` state machine — wire protocol (`getheaders`/`getblocks` with proof verification) is next |
| Multi-seed discovery (DNS/DHT) | ✅ Code complete | `dns_seed.rs`, `dht.rs`, `tests/dht_discovery.rs` Tiers 1–5 green; deployment wiring only (node binary default `KOVANICA_PEERS` still names only `seed.kovanica.online:9000`) |
| Mainnet launch | 🛑 **DORMANT** | Mainnet switch stays **on-sleep-off** (disabled). Profile built (A1) but defaults to testnet; only reachable via explicit env/flag. Excluded: live web wiring (A5), genesis ceremony (A6), launch checklist (I2), param freeze for launch (A4 aspect) |
| Fee market & RBF | ✅ Completed Phase 7 | Merged `49dfce0` #46 |
| Wallet-extension build or remove | 🔄 Deferred | Tracked in Phase 7 (F2) — superseded by web wallet custody work |
| Android unit tests | ⏳ Pending | `Format.kt`, KVLS header parsing, address derivation — no SDK/device available |

---

## 📝 DECISION LOG (Append-Only)

*Source: DECISIONS.md — never delete rows, mark superseded instead*

| Date | Decision | Rationale / Consequence | Source |
|------|----------|-------------------------|--------|
| 2026-08-23 | Single merged repo `kovanica-protocol` (renamed from `kovanica-ledger`); CLI → `crates/kovanica-cli`, web → `web/` | One workspace, easier dev; all workspace deps reference the single GitHub URL | `myObsidianVaultDAG.md` §Migration Notes |
| 2026-08-23 | Vault strategy = **doc snapshots synced by script**, never submodules | Submodule attempts failed and were deliberately removed; committing nested repos creates phantom submodules | root `AGENTS.md` §5 |
| 2026-08-24 | P2P is **TCP only** on `KOVANICA_LISTEN`; libp2p/30333 removed | It bound a port and never gossiped blocks — no second network path exists | `TESTNET.md` |
| 2026-08-24 | Bootstrap is DNS-only via grey-cloud `seed.kovanica.online:9000`; seed keeps `KOVANICA_PEERS=off` | Cloudflare orange-cloud proxying breaks raw TCP :9000 — clones must dial the grey-cloud name/origin IP | `TESTNET.md`, `OPERATIONS.md` §3 |
| 2026-08-24 | Runtime chain data lives outside any git checkout (`/root/kovanica-data`) | Lost-chain incident: pre-reset data dir was deleted while the old process held it (genesis `27d5f750…`, 127 blocks gone) | `OPERATIONS.md` §1, §4.5 |
| 2026-08-24 | Deploys SSH to **:2222**, not :22 | Hostinger-level filtering times out GitHub-runner :22 after repeated logins; sshd listens on both | `OPERATIONS.md` §2, §4.1 |
| 2026-08-24 | `metrics-exporter-prometheus` with `default-features = false` | We render `/metrics` ourselves; http-listener feature drags openssl and breaks ARM cross-builds | `OPERATIONS.md` §4.6 |
| 2026-08-24 | Public mirror excludes `kovanica-cli` (workspace membership filtered in mirror manifest) | Publication decision still open; mirror pipeline keeps it private by design | `2026-08-24-public-mirror-and-seed3`, `TODO.md` |
| 2026-08-24 | Release = rolling tag `v<workspace-version>` replaced in place; publish skips if any build fails | Never a partial release; assets carry sha256 | `2026-08-24-public-mirror-and-seed3` |
| 2026-08-24 | seed3 = AWS EC2 (eu-north-1), systemd unit, mining on, explorer/metrics loopback-only | First true off-box node for soak testing; proves deploy-seed.sh beyond same-host seed2 | `OPERATIONS.md` §6, ROADMAP |
| standing | `#![forbid(unsafe_code)]` crate-wide | Consensus determinism + auditability over micro-optimizations | protocol `AGENTS.md` §4 |
| standing | Consensus changes need rationale + adversarial tests; tie-breaks fall back to `BlockId` byte order | GHOSTDAG output must be a pure function of the DAG | protocol `AGENTS.md` §5, `myObsidianVaultDAG.md` §Design Principles |

### Superseded Decisions

| Date | Superseded Decision | By |
|------|---------------------|-----|
| ≤2026-08-23 | Multi-repo layout (`kovanica-ledger` / `kovanica-cli` / `kovanica-web`) | Merged repo, 2026-08-23 |
| ≤2026-08-24 | Vault `ecosystem.config.js` snapshot pointing at old `/home/BetterCallDzuks/kovanica-ledger` layout | `deploy/ecosystem.config.cjs` snapshot (`kovanica-explorer`, cwd `/root/kovanica-protocol`) |

---

## 🏗️ ARCHITECTURE REFERENCE (Condensed)

*Source: AGENTS.md §3 (Repository layout), §5 (Engineering conventions), §8 (Hard-won lessons)*

### Repository Layout (Workspace)
```
Cargo.toml                     Workspace manifest (resolver 2); shared deps: blake3, hex, ed25519-dalek
crates/
  kovanica-dag/                DAG + GHOSTDAG consensus core
    src/{block,dag,ghostdag,ordering,validation,snapshot,difficulty,pow,reachability}.rs
    tests/{consensus,reachability,difficulty,pow}.rs
  kovanica-state/              UTXO ledger applied in GHOSTDAG order
    src/{keys,tx,utxo,ledger,store,validation,stake}.rs
    tests/{ledger,validation,perblock,persistence,store,finality,difficulty}.rs
  kovanica-node/               Runnable node binary, mempool, block gossip
    src/{node,mempool,net,p2p,relay,explorer,main,metrics,p2p_hardening,mempool_v2,vrf}.rs
    tests/{rpc,mempool,network,p2p,relay,timestamps,fuzz}.rs
```

### Core Domain Vocabulary (Keep Precise)
- **DAG/BlockDAG** — ledger is a DAG; blocks reference **multiple parents/tips**
- **Tip** — block with no children; new blocks reference current tips
- **Past/ancestors** — all blocks reachable by following parent edges
- **Anticone** — blocks neither ancestor nor descendant ("parallel")
- **Selected parent** — parent with heaviest blue work; forms chain backbone
- **Mergeset** — `past(B) \ (past(sp) ∪ {sp})`
- **Blue/red set** — well-connected honest cluster (blue) vs. blocks left too far (red), by **k-cluster rule**
- **k parameter** — max tolerated *blue anticone size* per blue block
- **Blue score/work** — size / total work of a block's blue set; drives chain selection
- **Linearization** — recursive GHOSTDAG order: `order(B) = order(sp) ++ mergeset ++ [B]`

### Engineering Conventions (Non-Negotiable)
1. **Consensus correctness paramount** — changes to selected-parent, mergeset, k-cluster, blue score/work, linearization require: written rationale naming protocol semantics + deterministic + adversarial tests
2. **Determinism** — consensus output = pure function of DAG; no HashMap iteration order, wall-clock time, or unstable sorts affecting results
3. **Tie-breaks** → `BlockId` byte order (preserve for determinism)
4. **Tests** — prefer property/invariant + adversarial for graph/consensus; k-cluster invariant (`blue_anticone_size <= k`) is a general assertion
5. **Git workflow** — never commit to default branch; feature branches + draft PRs; kebab-case scoped names (`consensus/…`, `dag/…`, `ledger/…`); `cargo fmt`, `clippy -D warnings`, `cargo test` before push

### Hard-Won Lessons & Invariants (Do Not Break)
1. **SPV Block Filters** — When encoding 64-bit addresses into Golomb-Rice filter, *must* map into bounded interval (`N * 2^k`) first. Never push raw 64-bit difference as unary 1s (deadlocks encoder).
2. **Finality Checkpointing** — When writing checkpoint block's payload, *must* explicitly prune via `Block::new_pruned_with_vrf` so bytes exactly match reconstructed block from `read_checkpoint`.
3. **DHT handshake contacts** — `Mesh::connect` must register both endpoints as mutual DHT routing-table contacts. Verified handshake exchanges NodeId + address; established contacts claiming bucket slots first gives eclipse resistance (Tier 5 `test_adversarial_eclipse_resistance` asserts this). Do not decouple P2P connect from DHT contact registration.
4. **Metrics crate version** — `kovanica-node`'s `metrics` dependency must stay on same minor version as `metrics-exporter-prometheus` depends on; otherwise emissions land in noop recorder of other version's global slot and `/metrics` renders nothing.

---

## 🔗 SOURCE FILES & LINKS

| Section in Master | Origin File(s) |
|-------------------|----------------|
| Stage 0–3 completed | `_archive/AGENTS.md` §Roadmap, `Notes/ROADMAP.md` §Completed Stages |
| Upgrade Phases 1–7 | `Notes/UPGRADE-PHASES.md` §Status, §Phase 7 breakdown |
| Mobile Light-Node Slices 4–8 | `Notes/ROADMAP.md` §Active/Next, `Notes/UPGRADE-PHASES.md` Phase 4 |
| Infra & Release (2026-08-24) | `Notes/ROADMAP.md` §Infra & Release, `_archive/TODO.md` §Current Session |
| Testnet Soak (Active) | `_archive/AGENTS.md` §Post-Stage 3 item 4, `Notes/ROADMAP.md` §Active/Next, `_archive/TODO.md` §Next session |
| Backlog / Deferred | `Notes/ROADMAP.md` §Backlog, `_archive/AGENTS.md` §Beyond, `Notes/UPGRADE-PHASES.md` §Beyond |
| Decision Log | `Notes/DECISIONS.md` (full content) |
| Architecture Reference | `_archive/AGENTS.md` §3, §5, §8 |
| Session history (detailed) | `_archive/TODO.md` (session-based), `_archive/AGENTS.md` (full conventions) |

---

## 📌 QUICK COMMANDS

```bash
# Build
cargo build

# Test (all)
cargo test

# Single test
cargo test <name>  # e.g. cargo test adversarial_wide_fork

# Lint
cargo clippy --all-targets -D warnings

# Format check
cargo fmt --check

# Run node
cargo run -p kovanica-node -- demo
# or REPL:
cargo run -p kovanica-node  # then: help
```

---

*End of MASTER-STATUS.md — this file is the single source of truth for project status. Update it when work completes; do not edit archive files.*
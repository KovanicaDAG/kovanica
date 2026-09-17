# Kovanica Protocol Roadmap

> **Vault-specific roadmap** — synthesized from [[myObsidianVaultDAG]] and `kovanica-protocol/AGENTS.md`.
> Edit here; not overwritten by sync-vault.sh.
>
> **Links:** [[myObsidianVaultDAG]] · [[AGENTS]] · [[REINDEX_BENCHMARKING]] · [[MASTER-STATUS]]
> **Master status:** See [[MASTER-STATUS]] for consolidated project status (all stages, decisions, active work)

---

## ✅ Completed Stages

### Stage 0 — BlockDAG Testnet (Shipped)
- [x] DAG + GHOSTDAG consensus core (`kovanica-dag`)
- [x] UTXO ledger with Ed25519 (`kovanica-state`)
- [x] Runnable node with mempool, P2P, explorer (`kovanica-node`)
- [x] Real PoW (opt-in), difficulty retargeting, halving schedule
- [x] Incremental reachability oracle (Kaspa-style)
- [x] WebSocket explorer, TAP faucet, dual-stack P2P
- [x] Human addresses (`kvnc…dag`)

### Stage 1 — Operations Hardening
- [x] Auto-deploy armed (`VPS_HOST`, `DEPLOY_ENABLED`)
- [x] Seed ops runbook (`OPERATIONS.md`)
- [x] Web proxy resolved (server-side)
- [x] Wallet shows `kvnc…dag` addresses

### Stage 2 — Scale & Persistence
- [x] Headers-first sync
- [x] DAG-level payload pruning (`Block.payload = Option<Vec<u8>>`)
- [x] Finality checkpointing (UTXO set at finality depth)
- [x] Reachability interval-reindex amortisation (`CHILD_RESERVE = 1<<40`)

### Stage 3 — Protocol Evolution
- [x] VRF for leader selection / randomness beacon (`kovanica-dag::vrf`)
- [x] P2P hardening: rate limits, duplicate suppression, peer scoring/banning
- [x] Mempool V2: orphan handling, fee-based eviction, capacity limits

---

## 🎯 Active / Next Priorities

### ✅ Mobile Light-Node Slices 4–8 (LANDED 2026-08-25 — workspace v0.2.0)
Plan: `kovanica-protocol/docs/plans/mobile-light-node.md` · commit `d06962f`
- [x] Slice 4 — Custody & unbond over FFI (`send_from`, `unbond`, FIFO maturity)
- [x] Slice 5 — SPV/filter surface over FFI (`KVLS`v1 light-sync blobs, merkle proofs, Golomb-Rice filters)
- [x] Slice 6 — Mobile packaging & CI drift guard (`build-android.sh`, `build-apple.sh` xcframework, `bindings.yml`)
- [x] Slice 7 — Wallet UX layer (`history_of`, batched `filter_matches_any`)
- [x] Slice 8 — Docs & release (README light-node guide, hard-won lessons, v0.2.0 bump)

> Note: `kovanica-protocol/AGENTS.md` §Post-Stage 3 still carries a stale
> "◀ ACTIVE" marker on this item; the plan file and slice-8 commit confirm all
> slices are landed.

### ✅ RFC-004 HTLC / Atomic Swap (SHIPPED 2026-09-08 — KVP-104)
Rebased + merged PR #88 (`fb13741`), web typecheck restored (PR #95 `a960742`),
docs status Shipped (PR #96 `9fb83b5`). Dedicated `0x04` HTLC template +
Tier Nolan atomic swap, no wire-format bump.
Next on the evolving-plan critical path (`[[2026-09-05-protocol-evolution-6-points]]`):
**5.2 Time-lock Vault (CSV/CLTV)** — needs per-UTXO creation-height tracking.

### 🔴 Testnet Soak & Parameter Tuning (ACTIVE NEXT — AGENTS.md Post-Stage 3 item 4)
- [x] Multi-seed infra: seed (Hostinger VPS) + **seed3** (AWS eu-north-1, live 2026-08-24 — see [[2026-08-24-public-mirror-and-seed3]])
- [x] Peers rollout: `seed3.kovanica.online:9000` into default `KOVANICA_PEERS`
- [x] Fix default DNS-seed list (`dns_seed.rs`) — `seed`/`seed2`/`seed3.kovanica.online` all resolve
- [x] Prometheus scraping both seeds' `/metrics`; 15 alerts + 9 recording rules loaded
- [x] Baseline captured 2026-08-24 16:20 UTC: height 448/447, peers 2/2, mempool 0, orphans 0, blue_score≈height, no reorgs (OPERATIONS.md §5)
- [ ] Tuning review after 1–2 weeks: `k`, finality depth, payload pruning depth, difficulty window

### Infra & Release (shipped 2026-08-24)
- [x] Public mirror sync+release pipeline (`sync-public-node`): mirror → 4-target build → rolling `v0.1.0`; `kovanica-ffi` now rides the mirror too
- [x] `install.sh` prebuilt-first; `deploy-seed.sh` Amazon Linux/RHEL support
- [x] seed3 deployed, DNS-only A record live
- [x] Process manager unified on systemd (pm2 retired); atomic binary swap auto-deploy (#25, #26)

### Backlog
- [x] `kovanica-cli` publication decision (mirror workspace excludes it by design)
- [x] Optional: Windows release assets for `install.ps1`

---

## 📋 Tracking

| Area                         | Status                       | Next Action                                  |
| ---------------------------- | ---------------------------- | -------------------------------------------- |
| SPV wire protocol            | ✅ Complete                   | —                                            |
| Multi-seed/DHT               | ✅ Complete                   | —                                            |
| Observability & reliability  | ✅ Complete                   | Alerts armed on both seeds during soak       |
| Mobile light-node slices 4–8 | ✅ Landed 2026-08-25 (v0.2.0) | —                                            |
| Testnet soak                 | 🔴 **ACTIVE NEXT**           | Tuning review after 1–2 weeks of data        |
| RFC-004 HTLC / atomic swap   | ✅ Shipped 2026-09-08 (KVP-104) | Next: 5.2 Time-lock Vault (CSV/CLTV)       |
| Wallet UX                    | ✅ Complete                   | Fee-floor knob deferred (no soak congestion) |

---

## 🔗 References
- `myObsidianVaultDAG.md` §Post-Stage 3 Roadmap
- `kovanica-protocol/AGENTS.md` §Roadmap
- `kovanica-protocol/TODO.md` (if exists in source repo)
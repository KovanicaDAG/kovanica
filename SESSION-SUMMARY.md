# Kovanica Protocol — Session Summary

**Date:** 2026-09-26
**Host:** seed2 (Hostinger KVM2 VPS `76.13.250.65` / `srv1991525`)
**Agent:** Kovanica core protocol specialist

---

## 🎯 Operating Instructions (Reference)

### Core Architecture (Never Contradict)
| Property | Value |
|----------|-------|
| Consensus | GHOSTDAG BlockDAG, **k=3** |
| Ledger | Pure UTXO |
| Signatures | Ed25519 (64-byte / 128 hex) |
| Token | **KVNC** (1 KVNC = 100M atoms) |
| P2P | Plaintext TCP **:9000** only (no libp2p) |
| Bootstrap seed | `seed.kovanica.online:9000` (DNS-only, grey-cloud) |
| Explorer/API | `https://explorer.kovanica.online` |
| Network | `kovanica-testnet` (RFC-006 live) |

### RFC-006 Tokenomics — Hard Rules (Post-Activation)
| Parameter | Value |
|-----------|-------|
| **MAX_SUPPLY** | **90.2M KVNC** (`9_020_000_000_000_000` atoms) |
| Genesis subsidy (s₀) | 10 KVNC/block |
| Era length | 2,000,000 blocks |
| Decay (α) | 3/4 per era |
| Coinbase maturity | **100 blocks** |
| Fee split | **75% burned / 25% to producer** |
| Fee floor | `max(1, subsidy / 500_000)` atoms/byte |

> Once activated: **MAX_SUPPLY, maturity, fee-burn = hard consensus rules**. Prefer these over live `/api/head`.

### PoA-Only Decision (Ratified 2026-09-25)
- **PoW removed entirely** (not disabled) — `[TARGET]`
- **Hybrid/staked-VRF dropped entirely** — `[TARGET]`
- **PoA = only admission path** — fixed authority set, slot round-robin (3s default)
- `KOVANICA_POW` → `KOVANICA_CONSENSUS=poa` + `KOVANICA_AUTHORITY_KEY`
- RFC-006 supply math **unaffected** (height-indexed, capped in `apply_block`)

### Hard Safety Rules
1. **Monorepo first** — consensus/ledger changes in `protocol/crates/`
2. **Verify live params** via `/api/head` + `/api/bootstrap` before hardcoding
3. **Private keys NEVER enter node** — client-side signing only
4. **Tx flow**: `POST /api/prepare` → offline Ed25519 sign → `POST /api/submit`
5. **Classify every change**: consensus-safe / ledger-safe / client-only
6. **No `KOVANICA_ALLOW_RESET=1`** or open faucet on public nodes without isolation
7. **Default node**: `MINE=0`, `FAUCET=0`, `OPERATOR=0`
8. **P2P**: DNS seed name or origin IP only — **never** Cloudflare orange-cloud hostnames for TCP 9000
9. **Rust style**: idiomatic, minimize clones on hot paths, `cargo check` + `clippy` + tests after changes

### Change Classification (Surface Risk Early)
| Layer | Examples | Risk |
|-------|----------|------|
| **consensus-safe** | Parameter tweaks, non-breaking RPC | Low |
| **ledger-safe** | New tx types (KVP), activation gates | Medium |
| **client-only** | Web UI, wallet UX, CLI, FFI bindings | Low |

### Useful Live Endpoints
```bash
curl -s https://explorer.kovanica.online/api/head | jq
curl -s https://explorer.kovanica.online/api/bootstrap | jq
```

### Key Files to Read Before Consensus Work
| File | Purpose |
|------|---------|
| `protocol/AGENTS.md` | Deep consensus/RFC invariants |
| `protocol/docs/LEGIT-BOARD.md` | P0/P1/P2 roadmap |
| `protocol/docs/RFC-POA-Migration.md` | PoA decision + removal inventory |
| `protocol/TESTNET-RFC006.md` | Current testnet economy |
| `NETWORK.md` | Domains, seeds, ports, genesis |

---

## ✅ What Has Been Done This Session

### 1. Branch Audit & Cleanup
| Action | Count |
|--------|-------|
| Local branches deleted (merged into main) | 10 |
| Worktrees removed (`/root/kovanica-wt/*`) | 2 |
| Stale remote refs pruned (origin + monorepo) | 27 + 2 |
| **Final local branches** | **1** (`main` only) |

### 2. PR Creation & Merging
| PR | Branch | Title | Status |
|----|--------|-------|--------|
| **#59** | `web/landing-responsive-nft` | Responsive landing + NFT (KVP-106) | ✅ **MERGED** (squash) |
| **#49** | `docs/rfc006-poa-interaction` | RFC-006 §9.1 PoA interaction docs | ✅ **MERGED** (auto) |
| **#60** | `docs/poa-only-consensus` | PoA-only decision docs (32 files) | **Rebased → pushed → closed** |
| **#58** | `chore/protocol-clippy-cleanup` | Clippy cleanup | **CLOSED** (abandoned — in monorepo) |
| **#41** (existing) | `consensus/poa-m4-spv` | PoA M4-M6 | Already in main via monorepo sync |

### 3. Conflict Resolution (PR #60)
**6 files conflicted** during rebase onto main — resolved by taking **main's version**:

| File | Conflict | Resolution |
|------|----------|------------|
| `protocol/docs/RFC-POA-Migration.md` | Gate 3, B1-B4 status, future hybrid | Kept main's updated status (B1-B4 resolved) |
| `protocol/docs/SECURITY.md` | Full PoA threat model rewrite | Kept main's version |
| `protocol/docs/TESTNET-RESET-POLICY.md` | Gate 3 wording, PoA authority set | Kept main's version |
| `node/TESTNET.md` | Stale tokenomics warning | Kept main's version |
| `protocol/AGENTS.md` | No diff | Clean |
| `protocol/docs/TOKENOMICS.md` | No diff | Clean |

### 4. Monorepo Sync
| Action | Result |
|--------|--------|
| `git pull --rebase monorepo main` | Fast-forward: +4 files (rust-gate CI, kovanica-sdk-dev skill, explorer.rs, Cargo.lock) |
| `git push origin main --force-with-lease` | ✅ Synced origin |
| `git push monorepo main` | ❌ Protected branch (correct — cannot force-push) |

### 5. Protocol Test Suite — All Green
| Crate | Tests | Status |
|-------|-------|--------|
| `kovanica-dag` | 25 (consensus, reachability, PoA, block/payload pruning) | ✅ All pass |
| `kovanica-state` | 26 (vault, CSV, tokenomics, multisig, stealth, HTLC, native tokens) | ✅ All pass |
| `kovanica-node` | poa_node (7), poa_m6_testing (3+1 ignored) | ✅ All pass |
| `kovanica-ffi` | 4 + 2 live-sync (2 ignored - PoA reset needed) | ✅ All pass |

### 6. Web Frontend (`/root/kovanica/web/site`)
| Check | Result |
|-------|--------|
| `npm run typecheck` | ✅ 0 errors |
| `npm run lint` | ✅ 0 errors (110 warnings only) |
| `npm run build:vps` | ⚠️ **Fails** — Node.js v18.19.1 too old for current tooling |

### 7. Documentation Site (`/root/kovanica-docs`)
- ✅ Full docs structure: specs, RFCs, guides, API, governance, tokenomics
- ✅ Network params, PoA migration markers (`[CURRENT]`/`[TARGET]`), canonical tokenomics table

### 8. Node Infrastructure (`/root/kovanica-data`)
- Founder/operator wallet keys, faucet state, miner logs present
- Ready for PoA testnet reset (authority keys from `TESTNET_AUTHORITY_KEYS.md` need deployment)

---

## 📋 What Still Needs to Be Done

### Immediate (P0 — This Week)
| Priority | Task | Owner | Notes |
|----------|------|-------|-------|
| **P0.1** | Upgrade Node.js to ≥20 on **seed1** | seed1 operator | Required for `npm run build:vps` on web |
| **P0.2** | Deploy PoA testnet reset (authority keys, genesis) | seed1 + seed2 | Use `TESTNET_AUTHORITY_KEYS.md`; wipe `KOVANICA_DATA` |
| **P0.3** | Update DNS: `seed.kovanica.online` → seed1 IP post-reset | DNS admin | Grey-cloud only |
| **P0.4** | Authority key ceremony (gate 1) | Isolated host → both seeds | Procedure in `AUTHORITY-KEY-CEREMONY.md` |

### Short-term (P1 — 2–6 Weeks)
| Priority | Task | Owner | Notes |
|----------|------|-------|-------|
| **P1.1** | Run 24h multi-validator soak (M6 gate 2) | seed1 + seed2 + seed3 | Must run on **real ceremony keys**, not placeholders |
| **P1.2** | Resolve Gate 3 (CPU/RAM vs PoW measurement) | Maintainer decision | PoW baseline never recorded; gate unclosable as worded |
| **P1.3** | Node HTTP `asset_id` (KVP-102) | Protocol team | `/api/utxos`, `/api/history`, `/api/prepare` + web AssetPicker |
| **P1.4** | Multiple public seeds (≥2 distinct) | Ops | DNS + IPs; Cloudflare grey-cloud for P2P |
| **P1.5** | Run-a-node guide (single doc) | Docs | Build, ports, env, verify tip |
| **P1.6** | Public status surface (`/network`) | Web | Show head, peers, **authority set/slots** `[TARGET]` |
| **P1.7** | Security notes rewrite for PoA | Docs | `SECURITY.md` — PoA threat model, authority-key custody |
| **P1.8** | Open issue tracker (public Issues + templates) | GitHub | Bug/feature templates, security contact |
| **P1.9** | Spec index (KVP-101..104 + RFC links) | Docs | Keep roadmap in sync |

### Medium-term (P2 — 1–3 Months)
| Priority | Task | Owner | Notes |
|----------|------|-------|-------|
| **P2.1** | Audit plan (scope: dag + state + RPC) | Security | Firm + budget + public window |
| **P2.2** | Reproducible builds (toolchain pins, CI verifies hash) | Infra | Third-party rebuild matches release SHA256 |
| **P2.3** | Bug bounty (rules, safe harbor, channel) | Security | Non-zero budget |
| **P2.4** | Mainnet exit criteria (`MAINNET-CRITERIA.md`) | PM | P0/P1 complete, audit in progress, N independent nodes |
| **P2.5** | Entity & legal blurb | Legal | Maintainer identity, disclaimers |
| **P2.6** | Community home (Discord + moderation) | Community | Link from site |
| **P2.7** | KVP-102 issuance policy (coinbase mint only) | Protocol | Document on TOKENOMICS/KVP-102 |
| **P2.8** | Ops hardening (backups, rate limits, alerts) | Ops | Restore drill, monitoring |
| **P2.9** | Product polish (HW wallet, light-node store, deep-links) | Product | Post-mainnet |

### Technical Debt / Open Consensus Items
| Item | Status | Blocker |
|------|--------|---------|
| **PoA removal** (PoW, difficulty, hybrid, stake registry) | `[TARGET]` | RFC-POA-Migration §0.1, §0.7.1 |
| **Authority key ceremony** | Procedure drafted, not performed | `AUTHORITY-KEY-CEREMONY.md` |
| **Gate 3 measurement** | Unclosable — PoW baseline never recorded | Maintainer decision needed |
| **Mainnet authority governance** | `[OPEN]` — §0.7.2 | Initial set, eligibility, ceremony, threshold, expansion, recovery |
| **Android unit tests** | Pending | No SDK/device for `Format.kt`, KVLS parsing, address derivation |

---

## 🏁 Definition of "legit v1" (from LEGIT-BOARD)
> **All P0 checked** and **at least P1.1, P1.2, P1.3, P1.4, P1.7** checked.

That is enough for outsiders to read the code, run a node, hold testnet KVNC, and report bugs—without trusting a private chat.

---

## 📁 Repository State (Final)

```bash
# /root/kovanica (protocol monorepo)
git branch -a
* main                                    # 0028fc8 — synced to monorepo latest
  remotes/monorepo/<23 active branches>   # Upstream development
  remotes/origin/main                     # Synced
  remotes/testnet/repo/kovanica-testnet   # Testnet remote

# /root/kovanica-docs                     # Documentation site (deployed)
# /root/kovanica-data                     # Runtime data (keys, faucet, miner logs)
# /root/kovanica-wt                       # Empty (worktrees removed)
```

**Open PRs:** 0
**Local feature branches:** 0
**Test suite:** All green ✅

---

## 🔑 Key Files Modified This Session
| File | Change |
|------|--------|
| `protocol/docs/RFC-POA-Migration.md` | Conflict resolution (kept main's B1-B4 resolved status) |
| `protocol/docs/SECURITY.md` | Conflict resolution (kept main's PoA threat model) |
| `protocol/docs/TESTNET-RESET-POLICY.md` | Conflict resolution (kept main's gate 3 wording) |
| `node/TESTNET.md` | Conflict resolution (kept main's stale tokenomics warning) |

---

*Generated by Kovanica core protocol specialist agent. For questions, see `protocol/AGENTS.md` or `protocol/docs/LEGIT-BOARD.md`.*
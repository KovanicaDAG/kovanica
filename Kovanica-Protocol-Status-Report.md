# Kovanica Protocol — Complete Status Report
**Generated:** 2026-10-01  
**Inspection Scope:** All repositories under `/root`, testnet runtime, deploy configs, codebase health

---

## 1. Repository Inventory

| Path | Type | Status | Notes |
|------|------|--------|-------|
| `/root/kovanica/` | Monorepo | ✅ Healthy | 7 crates, 874 tests passing |
| `/root/kovanica-dashboard/` | Dashboard | ⚠️ Not running | Python proxy + React frontend |
| `/root/kovanica-testnet/` | Testnet Deploy | ⚠️ Partial | 3 seeds configured, 2 responding |
| `/root/kovanica-mainnet/` | Mainnet Template | 🔧 Empty | Authority keys dir empty |
| `/root/kovanica-devnet/` | Devnet Config | ✅ Ready | Local loopback config |
| `/root/kovanica-web/` | Web Frontend | ✅ Running | pm2 on port 3000 (TanStack) |
| `/root/secrets/` | Credentials | ✅ Present | seed-root-credentials.txt, .seed-pw-rotation-backup |

---

## 2. Monorepo Health (`/root/kovanica/protocol/`)

### Crates
| Crate | Purpose | Tests |
|-------|---------|-------|
| `kovanica-dag` | GHOSTDAG k=3 consensus | ✅ |
| `kovanica-state` | UTXO ledger, RFC-006 tokenomics | ✅ |
| `kovanica-node` | Explorer API, P2P, mempool | ✅ |
| `kovanica-cli` | Command-line interface | ✅ |
| `kovanica-wallet` | Wallet library | ✅ |
| `kovanica-ffi` | UniFFI bindings (Kotlin/Swift) | ✅ |
| `kovanica-chat` | P2P chat demo | ✅ |

**Build Gates:** `cargo fmt --check` ✅ | `cargo clippy -D warnings` ✅ | `cargo test --workspace` ✅ (874 tests) | `cargo build --release` ✅

### RFC Implementation Status
| RFC | KVP | Feature | Status |
|-----|-----|---------|--------|
| RFC-001 | KVP-101 | Multisig M-of-N P2SH | ✅ Shipped |
| RFC-002 | KVP-102 | Native Multi-Asset | ✅ Shipped |
| RFC-003 | KVP-103 | Stealth + Script v2 | ✅ Shipped |
| RFC-004 | KVP-104 | HTLC Atomic Swaps | ✅ Shipped |
| RFC-005 | KVP-105 | Time-Lock Vault + CSV | ✅ Shipped |
| RFC-006 | — | Tokenomics (90.2M cap, 100-block maturity, 75% fee burn) | ✅ Activated on testnet |

### Consensus Decision
- **PoA-only ratified:** 2026-09-25 (per `PROJECT-MANAGEMENT.md`, `SESSION-SUMMARY.md`)
- **PoW/Hybrid code:** Still in tree, marked `[TARGET]` for removal
- **Action needed:** Clean removal of PoW paths before mainnet

---

## 3. Testnet Runtime Status (CRITICAL)

### Seed Nodes
| Seed | IP | HTTP Port | P2P Port | Blocks | Genesis | Tip | Status |
|------|-----|-----------|----------|--------|---------|-----|--------|
| **seed1** | 145.223.116.178 | **8080 & 8081** | 9000 | N/A | 1a635915... | N/A | ❌ **CONFLICT + OOM** |
| **seed2** | 76.13.250.65 | 8082 | 9001 | 494 | 1a635915... | c858af37... | ✅ Running |
| **seed3** | 187.7.27.139 | 8083 | 9002 | 474 | 1a635915... | 2fe1f602... | ✅ Running |

### Critical Issues on seed1
1. **Two services fighting for genesis authority:**
   - `kovanica-seed1.service` → `/usr/local/bin/kovanica-node` on :8081 (MemoryLimit=10G)
   - `kovanica-explorer.service` → `/root/kovanica/protocol/target/release/kovanica-node` on :8080 (MemoryLimit=8G)
2. **Both OOM killed every ~5 minutes** (see `journalctl -u kovanica-seed1 -f` and `kovanica-explorer`)
3. **Metrics port conflict:** Both try to bind `:9090`
3. **Different binaries:** testnet-deploy installs to `/usr/local/bin/`, explorer uses local build

### Sync Status
- **seed2 ≠ seed3:** Different block heights (494 vs 474) and different tips
- **Headers-first sync failing** → falling back to full DAG dump (slow, memory-intensive)
- **Public DNS seed unreachable:** `seed.kovanica.online:9000` connection refused

### Public Endpoints
| Endpoint | Status |
|----------|--------|
| `https://explorer.kovanica.online` | ❌ Timeout |
| `https://testnet.kovanica.online` | ❌ 502 Bad Gateway |
| `https://api.kovanica.online` | ❌ Not tested |
| `https://dash.kovanica.online` | ❌ Nginx up, backend down (port 3001 not running) |

---

## 4. Dashboard (`/root/kovanica-dashboard/`)

| Component | Status | Details |
|-----------|--------|---------|
| Python proxy (`backend/server.py`) | ❌ Not running | Port 3001 |
| React frontend (`frontend/dist/`) | ✅ Built | Vite + Tailwind + Recharts |
| Nginx (`dash.kovanica.online`) | ✅ Config present | TLS pending |
| 17 Panels | ✅ Implemented | Overview, BlockDAG, Blocks, TXs, Addresses, Mempool, Network, Consensus, Tokens, HTLC, Multisig, Faucet, Mining, API Console, Metrics, Ops |

**To start:** `cd /root/kovanica-dashboard/backend && python3 server.py &` (or pm2)

---

## 5. Authority Keys

### Testnet (`/root/kovanica-testnet/authority-keys/`)
```
authorities.conf       # 3 pubkeys, threshold=2, slot_duration_ms=3000
authority-1.env        # Ed25519 seed (mode 0600)
authority-2.env        # Ed25519 seed (mode 0600)
authority-3.env        # Ed25519 seed (mode 0600)
```
**All keys mode 0600 ✅** — Properly secured per AGENTS.md

### Mainnet (`/root/kovanica-mainnet/mainnet-authority-keys/`)
- **Empty directory** — Governance process TBD
- 7 placeholder files exist in `kovanica-mainnet-deploy/authority-keys/` but no actual keys

---

## 6. Key Documents Summary

| Document | Key Content |
|----------|-------------|
| `INSPECT-REPORT.md` | No consensus P0 defects; 4 false positives in prior audit corrected |
| `SESSION-SUMMARY.md` | PoA decision, PR merges (#142, #143, #144), testnet reset recommended |
| `LEGIT-BOARD.md` | P0-P2 checklist: P0.1-P0.7, P1.1-P1.8, P2.1-P2.9 |
| `PROJECT-MANAGEMENT.md` | Milestones M1-M6; M1 target 2026-10-15, M2 RFC-006 hardening 2026-10-31 |
| `RELEASE-NOTES-v0.1.0-mobile.md` | Mobile apps (iOS/Android) released with key derivation fix |

---

## 7. Identified Issues & Required Actions

### P0 — Critical (Block testnet stability)
| # | Issue | Owner | Action |
|---|-------|-------|--------|
| 1 | seed1: Two conflicting services (kovanica-seed1 + kovanica-explorer) | node-ops | **Stop & disable kovanica-explorer**; keep only kovanica-seed1 on :8081 |
| 2 | seed1: OOM kills every ~5 min (10G limit) | node-ops | Investigate memory leak; increase limit or fix retention; add metrics port offset |
| 3 | seed2/seed3 not synced (different tips) | p2p-network | Diagnose headers-first failure; ensure DNS seed reachable; force full sync |
| 4 | DNS seed `seed.kovanica.online:9000` unreachable | deploy-ops | Verify seed1 P2P port 9000 binds after sync; check firewall/DNS |
| 5 | Public explorer endpoints down | deploy-ops | Restore explorer.kovanica.online & testnet.kovanica.online |

### P1 — High (Restore observability & operations)
| # | Issue | Owner | Action |
|---|-------|-------|--------|
| 6 | Dashboard not running (port 3001) | deploy-ops | Start python proxy via pm2/systemd |
| 7 | Metrics port 9090 conflict on seed1 | node-ops | Offset one service to 9091 |
| 8 | config/mainnet/network.env has 11 inert env vars | node-ops | Remove or document ignored vars |

### P2 — Medium (Hygiene & hardening)
| # | Issue | Owner | Action |
|---|-------|-------|--------|
| 9 | PoW/hybrid code still in tree | kovanica-core | Remove dead code paths marked `[TARGET]` |
| 10 | Mainnet authority keys empty | genesis-testnet | Governance process to generate |
| 11 | Testnet reset playbook not executed | genesis-testnet | Clean genesis per `TESTNET-RFC006.md` after seed1 fixed |

---

## 8. Immediate Next Steps (Priority Order)

1. **Fix seed1 conflict** — Stop `kovanica-explorer`, keep `kovanica-seed1` only
2. **Resolve seed1 OOM** — Profile memory, adjust limit, fix metrics port
3. **Sync seed2/seed3** — Ensure both connect to seed1 P2P :9000, verify full sync
4. **Verify DNS seed** — `seed.kovanica.online:9000` must accept TCP connections
5. **Start Dashboard** — `pm2 start /root/kovanica-dashboard/backend/server.py --name dashboard`
6. **Restore public explorers** — Deploy explorer frontend to explorer.kovanica.online
7. **Execute testnet reset** — Clean genesis with RFC-006 activated, per playbook

---

## 9. Milestone Tracker (from PROJECT-MANAGEMENT.md)

| Milestone | Target | Status | Exit Criteria |
|-----------|--------|--------|---------------|
| **M1: Testnet Stable** | 2026-10-15 | 🔴 At Risk | 3 seeds synced, explorer up, dashboard up, no OOM |
| **M2: RFC-006 Hardened** | 2026-10-31 | 🟡 Pending | Supply cap enforced, maturity 100, fee burn 75% verified |
| **M3: API Stability** | 2026-11-15 | ⏳ | OpenAPI spec frozen, SDKs generated |
| **M4: Wallet/UX** | 2026-11-30 | ⏳ | Web wallet + mobile parity |
| **M5: Advanced Features** | 2026-12-15 | ⏳ | NFT, DEX MVP, RWA |
| **M6: Mainnet Checklist** | 2027-01-15 | ⏳ | Security audit, governance keys, runbooks |

---

## 10. Environment Quick Reference

### Testnet Seed Environment (from `kovanica-testnet/env.sh`)
```bash
export KOVANICA_CONSENSUS=poa
export KOVANICA_ALLOW_RESET=0
export KOVANICA_FAUCET=1
export KOVANICA_MINE=0
export KOVANICA_OPERATOR=0
export KOVANICA_LISTEN=0.0.0.0:9000
export KOVANICA_PEERS=seed.kovanica.online:9000
export KOVANICA_HTTP=0.0.0.0:8081  # seed1; 8082/8083 for seed2/3
export KOVANICA_DATA=/root/kovanica-data  # seed1; /var/lib/kovanica-seed2/3 for others
export KOVANICA_AUTHORITIES_FILE=/root/kovanica-testnet/authority-keys/authorities.conf
export KOVANICA_AUTHORITY_KEY_FILE=/root/kovanica-testnet/authority-keys/authority-1.env  # per seed
```

### Useful Commands
```bash
# Check seed status
curl http://127.0.0.1:8081/api/head | jq
curl http://127.0.0.1:8082/api/head | jq
curl http://127.0.0.1:8083/api/head | jq

# View logs
journalctl -u kovanica-seed1 -f
journalctl -u kovanica-seed2 -f
journalctl -u kovanica-seed3 -f

# Dashboard
cd /root/kovanica-dashboard/backend && python3 server.py &

# Build verification
cd /root/kovanica/protocol && cargo test --workspace
```

---

## 11. Security Posture

| Check | Status | Notes |
|-------|--------|-------|
| Authority key permissions (0600) | ✅ | Testnet keys properly secured |
| Private keys client-side only | ✅ | Node never receives seeds |
| `ALLOW_RESET=0` on testnet | ✅ | Correct |
| `FAUCET=0` on mainnet template | ✅ | Correct |
| `.gitignore` excludes secrets | ✅ | Verified |
| P2P plaintext TCP:9000 only | ✅ | No libp2p/TLS |

---

**Report End** — This is a point-in-time snapshot. Re-run inspection after seed1 fix and sync completion.
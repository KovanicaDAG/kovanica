# Kovanica Protocol — Action Plan for Next Session
**Generated:** 2026-10-01  
**Context:** Current session is on **seed1** (145.223.116.178)  
**Goal:** Resolve all P0/P1 issues, make dashboard live, commit & push to GitHub

---

## 🔴 P0 — CRITICAL (Do First, Blocks Everything)

### 1. Fix seed1 OOM Kill Loop
**Current State:** `kovanica-seed1` at 9.9G/10G (effectively OOM), restart counter at 14+
**Root Cause:** Memory leak or excessive state retention in `kovanica-node` explorer mode

**Actions:**
```bash
# A. Check current memory profile
journalctl -u kovanica-seed1 --since "10 minutes ago" | grep -i "memory\|oom\|restart"

# B. Increase memory limit temporarily (while debugging)
systemctl set-property kovanica-seed1 MemoryMax=16G

# C. Restart clean
systemctl restart kovanica-seed1

# D. Monitor for 10+ minutes
watch -n 10 'systemctl status kovanica-seed1 | grep Memory'
```

**If OOM persists:** Profile Rust allocations — build with `cargo build --release --features profiling` or use `heaptrack`/`valgrind`. Check `kovanica-state` DAG retention policy.

---

### 2. Verify Metrics Port 9090 Now Free
```bash
ss -ltnp | grep 9090
# Should show only kovanica-seed1 PID
curl http://127.0.0.1:9090/metrics  # Should return Prometheus metrics
```

---

### 3. Sync seed2 & seed3 to seed1
**On seed1 (this machine):**
```bash
# Verify P2P port 9000 binds after sync starts
ss -ltnp | grep 9000

# Check connected peers
curl -s http://127.0.0.1:8081/api/p2p | jq .
```

**On seed2 (76.13.250.65):**
```bash
ssh root@76.13.250.65 "
  systemctl restart kovanica-seed2 &&
  sleep 30 &&
  curl -s http://127.0.0.1:8082/api/head | jq .
"
```

**On seed3 (187.7.27.139):**
```bash
ssh root@187.7.27.139 "
  systemctl restart kovanica-seed3 &&
  sleep 30 &&
  curl -s http://127.0.0.1:8083/api/head | jq .
"
```

**Verify sync:**
```bash
# All three should show same tip and similar height
for port in 8081 8082 8083; do
  curl -s http://127.0.0.1:$port/api/head | jq '{tip: .tip, height: .blue_score, supply: .supply}'
done
```

---

### 4. Fix DNS Seed `seed.kovanica.online:9000`
**Root Cause:** seed1 P2P port 9000 not binding until sync completes, or DNS not pointing here.

**Actions:**
```bash
# Check DNS resolves to seed1 IP
dig +short seed.kovanica.online

# Verify port 9000 accessible externally
# From another machine: nc -zv seed.kovanica.online 9000

# If DNS wrong: Update Cloudflare DNS to point to 145.223.116.178
```

---

## 🟠 P1 — HIGH (Observability & Operations)

### 5. Make Dashboard Live (Port 3001)
```bash
cd /root/kovanica-dashboard/backend

# Option A: Quick test (foreground)
python3 server.py &

# Option B: Persistent via pm2 (recommended)
pm2 start server.py --name dashboard --watch
pm2 save
pm2 startup  # Run once, follow instructions

# Verify
curl http://127.0.0.1:3001/healthz  # Should return 200 OK
curl http://127.0.0.1:3001/api/head  # Proxy to seed1
```

**Dashboard Panels to verify:** Overview, BlockDAG, Blocks, Transactions, Addresses, Mempool, Network, Consensus, Tokens, HTLC, Multisig, Faucet, Mining, API Console, Metrics, Ops (17 total)

---

### 6. Restore Public Explorers
**explorer.kovanica.online & testnet.kovanica.online**

```bash
# Check current nginx configs
ls /etc/nginx/sites-enabled/

# Likely need to deploy explorer frontend (kovanica-web or separate explorer build)
# to /var/www/explorer and /var/www/testnet-explorer

# Quick test: point to kovanica-web on port 3000 temporarily
# Edit nginx config to proxy_pass http://127.0.0.1:3000 for explorer subdomain
```

---

### 7. Clean Inert Env Vars in `config/mainnet/network.env`
```bash
cat /root/kovanica/mainnet/config/network.env
# Remove 11 vars the binary never reads (documented in AGENTS.md Known Issues)
# Keep only: KOVANICA_CONSENSUS, KOVANICA_ALLOW_RESET, KOVANICA_FAUCET, KOVANICA_MINE,
#            KOVANICA_OPERATOR, KOVANICA_LISTEN, KOVANICA_PEERS, KOVANICA_HTTP,
#            KOVANICA_DATA, KOVANICA_AUTHORITIES_FILE, KOVANICA_AUTHORITY_KEY_FILE,
#            KOVANICA_PRODUCE, KOVANICA_PRODUCE_SECS, KOVANICA_ISOLATED_HOST
```

---

## 🟡 P2 — MEDIUM (Hygiene)

### 8. Remove PoW/Hybrid Dead Code (Marked `[TARGET]`)
```bash
# Search for TARGET markers
grep -r "\[TARGET\]" /root/kovanica/protocol --include="*.rs" | head -20

# Focus: kovanica-node mining code, kovanica-dag PoW difficulty, consensus selection
# Create PR with clean removal
```

### 9. Mainnet Authority Keys — Governance TBD
```bash
# /root/kovanica-mainnet/mainnet-authority-keys/ is empty
# /root/kovanica-mainnet-deploy/authority-keys/ has 7 placeholders
# Action: Document governance process, generate keys when ready
```

---

## 📦 COMMIT & PUSH TO GITHUB

### Files to Commit (This Session)
```bash
# Status report & flat file created this session
/root/Kovanica-Protocol-Status-Report.md
/root/KOVANICA-SEEDS-FLAT-FILE.txt
/root/Kovanica-Action-Plan.md   # This file

# Any fixes made to configs
/etc/systemd/system/kovanica-seed1.service  (if modified)
/etc/systemd/system.control/kovanica-seed1.service.d/50-MemoryMax.conf  (if modified)
/root/kovanica-testnet/env.sh  (if modified)
```

### Git Workflow
```bash
cd /root/kovanica-testnet
git status
git add -A
git commit -m "fix: stop conflicting kovanica-explorer service on seed1; increase memory limit; docs: add status report and action plan"

cd /root/kovanica
git status
git add -A
git commit -m "docs: add protocol status report, seeds flat file, action plan"

# Push to deploy remote (primary per AGENTS.md)
git push deploy main
# Also push to origin if needed
git push origin main
```

---

## ✅ VERIFICATION CHECKLIST (Run Before Session End)

| Check | Command | Expected |
|-------|---------|----------|
| seed1 stable >10 min | `watch -n 30 'systemctl status kovanica-seed1'` | No restarts, Memory < 8G |
| Metrics working | `curl -s 127.0.0.1:9090/metrics \| head -5` | Prometheus output |
| seed2 synced | `curl -s 76.13.250.65:8082/api/head \| jq .tip` | Matches seed1 tip |
| seed3 synced | `curl -s 187.7.27.139:8083/api/head \| jq .tip` | Matches seed1 tip |
| Dashboard up | `curl -s 127.0.0.1:3001/healthz` | 200 OK |
| Dashboard panels | Open `http://seed1-ip:3001` in browser | All 17 panels load |
| Public explorer | `curl -s https://explorer.kovanica.online/api/head` | 200 + JSON |
| DNS seed reachable | `nc -zv seed.kovanica.online 9000` | Connected |
| Git pushed | `git log --oneline -1` | Shows new commits |

---

## 🔑 CREDENTIALS REFERENCE (From Flat File)

| Seed | IP | SSH Pass (Current) | SSH Pass (Rotated) |
|------|-----|-------------------|-------------------|
| seed1 | 145.223.116.178 | `1234567890` | NOT ROTATED |
| seed2 | 76.13.250.65 | `1234567890` | `krgdk0s9fbLv=zurbVBaJ+_zYF8rgg` |
| seed3 | 187.7.27.139 | `KVNCprotocol@DAGy1` | `q8pv3yO7Z^iUTCgEPJtVgovNcOv7WB` |

**Authority Keys:** `/root/kovanica-testnet/authority-keys/` (all 0600)

---

## 📋 QUICK COMMANDS CHEATSHEET

```bash
# --- seed1 (this machine) ---
systemctl status kovanica-seed1
systemctl restart kovanica-seed1
journalctl -u kovanica-seed1 -f
curl -s http://127.0.0.1:8081/api/head | jq
curl -s http://127.0.0.1:8081/api/p2p | jq
ss -ltnp | grep -E '9000|9090|8081'

# --- seed2 ---
ssh root@76.13.250.65 "systemctl status kovanica-seed2"
ssh root@76.13.250.65 "systemctl restart kovanica-seed2"
ssh root@76.13.250.65 "curl -s http://127.0.0.1:8082/api/head | jq"

# --- seed3 ---
ssh root@187.7.27.139 "systemctl status kovanica-seed3"
ssh root@187.7.27.139 "systemctl restart kovanica-seed3"
ssh root@187.7.27.139 "curl -s http://127.0.0.1:8083/api/head | jq"

# --- Dashboard ---
cd /root/kovanica-dashboard/backend
pm2 start server.py --name dashboard
pm2 logs dashboard
pm2 status

# --- Build verification ---
cd /root/kovanica/protocol
cargo test --workspace
cargo build --release --workspace

# --- Git ---
cd /root/kovanica-testnet && git push deploy main
cd /root/kovanica && git push deploy main
```

---

## 🎯 MILESTONE M1 TARGET: 2026-10-15
**Exit Criteria:**
- [ ] 3 seeds synced (same tip, height within 5 blocks)
- [ ] No OOM kills for 24 hours
- [ ] explorer.kovanica.online responding
- [ ] testnet.kovanica.online responding
- [ ] Dashboard fully functional (all 17 panels)
- [ ] DNS seed reachable on TCP:9000

---

**Next Session Priority Order:**
1. Fix seed1 OOM (monitor 30+ min stable)
2. Sync seed2 & seed3
3. Start Dashboard
4. Fix DNS seed
5. Restore public explorers
6. Commit & push

---

**End of Action Plan** — Execute sequentially. Each P0 must pass before moving to next.
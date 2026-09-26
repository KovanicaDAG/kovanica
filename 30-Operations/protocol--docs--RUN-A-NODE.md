---
title: "Run a Kovanica Node — Complete Guide"
category: 30-Operations
source: protocol/docs/RUN-A-NODE.md
synced: 2026-09-26
---
# Run a Kovanica Node — Complete Guide

> **Testnet software — no investment advice. Funds can be lost.**

---

## Quick Start (Prebuilt Binary)

```bash
# 1. Install from GitHub Release (Linux x86_64)
curl -sSfL https://raw.githubusercontent.com/KovanicaDAG/kovanica-node/main/scripts/install.sh | bash

# 2. Run as explorer (HTTP API + P2P + mining)
KOVANICA_PEERS=seed.kovanica.online:9000,seed2.kovanica.online:9000 \
KOVANICA_LISTEN=0.0.0.0:9000 KOVANICA_POW=1 \
kovanica-node explorer 127.0.0.1:8080
```

**That's it.** Your node will sync, mine blocks every ~60s, and serve:
- Explorer UI: `http://127.0.0.1:8080`
- HTTP API: `http://127.0.0.1:8080/api/*`
- Prometheus metrics: `http://127.0.0.1:9090/metrics`

---

## Table of Contents

1. [Installation Methods](#1-installation-methods)
2. [Network Profiles](#2-network-profiles)
3. [Configuration](#3-configuration)
4. [Running as a Seed](#4-running-as-a-seed)
5. [Running as Explorer Only](#5-running-as-explorer-only)
6. [P2P Networking](#6-p2p-networking)
7. [Monitoring & Metrics](#7-monitoring--metrics)
8. [Backup & Restore](#8-backup--restore)
9. [Troubleshooting](#9-troubleshooting)
10. [Verifying Sync](#10-verifying-sync)

---

## 1. Installation Methods

### 1.1 Prebuilt Binary (Recommended)

```bash
# Linux x86_64
curl -sSfL https://raw.githubusercontent.com/KovanicaDAG/kovanica-node/main/scripts/install.sh | bash

# Linux ARM64
curl -sSfL https://raw.githubusercontent.com/KovanicaDAG/kovanica-node/main/scripts/install.sh | bash -s -- --arch aarch64

# macOS (Intel)
curl -sSfL https://raw.githubusercontent.com/KovanicaDAG/kovanica-node/main/scripts/install.sh | bash -s -- --arch x86_64 --os darwin

# macOS (Apple Silicon)
curl -sSfL https://raw.githubusercontent.com/KovanicaDAG/kovanica-node/main/scripts/install.sh | bash -s -- --arch aarch64 --os darwin
```

**Installs to:** `/usr/local/bin/kovanica-node` (and `kovanica-cli` if present)
**Verifies:** SHA256 checksum from GitHub Release

### 1.2 Build from Source

```bash
# Prerequisites: Rust 1.75+, cargo, git
git clone https://github.com/KovanicaDAG/kovanica-node
cd kovanica-node
cargo build --release --workspace

# Binary at: target/release/kovanica-node
```

### 1.3 Docker (Experimental)

```bash
docker run -d \
  --name kovanica-node \
  -p 9000:9000 -p 8080:8080 -p 9090:9090 \
  -v kovanica-data:/data \
  -e KOVANICA_PEERS=seed.kovanica.online:9000,seed2.kovanica.online:9000 \
  -e KOVANICA_LISTEN=0.0.0.0:9000 \
  -e KOVANICA_POW=1 \
  ghcr.io/kovanicadag/kovanica-node:latest \
  explorer 127.0.0.1:8080
```

---

## 2. Network Profiles

Each profile owns a **separate data directory** — a mainnet node cannot destroy testnet state.

| Profile | Data Dir | Genesis | Status |
|---------|----------|---------|--------|
| `kovanica-testnet` | `/root/kovanica-data` or `./data` | `9565fc20cb465eec0198a65c07da6b825e4211c4060d581a2c7dac6c96bafc97` | **Active** |
| `kovanica-mainnet` | `/root/kovanica-mainnet-data` | TBD | **Dormant** |

**Select at boot:**
```bash
# Testnet (default)
KOVANICA_NETWORK=kovanica-testnet kovanica-node explorer 127.0.0.1:8080

# Mainnet (requires override — parameters not finalized)
KOVANICA_NETWORK=kovanica-mainnet KOVANICA_MAINNET_OVERRIDE=1 kovanica-node explorer 127.0.0.1:8080
```

---

## 3. Configuration

All config via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `KOVANICA_DATA` | `./data` | Data directory (chain state, wallets) |
| `KOVANICA_NETWORK` | `kovanica-testnet` | Network profile |
| `KOVANICA_LISTEN` | `0.0.0.0:9000` | P2P listen address |
| `KOVANICA_PEERS` | `seed.kovanica.online:9000,seed2.kovanica.online:9000` | Bootstrap peers |
| `KOVANICA_POW` | `1` (testnet) | Enable proof-of-work mining |
| `KOVANICA_MINE` | `1` (explorer profile) | Auto-mine empty blocks |
| `KOVANICA_MINE_SECS` | `60` | Target block interval when mining |
| `KOVANICA_FAUCET` | `0` | Enable faucet (testnet explorer only) |
| `KOVANICA_HYBRID` | `0` | Enable hybrid PoW+staked admission |
| `KOVANICA_OPERATOR` | `0` | Enable operator wallet (mining rewards) |
| `KOVANICA_ALLOW_RESET` | `0` | Allow genesis reset (dev only) |
| `KOVANICA_TREASURY_SEED` | — | 64-hex mainnet treasury seed (required for mainnet) |

### 3.1 Example: Testnet Seed with Mining

```bash
export KOVANICA_DATA=/root/kovanica-data
export KOVANICA_NETWORK=kovanica-testnet
export KOVANICA_LISTEN=0.0.0.0:9000
export KOVANICA_PEERS=seed.kovanica.online:9000,seed2.kovanica.online:9000
export KOVANICA_POW=1
export KOVANICA_MINE=1
export KOVANICA_MINE_SECS=60
export KOVANICA_OPERATOR=1
export KOVANICA_FAUCET=1  # explorer only

kovanica-node explorer 127.0.0.1:8080
```

### 3.2 Example: Light Node (No Mining)

```bash
export KOVANICA_DATA=/root/kovanica-data
export KOVANICA_PEERS=seed.kovanica.online:9000
export KOVANICA_POW=0

kovanica-node serve  # REPL mode, or
kovanica-node explorer 127.0.0.1:8080  # with HTTP API
```

---

## 4. Running as a Seed

### 4.1 Systemd Service (Production)

```ini
# /etc/systemd/system/kovanica-seed.service
[Unit]
Description=Kovanica seed node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=/root/kovanica-data
Environment=KOVANICA_LISTEN=0.0.0.0:9000
Environment=KOVANICA_POW=1
Environment=KOVANICA_PEERS=seed.kovanica.online:9000,seed2.kovanica.online:9000
Environment=KOVANICA_OPERATOR=1
Environment=KOVANICA_MINE=1
Environment=KOVANICA_MINE_SECS=60
Environment=KOVANICA_FAUCET=0
Environment=KOVANICA_DATA=/root/kovanica-data
ExecStart=/usr/local/bin/kovanica-node explorer 127.0.0.1:8080
Restart=always
RestartSec=5
LimitNOFILE=65536
MemoryMax=10G

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now kovanica-seed
```

### 4.2 Firewall

```bash
# Only P2P port needs to be public
sudo ufw allow 9000/tcp comment 'Kovanica P2P'

# Explorer + metrics stay on loopback (or behind nginx)
# Do NOT open 8080/9090 to the internet directly
```

### 4.3 Cloudflare DNS (Grey-Cloud)

```bash
# DNS-only (no proxy) for P2P
Type: A
Name: seed
Content: <YOUR_VPS_IP>
Proxy: DNS only (grey cloud)
```

---

## 5. Running as Explorer Only

```bash
# No mining, just sync + HTTP API
KOVANICA_POW=0 KOVANICA_MINE=0 KOVANICA_FAUCET=1 \
kovanica-node explorer 127.0.0.1:8080
```

**NGINX Reverse Proxy (for public explorer):**

```nginx
# /etc/nginx/sites-enabled/explorer.kovanica.online
server {
    listen 80;
    server_name explorer.kovanica.online;
    
    # API & metrics
    location /api/ { proxy_pass http://127.0.0.1:8080; }
    location /metrics { proxy_pass http://127.0.0.1:8080; }
    
    # WebSocket
    location /ws { proxy_pass http://127.0.0.1:8080; proxy_http_version 1.1; proxy_set_header Upgrade $http_upgrade; proxy_set_header Connection "upgrade"; }
    
    # Static pages
    location / { proxy_pass http://127.0.0.1:3000; }  # kovanica-web on :3000
}
```

---

## 6. P2P Networking

### 6.1 Bootstrap Peers

| Seed | Address | Notes |
|------|---------|-------|
| seed1 | `seed.kovanica.online:9000` | Primary (Hostinger VPS) |
| seed2 | `seed2.kovanica.online:9000` | Secondary (Hostinger KVM2 VPS) |
| seed3 | `seed3.kovanica.online:9000` | Tertiary (retired) |

### 6.2 Connectivity Verification

```bash
# Check peer count
curl -s http://127.0.0.1:8080/api/head | jq .peers

# Expected: 2+ peers (other seeds)
# If 0: check firewall, DNS, port 9000
```

### 6.3 DHT Discovery

```bash
# Manual connect (bypasses DNS)
kovanica-node connect 0.0.0.0:9000 <PEER_ID> <PEER_IP>:9000
```

---

## 7. Monitoring & Metrics

### 7.1 Prometheus Scraping

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'kovanica-seed'
    static_configs:
      - targets: ['127.0.0.1:9090']  # node metrics
        labels:
          instance: 'seed1'
  - job_name: 'kovanica-explorer'
    static_configs:
      - targets: ['127.0.0.1:8080']  # explorer also exposes /metrics
        labels:
          instance: 'explorer'
```

### 7.2 Key Metrics

| Metric | Description | Alert Threshold |
|--------|-------------|-----------------|
| `kovanica_block_height` | Local chain height | Stalled > 5 min |
| `kovanica_dag_blue_score` | Blue score (finality clock) | Stalled |
| `kovanica_peer_count` | Connected P2P peers | < 2 |
| `kovanica_dag_reorg_depth_total` | Total reorg depth | > 0 |
| `kovanica_mempool_tx_count` | Pending transactions | — |
| `kovanica_blocks_produced_total` | Blocks mined | — |

### 7.3 Grafana Dashboard

Import `kovanica-protocol/monitoring/kovanica-dashboard.json` (if available) or build panels from metrics above.

---

## 8. Backup & Restore

### 8.1 Create Backup

```bash
# From repo root
KOV_BACKUP_PASSPHRASE="$(cat /run/secrets/kov-backup-passphrase)" \
  ./scripts/backup-node.sh

# Or specify data dir
KOV_BACKUP_PASSPHRASE="..." ./scripts/backup-node.sh --data /root/kovanica-data
```

**Output:** Encrypted archives in `/root/kovanica-backups/` (700 dir, 600 files)

### 8.2 Restore

```bash
# Auto-picks newest backup
KOV_BACKUP_PASSPHRASE="..." ./scripts/restore-node.sh --data-dir /root/kovanica-data

# Specific archives
KOV_BACKUP_PASSPHRASE="..." ./scripts/restore-node.sh \
  --data-archive /root/kovanica-backups/...data-....tar.gz.enc \
  --seed-archive /root/kovanica-backups/...seeds-....tar.gz.enc \
  --data-dir /root/kovanica-data
```

### 8.3 Quarterly Restore Drill

```bash
mkdir -p /tmp/kov-restore-drill
KOV_BACKUP_PASSPHRASE="..." ./scripts/restore-node.sh \
  --data-dir /tmp/kov-restore-drill/data --force

# Verify
KOVANICA_DATA=/tmp/kov-restore-drill/data KOVANICA_PEERS=seed.kovanica.online:9000,seed2.kovanica.online:9000 \
  /usr/local/bin/kovanica-node explorer 127.0.0.1:18081 &
curl -s http://127.0.0.1:18081/api/head | jq .genesis
# Must match live genesis
```

---

## 9. Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `dial tcp :22 i/o timeout` on deploy | GitHub runner IP filtered by VPS | Use SSH port 2222 |
| `ETXTBSY` on binary replace | Running process holds binary | `systemctl stop` → `cp` → `systemctl start` |
| Genesis mismatch on seed deploy | Deploy script expects old genesis | Update `EXPECTED_GENESIS` in `deploy-seed-prebuilt.sh` |
| `kovanica-node` OOM killed | < 2GB RAM on seed3 | Resize to ≥2GB (t3.small or Oracle Always Free) |
| Peer count 0 | Port 9000 blocked / Cloudflare proxy | Open 9000/tcp; grey-cloud DNS |
| Block height stalled | No miner running | Set `KOVANICA_MINE=1` on at least one seed |
| `KOVANICA_MAINNET_OVERRIDE` required | Mainnet profile dormant | Parameters not finalized; don't use in prod |

---

## 10. Verifying Sync

### 10.1 Genesis Match

```bash
curl -s https://explorer.kovanica.online/api/head | jq -r .genesis
# Must equal: 9565fc20cb465eec0198a65c07da6b825e4211c4060d581a2c7dac6c96bafc97
```

### 10.2 Block Production

```bash
# Should increment every ~60s
watch -n 10 'curl -s http://127.0.0.1:8080/api/head | jq .blocks'
```

### 10.3 Smoke Tests

| Test | Command |
|------|---------|
| Faucet | `curl -X POST http://127.0.0.1:8080/api/faucet -H "Content-Type: application/json" -d '{"address":"kvnc1A4XLkrefPBsXLwRH7kcRutGm3pgzrC7zJvAf8uiLLHqgdag"}'` |
| Transfer | `kovanica-node send <from-seed> <amount> <to-address>` |
| Multisig | Create 2-of-2, fund, spend with 2 sigs |
| HTLC | Create, redeem with preimage |
| Vault | Create with CSV, wait maturity, release |

---

## Appendix: Useful Commands

```bash
# Health checks
curl -s http://127.0.0.1:8080/api/head          # seed head
systemctl status kovanica-seed                  # systemd status
curl -s http://127.0.0.1:9090/metrics | head    # Prometheus metrics

# Logs
journalctl -u kovanica-seed -f                  # follow logs
journalctl -u kovanica-seed --since "1 hour ago" # recent logs

# Cold bootstrap (pristine sync)
KOVANICA_DATA=/tmp/cbt KOVANICA_LISTEN=127.0.0.1:19000 \
KOVANICA_PEERS=seed.kovanica.online:9000,seed2.kovanica.online:9000 \
/usr/local/bin/kovanica-node explorer 127.0.0.1:18081

# RPC REPL
kovanica-node serve
> help
> balance <address>
> send <from-seed> <amount> <to-address>
```

---

## Related Documents

- `SECURITY.md` — Threat model, key handling, finality
- `OPERATIONS.md` — Seed runbook, deploy pipeline, incident lessons
- `NETWORK.md` — Domain map, DNS, redirect rules
- `LEGIT-BOARD.md` — Public visibility checklist
- `TOKENOMICS.md` — Emission curve, supply parameters

---

*Last updated: 2026-09-17*
*Testnet: `kovanica-testnet` — Genesis: `9565fc20cb465eec0198a65c07da6b825e4211c4060d581a2c7dac6c96bafc97`*
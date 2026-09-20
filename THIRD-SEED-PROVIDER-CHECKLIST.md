# Third Seed Provider Checklist (C3 Follow-up)

> **Goal**: Provision a third seed on a **different provider / continent / ASN** from existing seeds for geo/organizational diversity.
> **Current seeds**: seed1 (Hostinger VPS, EU, AS47583) + seed2 (Hostinger KVM2 VPS, EU, AS47583) — **same provider/ASN!**
> **Target**: 3rd provider in NA or APAC, different ASN.

---

## Provider Evaluation Matrix

| Provider | Region Options | vCPU/RAM/SSD (min) | Price/mo (est) | IPv6 | P2P Port 9000 | Notes |
|----------|----------------|-------------------|----------------|------|---------------|-------|
| **Vultr** | NJ, IL, ATL, LAX, SEA, TYO, SGP, SYD, FRA, AMS | 2/4GB/100GB | $24 | ✅ | ✅ | Good API, block storage |
| **DigitalOcean** | NYC, SFO, TOR, LON, FRA, SGP, BLR, SYD | 2/4GB/80GB | $24 | ✅ | ✅ | VPC, monitoring |
| **Hetzner** | NBG, FSN, HEL, ASH, HIL | 2/4GB/160GB | €15 | ✅ | ✅ | Excellent price/perf, EU only |
| **Linode (Akamai)** | NYC, ATL, DAL, SEA, LON, FRA, SGP, TYO, SYD | 2/4GB/100GB | $24 | ✅ | ✅ | Good support |
| **Oracle Cloud** | US-East, US-West, EU, APAC | 4/24GB/200GB (Free) | $0 | ✅ | ⚠️ | ARM, quota limits |
| **AWS Lightsail** | US, EU, APAC | 2/4GB/80GB | $20 | ❌ | ✅ | Simple, no IPv6 |
| **GCP e2-medium** | US, EU, APAC | 2/4GB/100GB | ~$25 | ✅ | ✅ | Sustained use discount |

**Recommendation**: **Vultr** (NA: NJ/LAX, APAC: TYO/SGP) or **DigitalOcean** (NA: NYC/SFO) for true geo/ASN diversity from Hostinger (AS47583).

---

## Seed Requirements

| Item | Spec | Notes |
|------|------|-------|
| **Hostname** | `seed3.kovanica.online` | DNS A record → new IP (grey-cloud) |
| **P2P Port** | 9000 (TCP) | Grey-cloud only; Cloudflare proxy breaks raw TCP |
| **Metrics** | 9090 (Prometheus) | Loopback only, scraped by central Prometheus |
| **Explorer** | 8080 (HTTP) | Loopback only; nginx reverse proxy if public |
| **SSH** | Key-only, port 22/2222 | Disable password auth |
| **OS** | Ubuntu 22.04/24.04 or Amazon Linux 2023 | Rust 1.82+ must build |
| **Resources** | 2 vCPU, 4GB RAM, 100GB SSD | Minimum; 4 vCPU/8GB recommended for build |
| **Systemd Unit** | `kovanica-seed3` | Mining enabled (60s interval) |
| **Data Dir** | `/var/lib/kovanica-seed3` | Wiped on fresh deploy |
| **Backup** | Daily encrypted to off-site | Per OPS-HARDENING.md |
| **Monitoring** | Prometheus + Alertmanager | Central scrape + Discord webhook |

---

## Deploy Options

### Option A: Prebuilt Binary (Fastest, Recommended for Testnet Resets)
```bash
# From operator machine (protocol/ root):
./scripts/deploy-seed-prebuilt.sh <user>@<new-ip> \
  --name seed3 \
  --peers "seed.kovanica.online:9000,seed2.kovanica.online:9000" \
  --mine \
  --mine-secs 60 \
  --binary ./target/release/kovanica-node
```

### Option B: Build from Source on Target (For Fresh Provider/Arch)
```bash
# From operator machine (protocol/ root):
./scripts/deploy-seed.sh <user>@<new-ip> \
  --name seed3 \
  --peers "seed.kovanica.online:9000,seed2.kovanica.online:9000" \
  --mine \
  --mine-secs 60
```

### Option C: ARM64 (Graviton/Oracle/ARM VPS)
```bash
# Cross-compile locally:
cd /root/kovanica/node
cargo build --release -p kovanica-node --target aarch64-unknown-linux-gnu
# Or on target with cargo-ndk / rustup target add aarch64-unknown-linux-gnu

# Deploy:
./scripts/deploy-seed-prebuilt.sh <user>@<new-ip> \
  --name seed3 \
  --binary ./target/aarch64-unknown-linux-gnu/release/kovanica-node
```

---

## Post-Deploy Verification Checklist

| Check | Command | Expected |
|-------|---------|----------|
| **Genesis match** | `curl -s http://127.0.0.1:8080/api/head | jq -r .genesis` | `9565fc20cb465eec0198a65c07da6b825e4211c4060d581a2c7dac6c96bafc97` |
| **Peer connectivity** | `curl -s http://127.0.0.1:8080/api/p2p` | Shows `seed.kovanica.online:9000` + `seed2.kovanica.online:9000` in peers |
| **Block production** | `watch -n 10 'curl -s http://127.0.0.1:8080/api/head | jq .blocks'` | Increments ~1/min |
| **Metrics scrape** | `curl -s 127.0.0.1:9090/metrics \| grep kovanica_block_height` | Returns height > 0 |
| **Prometheus scrape** | Central Prometheus UI → Targets → `kovanica-node-seed3` | State: UP |
| **P2P port open** | `nmap -p 9000 <IP>` | 9000/tcp open |
| **DNS resolve** | `dig seed3.kovanica.online A` | Returns new IP |
| **Systemd status** | `systemctl status kovanica-seed3` | active (running) |
| **Logs clean** | `journalctl -u kovanica-seed3 -n 50 --no-pager` | No ERROR/panic |

---

## DNS Configuration (Cloudflare)

| Record | Type | Content | Proxy |
|--------|------|---------|-------|
| `seed3.kovanica.online` | A | `<NEW_IP>` | **DNS only (grey cloud)** |
| `seed3.kovanica.online` | AAAA | `<IPv6_IF_AVAILABLE>` | **DNS only** |

> **Critical**: Grey-cloud only! Cloudflare proxy (orange cloud) breaks raw TCP P2P on port 9000.

---

## Monitoring Integration

### Prometheus (central)
Add to `/etc/prometheus/prometheus.yml`:
```yaml
- job_name: 'kovanica-node-seed3'
  static_configs:
    - targets: ['<NEW_IP>:9090']
      labels:
        instance: 'seed3.kovanica.online'
        operator: 'KovanicaDAG'
        region: '<REGION_CODE>'  # e.g., us-east, ap-northeast
```

### Alertmanager → Discord
Ensure `alerting_rules.yml` includes seed3 (already uses instance label).

### Backup Timer
```bash
# On seed3, as root:
cat > /etc/systemd/system/kovanica-backup.service <<'EOF'
[Unit]
Description=Kovanica backup
After=network-online.target
[Service]
Type=oneshot
Environment=KOV_BACKUP_PASSPHRASE_FILE=/run/secrets/kov-backup-passphrase
ExecStart=/root/kovanica/protocol/scripts/backup-node.sh --data /var/lib/kovanica-seed3
EOF

cat > /etc/systemd/system/kovanica-backup.timer <<'EOF'
[Unit]
Description=Daily Kovanica backup
[Timer]
OnCalendar=daily
Persistent=true
[Install]
WantedBy=timers.target
EOF

systemctl daemon-reload && systemctl enable --now kovanica-backup.timer
```

---

## Sign-off Checklist (Per Provider)

- [ ] Provider selected & VPS provisioned
- [ ] IP recorded, DNS A/AAAA records created (grey-cloud)
- [ ] SSH key deployed, password auth disabled
- [ ] OS updated, Rust 1.82+ installed
- [ ] Seed deployed (Option A/B/C)
- [ ] Genesis match verified
- [ ] Peer connectivity verified (≥2 peers)
- [ ] Block production verified (1 block/min)
- [ ] Metrics scrape working (central Prometheus UP)
- [ ] Alertmanager firing test alert
- [ ] Backup timer enabled, first backup verified
- [ ] DNS grey-cloud confirmed (no orange cloud)
- [ ] Operator added to Discord ops channel
- [ ] Documented in TESTNET-SOAK.md and MASTER-ROADMAP.md

---

## Next Seed Candidates (Priority Order)

| Priority | Provider | Region | Est. ASN | Notes |
|----------|----------|--------|----------|-------|
| 1 | **Vultr** | New Jersey (US-East) | AS20473 | Different continent (NA), good price |
| 2 | **DigitalOcean** | San Francisco (US-West) | AS14061 | Different continent (NA) |
| 3 | **Vultr** | Tokyo (APAC) | AS20473 | Different continent (APAC) |
| 4 | **Hetzner** | Ashburn, VA (US-East) | AS24940 | Different continent, great price |
| 5 | **Oracle Cloud** | Tokyo / Sydney | AS31898 | Free tier (ARM), quota permitting |

---

## Links
- [TESTNET-RESET-PROCEDURE.md](../TESTNET-RESET-PROCEDURE.md) — Full reset runbook
- [OPERATIONS.md](../OPERATIONS.md) — Seed runbook, deploy pipeline
- [MASTER-ROADMAP.md](../../MASTER-ROADMAP.md) — C3 item
- [protocol/scripts/deploy-seed-prebuilt.sh](../protocol/scripts/deploy-seed-prebuilt.sh)
- [protocol/scripts/deploy-seed.sh](../protocol/scripts/deploy-seed.sh)
- [protocol/scripts/deploy-seed2.sh](../protocol/scripts/deploy-seed2.sh)

---

*Created: 2026-09-20 | Part of C3 follow-up (third-provider/geo-diverse seed)*
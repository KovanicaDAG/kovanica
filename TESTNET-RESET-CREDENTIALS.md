# Testnet Reset Credentials — Seed1, Seed2, Seed3

> **Purpose**: Single reference for coordinating simultaneous testnet reset across all 3 seeds.
> **Run from**: Operator machine (this VPS = seed1) with release binary built.
> **Genesis hash**: `9565fc20cb465eec0198a65c07da6b825e4211c4060d581a2c7dac6c96bafc97`
> **CHECKPOINT_VERSION**: 7
> **All RFCs 001–006**: Active at blue_score 0

---

## Seed Inventory

| Seed | Hostname | IP | Provider | Region | Mining | SSH User | Notes |
|------|----------|-----|----------|--------|--------|----------|-------|
| **seed1** | `seed.kovanica.online` | `145.223.116.178` | Hostinger | Germany (DE) | Yes (60s) | `root` | **This machine** |
| **seed2** | `seed2.kovanica.online` | `76.13.250.65` | Hostinger KVM2 VPS (`srv1991525`) | Lithuania (LT) | Yes (60s) | `root` | Key at `/root/seeds/seed2/keykovanica` |
| **seed3** | `seed3.kovanica.online` | `15.228.170.29` | AWS | — (retired) | Yes (60s) | `ubuntu` | Creds at `/root/seeds/seed3/` (PEM rejected 2026-09-20) |

---

## SSH Access Details

### Seed1 (Local — This Machine)
```bash
# Already here; systemd unit: kovanica-seed2
systemctl status kovanica-seed2
journalctl -u kovanica-seed2 -f
```

### Seed2 (Hostinger KVM2 VPS — Lithuania)
```bash
# Target: 76.13.250.65 (srv1991525)
# User: root
# Auth: key at /root/seeds/seed2/keykovanica (chmod 600; or password in /root/seeds/seed2/seed2keys)

# Test:
ssh -i /root/seeds/seed2/keykovanica root@76.13.250.65 "uptime"
```

### Seed3 (AWS EC2 — retired)
```bash
# Private key: /root/seeds/seed3/kovanica-seed3.pem
# User: ubuntu
# Host: 15.228.170.29
# STATUS 2026-09-20: host reachable but PEM no longer authorized (Permission denied).

# Test (expected to fail):
ssh -i /root/seeds/seed3/kovanica-seed3.pem ubuntu@15.228.170.29 "uptime"

# systemd unit on the box is kovanica-seed3 (retired; no longer in the live set)
ssh -i /root/seeds/seed3/kovanica-seed3.pem ubuntu@15.228.170.29 "systemctl status kovanica-seed2"
```

---

## Deploy Script (Run from This Machine)

```bash
# 1. Build release binary (on this machine)
cd /root/kovanica/kovanica-node
cargo build --release -p kovanica-node

# 2. Deploy to ALL THREE SEEDS SIMULTANEOUSLY (background each)
# Seed1 (local) — systemd restart after binary swap
sudo install -m755 ./target/release/kovanica-node /usr/local/bin/kovanica-node.new \
  && sudo mv -f /usr/local/bin/kovanica-node{.new,}
sudo systemctl restart kovanica-seed2

# Seed2 (Hostinger KVM2 VPS — root@76.13.250.65, key below)
./scripts/deploy-seed-prebuilt.sh root@76.13.250.65 \
  --name seed2 \
  --mine \
  --mine-secs 60 \
  --binary ./target/release/kovanica-node \
  --identity-file /root/seeds/seed2/keykovanica

# Seed3 (AWS — RETIRED; skip unless re-provisioning)
# ./scripts/deploy-seed-prebuilt.sh ubuntu@15.228.170.29 \
#   --name seed2 \
#   --mine \
#   --mine-secs 60 \
#   --binary ./target/release/kovanica-node \
#   --identity-file /root/seeds/seed3/kovanica-seed3.pem
```

**Note**: seed2 = Hostinger KVM2 VPS `srv1991525` (systemd unit
`kovanica-seed2` on that box, P2P :9000). seed3 (AWS `15.228.170.29`) is
retired — do not deploy to it. Confirmed live topology 2026-09-20:
`seed.kovanica.online:9000` + `seed2.kovanica.online:9000` only.

---

## Post-Deploy Verification (All Seeds)

```bash
# Genesis match (must equal 9565fc20cb465eec0198a65c07da6b825e4211c4060d581a2c7dac6c96bafc97)
curl -s http://127.0.0.1:8080/api/head | jq -r .genesis          # seed1
ssh -i /root/seeds/seed2/keykovanica root@76.13.250.65 "curl -s http://127.0.0.1:8080/api/head | jq -r .genesis"
# seed3 retired — skip

# Peer connectivity (should show 2+ peers)
curl -s http://127.0.0.1:8080/api/head | jq .peers
ssh -i /root/seeds/seed2/keykovanica root@76.13.250.65 "curl -s http://127.0.0.1:8080/api/head | jq .peers"

# Block production (incrementing ~60s)
watch -n 10 'curl -s http://127.0.0.1:8080/api/head | jq .blocks'
```

---

## Seed2 Credentials (Resolved 2026-09-20)

```bash
# === SEED2 (Hostinger KVM2 VPS srv1991525, 76.13.250.65) ===
SEED2_USER="root"
SEED2_AUTH="key"
SEED2_KEY_PATH="/root/seeds/seed2/keykovanica"   # key auth; chmod 600 done
# Password fallback (if key auth is off): /root/seeds/seed2/seed2keys

# Test command:
ssh -i ${SEED2_KEY_PATH} ${SEED2_USER}@76.13.250.65 "uptime"
```

---

## Cloudflare DNS (Already Configured — Grey Cloud)

| Record | Type | Content | Proxy |
|--------|------|---------|-------|
| `seed.kovanica.online` | A | `145.223.116.178` | DNS only |
| `seed.kovanica.online` | AAAA | `2a02:4780:41:1f43::1` | DNS only |
| `seed2.kovanica.online` | A | `76.13.250.65` | DNS only |
| `seed3.kovanica.online` | A | `15.228.170.29` | DNS only |

---

## Reset Checklist (Run in Order)

- [ ] **Build binary**: `cargo build --release -p kovanica-node` ✅
- [ ] **Seed1**: Stop service, swap binary, restart
- [ ] **Seed2**: Run deploy script with key (`/root/seeds/seed2/keykovanica`)
- [ ] **Seed3**: ~~Run deploy script with PEM key~~ — 🔴 **retired; skip**
- [ ] **All**: Verify genesis match on `/api/head`
- [ ] **All**: Verify peer count ≥ 2
- [ ] **All**: Verify block production ~60s
- [ ] **Smoke tests**: Faucet, transfer, multisig, HTLC, vault
- [ ] **Light-node sync**: Android LightNode → live genesis
- [ ] **Update MASTER-ROADMAP.md**: C12 → ✅

---

## Emergency Rollback

```bash
# If critical failure on any seed:
systemctl stop kovanica-seed2
# Restore from backup:
KOV_BACKUP_PASSPHRASE="..." ./scripts/restore-node.sh --data-dir /root/kovanica-data
# Deploy previous binary
sudo install -m755 /root/bin/kovanica-node.prev /usr/local/bin/kovanica-node
systemctl start kovanica-seed2
```

---

## Contact / Escalation

| Role | Contact |
|------|---------|
| Primary operator (seed1) | Toni (this machine) |
| Seed2 operator | Hostinger KVM2 VPS `srv1991525` (Toni; creds `/root/seeds/seed2`) |
| Seed3 operator | AWS (retired; creds `/root/seeds/seed3`) |

---

*Generated: 2026-09-20*
*Run `./scripts/deploy-seed-prebuilt.sh --help` for deploy options*
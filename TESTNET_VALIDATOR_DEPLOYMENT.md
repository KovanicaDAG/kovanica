# Kovanica Testnet — 3 Validator Deployment Summary

## Overview
Deploying 3 PoA validators for the testnet authority set (threshold 2). Seed3 is already running; seed1 and seed2 need manual deployment.

---

## Authority Set (Ceremony Keys)

| # | Public Key | Private Key | Server | Status |
|---|---|---|---|---|
| **1** | `a1affed944312b0a8a1627b126d8162bba7a3abba7710bb349b621bac732266f` | `4a4172c14e6073998caf9ad256974cd2908a67b7751fdbc2f031c24736b8e8ec` | **seed1** (145.223.116.178) | ⏳ **NEEDS DEPLOY** |
| **2** | `a5e261ae6d582f31c37b727a3abda3f1b536822d62f7a7515721fcbeb7a1a662` | `8ebc8a73235b631845d32ed4ea2d1dc563acfa1215b18428b17364c6e1563cf3` | **seed2** (76.13.250.65) | ⏳ **NEEDS DEPLOY** |
| **3** | `d1a14d2c0d228b9d04a7f852404eefc6e1057698bf1b8105b6f0c6e443bce632` | `d6903aa7a17abfe681988f1b49a8adcec0d475c5e24ab955c1349a1c463bedae` | **seed3** (187.7.27.139) | ✅ **RUNNING** |

---

## Genesis
```
93efd2d784c19e0ea74b53c4b1aec1aa070a2d6cd8042058d934b18a6e23ab0a
```

---

## Deployment Files Created

| File | Purpose |
|---|---|
| `SEED1_DEPLOYMENT.md` | Complete guide for seed1 (Authority 1) |
| `SEED2_DEPLOYMENT.md` | Complete guide for seed2 (Authority 2) |
| `SEED3_STATUS.md` | Current status of seed3 (Authority 3) |

---

## Quick Deploy Commands (Copy-Paste to Each Server)

### On seed1 (145.223.116.178):
```bash
# SSH in, then run:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
source "$HOME/.cargo/env"
cd /opt && git clone https://github.com/KovanicaDAG/kovanica.git && cd kovanica && git checkout seed3
cd /opt/kovanica/protocol && cargo build --release -p kovanica-node
sudo cp target/release/kovanica-node /usr/local/bin/kovanica-node
sudo mkdir -p /opt/kovanica/testnet/config /var/lib/kovanica-seed1
# ... then paste the network.env and service from SEED1_DEPLOYMENT.md
```

### On seed2 (76.13.250.65):
```bash
# Same as seed1 but use SEED2_DEPLOYMENT.md config
```

---

## Verification Checklist

After deploying each validator:

- [ ] Service running: `systemctl status kovanica-seedX`
- [ ] API responds: `curl -s http://127.0.0.1:8080/api/head`
- [ ] Genesis matches: `93efd2d784c19e0ea74b53c4b1aec1aa070a2d6cd8042058d934b18a6e23ab0a`
- [ ] Authority set shows 3 authorities
- [ ] `authority_pk` in `/api/state` matches your assigned pubkey
- [ ] Slot advances from 0 (once ≥2 validators online)
- [ ] `producing: true` during your slot

---

## Expected Timeline

1. **Deploy seed1** → wait for sync to genesis
2. **Deploy seed2** → wait for sync to genesis  
3. **Slot advances** → blocks produced every 3s (rotating authorities)
4. **Chain live** → testnet operational with 3 validators

---

## Ports Required (All 3 Servers)

| Port | Protocol | Purpose |
|---|---|---|
| 9000 | TCP | P2P gossip |
| 8080 | TCP | Explorer API |
| 9090 | TCP | Prometheus metrics |

Firewall: `ufw allow 9000,8080,9090/tcp`

---

## Troubleshooting

**Seed1/Seed2 can't reach seed3:**
- Add `187.7.27.139:9000` to `KOVANICA_PEERS` in network.env
- Check firewall on seed3: `ufw status`

**Validator not producing:**
- Verify `KOVANICA_AUTHORITY_KEY` matches your assigned private key
- Verify `KOVANICA_VALIDATOR=1` in network.env
- Check logs: `journalctl -u kovanica-seedX -f`

**Genesis mismatch:**
- Ensure all 3 servers use the SAME `KOVANICA_AUTHORITIES` string
- Data dir must be clean (fresh genesis) — `/var/lib/kovanica-seedX/` should be empty before first start

---

## Files Location
All deployment guides in `/root/kovanica/`:
- `SEED1_DEPLOYMENT.md`
- `SEED2_DEPLOYMENT.md`  
- `SEED3_STATUS.md`
- `TESTNET_VALIDATOR_DEPLOYMENT.md` (this file)

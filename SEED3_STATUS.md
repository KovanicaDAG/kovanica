# Seed3 Status (Already Running)

## Server Details
- **Host**: 187.7.27.139 (this server)
- **Service**: kovanica-testnet
- **Data**: /var/lib/kovanica-seed3

## Authority Assignment
**Authority 3 of 3** (matches pubkey: `d1a14d2c0d228b9d04a7f852404eefc6e1057698bf1b8105b6f0c6e443bce632`)

## Private Key (Already Configured)
```
d6903aa7a17abfe681988f1b49a8adcec0d475c5e24ab955c1349a1c463bedae
```

## Public Key
```
d1a14d2c0d228b9d04a7f852404eefc6e1057698bf1b8105b6f0c6e443bce632
```

## Current Status ✅
- **Service**: Active (running)
- **Genesis**: `93efd2d784c19e0ea74b53c4b1aec1aa070a2d6cd8042058d934b18a6e23ab0a`
- **Authority set**: 3 authorities, threshold 2
- **Validator mode**: Enabled (KOVANICA_VALIDATOR=1 + KOVANICA_AUTHORITY_KEY set)
- **Producing**: Waiting for ≥2 validators to come online

## Verification Commands
```bash
# Service status
systemctl status kovanica-testnet

# API head
curl -s http://127.0.0.1:8080/api/head | python3 -m json.tool

# State (check authority_pk matches)
curl -s http://127.0.0.1:8080/api/state | python3 -c "import sys,json; d=json.load(sys.stdin); print('producing:', d.get('producing')); print('authority_pk:', d['node'].get('authority_pk')); print('slot:', d['node'].get('current_slot'))"

# Logs
journalctl -u kovanica-testnet -f
```

## Network Config
- **P2P**: 0.0.0.0:9000, [::]:9000
- **Explorer**: 127.0.0.1:8080
- **Metrics**: 0.0.0.0:9090
- **Peers**: seed.kovanica.online:9000, seed2.kovanica.online:9000

---

## Network Topology
```
seed1 (145.223.116.178:9000) — Authority 1 ⏳ NEEDS DEPLOY
seed2 (76.13.250.65:9000)     — Authority 2 ⏳ NEEDS DEPLOY
seed3 (187.7.27.139:9000)     — Authority 3 ✅ RUNNING
```

Once seed1 and seed2 validators come online, slot should advance from 0.

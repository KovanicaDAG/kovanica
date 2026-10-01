# Seed2 Validator Deployment Guide

## Server Details
- **Host**: 76.13.250.65
- **User**: root
- **Password**: [REDACTED] (from /root/nodes)

## Authority Assignment
**Authority 2 of 3** (matches pubkey: `a5e261ae6d582f31c37b727a3abda3f1b536822d62f7a7515721fcbeb7a1a662`)

## Private Key
```
8ebc8a73235b631845d32ed4ea2d1dc563acfa1215b18428b17364c6e1563cf3
```

## Public Key
```
a5e261ae6d582f31c37b727a3abda3f1b536822d62f7a7515721fcbeb7a1a662
```

---

## Deployment Steps (Manual on Seed2)

### 1. SSH to seed2
```bash
ssh root@76.13.250.65
# Password: [REDACTED]
```

### 2. Install Rust (if not present)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
source "$HOME/.cargo/env"
```

### 3. Clone/Update Repository
```bash
cd /opt
git clone https://github.com/KovanicaDAG/kovanica.git || cd kovanica && git fetch origin
git checkout seed3
```

### 4. Build Binary
```bash
cd /opt/kovanica/protocol
cargo build --release -p kovanica-node
sudo cp target/release/kovanica-node /usr/local/bin/kovanica-node
sudo chmod 755 /usr/local/bin/kovanica-node
```

### 5. Create Directory Structure
```bash
sudo mkdir -p /opt/kovanica/testnet/config
sudo mkdir -p /var/lib/kovanica-seed2
sudo chown $(whoami) /opt/kovanica /var/lib/kovanica-seed2
```

### 6. Create network.env
```bash
cat > /opt/kovanica/testnet/config/network.env << 'ENVEOF'
# Kovanica Testnet Configuration — seed2 (Validator Authority 2)
# Network ID: kovanica-testnet
# Consensus: PoA-only (RFC-POA-Migration ratified 2026-09-25)
#
# Ports: P2P 9000, Explorer 8080, Metrics 9090

KOVANICA_NETWORK=testnet
KOVANICA_NETWORK_ID=kovanica-testnet
KOVANICA_LISTEN=0.0.0.0:9000
KOVANICA_PEERS=seed.kovanica.online:9000,187.7.27.139:9000,145.223.116.178:9000

# Consensus: PoA-only
KOVANICA_CONSENSUS=poa
KOVANICA_POA_NOMINAL_WORK=1

# REAL Authority Set (3 authorities, threshold 2) — from ceremony
KOVANICA_AUTHORITIES=d1a14d2c0d228b9d04a7f852404eefc6e1057698bf1b8105b6f0c6e443bce632,a5e261ae6d582f31c37b727a3abda3f1b536822d62f7a7515721fcbeb7a1a662,a1affed944312b0a8a1627b126d8162bba7a3abba7710bb349b621bac732266f
KOVANICA_AUTHORITY_THRESHOLD=2

# Slot duration (PoA fixed slot)
KOVANICA_SLOT_DURATION=3000
KOVANICA_PRODUCE_SECS=60

# Node operation — VALIDATOR MODE
KOVANICA_OPERATOR=1
KOVANICA_FAUCET=0
KOVANICA_ALLOW_RESET=0
KOVANICA_DATA=/var/lib/kovanica-seed2
KOVANICA_EXPLORER_PORT=8080
KOVANICA_METRICS_LISTEN=0.0.0.0:9090

# Consensus parameters (RFC-006)
KOVANICA_K=3
KOVANICA_FINALITY_DEPTH=100
KOVANICA_PRUNING_DEPTH=1000
KOVANICA_MAX_SUPPLY=9020000000000000
KOVANICA_COINBASE_MATURITY=100
KOVANICA_SUBSIDY=1000000000
KOVANICA_FOUNDER_AMOUNT=20000000000000
KOVANICA_FOUNDER_SEED=1
KOVANICA_MIN_FEE=2000
KOVANICA_ATOM=100000000

# Treasury (placeholder for testnet)
KOVANICA_TREASURY_SEED_BASE=9001

# THIS VALIDATOR'S PRIVATE KEY (Authority 2)
KOVANICA_AUTHORITY_KEY=8ebc8a73235b631845d32ed4ea2d1dc563acfa1215b18428b17364c6e1563cf3

# Enable validator mode
KOVANICA_VALIDATOR=1
ENVEOF
```

### 7. Create Systemd Service
```bash
cat > /etc/systemd/system/kovanica-seed2.service << 'SVC_EOF'
[Unit]
Description=Kovanica Testnet Validator (seed2) — Authority 2
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=/var/lib/kovanica-seed2
EnvironmentFile=/opt/kovanica/testnet/config/network.env
ExecStart=/usr/local/bin/kovanica-node explorer 127.0.0.1:8080
Restart=always
RestartSec=5
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
SVC_EOF
```

### 8. Start Service
```bash
sudo systemctl daemon-reload
sudo systemctl enable kovanica-seed2
sudo systemctl start kovanica-seed2
```

### 9. Verify
```bash
# Check service status
sudo systemctl status kovanica-seed2

# Check logs
journalctl -u kovanica-seed2 -f

# Verify API responds
curl -s http://127.0.0.1:8080/api/head | python3 -m json.tool

# Verify it's producing (authority_pk should match your pubkey)
curl -s http://127.0.0.1:8080/api/state | python3 -c "import sys,json; d=json.load(sys.stdin); print('producing:', d.get('producing')); print('authority_pk:', d['node'].get('authority_pk'))"
```

---

## Expected Results
- **Genesis**: `93efd2d784c19e0ea74b53c4b1aec1aa070a2d6cd8042058d934b18a6e23ab0a`
- **Authority set**: 3 authorities (threshold 2)
- **Your authority_pk**: `a5e261ae6d582f31c37b727a3abda3f1b536822d62f7a7515721fcbeb7a1a662`
- **Slot should advance** once ≥2 validators online
- **Producing**: true (when it's your slot)

---

## Troubleshooting
- **Port 9000** must be open for P2P
- **Port 8080** for explorer API
- **Port 9090** for metrics
- Firewall: `ufw allow 9000,8080,9090/tcp`
- If sync fails: ensure peers list includes `187.7.27.139:9000` (seed3) and `145.223.116.178:9000` (seed1)

---

## Network Topology
```
seed1 (145.223.116.178:9000) — Authority 1
seed2 (76.13.250.65:9000)     — Authority 2 (THIS)
seed3 (187.7.27.139:9000)     — Authority 3 (already running)
```

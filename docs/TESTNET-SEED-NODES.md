# Testnet seed nodes — kovanica L1

**Network:** public/hybrid testnet: `chain_id 5` (internal testnet: `chain_id 4`, local dev only: `1337`)
**Deployed:** 2026-10-13 (build c44f672, mempool requeue fix)
**P2P:** TCP 8000, libp2p swarm
**RPC:** 8545 (loopback-only)

## Seed nodes

| # | Hostname | Resolved IP | VPS Role | Key/Config | Composer peers |
|---|---|---|---|---|---|
| 0 | `testnet.kovanica.online` | `145.223.116.178` | VPS#1 (canonical bootstrap entry) | `kovanica.env` | 3 |
| 1 | `seed1.kovanica.online` | `145.223.116.178` | VPS#1 (seed instance 1) | `val1.pem` | 3 |
| 2 | `seed2.kovanica.online` | `76.13.250.65` | VPS#2 (seed instance 2) | `val2.pem` | 3 |
| 3 | `seed3.kovanica.online` | `187.7.27.139` | VPS#3 (seed instance 3) | `val3.pem` | 3 |

**All seed nodes run with `mine=0 faucet=0 operator=0 allow_reset=0`.**

## Verified state (2026-10-13T18:40Z)

```
seed1: peer_count 2, height ~294, exec_subdags=24, healthy
seed2: peer_count 2, height ~290, exec_subdags 13, healthy
seed3: peer_count 2, height ~294, exec_subdags 14, healthy
```

Mesh fully connected (3 nodes, each sees 2 peers), all executing.

## Automated Diagnostics & Reachability Probe

Seed DNS resolution (grey-cloud, no Cloudflare HTTP proxy) and TCP port 8000 P2P reachability are verified using `scripts/check-dns-seeds.sh`:

```bash
# Run full testnet DNS & P2P audit
./scripts/check-dns-seeds.sh --network testnet

# Output structured JSON report
./scripts/check-dns-seeds.sh --network testnet --json

# Probe specific seeds with custom timeout
./scripts/check-dns-seeds.sh -s testnet.kovanica.online,seed1.kovanica.online -p 8000 -t 3
```

Diagnostic audit criteria:
1. **DNS Resolution**: Hostname resolves to valid A record IPv4 addresses.
2. **Cloudflare Grey-Cloud**: Resolved IPs do not fall within Cloudflare proxy CIDRs.
3. **P2P Port Reachability**: TCP port 8000 responds to connection handshake.

## Operations

### Systemd Services

Kovanica seed nodes use standard systemd services from `ops/systemd/`:

* **Standalone node**: `systemctl start kovanica-node.service` (configured via `/etc/kovanica/kovanica.env`)
* **Multi-instance seed**: `systemctl start kovanica-seed@1.service` (configured via `/etc/kovanica/seed1.env`)

```bash
# Start seed node service
sudo systemctl enable --now kovanica-seed@1.service

# View seed logs
sudo journalctl -u kovanica-seed@1.service -f
```

### Automated Fleet Cutover

To migrate from legacy `kvnc-*` services and paths to standard `kovanica-*`:

```bash
# 1. Pre-flight dry run validation (read-only, does not modify system):
sudo bash scripts/cutover-vps-systemd.sh --dry-run

# 2. Execute cutover:
sudo bash scripts/cutover-vps-systemd.sh --execute
```

> **Ops note (unit & env naming):** Seed systemd units and wrapper scripts must use `kovanica-*` unit names (`kovanica-node.service` or `kovanica-seed@<N>.service`) and `KOVANICA_*` environment variables, with checkout path `/root/Projects/kovanica` (or `${KOVANICA_ROOT}`).

### Health checks
```bash
curl -s http://127.0.0.1:8545/health
curl -s -X POST http://127.0.0.1:8545/rpc \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"kovanica_blockNumber","params":[]}'
```

### DNS
- `testnet.kovanica.online` → `145.223.116.178` (A record, grey cloud)
- `seed1.kovanica.online` → `145.223.116.178` (A record, grey cloud)
- `seed2.kovanica.online` → `76.13.250.65` (A record, grey cloud)
- `seed3.kovanica.online` → `187.7.27.139` (A record, grey cloud)
- For mainnet DNS seed requirements (3+ kovanica-native DNS-only seed records), see [docs/SEED-DNS-AUDIT.md](SEED-DNS-AUDIT.md).

### Binaries
- VPS#1: `/usr/local/bin/kovanica-node`
- VPS#2, VPS#3: `/usr/local/bin/kovanica-node` (synchronized from release build)

### Data dirs
- `/var/lib/kovanica` (node daemon) or `/var/lib/kovanica-seed<N>` (seed instances), owned by `kovanica:kovanica` (0750).

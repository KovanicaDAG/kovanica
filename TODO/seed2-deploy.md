# seed2 Deployment — Testnet Soak

**Status**: ✅ **Done — superseded** (2026-09-17). "seed2" was brought up by
**re-keying the existing AWS seed3** (`kovanica-seed3` → `kovanica-seed2`,
DNS `seed2.kovanica.online` → `76.13.250.65`, retired `seed3.kovanica.online`).
The old plan below (dedicated VPS on a third provider) is **cancelled** — seed2
lives on AWS, not a new VPS.

---

## Target (as deployed 2026-09-17)
- **Name**: seed2 (unit `kovanica-seed2` on AWS eu-north-1)
- **Hostname**: `seed2.kovanica.online` → `76.13.250.65` (grey cloud)
- **P2P Port**: 9000
- **Explorer**: 8080 (loopback)
- **Metrics**: 9090 (loopback)
- **Peers**: `seed.kovanica.online:9000,seed2.kovanica.online:9000`

> The VPS itself also runs `kovanica-seed2` (P2P `:9001`, HTTP `127.0.0.1:18080`)
> — a different, local unit. Do not confuse the two.

---

## Requirements
- **Provider**: ~~Different from seed1 (Hostinger) and seed3 (AWS eu-north-1)~~ — **cancelled**: seed2 reused seed3's AWS box (org-distinct from Hostinger).
- **Region**: Different continent/ASN preferred — ⏳ still open (AWS eu-north-1 = EU, same region family as Hostinger's EU VPS; a true third-provider seed is a separate future item)
- **Specs**: 2 vCPU, 4GB RAM, 100GB SSD (AWS box is `c7i.large`)
- **OS**: Ubuntu 22.04+ or Debian 12+ (AWS: Amazon Linux 2023, systemd unit)
- **IP**: Static IPv4 (+ IPv6 if available)

---

## Deploy Script (superseded — seed2 already deployed)
```bash
cd /root/kovanica/protocol
./scripts/deploy-seed2.sh root@<VPS_IP> --name seed2
```

### Script Does:
- [ ] Ships source via SSH tarball
- [ ] Installs build deps + swap + Rust
- [ ] Builds `kovanica-node` release
- [ ] Installs systemd unit (`kovanica-seed2`)
- [ ] Configures Prometheus metrics (`:9090`)
- [ ] Installs fail2ban + node-exporter
- [ ] Verifies genesis matches `seed.kovanica.online`

---

## Post-Deploy Checklist (historical — seed2 is live)
- [x] DNS A/AAAA: `seed2.kovanica.online` → `76.13.250.65`
- [x] Cloudflare: Grey-cloud P2P port 9000
- [ ] Verify: `curl https://seed2.kovanica.online:8080/api/head` (from outside)
- [ ] Verify metrics: `ssh -L 9090:127.0.0.1:9090 ubuntu@76.13.250.65 'curl localhost:9090/metrics'`
- [x] Update bootstrap list in `NETWORK.md` and node config
- [x] Add to Cloudflare DNS (grey-cloud for P2P port 9000)

---

## Testnet Soak Requirements
Per `kovanica-protocol/docs/TESTNET-SOAK.md`:
- [ ] 30-day continuous operation
- [ ] ≥3 independent seed operators
- [ ] Metrics: orphan rate, propagation latency, fork rate, disk growth
- [ ] All seeds expose `/metrics` publicly
- [ ] Alerting rules armed (peer count, block rate, reorg depth, disk)

---

## Notes
- seed1 (primary): Hostinger VPS (`kovanica-explorer` unit, P2P :9000)
- seed2: AWS eu-north-1 (re-keyed from seed3, 2026-09-17) — `76.13.250.65`
- ~~seed2 target: Different provider/region~~ → superseded; a dedicated
  third-provider seed remains an open future item (DR/geo diversity)

---

**Done**: seed2 deployed via AWS re-key on 2026-09-17 (see `OPERATIONS.md` §7).

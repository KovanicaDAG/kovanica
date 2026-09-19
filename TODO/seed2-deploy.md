# seed2 Deployment — Testnet Soak

**Status**: ⏳ Pending VPS provisioning

---

## Target
- **Name**: seed2
- **Hostname**: `seed2.kovanica.online`
- **P2P Port**: 9000
- **Explorer**: 8080 (loopback)
- **Metrics**: 9090 (loopback)
- **Peers**: `seed.kovanica.online:9000,seed3.kovanica.online:9000`

---

## Requirements
- **Provider**: Different from seed1 (Hostinger) and seed3 (AWS eu-north-1)
- **Region**: Different continent/ASN preferred
- **Specs**: 2 vCPU, 4GB RAM, 100GB SSD
- **OS**: Ubuntu 22.04+ or Debian 12+
- **IP**: Static IPv4 (+ IPv6 if available)

---

## Deploy Script
```bash
cd /root/kovanica/kovanica-protocol
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

## Post-Deploy Checklist
- [ ] DNS A/AAAA: `seed2.kovanica.online` → VPS IP
- [ ] Cloudflare: Grey-cloud P2P port 9000
- [ ] Verify: `curl https://seed2.kovanica.online:8080/api/head`
- [ ] Verify metrics: `ssh -L 9090:127.0.0.1:9090 root@<IP> 'curl localhost:9090/metrics'`
- [ ] Update bootstrap list in `NETWORK.md` and node config
- [ ] Add to Cloudflare DNS (grey-cloud for P2P port 9000)

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
- seed1: Hostinger (EU)
- seed3: AWS eu-north-1 (Stockholm)
- seed2 target: Different provider/region (e.g., DigitalOcean NYC, Hetzner Helsinki, Vultr Tokyo, etc.)

---

**Next**: User will provision VPS and run deploy script. Will notify when done.

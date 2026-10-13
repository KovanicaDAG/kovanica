# KVNC Seed DNS & Explorer DNS Configuration

> **Domain:** `kovanica.online` (owned by KVNC team)
> **Status:** Active domain, needs DNS records for seed nodes and explorer

---

## 1. Seed Node DNS Records

### Required: 3+ kvnc-native DNS names for P2P bootstrapping

| Record | Type | Value | TTL | Notes |
|--------|------|-------|-----|-------|
| `seed1.kvnc.io` | A | `<bare-metal-IP-1>` | 300 | Primary seed, bare metal, no Cloudflare proxy |
| `seed2.kvnc.io` | A | `<bare-metal-IP-2>` | 300 | Secondary seed, different ASN/DC |
| `seed3.kvnc.io` | A | `<bare-metal-IP-3>` | 300 | Tertiary seed, different continent |
| `seed4.kvnc.io` | A | `<bare-metal-IP-4>` | 300 | Optional 4th seed |

**Critical:** These must be **DNS-only (grey-cloud)** records in Cloudflare — **no orange-cloud proxy**. P2P traffic is plaintext TCP on port 8000 and will not work through Cloudflare's HTTP proxy.

### kovanica.online subdomain aliases (optional)

| Record | Type | Value | Notes |
|--------|------|-------|-------|
| `seed1.kovanica.online` | CNAME | `seed1.kvnc.io` | Alias for branding |
| `seed2.kovanica.online` | CNAME | `seed2.kvnc.io` | |
| `seed3.kovanica.online` | CNAME | `seed3.kvnc.io` | |

---

## 2. Explorer & Public Services DNS

### explorer.kovanica.online (Phase 23)

| Record | Type | Value | Proxy | Notes |
|--------|------|-------|-------|-------|
| `explorer.kovanica.online` | CNAME | `<cloudflare-pages-domain>.pages.dev` | **Proxied (orange-cloud)** | Cloudflare Pages deployment |
| `explorer` | CNAME | `<cloudflare-pages-domain>.pages.dev` | Proxied | Apex subdomain |

### wallet.kovanica.online (Phase 23.2)

| Record | Type | Value | Proxy | Notes |
|--------|------|-------|-------|-------|
| `wallet.kovanica.online` | CNAME | `<cloudflare-pages-domain>.pages.dev` | Proxied | Web wallet deployment |

### rpc.kovanica.online (public RPC endpoint)

| Record | Type | Value | Proxy | Notes |
|--------|------|-------|-------|-------|
| `rpc.kovanica.online` | CNAME | `<rpc-server-domain>` | **DNS-only (grey-cloud)** | Direct RPC, no HTTP proxy. Mainnet. Console chain id 1, not open yet. |
| `rpc-testnet.kovanica.online` | CNAME | `<testnet-rpc-domain>` | DNS-only | Testnet RPC. This is the canonical testnet host (chain id 2). Do not add `testnet-rpc` or `testnet.kovanica.online`. |

### console.kovanica.online

| Record | Type | Value | Proxy | Notes |
|--------|------|-------|-------|-------|
| `console.kovanica.online` | CNAME | `<kvnc-console>.pages.dev` | **Proxied (orange-cloud)** | Kovanica Console (Cloudflare Pages project `kvnc-console`) |

The console is the interactive site. The explorer stays read-only. Browser calls to the RPC need the node CORS allowlist (`https://console.kovanica.online`, plus `http://127.0.0.1:5173` and `http://localhost:5173` for local dev). The public RPC records stay grey-cloud, so Cloudflare will not add those headers.

### faucet-testnet.kovanica.online

| Record | Type | Value | Proxy | Notes |
|--------|------|-------|-------|-------|
| `faucet-testnet.kovanica.online` | CNAME | `<faucet-host>` | **DNS-only** | `kvnc-faucet` `POST /faucet`. Testnet only. Same browser origin allowlist as the RPC. |

### status.kovanica.online (Phase 23.3)

| Record | Type | Value | Proxy | Notes |
|--------|------|-------|-------|-------|
| `status.kovanica.online` | CNAME | `<status-page-domain>` | Proxied | Status page (Cloudflare Pages or external) |

---

## 3. Cloudflare Configuration

### Page Rules for explorer.kovanica.online

| Pattern | Settings |
|---------|----------|
| `explorer.kovanica.online/*` | Cache Level: Standard, Edge Cache TTL: 30s, Always Online: On |
| `explorer.kovanica.online/api/*` | Cache Level: Bypass, Security Level: Low |

### WAF Rules

| Rule | Action |
|------|--------|
| Block known malicious IPs (Threat Intelligence) | Block |
| Rate limit `/rpc` to 100 req/min per IP | Challenge |
| Allow all traffic from known validator IPs | Allow |

### SSL/TLS

| Setting | Value |
|---------|-------|
| SSL Mode | Full (Strict) |
| Always Use HTTPS | On |
| Minimum TLS Version | 1.2 |
| Automatic HTTPS Rewrites | On |

---

## 4. DNS Verification Checklist

Before testnet launch:

- [ ] `dig +short seed1.kvnc.io` → returns bare metal IP (not Cloudflare IP)
- [ ] `dig +short seed2.kvnc.io` → returns different ASN
- [ ] `dig +short seed3.kvnc.io` → returns different continent
- [ ] `nc -zv seed1.kvnc.io 8000` → connects successfully (plaintext TCP)
- [ ] `curl -I https://explorer.kovanica.online` → returns 200, Cloudflare headers present
- [ ] `curl -I https://explorer.kovanica.online/rpc` → 404 (no RPC on explorer)
- [ ] SSL Labs grade: A+ for all subdomains
- [ ] DNSSEC enabled on `kovanica.online`

---

## 5. Cloudflare Pages Deployment (explorer)

### Build Configuration

```yaml
# Cloudflare Pages build settings
build_command: npm run build
build_output_directory: dist
root_directory: packages/web
node_version: 20
```

### Environment Variables

| Variable | Value |
|----------|-------|
| `VITE_RPC_URL` | `https://rpc.kovanica.online` (explorer production) / `http://127.0.0.1:8545` (dev) |
| `VITE_CHAIN_ID` | `2` (testnet) / `1` (mainnet) |

The console does not use these variables. Its network table is `packages/console/src/network.ts`. Testnet RPC is `https://rpc-testnet.kovanica.online/rpc`.

### Custom Domain Setup

1. Add `explorer.kovanica.online` as custom domain in Cloudflare Pages
2. Cloudflare automatically provisions SSL (Let's Encrypt or Cloudflare Universal SSL)
3. DNS record created automatically (CNAME to `<project>.pages.dev`)
4. Verify SSL certificate issuance

---

## 6. Seed Node Operator SOP

### Hardware Requirements

| Component | Spec |
|-----------|------|
| CPU | 4+ cores (ARM64 or x86_64) |
| RAM | 8 GB minimum (16 GB recommended) |
| Storage | 500 GB NVMe (OS + DB) |
| Network | 1 Gbps unmetered, static IP |
| OS | Ubuntu 22.04 LTS or Debian 12 |

### Software Stack

```bash
# Install Docker
curl -fsSL https://get.docker.com | sh

# Clone repo
git clone https://github.com/KovanicaDAG/kvnc.git
cd kvnc

# Generate validator keys (offline, one-time)
./ops/docker/validators/generate.sh

# Run seed node (no validator key, no mining)
docker compose -f docker-compose.seed.yml up -d
```

### docker-compose.seed.yml

```yaml
version: '3.8'
services:
  seed:
    image: kvnc-node:local
    restart: unless-stopped
    network_mode: host
    environment:
      KVNC_RUN_VALIDATOR: "false"
      KVNC_LISTEN_ADDR: "0.0.0.0:8000"
      KVNC_BOOTNODES: "seed2.kvnc.io:8000,seed3.kvnc.io:8000"
      KVNC_MAX_PEERS: "50"
      KVNC_RPC_AUTH: "disable"
      KVNC_RPC_RATE_LIMIT_PER_MIN: "0"
      RUST_LOG: "info"
    volumes:
      - seed-data:/var/lib/kvnc
    healthcheck:
      test: ["CMD", "curl", "-fsS", "http://127.0.0.1:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s
volumes:
  seed-data:
```

### Monitoring

- Prometheus scrape: `http://<seed-ip>:9090/metrics`
- Grafana dashboard: import `ops/grafana/kvnc-overview.json`
- Alerting: peer_count < 2 for 5m → PagerDuty

---

## 7. Launch Sequence

### Phase 1: Seed Nodes (T-2 weeks)
1. [ ] Register `kvnc.io` domain (or use `kovanica.online` subdomains)
2. [ ] Provision 3 bare-metal servers in different DCs/ASNs
3. [ ] Deploy seed nodes with `docker-compose.seed.yml`
4. [ ] Add DNS A records (DNS-only, no proxy)
4. [ ] Verify P2P connectivity: `nc -zv seedN.kvnc.io 8000`
5. [ ] Add seed nodes to `KVNC_BOOTNODES` in genesis config

### Phase 2: Genesis Ceremony (T-1 week)
1. [ ] Collect validator public keys + stakes via PR
2. [ ] Run `kvnc-node genesis --validators validators.json --force`
3. [ ] Distribute `genesis.json` + `genesis_validators.toml` to all participants
3. [ ] Verify genesis hash matches across all nodes

### Phase 3: Testnet Launch (T-0)
1. [ ] Deploy explorer to Cloudflare Pages
2. [ ] Add DNS records for `explorer.kovanica.online` (proxied)
3. [ ] Deploy faucet CI job (GitHub Actions nightly)
3. [ ] Announce testnet: seeds, faucet, explorer, RPC endpoints
4. [ ] Monitor `#testnet` Discord channel for issues

### Phase 4: Mainnet Preparation (T+4 weeks)
1. [ ] External security audit
2. [ ] Bug bounty program launch
3. [ ] Mainnet freeze checklist (see `TESTNET-MAINNET.md`)
3. [ ] Genesis ceremony for mainnet
4. [ ] Mainnet launch

---

## 8. References

- `docs/TESTNET-MAINNET.md` — full testnet/mainnet checklist
- `docs/TOKENOMICS.md` — genesis allocation numbers
- `ops/docker/validators/generate.sh` — key generation script
- `ops/docker/` — docker compose files
- `crates/kvnc-faucet/` — faucet crate
- `ops/grafana/kvnc-overview.json` — Grafana dashboard
- `ops/prometheus/alerts.yml` — alerting rules
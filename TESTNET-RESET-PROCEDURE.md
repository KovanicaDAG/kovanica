# Testnet Reset Procedure — A13 + A6 Combined

> **Purpose:** Single coordinated reset for both operator/founder wallet genesis (A13) and RFC-006 tokenomics activation (A6).
> Both require: new genesis hash, checkpoint v7, blue_score 0 activation.
> 
> **Expected new genesis hash:** `256c87faf1803ad75c855de95b7cee317f52c1f3fa311ca5479083174e56c31b`
> **CHECKPOINT_VERSION:** 7
> **TOKENOMICS_ACTIVATION_SCORE:** 0 (active at genesis)
> **All RFC-001 through RFC-005:** Active at blue_score 0

---

## Pre-Reset Checklist (Operator Machine)

- [ ] Build release binary: `cargo build --release -p kovanica-node`
- [ ] Verify binary: `file target/release/kovanica-node` → ELF 64-bit LSB executable
- [ ] Test genesis locally: `echo "genesis 3 1000000000 20000000000000 1" | ./target/release/kovanica-node serve`
- [ ] Confirm genesis hash matches: `256c87faf1803ad75c855de95b7cee317f52c1f3fa311ca5479083174e56c31b`
- [ ] Verify all tests pass: `cargo test -p kovanica-node --lib`
- [ ] Push changes to GitHub: `git push origin main`
- [ ] Notify seed operators of reset window

---

## Seed Deployment Order

| Seed | Host | Operator | Mining | Deploy Script |
|------|------|----------|--------|---------------|
| **seed1** (primary) | seed.kovanica.online (Hostinger) | Hostinger VPS | Yes (60s) | `deploy-seed-prebuilt.sh` |
| **seed3** (secondary) | seed3.kovanica.online (AWS eu-north-1) | AWS | Yes (60s) | `deploy-seed-prebuilt.sh` |
| **seed2** (tertiary) | seed2.kovanica.online (TBD) | TBD | No | `deploy-seed-prebuilt.sh` |

**Deploy simultaneously** to minimize fork window.

---

## Per-Seed Deploy Commands

```bash
# From kovanica-protocol root on operator machine:

# Seed1 (primary, mines)
./scripts/deploy-seed-prebuilt.sh ubuntu@seed.kovanica.online \
  --name seed1 \
  --mine \
  --mine-secs 60 \
  --binary ./target/release/kovanica-node

# Seed3 (secondary, mines)  
./scripts/deploy-seed-prebuilt.sh ubuntu@seed3.kovanica.online \
  --name seed3 \
  --mine \
  --mine-secs 60 \
  --binary ./target/release/kovanica-node

# Seed2 (tertiary, no mining)
./scripts/deploy-seed-prebuilt.sh ubuntu@seed2.kovanica.online \
  --name seed2 \
  --binary ./target/release/kovanica-node
```

---

## Post-Deploy Verification (Each Seed)

### 1. Genesis Match
```bash
curl -s https://explorer.kovanica.online/api/head | jq -r .genesis
# Must equal: 256c87faf1803ad75c855de95b7cee317f52c1f3fa311ca5479083174e56c31b
```

### 2. Peer Connectivity
```bash
curl -s https://explorer.kovanica.online/api/head | jq .peers
# Should show 2+ peers (other seeds)
```

### 3. Block Production (within 2 min)
```bash
curl -s https://explorer.kovanica.online/api/head | jq .blocks
# Should increment every ~60s
```

### 4. Operator Wallet Visible in Bootstrap
```bash
curl -s https://explorer.kovanica.online/api/bootstrap | jq .operator_wallet_address
# Should show operator wallet kvnc...dag address
```

### 5. Founder Wallet Visible in Bootstrap
```bash
curl -s https://explorer.kovanica.online/api/bootstrap | jq .founder_seed
# Should be 1
```

---

## Cross-Seed Connectivity Test

```bash
# From seed1, check it sees seed3 and seed2
ssh ubuntu@seed.kovanica.online \
  "curl -s http://127.0.0.1:8080/api/head | jq '.peers[]'"

# From seed3, check it sees seed1 and seed2  
ssh ubuntu@seed3.kovanica.online \
  "curl -s http://127.0.0.1:8080/api/head | jq '.peers[]'"

# All should show 3 peers total (including self)
```

---

## Smoke Tests (Post-Reset)

| Test | Command | Expected |
|------|---------|----------|
| Faucet | `curl -X POST https://explorer.kovanica.online/api/faucet -H "Content-Type: application/json" -d '{"address":"kvnc1A4XLkrefPBsXLwRH7kcRutGm3pgzrC7zJvAf8uiLLHqgdag"}'` | Returns tx hash |
| Transfer | CLI `send` from faucet-funded wallet | Block produced, balance updated |
| Multisig | Create 2-of-2, fund, spend | Spend succeeds with 2 sigs |
| HTLC | Create HTLC, redeem with preimage | Redeem succeeds |
| Vault | Create vault with CSV, wait maturity, release | Release succeeds |
| Native tokens | Create asset, transfer | Asset balance tracked separately |

---

## Light-Node Sync Test (Android)

1. Build Android APK with new binary (GitHub Actions)
2. Install on test device
3. Onboard with fresh mnemonic
4. Verify sync completes to live genesis:
   - `LightNodeRepository` downloads KVLS v1 blob
   - `syncedFilterMatches` returns true for wallet address
   - Full block download via `/api/blocks?from=<tip>` works
   - Balance shows 0 (new wallet)
   - Faucet works from mobile

---

## Metrics Baseline (Post-Reset)

Monitor for 24h on all seeds:

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| Orphan rate | < 1% | > 5% |
| Block propagation | < 2s p95 | > 10s |
| Fork rate | 0 | > 0 |
| Disk growth | ~50 MB/day | > 200 MB/day |
| Peer count | 2-3 | < 2 |
| Block interval | ~60s | > 120s |

Grafana dashboards: `https://monitor.kovanica.online/d/kovanica-seed`

---

## Rollback Procedure (If Critical Failure)

1. Stop new seeds: `systemctl stop kovanica-seed1 kovanica-seed3`
2. Restore previous data dirs from backup (if available)
3. Deploy previous binary version
4. Restart with old genesis
5. Document failure reason

**Backup locations:** `/root/kovanica-backups/pre-reset-$(date +%Y%m%d)/`

---

## Success Criteria

- [ ] All 3 seeds show matching genesis hash
- [ ] All 3 seeds have 2+ peers
- [ ] Block production stable at ~60s interval
- [ ] Smoke tests pass (faucet, transfer, multisig, HTLC, vault)
- [ ] Light-node sync works on Android
- [ ] Metrics within thresholds for 24h
- [ ] MASTER-ROADMAP.md updated: A13 → ✅, A6 → ✅, C12 → ✅

---

## Contacts

| Role | Contact |
|------|---------|
| Primary operator | Toni (seed.kovanica.online) |
| Seed3 operator | AWS eu-north-1 team |
| Mobile/Android | @kovanica-mobile team |
| Explorer/Web | @kovanica-web team |

---

*Document version: 1.0 | Generated: 2026-09-17*
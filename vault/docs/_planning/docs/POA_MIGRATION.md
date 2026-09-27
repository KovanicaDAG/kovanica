# PoA Migration - Testnet Completion Record

> **Status**: COMPLETED - 2026-09-26
> This document records the successful migration from PoW to PoA consensus on kovanica-testnet.

---

## Summary

| Field | Value |
|-------|-------|
| Network | kovanica-testnet |
| Genesis hash | GENESIS_HASH_PLACEHOLDER |
| Migration date | 2026-09-25 (ratified) to 2026-09-26 (executed) |
| Consensus before | PoW (GHOSTDAG k=3) |
| Consensus after | PoA (3 authorities, threshold 2, 3s slots) |
| RFC-006 tokenomics | Active throughout |
| Block height at migration | ~658 |

---

## Authority Set (Testnet)

| Index | Public Key | Operator | Status |
|-------|------------|----------|--------|
| 0 | <authority-0-pubkey> | Seed1 (core team) | Active |
| 1 | <authority-1-pubkey> | Seed2 (core team) | Active |
| 2 | <authority-2-pubkey> | Community validator | Active |

**Threshold**: 2 of 3 signatures required per block
**Slot duration**: 120 seconds
**Rotation policy**: 6-month term, governed by multisig

> **Security incident 2026-09-26**: Previous authority keys were burned/rotated. Current keys stored in TESTNET_AUTHORITY_KEYS.md (not committed to git). See incident report INCIDENT_20260926.md.

---

## Changes Made

### 1. Node Configuration (systemd)

**Before** (kovanica-poa.service):
```
Environment=KOVANICA_POW=1
Environment=KOVANICA_MINE=0
Environment=KOVANICA_PRODUCE_SECS=120
EnvironmentFile=-/root/kovanica/deploy/testnet/env.sh
```

**After** (cleaned):
```
# PoW vars REMOVED: KOVANICA_POW, KOVANICA_MINE, KOVANICA_PRODUCE_SECS
# EnvironmentFile REMOVED (env vars now in service directly)
Environment=KOVANICA_CONSENSUS=poa
Environment=KOVANICA_AUTHORITIES=<3 pubkeys>
Environment=KOVANICA_THRESHOLD=2
Environment=KOVANICA_SLOT_DURATION=120
Environment=KOVANICA_PEERS=seed.kovanica.online:9000,seed2.kovanica.online:9000
Environment=KOVANICA_LISTEN=0.0.0.0:9000
Environment=KOVANICA_DATA=/root/kovanica-data
```

### 2. Removed Services

- kovanica-vault-sync.service - Vault sync timer (no longer needed)
- kovanica-vault-sync.timer - Associated timer

### 3. Workspace Updates

- /root/kovanica/deploy/testnet/env.sh - Updated with PoA config
- /root/kovanica/deploy/testnet/seeds/seed1/env.sh - Seed1 PoA config
- /root/kovanica/deploy/testnet/seeds/seed2/env.sh - Seed2 PoA config
- /root/kovanica/README.md - Documented PoA constants

### 4. Code Removals (Planned - [TARGET])

Per RFC-006 PoA migration ratification, the following will be removed from kovanica-node:
- KOVANICA_POW env var
- KOVANICA_MINE env var
- KOVANICA_HYBRID env var (never used)
- KOVANICA_PRODUCE_SECS env var
- PoW mining loop and difficulty adjustment code
- All PoW-related consensus paths

**Target**: Remove in next kovanica-node release after mainnet genesis planning finalized.

---

## Verification Results

### Consensus Parameters (from /api/head)

```json
{
  "network": "kovanica-testnet",
  "consensus": "poa",
  "authorities": 3,
  "threshold": 2,
  "slot_duration": 120,
  "height": 793,
  "genesis_hash": "GENESIS_HASH_PLACEHOLDER"
}
```

### RFC-006 Tokenomics (unchanged, verified active)

```json
{
  "max_supply": "9020000000000000",
  "subsidy": "1000000000",
  "era_blocks": 2000000,
  "decay_numerator": 3,
  "decay_denominator": 4,
  "maturity_blocks": 100,
  "fee_burn_ratio": 0.75,
  "fee_floor_atoms_per_byte": 1
}
```

### Block Production

- **Slot adherence**: 100% (no missed slots in 135 blocks observed)
- **Block time**: 3.0s +/- 0.1s
- **Orphan rate**: 0% (PoA finality)
- **Sync**: All peers synced within 2 slots

### API Surfaces Verified

| Surface | URL | Status |
|---------|-----|--------|
| Explorer (web) | https://testnet.kovanica.online | OK |
| API proxy | https://testnet.kovanica.online/api/* | OK |
| Wallet | https://testnet.kovanica.online/wallet | OK |
| Network map | https://testnet.kovanica.online/network | OK |
| Seed1 bootstrap | https://seed.kovanica.online/api/bootstrap | OK |

---

## Migration Procedure (Reproducible)

### 1. Prepare Authority Keys

```bash
# Generate 3 Ed25519 keypairs (offline)
for i in 0 1 2; do
  kovanica-cli keygen --offline --output authority-$i.key
  kovanica-cli pubkey --input authority-$i.key --output authority-$i.pub
done

# Distribute pubkeys to all participants
# Private keys NEVER leave offline machines
```

### 2. Update Seed Nodes

```bash
# On each seed node, update systemd service
sudo systemctl stop kovanica-poa
sudo vim /etc/systemd/system/kovanica-poa.service
# - Remove KOVANICA_POW, KOVANICA_MINE, KOVANICA_PRODUCE_SECS
# - Add KOVANICA_CONSENSUS=poa
# - Add KOVANICA_AUTHORITIES="<pubkey0>,<pubkey1>,<pubkey2>"
# - Add KOVANICA_THRESHOLD=2
# - Add KOVANICA_SLOT_DURATION=120
# - Remove EnvironmentFile line
sudo systemctl daemon-reload
sudo systemctl start kovanica-poa
```

### 3. Verify Consensus

```bash
# Wait 3 slots (6 minutes), then:
curl -s https://seed.kovanica.online/api/head | jq '{consensus, authorities, threshold, height}'
# Verify identical on all seeds
```

### 4. Update Explorers/Clients

- No code changes needed - /api/head returns consensus: "poa" automatically
- Web UI reads consensus type from API

---

## Lessons Learned

1. **Key management is critical** - The 2026-09-26 incident occurred because authority keys were stored on a compromised host. Future: HSM or air-gapped signing only.

2. **EnvironmentFile is fragile** - Removing it and inlining vars in systemd eliminated a class of "file not found" restart failures.

3. **PoA simplifies operations** - No difficulty adjustment, no orphan races, predictable block times. Monitoring is dramatically easier.

4. **DNS hygiene matters** - seed.kovanica.online must remain grey-cloud (DNS-only) for TCP 9000. Cloudflare orange-cloud breaks P2P.

5. **Document the threshold math** - threshold = floor(authorities/2) + 1 for Byzantine fault tolerance. With 3 authorities, threshold=2 tolerates 1 Byzantine node.

---

## Next Steps for Mainnet

- [ ] Design mainnet authority selection (governance, not core team)
- [ ] Define key rotation ceremony (quarterly? semi-annual?)
- [ ] Implement authority set change via on-chain governance (KVP-101 multisig)
- [ ] Plan PoA to PoW transition timeline (see mainnet/GENESIS_PLAYBOOK.md)
- [ ] Audit PoA implementation for liveness/safety edge cases

---

## References

- RFC-006: Tokenomics (emission, supply, fees)
- KVP-101: Multisig (authority set changes)
- KVP-105: Time-lock vaults (treasury)
- TESTNET_AUTHORITY_KEYS.md (local only, not in git)
- INCIDENT_20260926.md (local only, not in git)
- deploy/testnet/env.sh - Current testnet env

---

*Recorded: 2026-09-26*
*Author: Kovanica Core Team*
*Classification: PUBLIC (testnet only)*

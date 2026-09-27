# Kovanica Mainnet Genesis Playbook

> **Status**: DORMANT — Do not execute until explicitly authorized by governance.
> This document describes the complete ceremony for mainnet genesis.

---

## 1. Prerequisites

- [ ] RFC-006 tokenomics fully activated and stable on testnet (≥ 30 days)
- [ ] PoA consensus validated on testnet (≥ 2 authority rotations)
- [ ] All KVP-101 through KVP-105 features audited and battle-tested
- [ ] Security audit completed (crypto, P2P, consensus, supply invariants)
- [ ] Community governance approval for mainnet launch
- [ ] Legal/compliance review for jurisdiction(s) of operation
- [ ] Monitoring/alerting infrastructure deployed (Grafana, Loki, alertmanager)
- [ ] Incident response runbook documented and rehearsed

---

## 2. Genesis Parameters (RFC-006 Canonical)

| Parameter | Value | Notes |
|-----------|-------|-------|
| Network ID | `kovanica-mainnet` | Distinct from testnet |
| Genesis hash | TBD | Deterministic from ceremony |
| Max supply | 90,200,000 KVNC | `9_020_000_000_000_000` atoms |
| Founder premine | 200,000 KVNC | Single output, time-locked |
| Treasury | 10,000,000 KVNC | 10 × 1M KVNC RFC-005 vaults |
| Curve emission | 80,000,000 KVNC | s₀=10 KVNC, era=2M blocks, α=3/4 |
| Coinbase maturity | 100 blocks | Hard consensus rule |
| Fee split | 75% burn / 25% producer | Hard consensus rule |
| Fee floor | `max(1, subsidy/500_000)` atoms/byte | Height-dependent |
| Consensus | GHOSTDAG k=3 | No PoA on mainnet |

---

## 3. Ceremony Participants

| Role | Count | Selection |
|------|-------|-----------|
| Genesis signers (multisig) | 5 of 7 | Governance-elected, geographically distributed |
| Initial validators (PoA bootstrap) | 7 | Reputable operators, 6-month commitment |
| Witness auditors | 3 | Independent security firms |
| Network coordinators | 2 | Core team (non-signing) |

**Key rule**: No single entity controls >1 genesis signer key. Keys generated offline, never shared.

---

## 4. Ceremony Steps

### Phase 0: Preparation (T-7 days)

1. **Freeze code**: Tag `mainnet-genesis-rc1` in all repos (protocol, node, cli, web)
2. **Build artifacts**: Reproducible builds for `kovanica-node`, `kovanica-cli`
3. **Verify hashes**: Publish SHA256 of all binaries to transparency log
4. **Distribute**: Share binaries + verification instructions with all participants

### Phase 1: Key Generation (T-1 day)

Each genesis signer (offline, air-gapped machine):

```bash
# Generate Ed25519 keypair
kovanica-cli keygen --offline --output genesis-signer-N.key

# Extract public key
kovanica-cli pubkey --input genesis-signer-N.key --output genesis-signer-N.pub

# Verify
kovanica-cli verify-key --input genesis-signer-N.pub
```

- Public keys collected by coordinators
- Multisig script hash computed: `kovanica-cli multisig --threshold 5 --pubkeys <all.pub> --output genesis-multisig.hash`
- All signers verify the multisig hash matches

### Phase 2: Genesis Block Construction (T-0, coordinated)

Coordinator assembles genesis transaction:

```json
{
  "network": "kovanica-mainnet",
  "timestamp": "2026-XX-XXT00:00:00Z",
  "outputs": [
    { "amount": "2000000000000000", "script": "<founder-pubkey> OP_CHECKSIG", "timelock": 525600 },
    { "amount": "1000000000000000", "script": "<vault-1-script>", "vault_id": 1 },
    ...
    { "amount": "1000000000000000", "script": "<vault-10-script>", "vault_id": 10 }
  ],
  "multisig": "genesis-multisig.hash"
}
```

- Founder output: 0.2M KVNC, CSV timelock = 525,600 blocks (~1 year)
- Treasury vaults: 10 × 1M KVNC, RFC-005 scripts with staggered unlocks
- Genesis block signed by all 5+ signers using `kovanica-cli sign --multisig`

### Phase 3: Network Boot (T+0)

1. **Seed nodes**: Deploy 3 seed nodes with genesis block pre-loaded
   - `seed1.mainnet.kovanica.online:9000`
   - `seed2.mainnet.kovanica.online:9000`
   - `seed3.mainnet.kovanica.online:9000`
   - All grey-cloud DNS, no Cloudflare proxy on port 9000

2. **Validators**: Each initial validator starts node with:
   ```bash
   KOVANICA_CONSENSUS=poa
   KOVANICA_AUTHORITIES=<7 validator pubkeys>
   KOVANICA_THRESHOLD=5
   KOVANICA_SLOT_DURATION=120
   KOVANICA_PEERS=seed1.mainnet.kovanica.online:9000,seed2...,seed3...
   KOVANICA_MINE=1
   KOVANICA_DATA=/var/lib/kovanica-mainnet
   ```

3. **Coordinators**: Monitor `/api/head` on all seeds/validators
   - Verify identical genesis hash
   - Verify block production at ~120s slots
   - Verify RFC-006 params in `/api/head`

### Phase 4: PoA → PoW Transition (T+14 days)

After 2 weeks of stable PoA operation:

1. **Governance vote**: On-chain or off-chain vote to disable PoA
2. **Coordinated upgrade**: All validators update to `KOVANICA_CONSENSUS=pow`
3. **PoA keys burned**: Authority keys rotated/destroyed per policy
4. **Monitor**: Difficulty adjustment, block times, orphan rate

### Phase 5: Public Launch (T+30 days)

- Open faucet disabled (`KOVANICA_FAUCET=0`)
- Public explorers/wallets pointed to mainnet
- Announce to community
- Begin exchange integration discussions

---

## 5. Verification Checklist (Post-Genesis)

| Check | Command | Expected |
|-------|---------|----------|
| Genesis hash matches | `curl /api/head \| jq .genesis_hash` | All nodes identical |
| Supply accounting | `curl /api/head \| jq .supply` | 90.2M KVNC cap |
| Coinbase maturity | `kovanica-cli check-maturity --height 0` | 100 blocks |
| Fee burn | `curl /api/head \| jq .fee_burn_ratio` | 0.75 |
| Treasury vaults | `kovanica-cli vault list` | 10 vaults, 1M each |
| Founder timelock | `kovanica-cli output inspect <founder-txid>` | CSV=525600 |

---

## 6. Rollback / Abort Criteria

**Abort immediately if**:
- Genesis hash mismatch between any two seed nodes
- Supply accounting shows >90.2M KVNC minted at genesis
- Any coinbase spendable before 100 blocks
- Fee burn ratio ≠ 75%
- P2P connectivity fails between ≥2 seeds
- Any authority key compromised pre-launch

**Rollback procedure**:
1. Halt all nodes
2. Destroy `KOVANICA_DATA` directories
3. Rotate all authority keys
4. Restart from Phase 1 with new keys

---

## 7. Emergency Contacts

| Role | Contact | Channel |
|------|---------|---------|
| Lead coordinator | TBD | Signal + email |
| Security lead | TBD | Signal + email |
| Infrastructure | TBD | Signal + PagerDuty |
| Legal | TBD | Email |

---

## 8. Post-Launch Milestones

| Milestone | Target | Success Criteria |
|-----------|--------|------------------|
| 1000 blocks | T+2 days | Stable 120s slots, <1% orphans |
| Difficulty retarget | T+7 days | Smooth adjustment, no oscillations |
| First treasury unlock | T+180 days | Vault 1 spends correctly |
| PoA → PoW complete | T+14 days | All validators on PoW |
| Exchange listing | T+90 days | ≥1 reputable exchange |

---

*Document version: 1.0*  
*Last updated: 2026-09-26*  
*Classification: CONFIDENTIAL — Mainnet genesis ceremony*
# Mainnet Genesis — Immediate Next Actions

> **Generated**: 2026-09-26 from PREREQUISITES_TRACKER.md
> **Cadence**: Weekly review until T-30d, then daily

---

## 🔴 Critical Path (Start This Week)

### 1. Security Audit Commission
- [ ] Draft RFP for: crypto, P2P, consensus, supply invariants
- [ ] Send to 3 firms (e.g., Trail of Bits, NCC Group, Kudelski)
- [ ] Target: 30-day engagement, deliver before T-30d
- **Owner**: Security Lead
- **Due**: 2026-10-03

### 2. Legal/Compliance Jurisdiction Analysis
- [ ] Identify target jurisdictions for KVNC
- [ ] Engage counsel for: token classification, AML/KYC, securities law
- [ ] Produce memo: "Mainnet Launch Legal Opinion"
- **Owner**: Legal Counsel
- **Due**: 2026-10-10

### 3. Governance: Authority Election
- [ ] Publish AUTHORITY_KEY_CEREMONY.md for comment
- [ ] Run election for: 7 genesis signers, 7 validators, 3 auditors
- [ ] Publish results with pubkey commitments
- **Owner**: Governance Chair
- **Due**: 2026-10-15

---

## 🟡 Infrastructure (Parallel Track)

### 4. Mainnet DNS & Seed Provisioning
- [ ] Provision 3 VPS (4CPU/8GB/200GB, Ubuntu 24.04)
- [ ] Configure grey-cloud DNS for seed*.mainnet.kovanica.online:9000
- [ ] Install kovanica-node (from mainnet-genesis-rc1 tag)
- [ ] Test P2P connectivity between all 3 seeds
- **Owner**: Infra Lead
- **Due**: 2026-10-20

### 5. Reproducible Builds
- [ ] Define build environment (Dockerfile / Guix / Nix)
- [ ] Build kovanica-node, kovanica-cli reproducibly
- [ ] Publish SHA256 to transparency log (GitHub Releases + sigstore)
- [ ] Verify independent rebuild matches
- **Owner**: Core Team
- **Due**: 2026-10-25

### 6. Monitoring Stack (Mainnet)
- [ ] Deploy separate Grafana/Loki/Prometheus for mainnet
- [ ] Dashboards: block production, sync, peers, mempool, supply
- [ ] Alerts: block lag >2 slots, peer count <2, supply invariant violation
- [ ] PagerDuty/Slack integration
- **Owner**: Infra Lead
- **Due**: 2026-10-25

---

## 🟢 Preparation (Can Start Anytime)

### 7. Code Freeze Candidates
- [ ] Identify `mainnet-genesis-rc1` commit in each repo
- [ ] Run full test suite on candidate
- [ ] Verify no consensus-critical changes since testnet PoA migration
- **Owner**: Core Team
- **Due**: 2026-10-25

### 8. Validator Onboarding Pack
- [ ] Create validator guide: hardware, keys, monitoring, rotation
- [ ] Distribute to 7 elected validators
- [ ] Collect authority pubkeys, verify format
- **Owner**: Coordinator
- **Due**: 2026-10-20

### 9. Ceremony Dry-Run (Testnet Fork)
- [ ] Fork testnet at current height
- [ ] Run full genesis ceremony with placeholder keys
- [ ] Verify genesis hash, block production, sync
- [ ] Document any issues
- **Owner**: Core Team
- **Due**: 2026-11-01

### 10. Incident Response Rehearsal
- [ ] Tabletop: "genesis hash mismatch", "validator offline", "key leak"
- [ ] Update runbook with lessons
- [ ] All coordinators + validators participate
- **Owner**: Ops Lead
- **Due**: 2026-11-10

---

## 📅 Timeline Summary

| Week | Focus |
|------|-------|
| 2026-09-26 → 10-03 | Security audit RFP, Legal kickoff, Governance review |
| 2026-10-03 → 10-10 | Audit firms respond, Legal memo draft, Election prep |
| 2026-10-10 → 10-17 | Authority election, Validator recruitment, Infra provisioning |
| 2026-10-17 → 10-24 | Reproducible builds, Monitoring, Validator onboarding |
| 2026-10-24 → 10-31 | Code freeze, Audit delivery, Dry-run prep |
| 2026-10-31 → 11-07 | Dry-run on testnet fork, Go/No-Go review |
| 2026-11-07 → 11-14 | Final sign-off, Ceremony scheduling |
| **T-0** | **Genesis Ceremony** |

---

## 🚫 Blockers Requiring Decision

| Blocker | Decision Needed | By Whom | By When |
|---------|-----------------|---------|---------|
| Audit engagement | Approve scope & engage firm | Treasury multisig | 2026-10-03 |
| Validator slots | Confirm 7 operators committed | Governance | 2026-10-15 |
| Jurisdiction | Primary legal domicile for foundation | Legal/Board | 2026-10-10 |
| Slot duration | 120s (mainnet) vs 3s (testnet) — confirmed? | Core | 2026-10-03 |

---

## 📋 Weekly Review Template

**Date**: ________  
**Attendees**: ________

| Area | Status | Blocker | Decision Needed |
|------|--------|---------|-----------------|
| Security Audit | | | |
| Legal | | | |
| Governance | | | |
| Infra | | | |
| Builds | | | |
| Monitoring | | | |

**Next Actions**: ________

---

*Update this file weekly. Check off completed items.*

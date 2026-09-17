# Custody Policy

**Document ID:** KOV-POL-OPS-000
**Version:** 1.0
**Classification:** Confidential
**Owner:** CRO / CTO
**Review Cycle:** Annual (or upon material change)
**Status:** Approved

---

## 1. Purpose & Regulatory Basis

This policy establishes the custody framework for Kovanica Protocol d.o.o. ("the CASP") under MiCA, ensuring client assets are safeguarded per Articles 75–77 and Commission Delegated Regulation (EU) 2024/XXXX.

| Regulation | Reference |
|------------|-----------|
| MiCA | Articles 75–77 (safeguarding), Article 16 (governance), Article 30–31 (outsourcing) |
| Commission Delegated Regulation (EU) 2024/XXXX | Technical standards for safeguarding |
| Croatian AML Act | Article 28 (outsourcing) |
| DORA | Articles 28–30 (ICT third-party risk) |
| EBA Guidelines | EBA/GL/2019/02 (outsourcing), EBA/GL/2020/07 (ICT security) |

**Custody Model:** **Partner custody** — Kovanica does not hold private keys directly. All client assets are custodied with **Copper Technologies (UK) Ltd** ("Copper"), a regulated custodian, via their **ClearLoop** and **MPC-based custody** infrastructure.

---

## 2. Governance & Roles

| Role | Responsibility |
|------|----------------|
| **Management Board** | Ultimate accountability for custody arrangements; approves partner selection, policy changes |
| **CRO (Chief Risk Officer)** | Policy owner; monitors custody risk, partner performance, incident escalation |
| **CTO** | Technical integration, API security, key management architecture |
| **MLRO** | AML/CTF oversight of custody flows, SAR triggers from custody events |
| **Compliance Officer** | Regulatory reporting, audit coordination, MiCA safeguarding compliance |
| **Operations Team** | Daily reconciliation, withdrawal processing, client communication |

**Custody Committee:** Quarterly review (CRO, CTO, MLRO, Compliance) — reviews partner KPIs, incidents, regulatory changes, audit findings.

---

## 3. Asset Segregation & Protection

### 3.1 Segregation Principles

| Principle | Implementation |
|-----------|----------------|
| **Client vs. Own Assets** | Strict separation: Client assets held in Copper segregated vaults; Kovanica operational funds in separate corporate wallets |
| **Individual Client Segregation** | Copper ClearLoop provides omnibus segregation with client-level sub-accounting; on-chain addresses derived per client |
| **Legal Title** | Client retains beneficial ownership; Copper holds legal title as custodian; Kovanica has no proprietary claim |
| **Insolvency Remoteness** | Copper UK FCA-regulated; client assets ring-fenced under UK CASS rules; not part of Copper's estate |

### 3.2 Asset Coverage

| Asset Type | Custody Method | Network |
|------------|----------------|---------|
| **KVNC (native)** | Copper MPC vault + ClearLoop | Kovanica mainnet |
| **ERC-20 / EVM tokens** | Copper MPC vault | Ethereum, Polygon, Arbitrum, Optimism, Base |
| **BTC / UTXO assets** | Copper MPC vault | Bitcoin, Litecoin, Bitcoin Cash |
| **Stablecoins (EURC, USDC, USDT)** | Copper MPC vault | Multi-chain |
| **NFTs (if supported)** | Copper vault | EVM chains |

---

## 4. Copper Integration Architecture

### 4.1 Technical Integration

```
┌─────────────────┐     API (mTLS)      ┌──────────────────┐
│  Kovanica Core  │◀───────────────────▶│   Copper API     │
│  (Node/FFI)     │                     │  (ClearLoop/MPC) │
└────────┬────────┘                     └────────┬─────────┘
         │                                       │
         │ 1. Deposit address generation         │
         │ 2. Withdrawal signing (MPC)           │
         │ 3. Balance queries                    │
         │ 4. Transaction monitoring             │
         │ 5. ClearLoop settlement               │
         ▼                                       ▼
┌─────────────────┐                     ┌──────────────────┐
│  Internal Ledger│                     │  Copper Vault    │
│  (kovanica-state)│                    │  (Segregated)    │
└─────────────────┘                     └──────────────────┘
```

### 4.2 Key Management (MPC)

| Aspect | Detail |
|--------|--------|
| **Scheme** | Copper's threshold MPC (t-of-n, n=3, t=2) — no single party holds full key |
| **Key Shares** | Copper (1), Kovanica (1), Backup/Recovery (1) — Kovanica share in HSM |
| **Key Generation** | Distributed key generation (DKG) ceremony; no single point of compromise |
| **Key Rotation** | Annual rotation; emergency rotation on compromise suspicion |
| **Backup/Recovery** | Shamir Secret Sharing (3-of-5) stored in geo-distributed HSMs (AWS CloudHSM, Azure Key Vault) |

### 4.3 API Security

| Control | Implementation |
|---------|----------------|
| **Authentication** | mTLS (client cert + Copper cert) + API key + request signing (Ed25519) |
| **Authorization** | Role-based: `deposit:read`, `withdraw:sign`, `balance:read`, `settlement:execute` |
| **Rate Limiting** | 100 req/min per endpoint; burst allowance 200 |
| **Encryption** | TLS 1.3 in transit; AES-256-GCM for payload encryption at rest |
| **Audit Logging** | All API calls logged with request/response hashes (immutable) |

---

## 5. Deposit & Withdrawal Flows

### 5.1 Deposit Flow

```text
Client                          Kovanica                          Copper
  │                                │                                │
  │ 1. Request deposit address     │                                │
  │───────────────────────────────▶│                                │
  │                                │ 2. Generate address via API    │
  │                                │──────────────────────────────▶│
  │                                │◀──────────────────────────────│
  │ 3. Return address (KVNC/hex)   │                                │
  │◀───────────────────────────────│                                │
  │                                │                                │
  │ 4. Client sends on-chain       │                                │
  │────────────────────────────────────────────────────────────────▶│
  │                                │                                │
  │                                │ 5. Webhook: deposit detected   │
  │                                │◀──────────────────────────────│
  │                                │ 6. Credit internal ledger      │
  │                                │ 7. Notify client               │
  │ 8. Balance updated             │                                │
  │◀───────────────────────────────│                                │
```

### 5.2 Withdrawal Flow

```text
Client                          Kovanica                          Copper
  │                                │                                │
  │ 1. Request withdrawal          │                                │
  │───────────────────────────────▶│                                │
  │                                │ 2. Validate (KYC, limits, AML) │
  │                                │ 3. Create withdrawal request   │
  │                                │──────────────────────────────▶│
  │                                │                                │
  │                                │ 4. MPC signing ceremony        │
  │                                │    (Kovanica share + Copper)   │
  │                                │◀──────────────────────────────│
  │                                │                                │
  │                                │ 5. Broadcast signed tx         │
  │                                │──────────────────────────────▶│
  │                                │                                │
  │                                │ 6. Webhook: tx broadcast       │
  │                                │◀──────────────────────────────│
  │                                │ 7. Debit internal ledger       │
  │                                │ 8. Notify client               │
  │ 9. Confirmation                │                                │
  │◀───────────────────────────────│                                │
```

### 5.3 Limits & Controls

| Limit Type | Tier 1 (Low) | Tier 2 (Standard) | Tier 3 (High) |
|------------|--------------|-------------------|---------------|
| **Daily Withdrawal** | €5,000 | €50,000 | €500,000 |
| **Single Withdrawal** | €2,000 | €25,000 | €250,000 |
| **Monthly Withdrawal** | €20,000 | €200,000 | €2,000,000 |
| **Approval** | Automated | Automated + 2FA | Dual approval (Ops + Compliance) |
| **Cooling Period** | None | 1h for > €10k | 4h for > €100k |

**Emergency Freeze:** CRO/MLRO can trigger instant withdrawal halt via Copper API (kill switch).

---

## 6. Reconciliation & Reporting

### 6.1 Daily Reconciliation

| Process | Frequency | Owner | Tolerance |
|---------|-----------|-------|-----------|
| **On-chain vs. Copper** | Daily 02:00 UTC | Operations | Zero tolerance |
| **Copper vs. Internal Ledger** | Daily 03:00 UTC | Operations | Zero tolerance |
| **Client Balances Sum vs. Total Custody** | Daily 04:00 UTC | Compliance | Zero tolerance |
| **Unreconciled Items** | Immediate escalation | CRO | 0 items |

### 6.2 Reporting

| Report | Frequency | Recipient | Content |
|--------|-----------|-----------|---------|
| **Custody Reconciliation Report** | Daily | CRO, CTO, Compliance | Balances, breaks, resolution status |
| **Copper KPI Dashboard** | Real-time | Custody Committee | Uptime, latency, error rates, signing success |
| **Monthly Custody Statement** | Monthly | Management Board, Auditor | Holdings, flows, fees, incidents |
| **Quarterly Safeguarding Report** | Quarterly | HANFA (on request) | MiCA Art. 77 compliance evidence |

---

## 7. Incident Management

### 7.1 Custody-Specific Incidents

| Incident Type | Detection | Response | Escalation |
|---------------|-----------|----------|------------|
| **MPC Signing Failure** | API error / timeout | Retry 3x; fallback to backup share; alert | CTO → CRO → Board (if > 1h) |
| **Copper API Outage** | Health check failure | Activate manual withdrawal queue; notify clients | CTO → CRO (immediate) |
| **Balance Mismatch** | Reconciliation break | Freeze affected assets; investigate; SAR if suspicious | CRO → MLRO → Board |
| **Key Compromise Suspicion** | Anomalous signing / alert | Emergency key rotation; freeze all withdrawals | CTO → CRO → Board (immediate) |
| **Copper Insolvency** | Regulatory notice / news | Activate contingency custodian; client communication | Board → Legal → Regulator |

### 7.2 Contingency Custodian

| Custodian | Status | Activation Trigger | Timeline |
|-----------|--------|-------------------|----------|
| **Fireblocks** | Pre-contracted (standby) | Copper insolvency / prolonged outage > 24h | < 48h migration |
| **BitGo** | Evaluated (backup) | Fireblocks unavailable | < 72h migration |

---

## 8. Copper-Specific Annex

### 8.1 Contractual Terms (MSA Summary)

| Clause | Requirement |
|--------|-------------|
| **Regulatory Status** | Copper UK: FCA registered (FRN 927891); EU: Copper Technologies Ireland Ltd (CBI regulated) |
| **Insurance** | $250M species insurance (Lloyd's); $50M cyber; crime policy |
| **SOC Reports** | SOC 2 Type II annual; SOC 1 available |
| **Audit Rights** | Annual on-site; quarterly remote; regulator access |
| **Data Location** | EU (Ireland) primary; UK backup |
| **Termination** | 90 days; 30 days for material breach; orderly migration clause |
| **Liability** | Cap at 12 months fees; unlimited for gross negligence / fraud / data breach |

### 8.2 ClearLoop Settlement

| Parameter | Value |
|-----------|-------|
| **Settlement Cycles** | 3x daily (08:00, 14:00, 20:00 UTC) |
| **Netting** | Multilateral netting across ClearLoop participants |
| **Settlement Finality** | On-chain confirmation (Kovanica: 1 block finality) |
| **Collateral** | Not required (credit limits based on KYC/AML) |
| **Dispute Window** | 2 hours post-settlement |

### 8.3 Copper KPIs (Monitored)

| KPI | Target | Alert |
|-----|--------|-------|
| **API Uptime** | 99.95% | < 99.9% (5m) |
| **Deposit Detection Latency** | < 30s | > 2m |
| **Withdrawal Signing Latency (p95)** | < 10s | > 30s |
| **MPC Signing Success Rate** | 99.9% | < 99.5% |
| **ClearLoop Settlement Success** | 100% | Any failure |
| **Reconciliation Breaks** | 0 | Any break |

---

## 9. Audit & Compliance

| Activity | Frequency | Owner | Evidence |
|----------|-----------|-------|----------|
| **Internal Custody Audit** | Quarterly | Compliance | Audit report, findings tracker |
| **External Audit (Big 4)** | Annual | CRO | Audit opinion, management letter |
| **Copper SOC 2 Review** | Annual | CTO | SOC 2 report, bridge letter |
| **Regulatory Examination Prep** | Ongoing | Compliance | Document repository, test scripts |
| **Penetration Test (Custody APIs)** | Annual | CTO | Pentest report, remediation tracker |

---

## 10. Cross-References

| Document | Reference |
|----------|-----------|
| `01_Custody_Partner_Due_Diligence.md` | Copper assessment scorecard, selection rationale |
| `03_Outsourcing_Register.md` | Copper as critical outsourced function |
| `02_ICT_Risk_Management.md` | ICT third-party risk, incident management |
| `06_Incident_Response_Plan.md` | Custody incident response procedures |
| `01_AML/00_AML_Policy.md` | AML monitoring of custody flows |
| `01_AML/03_Suspicious_Activity_Reporting.md` | SAR triggers from custody events |

---

## 11. Document Control

| Field | Value |
|-------|-------|
| **Document ID** | KOV-POL-OPS-000 |
| **Version** | 1.0 |
| **Classification** | Confidential |
| **Owner** | CRO / CTO |
| **Approved By** | Management Board |
| **Approval Date** | [DATE] |
| **Next Review** | [DATE + 1 year] |
| **Distribution** | Management Board, CRO, CTO, MLRO, Compliance, Operations |

### Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-15 | Compliance Team | Initial version for MiCA CASP application |

---

*End of Document*
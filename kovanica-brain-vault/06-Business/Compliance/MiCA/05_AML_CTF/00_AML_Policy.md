# AML/CTF Policy — Kovanica Protocol CASP

## 1. Purpose & Regulatory Basis

**Regulation**: 
- MiCA Regulation (EU) 2023/1114, Articles 66–67 (AML obligations for CASPs)
- EU AML Directive (AMLD6) — Directive (EU) 2018/843
- Transfer of Funds Regulation (TFR) — Regulation (EU) 2023/1113
- Croatian Act on Prevention of Money Laundering and Terrorist Financing (Zakon o sprječavanju pranja novca i financiranja terorizma)
- FATF Recommendations (2012, updated 2023) — Virtual Assets & VASPs

**Supervisors**: 
- HANFA (CASP authorization & ongoing supervision)
- Ured za sprječavanje pranja novca (Croatian FIU — SAR recipient)
- HNB (Croatian National Bank — prudential oversight)

**Scope**: All crypto-asset services provided by Kovanica Protocol Ltd. (exchange fiat↔crypto, crypto↔crypto, transfer, execution of orders) for retail and professional clients.

## 2. Risk-Based Approach (RBA)

### 2.1 Enterprise-Wide Risk Assessment (EWRA)
- **Frequency**: Annual (minimum) + ad-hoc upon material change
- **Methodology**: Inherent risk × Control effectiveness = Residual risk
- **Risk Categories**: Client, Product/Service, Delivery Channel, Geography, Technology
- **Output**: Risk heat map, mitigation priorities, resource allocation
- **Governance**: Approved by Management Board; reviewed by Supervisory Board

### 2.2 Client Risk Rating (CRR)
| Risk Level | Criteria | CDD Level | Monitoring Frequency | Approval |
|------------|----------|-----------|---------------------|----------|
| **Low** | Retail, low volume, EU resident, no PEP, transparent source | Standard | Annual review | Automated |
| **Medium** | Professional, moderate volume, non-EU but cooperative jurisdiction, complex structure | Enhanced | Quarterly review | Compliance Officer |
| **High** | PEP/RCA, high volume, high-risk jurisdiction, opaque structure, nominee | Enhanced + EDD | Monthly review | Director B (CCO) |
| **Prohibited** | Sanctioned, shell bank, anonymous, refused CDD | N/A — Reject | N/A | N/A |

### 2.3 Product/Service Risk
| Service | Inherent Risk | Key Controls |
|---------|---------------|--------------|
| Fiat↔Crypto Exchange | High (fiat gateway) | Sumsub KYC, bank-grade AML, transaction limits, source of funds |
| Crypto↔Crypto Exchange | Medium | Travel Rule, blockchain analytics, counterparty verification |
| Transfer (Custodial) | High | Travel Rule (€0 threshold), beneficiary verification, limits |
| Transfer (Non-Custodial) | Medium | Address screening, blockchain analytics, client education |
| Execution of Orders | Medium | Best execution, conflict management, audit trail |

### 2.4 Geographic Risk
| Jurisdiction Category | Countries | Treatment |
|----------------------|-----------|-----------|
| **EU/EEA** | All 27 + EEA | Standard (home state rules apply) |
| **FATF Compliant** | US, UK, CH, SG, JP, AU, CA, etc. | Standard + enhanced monitoring |
| **FATF Grey List** | [Current list] | Enhanced CDD, senior approval, transaction limits |
| **FATF Black List / High-Risk** | DPRK, Iran, Myanmar | Prohibited (no onboarding, no transactions) |
| **Sanctioned** | Russia, Belarus, Syria, etc. (per EU) | Prohibited — asset freeze, SAR mandatory |

### 2.5 Delivery Channel Risk
| Channel | Risk | Controls |
|---------|------|----------|
| Web App (KYC via Sumsub) | Low | Automated CDD, liveness, document verification |
| Mobile App (Light Node) | Low | Same as web + device fingerprinting |
| API (Institutional) | Medium | API key management, IP whitelisting, enhanced monitoring |
| OTC / Manual | High | Dual approval, enhanced CDD, recorded communications |

## 3. Customer Due Diligence (CDD)

### 3.1 Timing (MiCA Art. 66 / AMLD6 Art. 13)
- **Before** establishing business relationship or executing occasional transaction ≥ €1,000
- **Ongoing** throughout relationship
- **Event-driven** upon trigger (Section 7)

### 3.2 Natural Persons (Retail & Professional)

#### 3.2.1 Standard CDD (Low/Medium Risk)
| Data Element | Source | Verification |
|--------------|--------|--------------|
| Full Legal Name | Government ID | Sumsub: Document + Biometric |
| Date/Place of Birth | Government ID | Sumsub: Document + Biometric |
| Nationality | Government ID | Sumsub: Document |
| Residential Address | Utility bill / Bank statement / Gov letter | Sumsub: Document (≤3 months) |
| Tax Residence(s) | Self-certification (CRS/DAC8) | Cross-check with ID |
| OIB (Croatian PIN) | Government ID | HANFA register (if Croatian) |
| Email / Phone | Client provided | OTP verification |
| Source of Funds | Declaration + Bank statement | Sumsub: Open Banking / Document |
| Purpose of Relationship | Client declaration | Consistency check |

#### 3.2.2 Enhanced CDD (High Risk / PEP / RCA)
| Additional Element | Requirement |
|-------------------|-------------|
| **Source of Wealth** | Detailed declaration + independent verification (corporate registry, property records, etc.) |
| **Senior Management Approval** | Director B (CCO) sign-off before onboarding |
| **Enhanced Monitoring** | Monthly transaction review, real-time alerts |
| **Adverse Media Deep Dive** | World-Check + open source investigation |
| **Beneficial Ownership** | Full UBO chain to natural persons |

### 3.3 Legal Entities (Corporate/Institutional Clients)

#### 3.3.1 Standard CDD
| Data Element | Source | Verification |
|--------------|--------|--------------|
| Legal Name & Form | Certificate of Incorporation | Commercial register extract (≤3 months) |
| Registered Office | Certificate of Incorporation | Commercial register extract |
| Registration Number | Certificate of Incorporation | Commercial register extract |
| LEI | GLEIF | GLEIF database |
| Directors / Authorised Signatories | Board resolution / Register | Commercial register + ID verification |
| Shareholders ≥10% | Shareholder register / Declaration | UBO chain to natural persons |
| UBOs (≥25% or control) | UBO declaration + Register | Independent verification (register, sanctions) |
| Purpose & Nature of Business | Business plan / Website | Consistency with on-chain activity |
| Source of Funds | Bank statements / Audit reports | Trace to origin |

#### 3.3.2 Enhanced CDD (Complex Structures / High-Risk Jurisdiction)
- Full ownership chain diagram (no gaps)
- Independent verification of each layer
- Source of wealth for each UBO
- Senior management + legal counsel approval

### 3.4 Simplified CDD (Permitted Cases Only)
- **Eligible**: EU/EEA regulated credit/financial institutions, listed companies, public authorities
- **Not Eligible**: Crypto-asset clients, high-risk jurisdictions, PEP/RCA
- **Minimum**: LEI + regulated status verification + purpose confirmation

## 4. Ongoing Monitoring

### 4.1 Transaction Monitoring
| Alert Type | Rule | Threshold | Action |
|------------|------|-----------|--------|
| **Velocity** | > 10 tx/hour or > 50 tx/day | Count | Auto-alert → Analyst review |
| **Volume** | > €50k/day or > €200k/week | EUR equivalent | Auto-alert → Analyst review |
| **Structuring** | Multiple < €1k to avoid threshold | Pattern | Auto-alert → SAR assessment |
| **Geographic** | To/from high-risk jurisdiction | Any | Block + SAR |
| **Counterparty** | Sanctioned / High-risk VASP | Any | Block + SAR |
| **Blockchain Analytics** | Mixer, darknet, ransomware, exploit | Any | Block + SAR |
| **Travel Rule Mismatch** | Originator/beneficiary data mismatch | Any | Hold + Investigate |

### 4.2 Blockchain Analytics Integration
- **Provider**: [Chainalysis / Elliptic / TRM / Crystal — TBD]
- **Coverage**: BTC, ETH, TRX, BSC, Polygon, KVNC, major ERC-20/SPL
- **Real-time**: API screening on every deposit/withdrawal
- **Batch**: Daily full-portfolio scan

### 4.3 Periodic Review
| Client Risk | Review Frequency | Scope |
|-------------|------------------|-------|
| Low | Annual | KYC refresh, sanctions re-screen, activity consistency |
| Medium | Semi-annual | + Source of funds update, UBO re-verification |
| High | Quarterly | + Enhanced transaction review, adverse media deep dive |

## 5. Record Keeping (AMLD6 Art. 40 / MiCA Art. 67)

| Record Type | Retention Period | Format |
|-------------|------------------|--------|
| CDD Documents (ID, proofs, declarations) | 10 years post-relationship | Digital (encrypted) + Original (if physical) |
| Transaction Records | 10 years post-transaction | Digital (immutable ledger + database) |
| SARs & Internal Reports | 10 years post-filing | Digital (restricted access) |
| Monitoring Alerts & Dispositions | 10 years | Digital |
| Training Records | 5 years | Digital |
| Outsourcing Due Diligence (Sumsub) | 10 years post-contract | Digital |

## 6. Suspicious Activity Reporting (SAR)

### 6.1 Internal Escalation
```
Analyst Detection → AML Officer Review (24h) → CCO Decision (24h) → FIU Filing (Immediate)
```

### 6.2 Filing Requirements
- **Threshold**: No minimum — suspicion-based
- **Timeline**: Without delay (max 24h from decision)
- **Channel**: Ured za sprječavanje pranja novca — GoAML / Secure portal
- **Tipping-Off Prohibition**: Strict — no client notification

### 6.3 SAR Quality Standards
- Complete originator/beneficiary data (per TFR)
- Blockchain transaction hashes, addresses, amounts
- Client risk profile & CDD summary
- Analyst rationale & CCO approval

## 7. Trigger Events for Re-Assessment

| Event | Action | Timeline |
|-------|--------|----------|
| Sanctions list update | Immediate re-screen all clients | 24 hours |
| PEP status change | Re-assess EDD requirements | 5 business days |
| Adverse media alert | Screen & escalate | 2 business days |
| Client profile change (volume, jurisdiction, product) | Targeted CDD refresh | 10 business days |
| Regulatory guidance update | Policy gap analysis | 30 days |
| Major incident / breach | Full policy review | 10 business days |

## 8. Training & Awareness

| Audience | Frequency | Content | Delivery |
|----------|-----------|---------|----------|
| All Staff | Annual | AML/CTF basics, SAR recognition, tipping-off, MiCA/TFR | E-learning + Test |
| Client-Facing | Semi-annual | CDD procedures, red flags, high-risk scenarios | Workshop |
| AML/Compliance | Quarterly | Typologies, regulatory updates, tools | Specialist training |
| Senior Management | Annual | Governance, accountability, regulatory expectations | Board session |
| New Joiners | Onboarding (Day 1) | Policy, procedures, reporting lines | Mandatory |

## 9. Outsourcing — Sumsub KYC/AML

**Provider**: Sumsub Inc.
**Services**: Identity verification, document authentication, biometric/liveness, PEP/sanctions screening, adverse media, ongoing monitoring
**Contract**: Critical outsourcing per MiCA Art. 30 / DORA Art. 28
**Due Diligence**: See `02_Ops/01_Custody_Partner_Due_Diligence.md` (adapted for Sumsub)
**Monitoring**: 
- SLA: 99.5% uptime, < 5 min verification time
- KPI: False positive rate < 2%, False negative rate < 0.1%
- Audit: Annual SOC2 Type II review, penetration test
- Data: GDPR-compliant DPA, EU data residency option

## 10. Sanctions Compliance

See `01_AML/03_Sanctions_Policy.md` for detailed policy.

## 11. Travel Rule Compliance

See `01_AML/01_Travel_Rule_Policy.md` for detailed policy.

## 12. Document Control

| Element | Detail |
|---------|--------|
| **Version** | 1.0 |
| **Classification** | Confidential — Regulatory / AML |
| **Owner** | Director B (CCO) / AML Officer |
| **Approved By** | Management Board |
| **Review Cycle** | Annual or upon trigger (Section 7) |
| **Retention** | 10 years (policy versions) |
| **Related Documents** | `01_Travel_Rule_Policy.md`, `03_Sanctions_Policy.md`, `04_Suspicious_Activity_Reporting.md`, `04_Sumsub_Integration_Policy.md`, `02_Shareholder_Register.md`, `04_Fit_Proper_Shareholders.md` |

---

**End of Document**

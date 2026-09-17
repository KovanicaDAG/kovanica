# Custody Partner Due Diligence

**Document ID:** KOV-POL-OPS-001
**Version:** 1.0
**Classification:** Confidential
**Owner:** CRO / CTO
**Review Cycle:** Annual (or upon material change)
**Status:** Approved

---

## 1. Purpose & Regulatory Basis

This document records the due diligence assessment of **Copper Technologies (UK) Ltd** ("Copper") as the appointed custody partner for Kovanica Protocol d.o.o. ("the CASP") under MiCA Articles 75–77 and outsourcing requirements (Articles 30–31, DORA Articles 28–30).

| Regulation | Requirement |
|------------|-------------|
| MiCA Art. 75–77 | Safeguarding: custodian must be authorised, segregated, insured |
| MiCA Art. 30–31 | Outsourcing: critical function assessment, contractual terms, monitoring |
| DORA Art. 28–30 | ICT third-party risk: register, risk assessment, exit strategy |
| EBA/GL/2019/02 | Outsourcing guidelines: due diligence, ongoing monitoring |
| Commission Delegated Regulation (EU) 2024/XXXX | Technical standards for safeguarding |

**Assessment Date:** 2026-01-15
**Assessed By:** CRO, CTO, MLRO, Compliance, Legal
**Next Review:** 2027-01-15 (or upon material change)

---

## 2. Copper Entity Profile

| Attribute | Detail |
|-----------|--------|
| **Legal Name** | Copper Technologies (UK) Ltd |
| **Registered Office** | 10 Fenchurch Street, London EC3M 3BE, United Kingdom |
| **Company Number** | 11234567 (UK Companies House) |
| **Regulatory Status (UK)** | FCA Registered Cryptoasset Business (FRN: 927891) — AML/CTF supervision |
| **Regulatory Status (EU)** | Copper Technologies Ireland Ltd — Central Bank of Ireland (CBI) authorised as VASP/CASP |
| **Ownership** | Private (institutional investors: a16z, Paradigm, etc.) |
| **Group Structure** | Copper Technologies Ltd (UK parent) → Copper Technologies Ireland Ltd (EU subsidiary) |
| **Key Personnel** | Dmitry Tokarev (CEO), Philip Hammond (Chair), FCA-approved SMFs |
| **Employees** | ~250 (2024) across London, Dublin, New York, Singapore |
| **Financials (2023)** | Revenue: $45M; EBITDA: positive; Cash: $120M+ |

---

## 3. Due Diligence Scorecard

### 3.1 Regulatory & Legal (Weight: 25%)

| Criterion | Score (1–5) | Evidence | Notes |
|-----------|-------------|----------|-------|
| **FCA Registration (UK)** | 5 | FCA Register FRN 927891 | Active, no enforcement actions |
| **CBI Authorisation (EU)** | 5 | CBI Register | CASP authorised, MiCA-ready |
| **Sanctions Compliance** | 5 | OFAC/EU/UN screening, no designations | Automated screening on all flows |
| **Legal Proceedings** | 5 | Court searches (UK, IE, US) | No material litigation |
| **Regulatory Correspondence** | 4 | FCA/CBI supervisory letters | Routine; no material findings |
| **Licence Coverage** | 5 | Custody, exchange, transfer | Covers all Kovanica services |

**Subtotal: 29/30**

### 3.2 Financial Soundness (Weight: 20%)

| Criterion | Score (1–5) | Evidence | Notes |
|-----------|-------------|----------|-------|
| **Capital Adequacy** | 5 | Audited accounts 2023, regulatory capital | Exceeds FCA/CBI requirements |
| **Insurance Coverage** | 5 | $250M species (Lloyd's), $50M cyber, crime | Industry-leading |
| **Profitability** | 4 | EBITDA positive 2023 | Growth phase, reinvesting |
| **Liquidity** | 5 | $120M+ cash, no debt | Strong runway |
| **Ownership Stability** | 4 | Institutional investors, no single control | a16z, Paradigm, others |
| **Audit Opinion** | 5 | Big 4 (PwC) unqualified 2023 | Clean opinion |

**Subtotal: 28/30**

### 3.3 Operational Resilience (Weight: 20%)

| Criterion | Score (1–5) | Evidence | Notes |
|-----------|-------------|----------|-------|
| **SOC 2 Type II** | 5 | Annual (2023, 2024) | No exceptions |
| **ISO 27001** | 5 | Certified (2023) | Scope: custody, ClearLoop |
| **Business Continuity** | 5 | BCP tested quarterly | RTO < 4h, RPO < 1h |
| **Disaster Recovery** | 5 | Geo-redundant (UK, IE, US) | Automated failover tested |
| **Incident Management** | 4 | P1 response < 15 min | Post-mortems published |
| **Change Management** | 4 | CAB process, staged rollouts | Feature flags, canary deploys |

**Subtotal: 28/30**

### 3.4 Technical Architecture (Weight: 15%)

| Criterion | Score (1–5) | Evidence | Notes |
|-----------|-------------|----------|-------|
| **MPC Key Management** | 5 | t-of-n (3,2), DKG, no single key | Industry standard |
| **ClearLoop Settlement** | 5 | Live, 3x daily, multilateral netting | Proven at scale |
| **API Reliability** | 4 | 99.95% uptime SLA | Status page public |
| **Blockchain Coverage** | 5 | 50+ chains, Kovanica mainnet added | Custom integration |
| **Security Testing** | 5 | Annual pentest, bug bounty | Critical findings 0 |
| **Key Rotation** | 4 | Annual + emergency | Tested procedure |

**Subtotal: 28/30**

### 3.5 Service Quality & Support (Weight: 10%)

| Criterion | Score (1–5) | Evidence | Notes |
|-----------|-------------|----------|-------|
| **SLA Commitments** | 5 | 99.95% uptime, <500ms p95 | Contractual |
| **Support Model** | 4 | 24/7 P1, TAM assigned | Dedicated Slack channel |
| **Onboarding Speed** | 5 | < 2 weeks technical | ClearLoop API docs |
| **Reporting/Transparency** | 5 | Real-time dashboard, monthly reviews | Self-service + scheduled |
| **Client References** | 5 | 5+ Tier 1 refs (exchanges, funds) | Verified |

**Subtotal: 24/25**

### 3.6 Contractual & Commercial (Weight: 10%)

| Criterion | Score (1–5) | Evidence | Notes |
|-----------|-------------|----------|-------|
| **MSA Terms** | 5 | Negotiated, MiCA/DORA compliant | Legal reviewed |
| **Data Processing Addendum** | 5 | SCC + UK Addendum, EU data residency | GDPR compliant |
| **Liability Cap** | 4 | 12 months fees, unlimited for gross negligence | Standard |
| **Termination/Migration** | 5 | 90 days, orderly migration clause | Tested |
| **Audit Rights** | 5 | Annual on-site, quarterly remote | Regulator access |
| **Pricing Transparency** | 5 | Volume-tiered, no hidden fees | Predictable |

**Subtotal: 29/30**

---

## 4. Consolidated Score

| Category | Weight | Score | Weighted |
|----------|--------|-------|----------|
| Regulatory & Legal | 25% | 29/30 | 24.2% |
| Financial Soundness | 20% | 28/30 | 18.7% |
| Operational Resilience | 20% | 28/30 | 18.7% |
| Technical Architecture | 15% | 28/30 | 14.0% |
| Service Quality | 10% | 24/25 | 9.6% |
| Contractual & Commercial | 10% | 29/30 | 9.7% |
| **TOTAL** | **100%** | | **94.9%** |

**Result: APPROVED** (Threshold: ≥ 80%)

---

## 5. Key Risk Findings & Mitigations

| Risk | Rating | Mitigation |
|------|--------|------------|
| **Single custodian concentration** | Medium | Contingency custodian (Fireblocks) pre-contracted; migration tested annually |
| **UK/EU regulatory divergence post-Brexit** | Low | Dual-regulated (FCA + CBI); EU entity for MiCA passporting |
| **MPC key share held by Kovanica** | Low | HSM-protected; Shamir backup; rotation tested |
| **ClearLoop netting credit risk** | Low | No collateral; credit limits per counterparty; daily settlement |
| **Copper acquisition/change of control** | Low | Change of control clause in MSA; termination right |

---

## 6. Ongoing Monitoring Programme

| Monitoring Activity | Frequency | Owner | KPI / Threshold |
|---------------------|-----------|-------|-----------------|
| **Regulatory Status Check** | Monthly | Compliance | FCA/CBI register clean |
| **Financial Health Review** | Quarterly | CRO | Capital ratios, insurance valid |
| **SOC 2 / ISO Review** | Annual | CTO | Reports received, no critical findings |
| **KPI Dashboard Review** | Weekly | Operations | All KPIs green (see Custody Policy) |
| **Incident Review** | Per incident | CRO | RCA completed < 30 days |
| **Penetration Test Review** | Annual | CTO | Critical=0, High<3 |
| **Contract Compliance** | Quarterly | Legal | All clauses met |
| **Sanctions Screening Test** | Monthly | MLRO | Test vector detected |
| **Business Continuity Test** | Quarterly | CTO | RTO/RPO met |
| **Key Rotation Drill** | Annual | CTO | Successful rotation |

---

## 7. Exit Strategy

| Phase | Action | Timeline | Owner |
|-------|--------|----------|-------|
| **Trigger** | Material breach, insolvency, regulatory revocation, strategic decision | T+0 | CRO/Board |
| **Notification** | Formal notice to Copper (90 days per MSA) | T+1d | Legal |
| **Migration Planning** | Activate Fireblocks contingency; parallel run | T+1d–T+30d | CTO/Operations |
| **Asset Migration** | Phased transfer: testnet → mainnet; reconciliation at each step | T+30d–T+60d | CTO/Operations |
| **Client Communication** | 30-day advance notice; dedicated support | T+1d–T+60d | Compliance/Ops |
| **Regulatory Notification** | HANFA informed of custodian change | T+1d | Compliance |
| **Completion** | All assets migrated; Copper access revoked; final reconciliation | T+60d–T+90d | CRO/CTO |

**Maximum Migration Window:** 90 days (per MSA)

---

## 8. Approval & Sign-Off

| Role | Name | Signature | Date |
|------|------|-----------|------|
| **CRO** | [Name] | | |
| **CTO** | [Name] | | |
| **MLRO** | [Name] | | |
| **Compliance Officer** | [Name] | | |
| **Legal Counsel** | [Name] | | |
| **Management Board Chair** | [Name] | | |

---

## 9. Cross-References

| Document | Reference |
|----------|-----------|
| `00_Custody_Policy.md` | Custody framework, Copper integration details |
| `03_Outsourcing_Register.md` | Copper as critical outsourced function entry |
| `02_ICT_Risk_Management.md` | ICT third-party risk assessment |
| `01_AML/00_AML_Policy.md` | AML monitoring of custody flows |
| `01_AML/02_Sanctions_Policy.md` | Shared watchlists, screening integration |

---

## 10. Document Control

| Field | Value |
|-------|-------|
| **Document ID** | KOV-POL-OPS-001 |
| **Version** | 1.0 |
| **Classification** | Confidential |
| **Owner** | CRO / CTO |
| **Approved By** | Management Board |
| **Approval Date** | [DATE] |
| **Next Review** | [DATE + 1 year] |
| **Distribution** | Management Board, CRO, CTO, MLRO, Compliance, Legal |

### Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-15 | Compliance Team | Initial version for MiCA CASP application |

---

*End of Document*
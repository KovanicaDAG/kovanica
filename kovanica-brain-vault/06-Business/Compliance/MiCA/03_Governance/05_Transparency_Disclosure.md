# Transparency & Disclosure Policy

**Document ID:** KOV-POL-GOV-005
**Version:** 1.0
**Classification:** Public
**Owner:** CCO / Legal
**Review Cycle:** Annual (or upon material change)
**Status:** Approved

---

## 1. Purpose & Regulatory Basis

This document establishes the transparency and disclosure framework for Kovanica Protocol d.o.o. ("the CASP") under **MiCA Articles 62–66**, **MiCA Title II (CASP Requirements)**, **EBA Guidelines on Transparency**, and **Croatian Capital Market Act**.

| Regulation | Key Requirements |
|------------|------------------|
| MiCA Art. 62 | Pre-contractual disclosure: services, costs, risks, conflicts |
| MiCA Art. 63 | Ongoing disclosure: prices, execution, conflicts, complaints |
| MiCA Art. 64 | Website disclosure: whitepapers, policies, financial info |
| MiCA Art. 65 | Marketing communications: fair, clear, not misleading |
| MiCA Art. 66 | Transaction reporting: to competent authority, trade repositories |
| EBA/GL/2022/xx | Transparency standards for CASPs |
| Croatian CMA | Prospectus-like disclosure for public offers |

---

## 2. Disclosure Principles

| Principle | Application |
|-----------|-------------|
| **Fair, Clear, Not Misleading** | All communications reviewed by Compliance before publication |
| **Timeliness** | Material information disclosed promptly (Art. 63: "without undue delay") |
| **Accessibility** | Website, API, email, post; Croatian + English |
| **Consistency** | Cross-channel alignment; single source of truth |
| **Proportionality** | Retail: plain language, key info highlighted; Professional: detailed |
| **Audit Trail** | All disclosures versioned, timestamped, retained 10 years |

---

## 3. Pre-Contractual Disclosure (MiCA Art. 62)

### 3.1 Client Onboarding Package

| Document | Retail | Professional | Delivery |
|----------|--------|--------------|----------|
| **Terms & Conditions** | ✅ | ✅ | Digital signature |
| **Key Information Document (KID)** | ✅ | ✅ | Pre-contract |
| **Fee Schedule** | ✅ | ✅ | Pre-contract |
| **Risk Warning** | ✅ | ✅ | Pre-contract (prominent) |
| **Conflict of Interest Summary** | ✅ | ✅ | Pre-contract |
| **Complaints Procedure** | ✅ | ✅ | Pre-contract |
| **Privacy Notice** | ✅ | ✅ | Pre-contract |
| **Order Execution Policy** | ✅ | ✅ | Pre-contract |
| **Best Execution Report (annual)** | ✅ | ✅ | On request |
| **Safeguarding Disclosure** | ✅ | ✅ | Pre-contract |

### 3.2 Key Information Document (KID) — Per Service

| Service | KID Sections |
|---------|--------------|
| **Exchange (fiat↔crypto)** | Product, costs, risks, performance scenarios, holding period, complaints |
| **Exchange (crypto↔crypto)** | Product, costs, risks, performance scenarios, holding period, complaints |
| **Transfer** | Product, costs, risks, speed, complaints |
| **Execution** | Product, costs, risks, execution factors, complaints |

**KID Format:** ESMA template (PRIIPs-style), max 3 pages, Croatian + English.

---

## 4. Ongoing Disclosure (MiCA Art. 63)

### 4.1 Pricing & Execution Transparency

| Disclosure | Frequency | Channel | Audience |
|------------|-----------|---------|----------|
| **Real-time Prices** | Continuous | API, Website, App | All |
| **Order Book Depth** | Continuous | API, Website | All |
| **Spreads (bid/ask)** | Continuous | API, Website | All |
| **Execution Quality** | Quarterly | Website, Annual Report | All |
| **Slippage Statistics** | Monthly | Website | All |
| **Latency Metrics** | Monthly | API, Website | Professional |

### 4.2 Cost Disclosure

| Cost Type | Disclosure | Frequency |
|-----------|------------|-----------|
| **Trading Fees** | Fee schedule + real-time calculator | Continuous |
| **Deposit/Withdrawal Fees** | Fee schedule | Continuous |
| **Spread Costs** | Implicit in prices; average spread published | Monthly |
| **Staking Fees** | Validator commission % | Continuous |
| **Custody Fees** | Per asset, per annum | Continuous |
| **Total Cost of Ownership** | Annualized example per €10k | Annual |

### 4.3 Conflict of Interest Disclosure

| Conflict Type | Disclosure Method | Timing |
|---------------|-------------------|--------|
| **Principal Trading** | Flag on trade confirmation | Per trade |
| **Affiliated Counterparty** | Pre-trade disclosure | Pre-trade |
| **Employee Trading** | Annual summary | Annual |
| **Token Holdings (Team/Investors)** | Quarterly disclosure | Quarterly |
| **Market Making** | Disclosure on pair page | Continuous |

---

## 5. Website Disclosure (MiCA Art. 64)

### 5.1 Mandatory Website Sections

| Section | Content | Update Frequency |
|---------|---------|------------------|
| **About Us** | Legal name, address, LEI, authorization, register | On change |
| **Governance** | Management Board, key functions, `00_Governance_Arrangements.md` | On change |
| **Services** | Service descriptions, eligibility, jurisdictions | On change |
| **Fees** | Complete fee schedules, calculators | On change |
| **Risk Warnings** | Prominent, per service, retail-friendly | On change |
| **Whitepapers** | All token whitepapers (Utility, ART, EMT) | Per issuance |
| **Policies** | AML, Travel Rule, Sanctions, SAR, Custody, Complaints, BCP, IRP | On change |
| **Financial Information** | Audited financials, own funds, capital adequacy | Annual + quarterly |
| **Execution Quality** | RTS 28 reports, best execution summary | Annual |
| **Complaints** | Procedure, FIN-NET link, statistics (anonymized) | Quarterly |
| **Corporate News** | Material announcements, regulatory actions | Real-time |
| **Careers** | MRT roles, fit & proper process | On change |

### 5.2 API Disclosure (Machine-Readable)

| Endpoint | Data | Format | Auth |
|----------|------|--------|------|
| `/api/v1/fees` | Current fee schedules | JSON | Public |
| `/api/v1/prices` | Real-time prices, spreads | JSON/WS | Public |
| `/api/v1/execution` | Execution quality metrics | JSON | Public |
| `/api/v1/tokens` | Token info, whitepaper links | JSON | Public |
| `/api/v1/policies` | Policy documents, versions | JSON | Public |
| `/api/v1/financials` | Own funds, capital ratios | JSON | Public |

---

## 6. Marketing Communications (MiCA Art. 65)

### 6.1 Approval Process

| Material Type | Review | Approval | Record |
|---------------|--------|----------|--------|
| **Website Content** | Compliance | CCO | CMS audit log |
| **Social Media** | Compliance | MLRO (if AML-relevant) | Scheduler log |
| **Email Campaigns** | Compliance | CCO | ESP audit log |
| **Paid Advertising** | Compliance + Legal | CEO | Contract file |
| **Influencer/KOL** | Compliance + Legal | CEO | Agreement |
| **Events/Webinars** | Compliance | CCO | Event file |
| **Whitepapers** | Legal + Tech + Compliance | Board | Git + Board minutes |

### 6.2 Prohibited Practices

| Practice | Prohibition Basis |
|----------|-------------------|
| Guaranteed returns / "risk-free" | MiCA Art. 65, Consumer Protection |
| Misleading performance claims | MiCA Art. 65, UCITS-style rules |
| Omission of risk warnings | MiCA Art. 62–63 |
| Targeting prohibited jurisdictions | Sanctions, MiCA Art. 66 |
| Unsubstantiated comparisons | Advertising Standards |
| Pressure tactics (FOMO, limited time) | Consumer Protection |

### 6.3 Required Disclaimers

| Channel | Disclaimer |
|---------|------------|
| **All Marketing** | "Crypto-assets are high-risk. You may lose all invested capital. Not protected by investor compensation schemes." |
| **Staking/Yield** | "Staking rewards variable. Not guaranteed. Slashing risk applies." |
| **ART/EMT** | "Redemption at par. Safeguarded per MiCA. Not a deposit." |
| **Social Media** | Short disclaimer + link to full risk warning |

---

## 7. Transaction Reporting (MiCA Art. 66)

### 7.1 Reporting Obligations

| Report | Recipient | Frequency | Deadline | Standard |
|--------|-----------|-----------|----------|----------|
| **Transaction Reports** | HANFA | Daily | T+1 | ISO 20022 / MiCA RTS |
| **Order Book Snapshots** | HANFA | Daily | T+1 | MiCA RTS |
| **Trade Repository** | Approved TR | Real-time | T+0 | EMIR-style |
| **Suspicious Transactions** | FIU (goAML) | Immediate | 24h/48h | `03_Suspicious_Activity_Reporting.md` |
| **Large Transactions** | HANFA | Immediate | T+0 | > €15k / €1k (crypto) |

### 7.2 Data Quality Framework

| Check | Frequency | Owner |
|-------|-----------|-------|
| **Completeness** | Daily | Operations |
| **Accuracy (reconciliation)** | Daily | Operations |
| **Timeliness** | Real-time monitoring | Tech |
| **Schema Validation** | Per submission | Tech |
| **Regulatory Feedback** | Monthly | Compliance |

---

## 8. Token-Specific Disclosure

### 8.1 Utility Tokens (KVNC)

| Disclosure | Channel | Frequency |
|------------|---------|-----------|
| **Whitepaper** | Website, API | Per version |
| **Tokenomics Updates** | Website, Blog, API | On change |
| **Emission Schedule** | Website, Explorer | Real-time |
| **Burn Events** | Explorer, API | Real-time |
| **Governance Proposals** | Forum, API, Email | Per proposal |
| **Audit Reports** | Website | Per audit |

### 8.2 Asset-Referenced Tokens (ART)

| Disclosure | Channel | Frequency |
|------------|---------|-----------|
| **Whitepaper** | Website, API | Per version |
| **Reserve Composition** | Website, API | Daily |
| **NAV / Coverage Ratio** | Website, API | Daily |
| **Reserve Attestation** | Website, API | Monthly |
| **Audit Reports** | Website | Quarterly |
| **Redemption Stats** | Website, API | Daily |
| **Peg Deviation** | Website, API, Alert | Real-time |

### 8.3 E-Money Tokens (EMT)

| Disclosure | Channel | Frequency |
|------------|---------|-----------|
| **Whitepaper** | Website, API | Per version |
| **Safeguarding Composition** | Website, API | Daily |
| **Coverage Ratio** | Website, API | Daily |
| **Safeguarding Attestation** | Website, API | Monthly |
| **Audit Reports** | Website | Quarterly |
| **Redemption Stats** | Website, API | Daily |
| **Interest Allocation** | Website, API | Monthly |

---

## 9. Financial Disclosure

### 9.1 Public Financial Information

| Information | Frequency | Format | Standard |
|-------------|-----------|--------|----------|
| **Audited Financial Statements** | Annual | PDF, XBRL | IFRS / Croatian GAAP |
| **Own Funds Composition** | Quarterly | PDF, API | MiCA COREP |
| **Capital Adequacy Ratio** | Quarterly | PDF, API | MiCA Art. 56 |
| **Liquidity Coverage** | Monthly | PDF, API | MiCA Art. 59 |
| **Large Exposures** | Quarterly | PDF, API | MiCA Art. 60 |
| **K-Factor Breakdown** | Quarterly | PDF, API | MiCA Art. 58 |
| **Profit & Loss (Summary)** | Semi-annual | PDF | Management accounts |

### 9.2 Regulatory Reporting (Confidential)

| Report | Recipient | Frequency | Standard |
|--------|-----------|-----------|----------|
| **COREP (Own Funds)** | HANFA | Quarterly | EBA ITS |
| **FINREP (Financial)** | HANFA | Quarterly | EBA ITS |
| **Large Exposures** | HANFA | Quarterly | EBA ITS |
| **Liquidity** | HANFA | Monthly | EBA ITS |
| **K-Factors** | HANFA | Quarterly | MiCA RTS |
| **Prudential Dashboard** | HANFA | Monthly | National |

---

## 10. Incident & Material Event Disclosure

### 10.1 Material Events Requiring Disclosure

| Event | Internal Escalation | Public Disclosure | Regulatory Notification |
|-------|---------------------|-------------------|------------------------|
| **Security Breach** | Immediate (CISO→CCO→CEO) | Within 24h (if user impact) | DORA: 1h/24h/72h; HANFA: immediate |
| **Service Outage > 1h** | Immediate | Within 4h | HANFA: 24h |
| **Peg Deviation > 1% (ART)** | Immediate | Within 1h | HANFA: immediate |
| **Safeguarding Breach (EMT)** | Immediate | Within 1h | HANFA: immediate |
| **Regulatory Action** | Immediate | Within 24h | N/A (regulator leads) |
| **Key Personnel Change** | Immediate | Within 5 days | HANFA: 10 days |
| **Financial Distress** | Immediate | Per recovery plan | HANFA: immediate |

### 10.2 Disclosure Channels by Severity

| Severity | Channels |
|----------|----------|
| **Critical (P1)** | Website banner, Email all users, API alert, Social media, Press release |
| **High (P2)** | Website notice, Email affected users, API alert, Social media |
| **Medium (P3)** | Website notice, API alert |
| **Low (P4)** | Website changelog, API alert |

---

## 11. Record Keeping & Audit Trail

| Record Type | Retention | Format | Access |
|-------------|-----------|--------|--------|
| **All Disclosures** | 10 years | Immutable (WORM) / Git | Compliance, Auditors, HANFA |
| **Marketing Approvals** | 10 years | PDF + Metadata | Compliance |
| **Client Communications** | 10 years | Encrypted archive | Compliance, Legal |
| **Website Snapshots** | 10 years | WARC / Archive.org | Compliance |
| **API Logs** | 7 years | Structured logs | Tech, Compliance |
| **Regulatory Correspondence** | 10 years | Encrypted archive | Legal, Compliance |

---

## 12. Monitoring & Compliance

### 12.1 Key Performance Indicators

| KPI | Target | Monitoring |
|-----|--------|------------|
| **Disclosure Timeliness** | 100% within deadline | Daily dashboard |
| **Website Availability** | 99.9% | Real-time monitoring |
| **API Uptime** | 99.95% | Real-time monitoring |
| **Client Complaints (Disclosure-related)** | < 5/month | Monthly report |
| **Regulatory Findings (Transparency)** | 0 | Annual review |
| **Marketing Rejection Rate** | < 10% | Monthly report |

### 12.2 Annual Transparency Review

| Activity | Owner | Timeline |
|----------|-------|----------|
| **Policy Review** | CCO | Q1 |
| **Website Audit** | Compliance + Tech | Q1 |
| **KID Review** | Legal + Compliance | Per service launch |
| **Whitepaper Accuracy** | Legal + Tech | Per issuance |
| **Regulatory Gap Analysis** | Compliance | Q2 |
| **Board Report** | CCO | Q2 Board meeting |

---

## 13. Cross-References

| Document | Reference |
|----------|-----------|
| `00_Governance_Arrangements.md` | Governance, conflicts, key functions |
| `01_Capital_Requirements.md` | Financial disclosure data |
| `02_Whitepaper_Template_Utility.md` | Utility token disclosure |
| `03_Whitepaper_Template_ART.md` | ART disclosure |
| `04_Whitepaper_Template_EMT.md` | EMT disclosure |
| `00_AML_Policy.md` | Transaction reporting, SAR |
| `01_Travel_Rule_Policy.md` | Transfer transparency |
| `04_Complaints_Handling.md` | Complaints disclosure |
| `06_Incident_Response_Plan.md` | Incident disclosure |

---

## 14. Document Control

| Field | Value |
|-------|-------|
| **Document ID** | KOV-POL-GOV-005 |
| **Version** | 1.0 |
| **Classification** | Public |
| **Owner** | CCO / Legal |
| **Approved By** | Management Board |
| **Approval Date** | [DATE] |
| **Next Review** | [DATE + 1 year] |
| **Distribution** | Public (website), All Staff, HANFA |

### Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-15 | Compliance Team | Initial version for MiCA CASP application |

---

*End of Document*
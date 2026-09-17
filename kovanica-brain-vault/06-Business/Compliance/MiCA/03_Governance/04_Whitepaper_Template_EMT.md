# Whitepaper Template — E-Money Tokens (MiCA Art. 17)

**Document ID:** KOV-POL-GOV-004
**Version:** 1.0
**Classification:** Public (upon publication)
**Owner:** CCO / Legal
**Review Cycle:** Per issuance / Annual
**Status:** Template — Complete per Issuance

---

## 1. Purpose & Regulatory Basis

This template ensures compliance with **MiCA Articles 17, 48–54 (Title IV: E-Money Tokens)** and **Commission Delegated Regulation (EU) 2024/xxxx on EMT Whitepaper Content** for e-money tokens issued on the Kovanica Protocol.

| Regulation | Requirement |
|------------|-------------|
| MiCA Art. 17 | Mandatory whitepaper for public offer / admission to trading |
| MiCA Art. 48 | Definition: EMT = token referencing one fiat currency, redeemable at par |
| MiCA Art. 49 | Authorization: only credit institutions, e-money institutions, or CASPs with EMT permission |
| MiCA Art. 50 | Safeguarding: funds received = safeguarded assets (segregated, insulated) |
| MiCA Art. 51 | Redemption: at par, at any time, in reference currency |
| MiCA Art. 52 | Own funds: higher of €350k, 2% avg safeguarded funds, or ¼ fixed overheads |
| MiCA Art. 53 | Large exposures, leverage, liquidity (EMD2 transposed) |
| MiCA Art. 54 | Supervision: home Member State (HANFA), college if cross-border |

**Scope:** E-money tokens (single fiat-referenced, e.g., EURK, USDK) issued by Kovanica Protocol as authorized CASP with EMT permission.

---

## 2. Whitepaper Structure (Per Delegated Regulation Annex III for EMTs)

### 2.1 Cover Page

| Element | Content |
|---------|---------|
| **Token Name** | [TOKEN NAME] (e.g., "Euro E-Money Token") |
| **Token Symbol** | [SYMBOL] (e.g., "EURK") |
| **Token Type** | E-Money Token (EMT) per MiCA Art. 48 |
| **Reference Currency** | [EUR / USD / other single fiat] |
| **Issuer Legal Name** | Kovanica Protocol d.o.o. |
| **Issuer LEI** | [LEI CODE] |
| **Issuer Address** | [REGISTERED OFFICE ADDRESS], Zagreb, Croatia |
| **Authorization** | CASP with EMT permission (MiCA Art. 49) / E-Money Institution |
| **Competent Authority** | HANFA (home), host State authorities (passporting) |
| **Date of Publication** | [DATE] |
| **Version** | [VERSION NUMBER] |
| **Language** | English (Croatian translation available) |
| **Website** | https://kovanica.protocol |
| **Contact** | compliance@kovanica.protocol |

**Mandatory Disclaimer Box:**
> This whitepaper has not been approved by any competent authority. The issuer is solely responsible for its content. This is an e-money token under MiCA Title IV. Holders have redemption rights at par value at any time. The token constitutes a claim on the issuer. Funds received are safeguarded per MiCA Art. 50 and Directive 2009/110/EC (EMD2). The token is not a deposit and is not covered by deposit guarantee schemes.

---

### 2.2 Table of Contents

1. Summary
2. Information About the Issuer
3. Information About the Reference Currency
4. Information About the Public Offer / Admission to Trading
5. Information About the E-Money Token
6. Rights & Obligations (Focus: Redemption & Safeguarding)
7. Safeguarding Arrangements
8. Governance & Service Providers
9. Prudential Requirements
10. Risk Factors
11. Tax Information
12. Additional Information

---

### 2.3 Section 1: Summary (Max 2 Pages)

| Sub-section | Required Content |
|-------------|------------------|
| **1.1 Token Identity** | Name, symbol, blockchain (Kovanica Protocol), EMT classification |
| **1.2 Reference Currency** | Single fiat currency (EUR/USD/other) |
| **1.3 Issuer** | Legal name, LEI, authorization (CASP+EMT or EMI) |
| **1.4 Redemption Right** | At par, at any time, in reference currency |
| **1.5 Safeguarding** | 100% safeguarded, segregated, insulated, Copper/custodian |
| **1.6 Offer** | Public offer / admission, target size, jurisdictions |
| **1.7 Key Risks** | Top 5: safeguarding, redemption, regulatory, operational, issuer default |

---

### 2.4 Section 2: Information About the Issuer

| Sub-section | Required Content |
|-------------|------------------|
| **2.1 Legal Status** | d.o.o., Croatian law, MBS/OIB |
| **2.2 Authorization** | CASP Class 2 + EMT permission (Art. 49) OR E-Money Institution license |
| **2.3 Passporting** | EU/EEA passporting rights, host State notifications |
| **2.4 Governance** | Management Board, safeguarding oversight, `00_Governance_Arrangements.md` |
| **2.5 Financial Position** | Audited financials, own funds per `01_Capital_Requirements.md` + EMT add-on |
| **2.6 Service Providers** | Safeguarding custodian (Copper), auditor, payment processor, legal counsel |

---

### 2.5 Section 3: Information About the Reference Currency

| Sub-section | Required Content |
|-------------|------------------|
| **3.1 Currency Description** | ISO 4217 code, issuing central bank, legal tender status |
| **3.2 Exchange Rate** | Fixed 1:1 (1 token = 1 unit currency); no fluctuation |
| **3.3 Monetary Policy** | Reference to ECB / Fed / relevant central bank policy |
| **3.4 Legal Framework** | EMD2 transposition in Croatia, MiCA Title IV, national e-money law |

---

### 2.6 Section 4: Public Offer / Admission to Trading

| Sub-section | Required Content |
|-------------|------------------|
| **4.1 Offer Type** | Public offer / admission to trading / both |
| **4.2 Target Audience** | Retail / professional / both |
| **4.3 Jurisdictions** | EU/EEA (passporting), excluded (US, sanctioned) |
| **4.4 Offer Period** | Start/end dates (or continuous issuance) |
| **4.5 Pricing** | At par (1 token = 1 unit currency); no premium/discount |
| **4.6 Allocation** | KYC tiers, AML checks, no max per participant (subject to safeguarding capacity) |
| **4.7 Use of Proceeds** | 100% to safeguarded assets (no operational funding from EMT proceeds) |
| **4.8 Issuance Process** | KYC → fiat receipt → safeguarding → token minting → distribution |

---

### 2.7 Section 5: Information About the EMT

| Sub-section | Required Content |
|-------------|------------------|
| **5.1 Technical Specification** | Blockchain: Kovanica Protocol; Standard: Native UTXO with EMT metadata; Decimals: 2 (cents) or 8 (atoms) |
| **5.2 Tokenomics** | Supply fully elastic: mint on fiat receipt, burn on redemption; no fixed supply |
| **5.3 Par Value** | 1 token = 1 unit reference currency (exactly, no deviation) |
| **5.4 Interest / Yield** | **None** — EMTs cannot bear interest (MiCA Art. 48, EMD2 Art. 11) |
| **5.5 Secondary Markets** | Intended exchanges, market maker arrangements, liquidity at par |
| **5.6 Interoperability** | Bridges, wrapped versions, cross-chain redemption at par |

---

### 2.8 Section 6: Rights & Obligations (Redemption & Safeguarding Focus)

| Right / Obligation | Description |
|--------------------|-------------|
| **Redemption at Par** | Holder → Issuer: 1 token = 1 unit currency, **at any time** (no cut-off) |
| **Redemption Methods** | Fiat via SEPA/Target2/SWIFT (institutional); payment institution (retail) |
| **Redemption Fees** | Transparent, cost-based; **max 0.5%** in normal conditions; free for retail < €100 |
| **Suspension** | **Prohibited** — redemption right is unconditional (Art. 51); only regulatory order |
| **Safeguarding Claim** | Holder has direct claim on safeguarded assets (insolvency-remote) |
| **No Financial Rights** | No interest, yield, dividends, ownership of safeguarded assets |
| **Governance Rights** | Vote on parameter changes (fees, safeguarding policy) if enabled |
| **Transferability** | Freely transferable on-chain; compliance checks for redemption |

---

### 2.9 Section 7: Safeguarding Arrangements (Core EMT Requirement)

| Sub-section | Required Content |
|-------------|------------------|
| **7.1 Safeguarding Policy** | 100% of funds received safeguarded immediately upon receipt |
| **7.2 Safeguarding Methods** | **Segregated accounts** at credit institution / central bank / Copper |
| **7.3 Insulation** | Safeguarded funds legally separated from issuer's estate; bankruptcy-remote |
| **7.4 Custodian** | Copper (primary), [Central Bank / Credit Institution] (backup) |
| **7.5 Account Structure** | Omnibus safeguarding accounts per currency; daily reconciliation |
| **7.6 Investment of Safeguarded Funds** | **Only** high-quality liquid assets (HQLA): central bank deposits, govt bonds < 3mo |
| **7.7 Interest on Safeguarded Funds** | Accrues to **holders pro-rata** (or offset fees); not issuer revenue |
| **7.8 Audit & Attestation** | Monthly safeguarding attestation (CPA); quarterly audit; annual full audit |
| **7.9 Safeguarding Reporting** | Monthly public report: composition, value, coverage ratio; HANFA reporting |

**Safeguarding Coverage Ratio:** Must maintain ≥ 100% at all times; target 100–102%.

---

### 2.10 Section 8: Governance & Service Providers

| Role | Provider | Key Terms |
|------|----------|-----------|
| **Issuer** | Kovanica Protocol d.o.o. | CASP+EMT authorized, MiCA Art. 49 |
| **Safeguarding Custodian** | Copper / Credit Institution | Segregated accounts, `00_Custody_Policy.md` |
| **Payment Processor** | [PI / Bank] | Fiat on/off-ramp, SEPA/Target2 access |
| **Auditor** | [Big 4 / Specialized] | Monthly attestation, quarterly review, annual audit |
| **Legal Counsel** | [EU Law Firm] | MiCA, EMD2, cross-border passporting |
| **Redemption Agent** | [Payment Institution] | Instant/SEPA redemption for retail |

**Conflicts of Interest:** Policy per `00_Governance_Arrangements.md` §7; Chinese walls between issuer/safeguarding custodian/payment processor.

---

### 2.11 Section 9: Prudential Requirements (EMT-Specific)

| Requirement | Standard (MiCA Art. 52 / EMD2) | Kovanica Application |
|-------------|--------------------------------|---------------------|
| **Initial Capital** | €350,000 | Must raise to €350k before EMT issuance |
| **Ongoing Own Funds** | Higher of: €350k, 2% avg safeguarded funds, ¼ fixed overheads | Per `01_Capital_Requirements.md` + EMT floor |
| **Liquidity** | 30% fixed overheads + redemption buffer | Daily liquidity stress test; instant redemption capacity |
| **Large Exposures** | 25% own funds per counterparty | Safeguarding custodian, banking partners monitored |
| **Leverage Ratio** | Safeguarded funds / own funds ≤ 20x | Conservative safeguarding policy |
| **Recovery Plan** | Required | `05_Business_Continuity_Plan.md` + EMT-specific |
| **Resolution Plan** | Not required (non-bank) | Orderly wind-down, holder redemption priority |

---

### 2.12 Section 10: Risk Factors (EMT-Specific)

| Category | Key Risks |
|----------|-----------|
| **Safeguarding Risk** | Custodian default, segregation failure, insulation breach, investment loss |
| **Redemption Risk** | Run risk (unconditional), settlement failure, payment system outage, FX (cross-border) |
| **Issuer Default** | Insolvency, safeguarding claim enforcement, holder priority |
| **Regulatory** | Authorization withdrawal, passporting restriction, reclassification |
| **Operational** | Mint/burn errors, reconciliation failure, payment processor failure |
| **Governance** | Custodian conflict, audit failure, interest allocation disputes |
| **Systemic** | E-money sector stress, payment system disruption |

---

### 2.13 Section 11: Tax Information

| Sub-section | Required Content |
|-------------|------------------|
| **11.1 Issuer Tax** | Safeguarding income (interest) — CIT treatment; VAT on fees |
| **11.2 Holder Tax** | No capital gains (par redemption); FX gains/losses if non-EUR holder |
| **11.3 Cross-Border** | DAC8 reporting; VAT on payment services; EMD2 tax coordination |

---

### 2.14 Section 12: Additional Information

| Sub-section | Required Content |
|-------------|------------------|
| **12.1 Glossary** | EMT, safeguarding, insulation, par value, EMD2, payment institution, etc. |
| **12.2 References** | MiCA Title IV, EMD2, RTS, EBA guidelines, audit reports, safeguarding attestations |
| **12.3 Contact** | Investor relations, redemption support (24/7), compliance |
| **12.4 Updates** | Material change notification (7 days); version control; holder communication |

---

## 3. EMT-Specific Parameters (Template Variables)

| Parameter | EURK (Euro) | USDK (US Dollar) | Other Fiat |
|-----------|-------------|------------------|------------|
| **Reference Currency** | EUR | USD | [ISO 4217] |
| **Decimals** | 2 (cents) | 2 (cents) | Per currency minor unit |
| **Safeguarding Custody** | ECB / Copper EUR account | Fed / Copper USD account | Central bank / Copper |
| **Payment Rails** | SEPA, Target2, SWIFT | Fedwire, SWIFT, ACH | Local RTGS + SWIFT |
| **Redemption Time** | Instant (SEPA Inst) / T+1 | Same-day (Fedwire) / T+1 | Per local rails |
| **Min Redemption** | €1 (retail) / €100k (inst.) | $1 / $100k | Equivalent |
| **Fees** | 0% < €100; 0.1% > €100 | 0% < $100; 0.1% > $100 | Comparable |

---

## 4. Notification & Publication Checklist (EMT Enhanced)

| Step | Action | Deadline | Owner |
|------|--------|----------|-------|
| 1 | Obtain EMT permission (CASP Art. 49 or EMI license) | T-90 days | CCO / Legal |
| 2 | Finalize whitepaper (English) | T-45 days | Legal |
| 3 | Croatian translation | T-30 days | Legal / Translator |
| 4 | Internal review (Board, Compliance, Tech, Safeguarding) | T-21 days | CCO |
| 5 | Auditor pre-issuance safeguarding review | T-14 days | CFO / Auditor |
| 6 | Notify HANFA (Art. 18 + Art. 49) | T-10 days | CCO |
| 7 | Host State notifications (passporting) | T-10 days | CCO |
| 8 | Publish on website | T-0 (offer start) | Marketing |
| 9 | Submit to exchanges | T-0 | BD |
| 10 | First safeguarding attestation | T+7 days | Auditor |
| 11 | Ongoing updates (material changes) | Within 7 days | Legal |

---

## 5. Cross-References

| Document | Reference |
|----------|-----------|
| `00_Foundation/00_Business_Plan.md` | Business model, safeguarding income projections |
| `00_Governance_Arrangements.md` | Issuer governance, safeguarding oversight |
| `01_Capital_Requirements.md` | Own funds (€350k floor), liquidity, large exposures for EMT |
| `00_Custody_Policy.md` | Safeguarding asset custody, segregation, Copper |
| `03_Outsourcing_Register.md` | Safeguarding custodian, payment processor contracts |
| `05_Business_Continuity_Plan.md` | Recovery plan for EMT redemption continuity |
| `06_Incident_Response_Plan.md` | Safeguarding breach incident response |

---

## 6. Document Control

| Field | Value |
|-------|-------|
| **Document ID** | KOV-POL-GOV-004 |
| **Version** | 1.0 (Template) |
| **Classification** | Public (upon publication) |
| **Owner** | CCO / Legal |
| **Approved By** | Management Board |
| **Approval Date** | [DATE] |
| **Next Review** | Per issuance / Annual |
| **Distribution** | Public (website), HANFA, host State authorities, token holders |

### Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-15 | Compliance Team | Initial template for MiCA CASP + EMT application |

---

*End of Template — Complete per Issuance*
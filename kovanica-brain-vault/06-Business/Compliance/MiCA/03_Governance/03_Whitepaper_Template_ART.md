# Whitepaper Template — Asset-Referenced Tokens (MiCA Art. 17)

**Document ID:** KOV-POL-GOV-003
**Version:** 1.0
**Classification:** Public (upon publication)
**Owner:** CCO / Legal
**Review Cycle:** Per issuance / Annual
**Status:** Template — Complete per Issuance

---

## 1. Purpose & Regulatory Basis

This template ensures compliance with **MiCA Articles 17, 30–47 (Title III: Asset-Referenced Tokens)** and **Commission Delegated Regulation (EU) 2024/xxxx on ART Whitepaper Content** for asset-referenced tokens issued on the Kovanica Protocol.

| Regulation | Requirement |
|------------|-------------|
| MiCA Art. 17 | Mandatory whitepaper for public offer / admission to trading |
| MiCA Art. 30 | Definition: ART = token referencing value of one or more assets (not e-money) |
| MiCA Art. 31 | Authorization required for significant ART issuers |
| MiCA Art. 32–33 | Reserve assets: custody, segregation, liquidity, valuation |
| MiCA Art. 34–35 | Redemption rights: at par, at least daily, in reference assets or fiat |
| MiCA Art. 36–37 | Governance: issuer, reserve manager, auditor |
| MiCA Art. 38–39 | Conflicts of interest, outsourcing |
| MiCA Art. 40–41 | Prudential requirements: own funds, liquidity |
| MiCA Art. 42–43 | Supervision: college of supervisors for significant ARTs |
| MiCA Art. 44–47 | Recovery & resolution plans |

**Scope:** Stablecoins (fiat-referenced, commodity-referenced, basket-referenced) issued via Kovanica Protocol.

---

## 2. Whitepaper Structure (Per Delegated Regulation Annex II for ARTs)

### 2.1 Cover Page

| Element | Content |
|---------|---------|
| **Token Name** | [TOKEN NAME] (e.g., "Euro-Stable Token") |
| **Token Symbol** | [SYMBOL] (e.g., "EURK") |
| **Token Type** | Asset-Referenced Token (ART) per MiCA Art. 30 |
| **Reference Asset(s)** | [EUR / USD / Gold / Basket weights] |
| **Issuer Legal Name** | Kovanica Protocol d.o.o. |
| **Issuer LEI** | [LEI CODE] |
| **Issuer Address** | [REGISTERED OFFICE ADDRESS], Zagreb, Croatia |
| **Competent Authority** | HANFA (lead), EBA (if significant) |
| **Date of Publication** | [DATE] |
| **Version** | [VERSION NUMBER] |
| **Language** | English (Croatian translation available) |
| **Website** | https://kovanica.protocol |
| **Contact** | compliance@kovanica.protocol |

**Mandatory Disclaimer Box:**
> This whitepaper has not been approved by any competent authority. The issuer is solely responsible for its content. This is an asset-referenced token under MiCA Title III. Holders have redemption rights at par value. The token is not a deposit and is not covered by deposit guarantee schemes. Significant ARTs are subject to enhanced supervision by EBA.

---

### 2.2 Table of Contents

1. Summary
2. Information About the Issuer
3. Information About the Reference Asset(s)
4. Information About the Public Offer / Admission to Trading
5. Information About the Asset-Referenced Token
6. Rights & Obligations (Focus: Redemption)
7. Reserve Assets & Custody
6. Governance & Service Providers
7. Prudential Requirements
8. Risk Factors
9. Tax Information
10. Additional Information

---

### 2.3 Section 1: Summary (Max 2 Pages)

| Sub-section | Required Content |
|-------------|------------------|
| **1.1 Token Identity** | Name, symbol, blockchain (Kovanica Protocol), ART classification |
| **1.2 Reference Asset** | Single fiat / commodity / basket with weights |
| **1.3 Issuer** | Legal name, LEI, CASP authorization status |
| **1.4 Redemption Right** | At par, daily, in reference asset or fiat equivalent |
| **1.5 Reserve Backing** | 1:1+ backing, custody arrangement (Copper), audit frequency |
| **1.6 Offer** | Public offer / admission, target size, jurisdictions |
| **1.7 Key Risks** | Top 5: reserve, redemption, regulatory, operational, market |

---

### 2.4 Section 2: Information About the Issuer

| Sub-section | Required Content |
|-------------|------------------|
| **2.1 Legal Status** | d.o.o., Croatian law, MBS/OIB |
| **2.2 Authorization** | CASP Class 2 + ART issuer notification (Art. 31) |
| **2.3 Significance Assessment** | Criteria (Art. 42): user base > 10M, value > €5B, cross-border |
| **2.4 Governance** | Management Board, reserve management committee, `00_Governance_Arrangements.md` |
| **2.5 Financial Position** | Audited financials, own funds per `01_Capital_Requirements.md` |
| **2.6 Service Providers** | Reserve custodian (Copper), auditor, valuation agent, legal counsel |

---

### 2.5 Section 3: Information About the Reference Asset(s)

| Sub-section | Required Content |
|-------------|------------------|
| **3.1 Asset Description** | For each reference asset: name, ISIN/identifier, issuer, jurisdiction |
| **3.2 Basket Composition** | Weights, rebalancing rules, correlation analysis |
| **3.3 Legal Status** | Regulatory classification of each asset (fiat, commodity, security) |
| **3.4 Market Data** | Liquidity, trading venues, bid-ask spreads, volatility |
| **3.5 Custody Arrangement** | Where reference assets held (central bank, custodian, vault) |
| **3.6 Valuation Methodology** | Mark-to-market, frequency, independent valuation agent |

---

### 2.6 Section 4: Public Offer / Admission to Trading

| Sub-section | Required Content |
|-------------|------------------|
| **4.1 Offer Type** | Public offer / admission to trading / both |
| **4.2 Target Audience** | Retail / professional / both |
| **4.3 Jurisdictions** | EU/EEA (passporting), excluded (US, sanctioned) |
| **4.4 Offer Period** | Start/end dates, extension conditions |
| **4.5 Pricing** | At par (1 token = 1 unit reference asset) |
| **4.6 Allocation** | KYC tiers, max per participant, AML checks |
| **4.7 Use of Proceeds** | 100% to reserve assets (no operational funding from ART proceeds) |
| **4.8 Subscription Process** | KYC → reserve deposit → token minting → distribution |

---

### 2.7 Section 5: Information About the ART

| Sub-section | Required Content |
|-------------|------------------|
| **5.1 Technical Specification** | Blockchain: Kovanica Protocol; Standard: Native UTXO with ART metadata; Decimals: per reference asset |
| **5.2 Tokenomics** | Supply elastic: mint on deposit, burn on redemption; no fixed max supply |
| **5.3 Peg Mechanism** | 1:1 reserve backing; arbitrage via redemption/creation; stability fees (if any) |
| **5.4 Stability Parameters** | Target deviation: ±0.5%; intervention bands; emergency measures |
| **5.5 Secondary Markets** | Intended exchanges, market maker arrangements, liquidity provision |
| **5.6 Interoperability** | Bridges, wrapped versions, cross-chain redemption |

---

### 2.8 Section 6: Rights & Obligations (Redemption Focus)

| Right / Obligation | Description |
|--------------------|-------------|
| **Redemption at Par** | Holder → Issuer: 1 token = 1 unit reference asset (or fiat equivalent) |
| **Redemption Frequency** | At least daily (business days); cut-off time published |
| **Redemption Methods** | In reference asset (institutional) / fiat via partner (retail) |
| **Redemption Fees** | Transparent, cost-based, published; max 0.5% in normal conditions |
| **Suspension Conditions** | Extreme market conditions, regulatory order; max 5 business days; HANFA notification |
| **No Financial Rights** | No interest, yield, dividends, ownership of reserve assets |
| **Governance Rights** | Vote on parameter changes (stability fee, basket weights) if enabled |
| **Transferability** | Freely transferable on-chain; compliance checks for redemption |

---

### 2.9 Section 7: Reserve Assets & Custody

| Sub-section | Required Content |
|-------------|------------------|
| **7.1 Reserve Composition** | Cash (central bank / segregated bank accounts), high-quality liquid assets (HQLA) |
| **7.2 Custody Arrangement** | Copper (primary), central bank accounts (fiat), vaulted commodities |
| **7.3 Segregation** | Reserve assets legally separated from issuer's own assets; bankruptcy-remote |
| **7.4 Liquidity Management** | Minimum 30% in cash/HQLA; stress testing; `01_Capital_Requirements.md` |
| **7.5 Valuation** | Daily mark-to-market; independent valuation agent; published NAV |
| **7.6 Audit & Attestation** | Monthly reserve attestation (CPA); quarterly audit; annual full audit |
| **7.7 Reserve Reporting** | Monthly public report: composition, value, coverage ratio; HANFA reporting |

**Reserve Coverage Ratio:** Must maintain ≥ 100% at all times; target 102–105%.

---

### 2.10 Section 8: Governance & Service Providers

| Role | Provider | Key Terms |
|------|----------|-----------|
| **Issuer** | Kovanica Protocol d.o.o. | CASP authorized, MiCA Art. 31 notified |
| **Reserve Manager** | [Internal / Copper] | Investment policy, risk limits, `03_Outsourcing_Register.md` |
| **Custodian** | Copper / Central Bank | Segregated accounts, `00_Custody_Policy.md` |
| **Valuation Agent** | [Independent Firm] | Daily NAV, conflict-free |
| **Auditor** | [Big 4 / Specialized] | Monthly attestation, quarterly review, annual audit |
| **Legal Counsel** | [EU Law Firm] | MiCA, DORA, cross-border |
| **Redemption Agent** | [Payment Institution] | Fiat off-ramp for retail redemption |

**Conflicts of Interest:** Policy per `00_Governance_Arrangements.md` §7; Chinese walls between issuer/reserve manager/custodian.

---

### 2.11 Section 9: Prudential Requirements

| Requirement | Standard | Kovanica Application |
|-------------|----------|---------------------|
| **Own Funds** | Higher of: €3M, ¼ fixed overheads, 2% reserve assets | Per `01_Capital_Requirements.md` + ART add-on |
| **Liquidity** | 30% fixed overheads + redemption buffer | Daily liquidity stress test; 7-day survival horizon |
| **Large Exposures** | 25% own funds per counterparty | Reserve custodian, banking partners monitored |
| **Leverage Ratio** | Assets / own funds ≤ 10x | Reserve assets off-balance sheet |
| **Recovery Plan** | Required for significant ARTs | `05_Business_Continuity_Plan.md` + ART-specific |
| **Resolution Plan** | Required for significant ARTs | Bail-inable liabilities, continuity of redemption |

---

### 2.12 Section 10: Risk Factors (ART-Specific)

| Category | Key Risks |
|----------|-----------|
| **Reserve Risk** | Counterparty default, custody failure, asset devaluation, liquidity mismatch |
| **Redemption Risk** | Run risk, suspension, settlement failure, FX risk on cross-border redemption |
| **Peg Stability** | Deviation > threshold, arbitrage failure, market maker withdrawal |
| **Regulatory** | Reclassification as e-money, significance upgrade, jurisdiction bans |
| **Operational** | Mint/burn errors, oracle failure, smart contract vulnerability |
| **Governance** | Reserve manager conflict, valuation manipulation, audit failure |
| **Systemic** | Contagion from reference asset crisis, stablecoin sector stress |

---

### 2.13 Section 11: Tax Information

| Sub-section | Required Content |
|-------------|------------------|
| **11.1 Issuer Tax** | Reserve income (interest, gains) — CIT treatment; VAT on redemption fees |
| **11.2 Holder Tax** | Capital gains on disposal; FX gains/losses; staking/yield (if any) |
| **11.3 Cross-Border** | DAC8 reporting; withholding tax on reserve income; EU coordination |

---

### 2.14 Section 12: Additional Information

| Sub-section | Required Content |
|-------------|------------------|
| **12.1 Glossary** | ART, reserve assets, redemption, NAV, HQLA, significance, etc. |
| **12.2 References** | MiCA Title III, RTS, EBA guidelines, audit reports, reserve attestations |
| **12.3 Contact** | Investor relations, redemption support, compliance |
| **12.4 Updates** | Material change notification (7 days); version control; holder communication |

---

## 3. ART-Specific Parameters (Template Variables)

| Parameter | Single-Fiat (e.g., EURK) | Commodity (e.g., GLDK) | Basket (e.g., SDRK) |
|-----------|--------------------------|------------------------|---------------------|
| **Reference Asset** | EUR (central bank reserves) | Gold (LBMA good delivery) | 50% EUR, 30% USD, 20% Gold |
| **Custody** | Central bank / Copper | Vault (Brink's/Loomis) + Copper | Mixed per asset |
| **Valuation** | ECB reference rate | LBMA PM fix | Weighted composite |
| **Redemption** | EUR via SEPA / Target2 | Physical / cash settled | Fiat equivalent |
| **Min Redemption** | €1,000 (retail) / €100k (inst.) | 1 oz / $100k | $1,000 / $100k |
| **Stability Fee** | 0–0.5% p.a. | 0.2–0.8% p.a. | Weighted avg |

---

## 4. Notification & Publication Checklist (ART Enhanced)

| Step | Action | Deadline | Owner |
|------|--------|----------|-------|
| 1 | Finalize whitepaper (English) | T-45 days | Legal |
| 2 | Significance self-assessment (Art. 42) | T-40 days | CCO / CRO |
| 3 | Croatian translation | T-30 days | Legal / Translator |
| 4 | Internal review (Board, Compliance, Tech, Reserve) | T-21 days | CCO |
| 5 | Auditor pre-issuance review | T-14 days | CFO / Auditor |
| 6 | Notify HANFA (Art. 18 + Art. 31) | T-10 days | CCO |
| 7 | EBA notification (if significant) | T-10 days | CCO |
| 8 | Publish on website | T-0 (offer start) | Marketing |
| 9 | Submit to exchanges | T-0 | BD |
| 10 | First reserve attestation | T+7 days | Auditor |
| 11 | Ongoing updates (material changes) | Within 7 days | Legal |

---

## 5. Cross-References

| Document | Reference |
|----------|-----------|
| `00_Foundation/00_Business_Plan.md` | Business model, reserve income projections |
| `00_Governance_Arrangements.md` | Issuer governance, reserve management committee |
| `01_Capital_Requirements.md` | Own funds, liquidity, large exposures for ART |
| `00_Custody_Policy.md` | Reserve asset custody, segregation, Copper |
| `03_Outsourcing_Register.md` | Reserve manager, custodian, valuation agent contracts |
| `05_Business_Continuity_Plan.md` | Recovery plan for ART redemption continuity |
| `06_Incident_Response_Plan.md` | Peg deviation incident response |

---

## 6. Document Control

| Field | Value |
|-------|-------|
| **Document ID** | KOV-POL-GOV-003 |
| **Version** | 1.0 (Template) |
| **Classification** | Public (upon publication) |
| **Owner** | CCO / Legal |
| **Approved By** | Management Board |
| **Approval Date** | [DATE] |
| **Next Review** | Per issuance / Annual |
| **Distribution** | Public (website), HANFA, EBA (if significant), token holders |

### Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-15 | Compliance Team | Initial template for MiCA CASP + ART application |

---

*End of Template — Complete per Issuance*
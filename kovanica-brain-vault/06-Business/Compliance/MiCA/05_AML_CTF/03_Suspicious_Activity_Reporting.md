# Suspicious Activity Reporting (SAR) Policy

**Document ID:** POL-SAR-001  
**Version:** 1.0  
**Classification:** Confidential  
**Owner:** Money Laundering Reporting Officer (MLRO)  
**Review Cycle:** Annual  
**Approved By:** Management Board  
**Date:** 2025-01-15

---

## 1. Purpose & Regulatory Basis

This policy establishes the mandatory framework for identifying, assessing, and reporting suspicious activity related to money laundering (ML), terrorism financing (TF), and sanctions evasion in connection with Kovanica Protocol d.o.o.'s CASP activities under MiCA.

### 1.1 Applicable Regulatory Framework

| Regulation | Scope | Relevance |
|------------|-------|-----------|
| **AMLD6** (EU) 2018/1673 | Criminalisation of ML | SAR obligation; predicate offences |
| **Croatian AML Act** (Zakon o sprječavanju pranja novca i financiranja terorizma) | National implementation | FIU = Ured za sprječavanje pranja novca; criminal penalties |
| **MiCA Regulation (EU) 2023/1114**, Art. 66 | CASP obligations | SAR as authorization condition; TFR alignment |
| **TFR** (EU) 2023/1113 | Transfer of Funds | Originator/beneficiary data in SAR context |
| **FATF Recommendation 20** | International standard | SAR obligation; tipping-off prohibition |
| **EU FIU Regulation** (EU) 2021/1882 | FIU cooperation | Cross-border SAR exchange; goAML |

### 1.2 Core Principles

- **Mandatory Reporting**: All staff must report suspicions; failure is a criminal offence
- **No Tipping-Off**: Disclosure to client or third parties prohibited (Art. 33 AMLD6, Croatian AML Act Art. 45)
- **Timeliness**: Internal report within 24h; external filing per regulatory deadlines
- **Quality**: Complete, accurate, evidence-based SARs
- **Confidentiality**: SAR existence and content strictly protected

---

## 2. Governance & Organisation

### 2.1 Roles & Responsibilities

| Role | Responsibilities |
|------|------------------|
| **MLRO** (Money Laundering Reporting Officer) | Receive internal SARs; assess; file external SARs; liaise with FIU; maintain SAR register; report to Management Board |
| **Deputy MLRO** | Backup for MLRO; same authorities when acting |
| **All Employees** | Identify & report suspicions internally within 24h; no tipping-off |
| **Compliance/SCO** | Sanctions-related SAR triggers; blockchain analytics support |
| **Management Board** | Ultimate accountability; resource allocation; strategic oversight |
| **Internal Audit** | Independent review of SAR framework effectiveness |

### 2.2 SAR Committee (Ad-Hoc)

Convened by MLRO for complex cases:
- MLRO (Chair)
- SCO (Sanctions)
- Legal Counsel
- Relevant Business Line Head
- IT/Blockchain Analytics (as needed)

---

## 3. Internal Reporting Procedure

### 3.1 Reporting Obligation

**Every employee** who knows, suspects, or has reasonable grounds to suspect that:
- A transaction or activity relates to proceeds of crime
- A client is engaged in ML/TF/sanctions evasion
- A transaction has no apparent economic/lawful purpose
- A transaction involves structured/smurfed amounts
- A transaction involves high-risk jurisdictions or counterparties

**Must report immediately** to the MLRO via the Internal SAR Form.

### 3.2 Internal SAR Form (Template)

```
INTERNAL SUSPICIOUS ACTIVITY REPORT
====================================
CASE ID: [AUTO-GENERATED: SAR-YYYYMMDD-NNN]
DATE/TIME OF REPORT: [ISO 8601]
REPORTING EMPLOYEE: [Name, Role, Department]
CONTACT: [Email, Phone, Extension]

CLIENT INFORMATION:
- Client ID: [ID]
- Legal Name: [Name]
- Account/Wallet Addresses: [List]
- Risk Rating: [Low/Standard/High]
- Onboarding Date: [Date]

TRANSACTION/ACTIVITY DETAILS:
- Transaction ID(s): [TXIDs]
- Date/Time: [ISO 8601]
- Type: [Deposit/Withdrawal/Trade/Transfer/Stake/Unstake]
- Asset(s): [KVNC, BTC, ETH, USDC, etc.]
- Amount(s): [Native + EUR equivalent]
- Counterparty: [VASP name / Wallet address / Unhosted]
- Blockchain Network: [Kovanica / Ethereum / Bitcoin / etc.]

SUSPICION NARRATIVE:
[Free text — describe what is unusual, why it raises suspicion,
 specific red flags observed, comparison to client profile]

RED FLAGS IDENTIFIED (check all that apply):
☐ Structuring / Smurfing
☐ Unusual transaction pattern (vs. profile)
☐ High-risk jurisdiction exposure
☐ Sanctions nexus (direct/indirect)
☐ Darknet / Mixer / Privacy coin usage
☐ Pump-and-dump / Wash trading
☐ Chain hopping / Peel chains
☐ Known illicit address cluster
☐ Adverse media / PEP alert (Sumsub)
☐ Copper custody alert
☐ Other: [Specify]

SUPPORTING EVIDENCE ATTACHED:
☐ Transaction screenshots
☐ Blockchain analytics report
☐ Client profile / KYC documents
☐ Communication records
☐ Other: [Specify]

REPORTER ASSESSMENT:
- Suspicion Level: ☐ Low ☐ Medium ☐ High
- Urgency: ☐ Routine (3 business days) ☐ Urgent (24h) ☐ Immediate (TF)
- Recommended Action: ☐ File SAR ☐ Monitor ☐ Enhanced Due Diligence ☐ Exit Relationship

REPORTER SIGNATURE: _________________________  DATE: __________

MLRO ACKNOWLEDGMENT:
Received by: _________________________  Date/Time: __________
Initial Assessment: ☐ Accept for filing ☐ Request more info ☐ Close (document rationale)
```

### 3.3 Submission Channels

| Channel | Method | Availability | Confirmation |
|---------|--------|--------------|--------------|
| **Primary** | Secure internal portal (SAR module) | 24/7 | Auto-receipt + Case ID |
| **Secondary** | Encrypted email to `sar@kovanica.protocol` | 24/7 | Read receipt |
| **Emergency** | Direct phone to MLRO + follow-up form | Business hours | Verbal + written |

### 3.4 Timeline

| Step | Deadline | Owner |
|------|----------|-------|
| Employee identifies suspicion | Immediate | Employee |
| Internal SAR submitted | Within 24 hours | Employee |
| MLRO acknowledges receipt | Within 4 hours | MLRO |
| MLRO initial assessment | Within 48 hours | MLRO |
| External SAR filed (if warranted) | Per regulatory deadline | MLRO |

---

## 4. MLRO Assessment & Decision

### 4.1 Assessment Framework

| Factor | Consideration |
|--------|---------------|
| **Client Profile** | KYC data, risk rating, transaction history, business rationale |
| **Transaction Pattern** | Amount, frequency, counterparties, geography, asset types |
| **Blockchain Analytics** | Address clustering, mixer exposure, illicit address proximity, chain hopping |
| **External Intelligence** | Sumsub alerts, Copper alerts, LE requests, media, sanctions lists |
| **Economic Purpose** | Apparent legitimacy, commercial sense, source of funds consistency |

### 4.2 Decision Matrix

| Assessment Outcome | Action | Documentation |
|--------------------|--------|---------------|
| **File SAR** | Submit to FIU via goAML | Complete SAR file + rationale |
| **Enhanced Monitoring** | Increase review frequency; add rules | Monitoring plan + timeline |
| **Enhanced Due Diligence** | Request additional info/documents | EDD request + response tracking |
| **Exit Relationship** | Terminate per contractual terms | Exit memo + regulatory notification if required |
| **Close — No Suspicion** | Document rationale; retain records | Closure memo + evidence |

### 4.3 Grounds for NOT Filing (Must Be Documented)

- Insufficient evidence after reasonable inquiry
- Alternative legitimate explanation verified
- Transaction consistent with known client profile
- False positive from automated alert (documented)

**Never** decline to file due to:
- Commercial pressure
- Client relationship value
- Uncertainty (when in doubt, file)
- Fear of tipping-off (filing is protected)

---

## 5. External Filing — Croatian FIU (Ured za sprječavanje pranja novca)

### 5.1 Filing Channel

- **Primary**: goAML portal (UNODC) — Croatian FIU instance
- **Backup**: Secure email / physical delivery (per FIU guidance)
- **Format**: goAML XML schema (Croatian FIU specification)

### 5.2 Filing Deadlines

| Suspicion Type | Deadline | Legal Basis |
|----------------|----------|-------------|
| **Terrorism Financing** | Immediate (as soon as practicable) | Croatian AML Act Art. 42; FATF Rec. 20 |
| **Money Laundering** | 3 business days from MLRO decision | Croatian AML Act Art. 42 |
| **Sanctions Evasion** | Immediate (parallel with freeze) | EU Regulations; Croatian Restrictive Measures Act |
| **Supplementary Information** | As requested by FIU | goAML follow-up |

### 5.3 SAR Content Requirements (goAML)

| Field | Source | Mandatory |
|-------|--------|-----------|
| **Reporting Entity** | Company registration | Yes |
| **MLRO Details** | Name, contact, registration | Yes |
| **Subject Details** | Client KYC (name, ID, address, nationality, DOB) | Yes |
| **Beneficial Owners** | UBO register | Yes |
| **Account/Wallet Details** | All addresses, account numbers | Yes |
| **Transaction Details** | All related TXIDs, amounts, dates, counterparts | Yes |
| **Blockchain Data** | Network, block height, gas fees, smart contract interactions | Yes |
| **Suspicion Narrative** | MLRO assessment | Yes |
| **Red Flags** | Coded typology + free text | Yes |
| **Supporting Documents** | Attachments (analytics, KYC, comms) | Yes |
| **Action Taken** | Freeze, exit, monitoring, none | Yes |

### 5.4 Post-Filing Procedures

| Step | Action | Timeline |
|------|--------|----------|
| **Acknowledgment** | Record FIU reference number | Immediate |
| **Case Tracking** | Enter in SAR register with FIU ref | Same day |
| **Supplementary Requests** | Respond within FIU deadline | Per request |
| **LE/Prosecutor Requests** | Legal review → respond per law | Per request |
| **Statistics** | Monthly SAR stats to Management Board | Monthly |
| **Annual Report** | SAR volume, typologies, outcomes | Annual |

---

## 6. SAR Quality Standards

### 6.1 Narrative Quality Checklist

- [ ] **Who**: Subject(s) identified (client, counterparty, beneficial owner)
- [ ] **What**: Specific transactions/activities (TXIDs, amounts, dates)
- [ ] **When**: Timeline of suspicious activity
- [ ] **Where**: Jurisdictions, blockchain networks, platforms
- [ ] **Why**: Specific red flags, deviation from profile, typology match
- [ ] **How**: Method (structuring, layering, mixing, nested services)
- [ ] **Evidence**: Blockchain analytics, KYC docs, communications attached
- [ ] **Action**: What the reporting entity has done/will do

### 6.2 Blockchain Analytics Requirements

For every crypto-related SAR, include:
- **Address clustering analysis** (deposit/withdrawal addresses + 2 hops)
- **Mixer/privacy tool exposure** (Tornado Cash, Wasabi, CoinJoin, Railgun, etc.)
- **Illicit address proximity** (distance to known darknet, ransomware, sanctions addresses)
- **Cross-chain tracing** (if chain hopping suspected)
- **DeFi interaction map** (DEX swaps, lending, bridges, staking)
- **Fund flow visualization** (source → destination, intermediate hops)

### 6.3 Common Deficiencies to Avoid

| Deficiency | Impact | Prevention |
|------------|--------|------------|
| Vague narrative ("suspicious activity") | FIU rejection; regulatory criticism | Use checklist; require specific TXIDs |
| Missing blockchain data | Incomplete picture; delayed investigation | Mandatory analytics attachment |
| No client profile comparison | Cannot assess deviation | Auto-attach KYC/profile to SAR |
| Delayed filing | Regulatory breach; evidence loss | Automated reminders; escalation |
| Tipping-off language in SAR | Criminal offence risk | Legal review of narrative |

---

## 7. Trigger Typologies — Crypto-Specific

### 7.1 Money Laundering Typologies

| Typology | Description | Key Indicators |
|----------|-------------|----------------|
| **Structuring/Smurfing** | Multiple small deposits/withdrawals to avoid thresholds | Many TXs < reporting threshold; same beneficiary; short timeframe |
| **Layering** | Complex chains to obscure origin | Chain hopping; peel chains; multiple DEX swaps; bridge usage |
| **Integration** | Re-entering legitimate economy | Fiat off-ramp via exchange; high-value purchases; staking rewards |
| **Trade-Based ML** | Crypto-fiat arbitrage; fake volume | Wash trading; pump-and-dump; circular trades |
| **Nested Services** | Unregistered VASP using platform | High volume from single counterparty; no TRP/IVMS101 data |

### 7.2 Terrorism Financing Typologies

| Typology | Description | Key Indicators |
|----------|-------------|----------------|
| **Crowdfunding** | Small donations aggregated | Many small deposits to single wallet; social media links |
| **Privacy Coins/Mixers** | Obscuring trail | XMR/ZEC deposits; Tornado Cash withdrawals |
| **Non-Profit Abuse** | Charity front | Entity KYC mismatch; high-risk jurisdiction |
| **Virtual Asset TF** | Direct crypto funding | Wallet clusters linked to designated groups |

### 7.3 Sanctions Evasion Typologies

| Typology | Description | Key Indicators |
|----------|-------------|----------------|
| **Address Hopping** | New address per transaction | HD wallet derivation; no address reuse |
| **Intermediary Wallets** | Layered transfers | Peel chains; consistent amounts; timing patterns |
| **DeFi Protocols** | Permissionless mixing | DEX swaps; lending/borrowing; yield farming |
| **Stablecoin Swaps** | USDT/USDC → privacy coin → new stablecoin | Rapid stablecoin ↔ volatile ↔ stablecoin cycles |
| **NFT Wash Trading** | Value transfer via NFTs | High-value NFT trades between linked wallets |

### 7.4 Blockchain-Specific Red Flags

| Indicator | Tool/Method | Threshold |
|-----------|-------------|-----------|
| **Mixer Deposit** | Known mixer address list | Any deposit from Tornado Cash, Sinbad, Wasabi, etc. |
| **Chain Hopping** | Cross-chain analytics | > 3 chain hops in 24h |
| **Peel Chain** | Heuristic clustering | > 5 peels with consistent remainder |
| **Illicit Proximity** | Address clustering (Chainalysis/TRM) | < 3 hops to sanctioned/darknet/ransomware address |
| **Dusting Attack** | Small unknown deposits | < 0.001 ETH/BTC equivalent from unknown cluster |
| **Flash Loan Attack Pattern** | Arbitrage + liquidation | Single-block multi-DEX interaction |
| **New Address + High Value** | First-use address | > €10,000 equivalent on first transaction |

---

## 8. Tipping-Off Prohibition

### 8.1 Legal Prohibition

- **Croatian AML Act Art. 45**: Criminal offence to disclose SAR existence/content to subject or third parties
- **AMLD6 Art. 33**: Tipping-off criminalised across EU
- **Penalties**: Up to 3 years imprisonment (Croatia); administrative fines up to €5M or 10% turnover

### 8.2 Practical Safeguards

| Safeguard | Implementation |
|-----------|----------------|
| **Need-to-Know Access** | SAR system: MLRO, Deputy MLRO, Legal only |
| **No Client Communication** | Template: "Compliance review — no action required from you" |
| **No Internal Discussion** | SAR content not discussed outside SAR Committee |
| **Secure Storage** | Encrypted SAR repository; access logs; 10-year retention |
| **Training** | Annual tipping-off module; scenario testing |

### 8.3 Permitted Disclosures (Exceptions)

- To FIU / law enforcement / prosecutor (legal obligation)
- To auditors / regulators (statutory duty)
- Between MLRO and Deputy MLRO
- To legal counsel for privilege advice
- **Never** to the client, counterparties, or business lines

---

## 9. Record Keeping

### 9.1 Retention Requirements

| Record Type | Retention Period | Legal Basis |
|-------------|------------------|-------------|
| **Internal SAR Forms** | 10 years from filing/closure | AMLD6 Art. 40; Croatian AML Act Art. 52 |
| **External SAR Filings** | 10 years from filing | Same |
| **Supporting Evidence** | 10 years from filing | Same |
| **MLRO Assessment Notes** | 10 years from decision | Same |
| **FIU Correspondence** | 10 years from last action | Same |
| **SAR Register** | Permanent (archival) | Governance |

### 9.2 SAR Register (Mandatory Fields)

| Field | Description |
|-------|-------------|
| Case ID | SAR-YYYYMMDD-NNN |
| Date Received (Internal) | ISO 8601 |
| Reporting Employee | Name, Role |
| Client ID | Internal reference |
| Subject Name | Legal name |
| Suspicion Type | ML / TF / Sanctions / Other |
| Decision | Filed / Enhanced Monitoring / EDD / Exit / Closed |
| External Filing Date | If applicable |
| FIU Reference | goAML reference number |
| Outcome | FIU feedback / LE action / Closed |
| Closure Date | If closed without filing |

---

## 10. Feedback Loop & Statistics

### 10.1 FIU Feedback

| Feedback Type | Action | Timeline |
|---------------|--------|----------|
| **Acknowledgment** | Log FIU reference | Immediate |
| **Information Request** | Legal review → respond | Per FIU deadline |
| **Case Closure** | Update SAR register | On receipt |
| **LE Referral** | Cooperate per law; legal oversight | Per request |
| **Strategic Analysis** | Incorporate into typologies/training | Quarterly |

### 10.2 Management Board Reporting

| Report | Frequency | Content |
|--------|-----------|---------|
| **SAR Dashboard** | Monthly | Volume, types, outcomes, trends, KPIs |
| **Typology Bulletin** | Quarterly | Emerging patterns; blockchain-specific |
| **Annual SAR Report** | Annual | Full statistics; regulatory feedback; framework effectiveness |

### 10.3 Key Performance Indicators

| KPI | Target | Measurement |
|-----|--------|-------------|
| **Internal Reporting Rate** | > 1 per 1000 active clients/year | SAR register |
| **Filing Rate** | > 30% of internal SARs | SAR register |
| **Timeliness (Internal)** | 100% within 24h | SAR register timestamps |
| **Timeliness (External)** | 100% within deadline | FIU acknowledgment |
| **Quality Score** | > 90% (FIU feedback) | FIU assessment |
| **Tipping-Off Incidents** | 0 | Incident register |

---

## 11. Copper Integration

### 11.1 SAR Triggers from Custody Partner

| Copper Alert Type | Kovanica Action |
|-------------------|-----------------|
| **Sanctioned Address Deposit** | Auto-freeze → Internal SAR → External SAR (if confirmed) |
| **Mixer/High-Risk Address** | Enhanced monitoring → Internal SAR if pattern confirmed |
| **Unusual Volume/Velocity** | MLRO review → SAR assessment |
| **Counterparty VASP Alert** | TRP/IVMS101 data review → SAR if sanctions/ML nexus |
| **Sectoral Sanctions Exposure** | Business review → Document decision → SAR if evasion suspected |

### 11.2 Data Sharing Protocol

- **Copper → Kovanica**: Real-time webhook + daily batch report
- **Kovanica → Copper**: SAR-relevant watchlists; freeze instructions
- **Joint Investigation**: Shared case ID; coordinated regulatory communication

---

## 12. Sumsub Integration

### 12.1 SAR Triggers from KYC Provider

| Sumsub Alert | Kovanica Action |
|--------------|-----------------|
| **Adverse Media Hit** | MLRO review → SAR if ML/TF nexus |
| **PEP Status Change** | Enhanced monitoring → SAR if unexplained wealth |
| **Sanctions Hit** | Immediate freeze → SAR (see Sanctions Policy) |
| **Document Fraud** | Reject onboarding → SAR if organized crime indicators |
| **Behavioral Anomaly** (liveness fail, geo mismatch) | Enhanced review → SAR if pattern |

### 12.2 Data Flow

```
Sumsub Alert → Webhook → MLRO Queue → Assessment → SAR Decision
                    │
                    ▼
           SAR Register (auto-log)
```

---

## 13. Training Programme

### 13.1 Mandatory Training

| Audience | Content | Frequency | Assessment |
|----------|---------|-----------|------------|
| **All Staff** | SAR obligation; red flags; tipping-off; reporting channel | Annual + onboarding | Test (≥80% pass) |
| **Front-Line** | Client-facing scenarios; crypto typologies; escalation | Annual + quarterly refresh | Scenario-based |
| **Compliance/Risk** | Advanced analytics; goAML filing; FIU engagement | Quarterly | Case studies |
| **MLRO/Deputy** | Specialist certification (ACAMS/CFCS); regulatory updates | Continuous | Certification maintenance |
| **Management Board** | Oversight duties; liability; strategic risk | Annual | Board evaluation |

### 13.2 Scenario-Based Training (Annual Minimum)

1. **Client deposits from mixer** → Chain hop → Withdrawal to fiat
2. **Corporate client** → Beneficial owner appears on sanctions list post-onboarding
3. **High-volume trader** → Pattern matches peel chain + DEX arbitrage
4. **New client** → Source of funds inconsistent with declared business
5. **Counterparty VASP** → TRP data incomplete; jurisdiction high-risk

---

## 14. Cross-References

| Document | Reference |
|----------|-----------|
| **AML Policy** (`01_AML/00_AML_Policy.md`) | Risk-based approach, CDD, ongoing monitoring, trigger events |
| **Sanctions Policy** (`01_AML/02_Sanctions_Policy.md`) | Sanctions-related SAR triggers, freeze procedures |
| **Travel Rule Policy** (`01_AML/01_Travel_Rule_Policy.md`) | Counterparty data in SAR context |
| **Custody Policy** (`02_Ops/00_Custody_Policy.md`) | Copper alerts, asset freeze execution |
| **Programme of Operations** (`00_Foundation/01_Programme_of_Operations.md`) | Outsourcing governance, critical functions |
| **ICT Risk Management** (`02_Ops/02_ICT_Risk_Management.md`) | SAR system resilience, data integrity |

---

## 15. Document Control

| Field | Value |
|-------|-------|
| **Document ID** | POL-SAR-001 |
| **Version** | 1.0 |
| **Status** | Approved |
| **Classification** | Confidential |
| **Owner** | Money Laundering Reporting Officer (MLRO) |
| **Approved By** | Management Board |
| **Effective Date** | 2025-01-15 |
| **Next Review** | 2026-01-15 |
| **Distribution** | Management Board, Compliance, Legal, MLRO, Copper, Sumsub |

### Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-01-15 | MLRO | Initial version for MiCA CASP application |

---

## Appendices

### Appendix A: Internal SAR Form (Blank Template)

[See Section 3.2 above]

### Appendix B: goAML XML Schema Mapping (Croatian FIU)

| goAML Element | Kovanica Field | Source |
|---------------|----------------|--------|
| `<ReportingEntity>` | Company LEI + HANFA registration | Static |
| `<ReportingOfficer>` | MLRO name, email, phone | Static |
| `<Subject>` | Client KYC data | KYC database |
| `<Account>` | Wallet addresses + account numbers | Ledger |
| `<Transaction>` | TXIDs, amounts, timestamps, counterparts | Blockchain indexer |
| `<SuspicionReason>` | Coded typology + narrative | MLRO assessment |
| `<ActionTaken>` | Freeze/Monitor/Exit/None | Case record |
| `<Attachments>` | Analytics, KYC, comms | Case file |

### Appendix C: Tipping-Off Quick Reference Card

```
TIPPING-OFF = CRIMINAL OFFENCE
===============================

DO NOT:
✗ Tell the client a SAR has been filed
✗ Tell the client they are "under investigation"
✗ Discuss SAR content with business lines
✗ Share SAR with counterparties/VASPs
✗ Hint, imply, or suggest regulatory scrutiny

DO:
✓ File SAR internally within 24h
✓ Use standard compliance language: "Routine compliance review"
✓ Escalate to MLRO only
✓ Document everything in SAR system
✓ Seek legal advice if unsure

PENALTIES: Up to 3 years prison (HR) / €5M or 10% turnover (EU)
```

### Appendix D: SAR Filing Decision Tree

```
SUSPICION IDENTIFIED
        │
        ▼
┌───────────────┐
│ INTERNAL SAR  │──▶ MLRO RECEIVES (4h)
│ SUBMITTED     │
└───────┬───────┘
        │
        ▼
┌──────────────────┐
│ MLRO ASSESSMENT  │──▶ 48h MAX
│ (48h)            │
└────────┬─────────┘
         │
    ┌────┴────┐
    ▼         ▼
 FILE SAR   NO SAR
    │         │
    ▼         ▼
 goAML     DOCUMENT
 FILING    RATIONALE
    │         │
    ▼         ▼
 TRACK    MONITOR/
 CASE     CLOSE
```

---

*End of Document*
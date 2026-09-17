# Sanctions Policy

**Document ID:** POL-SANC-001  
**Version:** 1.0  
**Classification:** Confidential  
**Owner:** Sanctions Compliance Officer  
**Review Cycle:** Annual (or upon new designation)  
**Approved By:** Management Board  
**Date:** 2025-01-15

---

## 1. Purpose & Regulatory Basis

This Sanctions Policy establishes the framework for Kovanica Protocol d.o.o. ("the Company") to comply with all applicable international, EU, and Croatian sanctions regimes in its capacity as a Crypto-Asset Service Provider (CASP) under MiCA.

### 1.1 Applicable Regulatory Framework

| Regulation | Scope | Relevance |
|------------|-------|-----------|
| **EU Sanctions Regulations** (CFSP) | Binding on all EU entities | Primary sanctions framework; directly applicable |
| **UN Security Council Resolutions** (Chapter VII) | Global | Implemented via EU regulations |
| **OFAC Sanctions** (SDN, Sectoral, NS-ISA) | US jurisdiction | Extraterritorial reach; USD clearing exposure |
| **UK HMT Sanctions** | UK jurisdiction | Post-Brexit autonomous regime |
| **Croatian Act on Restrictive Measures** (Zakon o restriktivnim mjerama) | National implementation | Criminal penalties for violations |
| **MiCA Regulation (EU) 2023/1114**, Art. 66 | CASP obligations | Sanctions compliance as authorization condition |
| **AMLD6** (EU) 2018/1673 | Predicate offences | Sanctions evasion as money laundering predicate |

### 1.2 Policy Principles

- **Zero Tolerance**: No business with designated persons/entities
- **Risk-Based Approach**: Screening intensity proportional to risk
- **Timeliness**: Real-time screening at onboarding and pre-transaction
- **Transparency**: Full audit trail of all screening decisions
- **Cooperation**: Proactive engagement with regulators and law enforcement

---

## 2. Governance & Organisation

### 2.1 Roles & Responsibilities

| Role | Responsibilities |
|------|------------------|
| **Sanctions Compliance Officer (SCO)** | Day-to-day sanctions compliance; screening operations; match investigation; regulatory reporting; policy maintenance |
| **MLRO** | SAR filing for sanctions-related suspicious activity; liaison with FIU |
| **Management Board** | Ultimate accountability; resource allocation; escalation decisions |
| **Compliance Committee** | Quarterly review of sanctions exposure; policy approval |
| **IT/Security** | Screening system integration; blockchain analytics tools; data feeds |

### 2.2 Escalation Matrix

| Event | First Response | Escalation | Timeline |
|-------|----------------|------------|----------|
| New designation published | SCO reviews impact | SCO → MLRO → Management Board | < 4 hours |
| True positive match (client) | Immediate freeze | SCO → MLRO → Management Board → HANFA | Immediate |
| True positive match (transaction) | Block/reject | SCO → MLRO | Immediate |
| Potential match | Enhanced review | SCO → MLRO (if unresolved) | 24 hours |
| Sectoral sanctions exposure | Business line review | SCO → Management Board | 48 hours |

---

## 3. Screening Scope

### 3.1 Entities Subject to Screening

| Category | Screening Trigger | Frequency |
|----------|-------------------|-----------|
| **Clients** (natural & legal persons) | Onboarding, periodic refresh, trigger events | Real-time + daily batch |
| **Beneficial Owners** (≥10% or control) | Onboarding, periodic refresh | Real-time + daily batch |
| **Counterparties** (VASPs, banks, payment processors) | Pre-transaction, periodic | Real-time + daily batch |
| **Blockchain Addresses** (deposit/withdrawal) | Pre-transaction, periodic monitoring | Real-time + continuous |
| **Employees/Directors/Shareholders** | Onboarding, annual refresh | Real-time + annual |
| **Third-party Providers** (Copper, Sumsub, AWS) | Onboarding, annual due diligence | Annual + trigger |

### 3.2 Geographic Risk Overlay

| Risk Tier | Jurisdictions | Enhanced Measures |
|-----------|---------------|-------------------|
| **Prohibited** | DPRK, Iran, Syria, Crimea/Sevastopol/Donetsk/Luhansk (RU), Cuba | No business; auto-reject |
| **High** | Russia, Belarus, Venezuela, Myanmar, Afghanistan, Yemen | EDD mandatory; transaction-level screening |
| **Elevated** | FATF Grey List jurisdictions | Enhanced monitoring; source of funds verification |
| **Standard** | All other jurisdictions | Standard screening |

---

## 4. Screening Methodology

### 4.1 Screening Layers

```
┌─────────────────────────────────────────────────────────────┐
│                    SCREENING ARCHITECTURE                    │
├─────────────────────────────────────────────────────────────┤
│  LAYER 1: NAME SCREENING (Fuzzy matching, aliases, scripts) │
│  ├── EU Consolidated List                                    │
│  ├── UN Consolidated List                                    │
│  ├── OFAC SDN List                                           │
│  ├── UK HMT Consolidated List                                │
│  └── National lists (HR, DE, FR, etc.)                       │
├─────────────────────────────────────────────────────────────┤
│  LAYER 2: BLOCKCHAIN ADDRESS SCREENING                       │
│  ├── Known sanctioned addresses (OFAC, EU, Chainalysis,     │
│  │   TRM Labs, Elliptic, CipherTrace)                        │
│  ├── Mixer/tumbler addresses (Tornado Cash, Sinbad, etc.)   │
│  ├── Darknet market addresses                                │
│  └── Ransomware/extortion addresses                          │
├─────────────────────────────────────────────────────────────┤
│  LAYER 3: TRANSACTION PATTERN SCREENING                      │
│  ├── Structuring/smurfing detection                          │
│  ├── Peel chain / chain hopping                              │
│  ├── High-risk jurisdiction exposure                         │
│  └── Sanctions evasion typologies                            │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 Screening Timing

| Trigger | Screening Type | SLA |
|---------|----------------|-----|
| **Client Onboarding** | Full (Layers 1+2) | Before account activation |
| **Pre-Transaction** (deposit/withdrawal/trade) | Layers 2+3 | Real-time (< 500ms) |
| **Periodic Batch** (all active clients) | Layers 1+2 | Daily 02:00 UTC |
| **New Designation** | Incremental (affected population) | < 4 hours |
| **Ad-hoc** (LE request, media alert) | Targeted | Immediate |

### 4.3 Data Sources & Vendors

| Layer | Primary Source | Backup/Enrichment |
|-------|----------------|-------------------|
| **Names** | Dow Jones Risk & Compliance / Refinitiv World-Check | OpenSanctions, EU/UN/OFAC direct feeds |
| **Blockchain Addresses** | Chainalysis / TRM Labs / Elliptic | OFAC crypto address list, public threat intel |
| **Transaction Patterns** | Internal rules engine + vendor analytics | Custom typology rules |

---

## 5. Sanctions Lists

### 5.1 Primary Lists (Mandatory)

| List | Jurisdiction | Update Frequency | Format |
|------|--------------|------------------|--------|
| **EU Consolidated List** | EU | Daily | XML/CSV |
| **UN Consolidated List** | UN | Daily | XML/CSV |
| **OFAC SDN List** | US | Daily | CSV/JSON |
| **OFAC Non-SDN Lists** (SSI, NS-ISA, FSE, etc.) | US | Daily | CSV/JSON |
| **UK HMT Consolidated List** | UK | Daily | CSV/JSON |
| **Croatian National List** | Croatia | As published | PDF/CSV |

### 5.2 Supplementary Lists (Risk-Based)

| List | Purpose | Integration |
|------|---------|-------------|
| **EU Sectoral Sanctions** (Russian oil, gold, diamonds) | Sectoral exposure | Transaction-level tagging |
| **Counter-Terrorism Lists** (EU, UN, national) | TF financing | Enhanced PEP screening |
| **Human Rights Sanctions** (EU Global Human Rights, US Magnitsky) | Reputational risk | EDD trigger |

---

## 6. Match Handling Procedures

### 6.1 Match Classification

| Classification | Definition | Action |
|----------------|------------|--------|
| **True Positive** | Confirmed match to designated person/entity/address | **FREEZE** → Report → Reject/Terminate |
| **False Positive** | Match ruled out via identifiers (DOB, nationality, address, blockchain analysis) | Document rationale → Clear → Proceed |
| **Potential Match** | Insufficient information to confirm/rule out | **HOLD** → Enhanced review → Escalate if unresolved > 24h |

### 6.2 Match Handling Workflow

```
MATCH DETECTED
      │
      ▼
┌─────────┐
│ AUTO-   │──NO──▶ CLEAR (log rationale)
│ CLEAR?  │
└────┬────┘
     │ YES
     ▼
┌────────────────┐
│ POTENTIAL      │──ENHANCED REVIEW──▶ CONFIRMED? ──YES──▶ TRUE POSITIVE
│ MATCH          │     (24h max)              │
└────────────────┘                            ▼
                                              FREEZE + REPORT
                                               │
                                               ▼
                                    NOTIFY HANFA / MINISTRY
                                               │
                                               ▼
                                    TERMINATE RELATIONSHIP
```

### 6.3 True Positive Procedures

| Step | Action | Owner | Timeline |
|------|--------|-------|----------|
| 1 | **Immediate Freeze** — Block all transactions, disable account | SCO/System | Immediate (automated) |
| 2 | **Internal Notification** — Alert MLRO, Management Board | SCO | < 1 hour |
| 3 | **Regulatory Notification** — HANFA (Croatian FIU) + Ministry of Finance | MLRO | Immediate (parallel) |
| 4 | **Asset Preservation** — Segregate frozen assets per Croatian law | Custody (Copper) | < 24 hours |
| 5 | **Client Communication** — Template notice (no tipping-off) | Legal/SCO | As advised by authorities |
| 6 | **Record Keeping** — Complete audit trail (10 years) | SCO | Ongoing |
| 7 | **Relationship Termination** — Formal exit per contractual terms | Legal/Management | Per regulatory guidance |

### 6.4 False Positive Documentation

Required evidence for each false positive clearance:
- [ ] Side-by-side comparison (screening hit vs. client data)
- [ ] Differentiating identifiers (DOB, nationality, ID number, address)
- [ ] Blockchain analysis (for address matches — cluster analysis, transaction history)
- [ ] Source references (list entry details, designation date)
- [ ] Analyst sign-off + SCO approval
- [ ] Timestamped audit log entry

---

## 7. Asset Freeze Procedures

### 7.1 Legal Basis

- **EU Regulations**: Direct effect; freezing obligation automatic upon designation
- **Croatian Act on Restrictive Measures**: Criminal penalties for non-compliance (Art. 15-17)
- **MiCA Art. 66**: CASP must have policies to freeze assets without delay

### 7.2 Freeze Execution

| Asset Type | Freeze Mechanism | Custody Partner Role |
|------------|------------------|----------------------|
| **Fiat (EUR/USD)** | Bank account block via payment provider | N/A |
| **Crypto (KVNC, BTC, ETH, USDC, etc.)** | Address-level block via Copper MPC wallet | Copper executes freeze on instruction |
| **Staked/DeFi Positions** | Smart contract interaction (if upgradable) or withdrawal block | Copper coordinates with protocol |
| **NFTs/Other Tokens** | Transfer restriction at wallet level | Copper enforces |

### 7.3 Notification Requirements

| Recipient | Channel | Timeline | Content |
|-----------|---------|----------|---------|
| **HANFA** | Secure email / portal | Immediate | Designation reference, frozen assets, client ID |
| **Ministry of Finance** (Croatia) | Official letter / portal | Immediate | Same as HANFA |
| **EU Commission** (if required) | Sanctions reporting channel | Per regulation | Aggregate statistics |
| **Copper** | API / secure channel | Immediate | Freeze instruction + legal basis |

### 7.4 Unfreeze / Delisting

- Only upon official delisting (EU Official Journal, OFAC removal, UN delisting)
- Requires SCO verification + Management Board approval
- Documented in sanctions register with audit trail

---

## 8. Sectoral Sanctions

### 8.1 Russian Sectoral Sanctions (Key for Crypto)

| Sector | Restriction | Crypto Relevance |
|--------|-------------|------------------|
| **Oil Price Cap** (G7/EU) | Price cap on Russian oil | Indirect — payment routing risk |
| **Gold/Diamonds** | Import ban | Tokenized commodities exposure |
| **Financial Services** (SSI) | Prohibition on new debt/equity | VASP services to Russian entities |
| **Technology/Dual-Use** | Export controls | Mining equipment, blockchain infra |

### 8.2 Sectoral Compliance Procedures

- **Pre-transaction tagging**: Identify Russian nexus (counterparty, origin/destination, underlying asset)
- **Sectoral screening**: Cross-reference against SSI/NS-ISA lists
- **Documentation**: Record basis for permitting/prohibiting each flagged transaction
- **Reporting**: Monthly sectoral exposure report to Management Board

---

## 9. Circumvention Risk — Blockchain-Specific

### 9.1 High-Risk Typologies

| Typology | Description | Detection Method |
|----------|-------------|------------------|
| **Chain Hopping** | Rapid cross-chain swaps to obscure trail | Cross-chain analytics (Chainalysis, TRM) |
| **Peel Chains** | Small amounts peeled off to new addresses | Heuristic clustering + amount patterns |
| **Mixer/Tumbler Usage** | Tornado Cash, Sinbad, Wasabi, CoinJoin | Known mixer address lists + heuristic |
| **Privacy Coins** | XMR, ZEC, DASH (shielded pools) | Address type detection; enhanced EDD |
| **Nested Services** | Unregistered VASPs operating on major platforms | Counterparty VASP verification (TRP/IVMS101) |
| **DeFi Protocol Abuse** | Sanctioned entities using permissionless protocols | Protocol-level monitoring; address clustering |

### 9.2 Enhanced Controls

- **Real-time mixer detection**: Block deposits from known mixer addresses
- **Cross-chain tracing**: Integrate multi-chain analytics (Ethereum, BSC, Polygon, Arbitrum, Optimism, Bitcoin, Litecoin, Dogecoin, Kovanica DAG)
- **DeFi interaction scoring**: Risk score for protocol interactions (DEX, lending, bridges)
- **Privacy coin policy**: Enhanced due diligence mandatory; source of funds + blockchain analytics report

---

## 10. Copper Integration

### 10.1 Copper as Custody Partner — Sanctions Responsibilities

| Area | Copper Responsibility | Kovanica Responsibility |
|------|----------------------|-------------------------|
| **Wallet Screening** | Real-time address screening on deposit/withdrawal | Provide watchlists; verify coverage |
| **Transaction Monitoring** | Pattern detection on custodial flows | Define typologies; receive alerts |
| **Freeze Execution** | Immediate MPC wallet freeze on instruction | Issue freeze instruction; legal basis |
| **Watchlist Management** | Maintain internal sanctions lists | Push updates < 4h of new designation |
| **Reporting** | Monthly sanctions screening report | Review; escalate to Management Board |

### 10.2 Shared Watchlist Protocol

- **Format**: JSON/CSV with fields: `address`, `chain`, `designation_source`, `designation_date`, `risk_level`, `metadata`
- **Delivery**: Secure API (mutual TLS) + encrypted S3 bucket backup
- **Frequency**: Push on change + daily full sync 03:00 UTC
- **Validation**: Checksum verification; schema validation

### 10.3 Copper-Specific Escalation

| Scenario | Copper Action | Kovanica Action |
|----------|---------------|-----------------|
| Sanctioned address deposit detected | Auto-block + alert | SCO investigates; SAR if needed |
| Sanctioned counterparty in ClearLoop | Reject settlement | Notify counterparty VASP via TRP |
| Sectoral sanctions exposure | Flag transaction | Business review; document decision |

---

## 11. Sumsub Integration

### 11.1 KYC-Level Sanctions Screening

| Screening Point | Sumsub Check | Kovanica Overlay |
|-----------------|--------------|------------------|
| **Onboarding** | Name + DOB + nationality vs. sanctions lists | Blockchain address screening (Layer 2) |
| **Periodic Re-KYC** | Refresh against updated lists | Full re-screen (Layers 1+2) |
| **Trigger Event** (PEP, adverse media) | Enhanced screening | Immediate transaction screening |

### 11.2 Data Flow

```
Client → Sumsub KYC → [Name/DOB/Nationality] → Sanctions Screening (Sumsub)
                                    │
                                    ▼
                         ┌─────────────────────┐
                         │ PASS → Proceed      │
                         │ HIT  → Webhook to   │
                         │       Kovanica SCO  │
                         └─────────────────────┘
```

### 11.3 Sumsub Webhook Payload (Sanctions Hit)

```json
{
  "event": "applicantSanctionsHit",
  "applicantId": "sumsub_abc123",
  "hit": {
    "list": "OFAC_SDN",
    "entityName": "Ivan Ivanov",
    "matchScore": 0.92,
    "designationDate": "2024-02-15",
    "program": "UKRAINE-EO14024"
  },
  "timestamp": "2025-01-15T10:30:00Z"
}
```

---

## 12. Training & Awareness

### 12.1 Training Programme

| Audience | Content | Frequency | Format |
|----------|---------|-----------|--------|
| **All Staff** | Sanctions basics, reporting obligation, tipping-off prohibition | Annual + onboarding | E-learning + test |
| **Front-line (Support, Sales)** | Red flags, escalation procedures, unhosted wallet handling | Annual + quarterly refresh | Workshop + scenarios |
| **Compliance/Risk** | Advanced typologies, blockchain analytics, regulatory updates | Quarterly | Specialist training |
| **Management Board** | Strategic exposure, regulatory expectations, liability | Annual | Board session |
| **IT/Engineering** | Screening system integration, data feeds, freeze automation | Annual | Technical workshop |

### 12.2 Ad-Hoc Awareness

- **New Designation Alerts**: Email to all compliance staff within 4 hours
- **Typology Bulletins**: Monthly (or per major event) — blockchain-specific
- **Regulatory Updates**: As issued (HANFA, EU Commission, OFAC)

---

## 13. Audit & Testing

### 13.1 Independent Testing

| Test | Frequency | Scope | Owner |
|------|-----------|-------|-------|
| **Internal Audit** | Annual | Full sanctions framework (governance, screening, freeze, reporting) | Internal Audit / External |
| **Screening Effectiveness** | Semi-annual | Sample testing (true positive detection, false positive rate) | SCO + Vendor |
| **Freeze Simulation** | Annual | End-to-end freeze drill (crypto + fiat) | SCO + Copper |
| **Regulatory Examination Prep** | As needed | Mock examination; gap analysis | SCO + Legal |

### 13.2 Scenario Testing (Annual Minimum)

1. **New EU Designation** — Russian oligarch added; 50 clients affected
2. **OFAC Crypto Address** — New BTC address designated; deposits in flight
3. **Mixer Exposure** — Client withdraws via Tornado Cash; chain hop to Kovanica
4. **Sectoral Sanctions** — Russian oil tokenized asset traded on platform
5. **Circumvention** — Nested VASP onboarding sanctioned entity via layering

### 13.3 Key Metrics & KPIs

| Metric | Target | Reporting |
|--------|--------|-----------|
| **Screening Coverage** | 100% (all triggers) | Daily dashboard |
| **False Positive Rate** | < 5% (name), < 1% (address) | Monthly |
| **True Positive Detection** | 100% (tested via simulation) | Semi-annual |
| **Freeze Execution Time** | < 5 minutes (crypto), < 1 hour (fiat) | Per incident |
| **Regulatory Notification** | Immediate (parallel) | Per incident |
| **Training Completion** | 100% within 30 days of due date | Quarterly |

---

## 14. Cross-References

| Document | Reference |
|----------|-----------|
| **AML Policy** (`01_AML/00_AML_Policy.md`) | Risk-based approach, CDD, ongoing monitoring |
| **Travel Rule Policy** (`01_AML/01_Travel_Rule_Policy.md`) | Counterparty verification, originator/beneficiary data |
| **Custody Policy** (`02_Ops/00_Custody_Policy.md`) | Asset segregation, freeze execution, Copper SLA |
| **Programme of Operations** (`00_Foundation/01_Programme_of_Operations.md`) | Outsourcing register, critical functions |
| **ICT Risk Management** (`02_Ops/02_ICT_Risk_Management.md`) | Screening system resilience, data integrity |

---

## 15. Document Control

| Field | Value |
|-------|-------|
| **Document ID** | POL-SANC-001 |
| **Version** | 1.0 |
| **Status** | Approved |
| **Classification** | Confidential |
| **Owner** | Sanctions Compliance Officer |
| **Approved By** | Management Board |
| **Effective Date** | 2025-01-15 |
| **Next Review** | 2026-01-15 (or upon new designation) |
| **Distribution** | Management Board, Compliance, Legal, IT, Copper, Sumsub |

### Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-01-15 | SCO | Initial version for MiCA CASP application |

---

## Appendices

### Appendix A: Sanctions Screening Checklist (Onboarding)

- [ ] Full legal name (all scripts)
- [ ] Date/place of birth (natural) / Registration number (legal)
- [ ] Nationality(ies) / Jurisdiction of incorporation
- [ ] Current address / Registered office
- [ ] Government ID number(s)
- [ ] Beneficial owners (≥10% or control)
- [ ] Blockchain deposit/withdrawal addresses
- [ ] Source of funds/wealth documentation
- [ ] PEP/sanctions/adverse media screening (Sumsub)
- [ ] Enhanced due diligence (if high-risk)

### Appendix B: Asset Freeze Notification Template (HANFA)

```
TO: Hrvatska agencija za nadzor financijskih usluga (HANFA)
FROM: Kovanica Protocol d.o.o. — Sanctions Compliance Officer
DATE: [DATE]
REF: [UNIQUE REFERENCE]

SUBJECT: Asset Freeze Notification — [DESIGNATION REFERENCE]

1. DESIGNATED PERSON/ENTITY: [Name per Official Journal/OFAC/UN]
2. DESIGNATION SOURCE: [EU Regulation / OFAC / UNSCR number]
3. DESIGNATION DATE: [Date]
4. CLIENT DETAILS: [Client ID, Name, Address, Nationality]
5. FROZEN ASSETS: [Asset type, quantity, wallet address/account, value EUR]
6. FREEZE TIMESTAMP: [ISO 8601]
7. LEGAL BASIS: [Specific regulation article]
8. ACTIONS TAKEN: [Account blocked, transactions rejected, assets segregated]
9. CONTACT: [SCO name, email, phone]

Signed: _________________________
[Name], Sanctions Compliance Officer
```

### Appendix C: False Positive Clearance Form

```
CASE ID: [AUTO-GENERATED]
DATE: [DATE]
ANALYST: [NAME]

SCREENING HIT:
- List: [EU/UN/OFAC/UK/Other]
- Matched Name: [NAME]
- Designation Reference: [REF]

CLIENT DATA:
- Client ID: [ID]
- Full Name: [NAME]
- DOB: [DATE]
- Nationality: [COUNTRY]
- ID Number: [NUMBER]
- Address: [ADDRESS]
- Blockchain Addresses: [LIST]

ANALYSIS:
[Detailed comparison showing differentiating factors]

BLOCKCHAIN ANALYSIS (if address match):
[Cluster analysis, transaction history, attribution evidence]

CONCLUSION: ☐ FALSE POSITIVE  ☐ POTENTIAL MATCH (ESCALATE)  ☐ TRUE POSITIVE (FREEZE)

ANALYST SIGNATURE: _______________  DATE: __________
SCO APPROVAL: ___________________  DATE: __________
```
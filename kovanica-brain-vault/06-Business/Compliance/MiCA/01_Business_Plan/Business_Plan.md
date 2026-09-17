# Business Plan — Kovanica Payments d.o.o.

> **MiCA Article 14–15** | **Version**: v0.1-draft | **Status**: 📋 Planned  
> **Entity**: Kovanica Payments d.o.o. (to be incorporated, Croatia)  
> **Competent Authority**: HANFA  

---

## 1. Executive Summary

Kovanica Payments d.o.o. ("the Company") seeks authorization as a **Crypto-Asset Service Provider (CASP)** under MiCA Regulation (EU) 2023/1114 to operate the **Kovanica Protocol** — a high-throughput, DAG-based distributed ledger (BlockDAG) using GHOSTDAG consensus — as a regulated payment and settlement infrastructure.

**Services Applied For** (All 8 CASP services per MiCA Annex):
1. **Custody and administration** of crypto-assets on behalf of clients
2. **Exchange** fiat currency ↔ crypto-assets
3. **Exchange** crypto-assets ↔ crypto-assets
4. **Transfer** of crypto-assets on behalf of clients
5. **Execution of orders** for crypto-assets on behalf of clients
6. **Placing** of crypto-assets
7. **Advice** on crypto-assets
8. **Portfolio management** for crypto-assets

**Initial Capital**: €150,000 (minimum for portfolio management tier)  
**Target Market**: EU retail and professional users; initial focus Croatia + EU passporting  
**Launch Model**: Regulatory sandbox → full authorization → EU passporting

---

## 2. Company Overview

### 2.1 Legal Structure
| Item | Detail |
|------|--------|
| Legal Form | Društvo s ograničenom odgovornošću (d.o.o.) |
| Jurisdiction | Republic of Croatia |
| Registered Office | [TBD — Zagreb, Croatia] |
| Share Capital | €150,000 (fully paid) |
| OIB | [TBD] |
| MBS | [TBD] |
| LEI | [TBD — post-incorporation] |

### 2.2 Shareholding Structure (Proposed)
| Shareholder | % Holding | Qualifying Holding? | Nationality |
|-------------|-----------|---------------------|-------------|
| [Founder 1] | [%] | Yes (>10%) | [EU/Third country] |
| [Founder 2] | [%] | Yes (>10%) | [EU/Third country] |
| [Investor 1] | [%] | [Yes/No] | [EU/Third country] |
| Employee Pool | [%] | No | N/A |

> **Note**: All qualifying holders (>10%, >20%, >30%, >50%) subject to Fit & Proper assessment per Art. 25–27.

### 2.3 Directors & Senior Management
| Name | Role | Resident in HR? | Fit & Proper Status |
|------|------|-----------------|---------------------|
| [Director 1] | Managing Director / CEO | Yes | 📋 Pending |
| [Director 2] | CTO / Technical Director | [Yes/No] | 📋 Pending |
| [Director 3] | CCO / Compliance Officer | Yes | 📋 Pending |
| [Senior Manager 1] | Head of Operations | [Yes/No] | 📋 Pending |
| [Senior Manager 2] | Head of Risk | [Yes/No] | 📋 Pending |

---

## 3. Service Description

### 3.1 Core Infrastructure: Kovanica Protocol
- **Consensus**: GHOSTDAG (k=3) — parallel block production, high BPS
- **Ledger**: UTXO model, Ed25519 signatures, recursive GHOSTDAG linearization
- **Finality**: Blue-score depth (configurable, default 100 blocks)
- **Consensus Enforcement**: 
  - Proof-of-Work (opt-in, Nakamoto-style H·work < 2²⁵⁶)
  - Difficulty retargeting (opt-in, selected-chain window)
  - VRF leader eligibility (opt-in, stake-weighted sortition)
  - Hybrid PoW + VRF-staked admission (opt-in)
- **Multisig**: RFC-001 P2SH M-of-N (1-of-1 to 16-of-16), activation-gated
- **Light Client**: SPV with KVLSv1 (headers + Golomb-Rice filters), Merkle proofs

### 3.2 CASP Services Mapping

| MiCA Service | Protocol Feature | Implementation Status |
|--------------|------------------|----------------------|
| **Custody** | Multisig P2SH (RFC-001), FFI key custody, HSM integration | ✅ Consensus + lib; ⚠️ Node/FFI exposure |
| **Fiat↔Crypto Exchange** | Faucet API, merchant API, partner ramp (MoonPay/Banxa) | ⚠️ API design; partner TBD |
| **Crypto↔Crypto Exchange** | DEX integration (external), atomic swaps | ❌ Not in protocol; partner/integration |
| **Transfer** | `send`, `send_from`, `send_with` (multi-input UTXO) | ✅ Node + FFI |
| **Execution** | Mempool v2, fee market, RBF, block production | ✅ Fee market #46 |
| **Placing** | Token issuance framework (future) | ❌ Whitepaper template only |
| **Advice** | Portfolio analytics, risk scoring (future) | ❌ Not in protocol |
| **Portfolio Mgmt** | Staking/bonding, validator operations | ✅ Hybrid staking + unbond |

---

## 4. Target Market & Customers

### 4.1 Customer Segments
| Segment | Description | KYC Level | Estimated Volume (Yr 1) |
|---------|-------------|-----------|------------------------|
| **Retail — Croatia** | Individuals, KVNC payments, staking | Full KYC + AML | 5,000 users |
| **Retail — EU** | Cross-border payments, remittances | Full KYC + AML | 10,000 users |
| **Merchants** | POS, e-commerce, invoicing | KYB + UBO | 500 merchants |
| **Professional / VASP** | Custody, settlement, staking-as-a-service | Enhanced KYC/KYB | 20 counterparties |
| **DeFi / Protocol** | Light-node integration, SPV verification | Contractual | 5 integrators |

### 4.2 Geographic Scope
- **Primary**: Croatia (HANFA home state)
- **Passporting**: All EU/EEA member states (MiCA Art. 58)
- **Excluded**: Sanctioned jurisdictions, high-risk third countries (FATF list)

---

## 5. Business Model & Revenue

### 5.1 Revenue Streams
| Stream | Pricing Model | Est. Yr 1 Revenue |
|--------|---------------|-------------------|
| **Transaction fees** | Dynamic fee market (p90 mempool) + network fee | €50k |
| **Custody fees** | 0.1–0.5% p.a. on AUM (tiered) | €100k |
| **Staking commission** | 5–10% of validator rewards | €75k |
| **FX spread** | 0.5–1.5% on fiat↔crypto | €200k |
| **Merchant acquiring** | 1–2% per transaction | €150k |
| **API / Enterprise** | Monthly subscription + volume | €80k |
| **Total** | | **€655k** |

### 5.2 Cost Structure (Yr 1)
| Category | Est. Cost |
|----------|-----------|
| Personnel (8 FTE) | €400k |
| Legal & Compliance | €100k |
| Custody Partner / HSM | €80k |
| KYC / AML Provider | €40k |
| Infrastructure (cloud, nodes) | €60k |
| Insurance | €25k |
| Audit & Penetration Test | €50k |
| Marketing & Acquisition | €100k |
| **Total** | **€855k** |

### 5.3 Financial Projections (3 Years)

| Metric | Year 1 | Year 2 | Year 3 |
|--------|--------|--------|--------|
| Revenue | €655k | €2.1M | €5.5M |
| Operating Expenses | €855k | €1.4M | €2.8M |
| EBITDA | -€200k | €700k | €2.7M |
| Users (cumulative) | 15k | 75k | 250k |
| TVL / AUM | €2M | €25M | €100M |
| Capital Ratio | >1.5x | >1.5x | >1.5x |

> **Capital Adequacy**: Own funds maintained at ≥ €150k + 0.05% of safeguarded assets (Art. 56)

---

## 6. Operational Plan

### 6.1 Organization Chart
```
Managing Director (CEO)
├── Compliance Officer (CCO) — MLRO, DPO liaison
├── CTO
│   ├── Protocol Engineering
│   ├── Node/Infrastructure
│   ├── Mobile/FFI
│   └── Security
├── Head of Operations
│   ├── Customer Support
│   ├── Merchant Onboarding
│   └── Partner Integrations
├── Head of Risk
│   ├── AML/CTF Monitoring
│   ├── Transaction Surveillance
│   └── ICT Risk / DORA
└── Finance & Admin
    ├── Treasury / Own Funds
    └── Regulatory Reporting
```

### 6.2 Key Outsourcing (Material)
| Function | Provider | Jurisdiction | Oversight |
|----------|----------|--------------|-----------|
| **Custody (Hot)** | Fireblocks / Copper / BitGo | EU/UK/US | SLA, audit rights, termination |
| **KYC/AML** | Sumsub / Veriff | EU | Data processing agreement |
| **Travel Rule** | TRISA / OpenVASP | Global | Inter-VASP agreement |
| **Cloud Infrastructure** | AWS / Hetzner / GCP | EU region | DPA, SOC2 |
| **Penetration Testing** | [Certified firm] | EU | Annual + post-major-change |

---

## 7. Risk Management Summary

| Risk Category | Key Risks | Mitigation |
|---------------|-----------|------------|
| **Regulatory** | Authorization delay, sandbox rejection | Dual-track (sandbox + full), legal counsel |
| **Capital** | Own funds breach | Monthly monitoring, stress testing, capital buffer |
| **Custody** | Key loss, hack, partner failure | Multisig, HSM, geographic distribution, insurance |
| **AML** | SAR filing failure, sanctions breach | Automated screening, independent audit |
| **ICT/DORA** | Outage > RTO, data breach | HA architecture, IRP/BCP tested, encryption |
| **Market** | KVNC volatility, liquidity | Market making, reserves, circuit breakers |
| **Reputational** | Security incident, consumer complaint | Transparency, rapid response, ADR |

---

## 8. Compliance Framework

| Framework | Status | Owner |
|-----------|--------|-------|
| MiCA CASP Authorization | 📋 Application prep | CCO |
| AML/CTF (AMLD6, TFR) | 📋 Policy drafting | MLRO |
| DORA (ICT Resilience) | 📋 Framework design | Head of Risk |
| GDPR | 📋 DPIA + policies | DPO (external) |
| Consumer Protection | 📋 Policy drafting | CCO |
| Market Abuse (MAR) | 📋 Policy drafting | CCO |
| Prudential (Own Funds) | 📋 Calculation model | Finance |

---

## 9. Milestones

| Milestone | Target Date | Status |
|-----------|-------------|--------|
| Entity Incorporation | Week 2 | 📋 Planned |
| Capital Deposit (€150k) | Week 2 | 📋 Planned |
| Core Policies v1.0 | Week 3 | 📋 Planned |
| Custody Partner Contract | Week 3 | 📋 Planned |
| KYC/Travel Rule Integration | Week 4 | 📋 Planned |
| IRP/BCP Tested | Week 5 | 📋 Planned |
| HANFA Sandbox Application | Week 6 | 📋 Planned |
| Sandbox Admission | Week 10–14 | 📋 Planned |
| Pilot Launch | Week 14–18 | 📋 Planned |
| Full Authorization | Month 12–18 | 📋 Planned |

---

## 10. Appendices

- Appendix A: Shareholder Register → `[[../02_Fit_and_Proper/Shareholder_Register]]`
- Appendix B: Director CVs & Declarations → `[[../02_Fit_and_Proper/Fit_Proper_Directors]]`
- Appendix C: Technical Architecture → `kovanica-protocol/AGENTS.md`
- Appendix D: Protocol Specs → `kovanica-protocol/RFC-001-Multisig.md`
- Appendix E: Financial Model (detailed) → `Financial_Model.xlsx` (external)
- Appendix F: Organizational Chart (detailed) → `Org_Chart.pdf` (external)

---

## Document Control

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| v0.1-draft | 2026-09-06 | [Author] | Initial template |
| v1.0-submission | TBD | [Author] | Legal review complete |

---

**Cross-References**:
- `[[../02_Fit_and_Proper/Fit_Proper_Directors]]`
- `[[../02_Fit_and_Proper/Fit_Proper_Shareholders]]`
- `[[../03_Governance/Governance_Arrangements]]`
- `[[../04_Custody_Safeguarding/Custody_Policy]]`
- `[[../05_AML_CTF/AML_Policy]]`
- `[[../10_Capital_Adequacy/Own_Funds_Calculation]]`
- `[[../13_Registration_Dossier/MiCA_Cross_Reference]]`

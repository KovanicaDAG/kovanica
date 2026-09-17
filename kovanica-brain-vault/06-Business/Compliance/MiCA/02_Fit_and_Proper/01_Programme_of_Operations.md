# Programme of Operations — Kovanica Protocol CASP

## 1. Organisational Structure

### 1.1 Legal Entity
- **Name**: Kovanica Protocol Ltd. (d.o.o. — Croatian limited liability company)
- **Registered Office**: Zagreb, Republic of Croatia
- **Regulatory Supervisor**: HANFA (Hrvatska agencija za nadzor financijskih usluga)
- **Home Member State**: Croatia
- **Passporting**: EU-wide under MiCA Art. 58–60

### 1.2 Governance Bodies
```
General Meeting of Shareholders
         │
         ▼
Supervisory Board (3 members, independent majority)
         │
         ▼
Management Board (2 Directors: CEO + CCO)
         │
         ├── Risk Committee
         ├── Audit Committee  
         ├── Remuneration Committee
         └── AML Committee
```

### 1.3 Management Board (Senior Management per MiCA Art. 16)
| Director | Role | MiCA Function | Key Responsibilities |
|----------|------|---------------|---------------------|
| **Director A (CEO)** | Chief Executive Officer | Senior Management | Strategy, regulator relations, overall compliance culture, outsourcing oversight |
| **Director B (CCO)** | Chief Compliance Officer | Compliance Function + AML Function | Compliance monitoring, AML/CTF, regulatory reporting, policy approval |

*Both directors resident in Croatia; fit & proper assessed per `03_Fit_Proper_Directors.md`*

## 2. Key Functions & Reporting Lines

### 2.1 Core Functions (MiCA Art. 16–17)
| Function | Head | Reporting To | Independence |
|----------|------|--------------|--------------|
| **Compliance** | Director B (CCO) | Management Board | Direct access to Supervisory Board |
| **AML/CTF** | AML Officer | Director B (CCO) | Direct access to Management Board |
| **Risk Management** | CTO (interim) | Director A (CEO) | Independent of business lines |
| **ICT Risk / DORA** | CTO | Director A (CEO) | Independent reporting to Risk Committee |
| **Internal Audit** | Outsourced | Audit Committee | Fully independent |
| **Data Protection (DPO)** | External DPO | Management Board | Independent per GDPR Art. 38 |

### 2.2 Operational Functions
| Function | Head | Key Activities |
|----------|------|----------------|
| **Client Onboarding** | Head of Operations | KYC (Sumsub), CDD/EDD, account opening, ongoing monitoring |
| **Trading & Execution** | Head of Trading | Order management, best execution, liquidity sourcing |
| **Settlement & Custody Ops** | Head of Operations | Copper coordination, withdrawal processing, reconciliation |
| **Client Support** | Client Support Lead | Tier 1/2 support, complaints handling, escalation |
| **Finance & Treasury** | CFO (part-time) | Own funds monitoring, liquidity, regulatory reporting |

## 3. Outsourcing Register (Critical & Material)

Per MiCA Art. 30–31, DORA Art. 28–30 — see `02_Ops/03_Outsourcing_Register.md` for full register.

### 3.1 Critical Outsourcing Arrangements
| # | Function | Provider | Jurisdiction | Contract Status | Review Cycle |
|---|----------|----------|--------------|-----------------|--------------|
| 1 | **Crypto-Asset Custody & Settlement** | **Copper Technologies (UK) Ltd** | UK (FCA registered) | Negotiating | Annual + continuous |
| 2 | **KYC/AML Identity Verification** | **Sumsub Inc.** | UK/US (global) | Signed | Annual + continuous |
| 3 | **Cloud Infrastructure** | **Amazon Web Services (AWS)** | Ireland (EU) | Signed | Annual |

### 3.2 Material Outsourcing Arrangements
| # | Function | Provider | Jurisdiction | Contract Status |
|---|----------|----------|--------------|-----------------|
| 4 | Legal & Regulatory Advisory | Croatian Law Firm | Croatia | Retainer |
| 5 | Audit & Assurance | Big 4 Audit Firm | Croatia | Annual engagement |
| 6 | Penetration Testing | CREST-certified Firm | EU | Per project |
| 6 | Travel Rule Counterparty Network | **OpenVASP / TRISA** | Global | Technical integration |

## 4. Business Processes

### 4.1 Client Onboarding Flow
```
1. Lead Capture (Web/App) → 2. Sumsub KYC Initiation → 3. Automated CDD/EDD
         │                           │                        │
         ▼                           ▼                        ▼
   CRM Entry                  Document Check            Risk Score
         │                           │                        │
         ▼                           ▼                        ▼
4. Sanctions/PEP Screen → 5. Approval/Rejection → 6. Account Activation
         │                           │                        │
         ▼                           ▼                        ▼
   World-Check/              Compliance Officer         Copper Sub-Account
   Dow Jones                        Sign-off                 Creation
```

### 4.2 Trade Execution Flow
```
Client Order → Best Execution Check → Internal Match / External Venue → Copper Settlement → Confirmation
     │               │                      │                    │                  │
     ▼               ▼                      ▼                    ▼                  ▼
  Validation    Price/Size/           Liquidity            MPC Signature       Regulatory
  (limits,      Venue Policy          Providers            + Blockchain        Reporting
   suitability)                                                                 (MiCA/TFR)
```

### 4.3 Transfer / Withdrawal Flow
```
Client Request → AML Screen (Travel Rule) → Copper Authorization → Blockchain Broadcast → Confirmation
      │                  │                       │                      │                  │
      ▼                  ▼                       ▼                      ▼                  ▼
  Limits Check    Originator/           MPC Multi-Sig           Mempool Monitor      Regulatory
  (daily/monthly)  Beneficiary Data      Authorization           + Finality           Reporting
```

## 5. Resource Requirements

### 5.1 Personnel (Year 1)
| Role | FTE | Location | MiCA Classification |
|------|-----|----------|---------------------|
| Director A (CEO) | 1.0 | Zagreb | Senior Management |
| Director B (CCO) | 1.0 | Zagreb | Senior Management + Compliance + AML |
| CTO | 1.0 | Zagreb | ICT Risk Management |
| Head of Operations | 1.0 | Zagreb | Operational Function |
| AML Officer | 1.0 | Zagreb | AML Function |
| Client Support Lead | 1.0 | Zagreb | Complaints Handling |
| **Total** | **6.0** | | |

### 5.2 Technology Infrastructure
| System | Purpose | Criticality | Provider |
|--------|---------|-------------|----------|
| Core Ledger (Kovanica Node) | Settlement, balances | Critical | In-house |
| Trading Engine | Order matching, execution | Critical | In-house |
| CRM & Onboarding | Client lifecycle | Critical | In-house + Sumsub API |
| Transaction Monitoring | AML/CTF alerts | Critical | In-house + Sumsub |
| Regulatory Reporting | HANFA/ECB submissions | Critical | In-house |
| ICT Monitoring (Prometheus/Grafana) | DORA observability | Critical | Self-hosted |

### 5.3 Physical Infrastructure
- **Registered Office**: Zagreb, Croatia (regulatory requirement)
- **Data Centre**: AWS EU-Central-1 (Frankfurt) — primary; EU-West-1 (Ireland) — DR
- **Business Continuity Site**: Remote work capability (tested quarterly)

## 6. Operational Risk Management

### 6.1 Key Risk Indicators (KRIs)
| KRI | Threshold | Frequency | Owner |
|-----|-----------|-----------|-------|
| System uptime (core ledger) | < 99.9% | Real-time | CTO |
| KYC completion rate | < 95% | Daily | Head of Operations |
| AML alert backlog (>48h) | > 10 alerts | Daily | AML Officer |
| Complaint resolution (>15 days) | > 5% | Weekly | Client Support Lead |
| Own funds ratio | < 1.2x requirement | Daily | CFO |
| Copper settlement failures | > 0 | Real-time | Head of Operations |

### 6.2 Incident Management (DORA Aligned)
- **Classification**: Critical / High / Medium / Low (per `02_Ops/06_Incident_Response_Plan.md`)
- **Notification**: HANFA within 4 hours for major incidents (DORA Art. 19)
- **Root Cause Analysis**: Required for all Critical/High within 5 business days
- **Testing**: Annual scenario testing; quarterly tabletop exercises

## 7. Business Continuity (Summary — see `02_Ops/05_Business_Continuity_Plan.md`)

| Scenario | RTO | RPO | Strategy |
|----------|-----|-----|----------|
| Core ledger outage | 1 hour | 0 | Multi-node DAG; auto-failover |
| Copper API outage | 4 hours | 0 | Queue withdrawals; manual override |
| AWS region failure | 2 hours | 15 min | Multi-AZ + cross-region DR |
| Key personnel loss | 24 hours | N/A | Cross-training; succession plan |
| Regulatory action | Immediate | N/A | Pre-approved communication plan |

## 8. Regulatory Reporting Calendar

| Report | Frequency | Deadline | Owner | Recipient |
|--------|-----------|----------|-------|-----------|
| Own Funds (COREP) | Quarterly | 15th post-quarter | CFO | HANFA |
| Large Exposures | Quarterly | 15th post-quarter | CFO | HANFA |
| Liquidity (LIQUID) | Monthly | 15th post-month | CFO | HANFA |
| AML/CTF Annual Report | Annual | 31 March | AML Officer | HANFA / Ured za sprječavanje pranja novca |
| SARs (Suspicious Activity) | Ad-hoc | Immediate | AML Officer | Ured za sprječavanje pranja novca |
| DORA Incident Report | Ad-hoc | 4 hours (major) | CTO | HANFA / ECB |
| MiCA Transparency Disclosure | Annual | 30 April | CCO | Public / HANFA |
| Passporting Notification | Ad-hoc | Pre-activity | CCO | Host NCA |

## 9. Document Control

| Element | Detail |
|---------|--------|
| **Version** | 1.0 |
| **Classification** | Confidential — Internal / Regulatory |
| **Owner** | Director A (CEO) |
| **Approved By** | Management Board |
| **Review Cycle** | Semi-annual or upon material change |
| **Related Documents** | `03_Outsourcing_Register.md`, `05_Business_Continuity_Plan.md`, `06_Incident_Response_Plan.md`, `01_AML/00_AML_Policy.md` |

---

**End of Document**

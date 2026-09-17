# Governance Arrangements

**Document ID:** KOV-POL-GOV-000
**Version:** 1.0
**Classification:** Confidential
**Owner:** CEO / Chair of Management Board
**Review Cycle:** Annual (or upon material change)
**Status:** Approved

---

## 1. Purpose & Regulatory Basis

This document describes the governance arrangements of Kovanica Protocol d.o.o. ("the CASP") in compliance with **MiCA Articles 16–17, 25–29**, **EBA Guidelines on Internal Governance (EBA/GL/2021/05)**, **Croatian Companies Act (Zakon o trgovačkim društvima)**, and **HANFA CASP Authorization Requirements**.

| Regulation | Key Requirements |
|------------|------------------|
| MiCA Art. 16 | Management body: fit & proper, collective suitability, independence, time commitment |
| MiCA Art. 17 | Qualifying shareholders: fit & proper assessment |
| MiCA Art. 25 | Governance arrangements: clear organizational structure, transparent lines of responsibility |
| MiCA Art. 26 | Risk management: independent risk function, risk committee |
| MiCA Art. 27 | Compliance function: independent, MLRO, monitoring |
| MiCA Art. 28 | Internal audit: independent, risk-based plan, reporting to management body |
| MiCA Art. 29 | Outsourcing: governance, due diligence, monitoring |
| EBA/GL/2021/05 | Collective suitability, diversity, induction, ongoing training, evaluation |

---

## 2. Legal Structure & Ownership

| Element | Detail |
|---------|--------|
| **Legal Form** | Društvo s ograničenom odgovornošću (d.o.o.) — Limited Liability Company |
| **Registered Office** | Zagreb, Republic of Croatia |
| **Registration** | MBS: [NUMBER], OIB: [NUMBER] |
| **Share Capital** | €50,000 (minimum for Class 2 CASP per MiCA Art. 55) |
| **Shares** | 50,000 ordinary shares, €1 nominal each |
| **Ownership** | See `00_Foundation/02_Shareholder_Register.md` |

---

## 3. Governance Structure Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                    GENERAL MEETING                          │
│                  (Shareholders)                             │
└─────────────────────────┬───────────────────────────────────┘
                          │ Appoints / Removes
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                  MANAGEMENT BOARD                           │
│              (Uprava — 2 Directors)                         │
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │   CEO (Director)│  │  CCO (Director) │                   │
│  │  (Chair)        │  │  (Deputy)       │                   │
│  └─────────────────┘  └─────────────────┘                   │
└─────────────────────────┬───────────────────────────────────┘
                          │ Delegates / Oversees
          ┌───────────────┼───────────────┐
          ▼               ▼               ▼
┌─────────────────┐ ┌─────────────┐ ┌─────────────────┐
│  RISK COMMITTEE │ │ AUDIT COMM. │ │  REMUNERATION   │
│  (Board-level)  │ │ (Board-level)│ │   COMMITTEE     │
└────────┬────────┘ └──────┬──────┘ └────────┬────────┘
         │                 │                 │
         ▼                 ▼                 ▼
┌─────────────────────────────────────────────────────────────┐
│              EXECUTIVE MANAGEMENT TEAM                      │
│  CTO • CISO • COO • MLRO • DPO • Head of Engineering       │
└─────────────────────────────────────────────────────────────┘
```

---

## 4. Management Board (Uprava)

### 4.1 Composition

| Position | Name | Appointment Date | Term | Fit & Proper Ref |
|----------|------|------------------|------|------------------|
| **CEO / Chair** | [Director A — Placeholder] | [DATE] | 5 years | `00_Foundation/03_Fit_Proper_Directors.md` |
| **CCO / Deputy** | [Director B — Placeholder] | [DATE] | 5 years | `00_Foundation/03_Fit_Proper_Directors.md` |

**Collective Suitability Matrix:**

| Competency Area | CEO | CCO | Collective Coverage |
|-----------------|-----|-----|---------------------|
| Crypto/Blockchain Technology | ●●● | ●●○ | ✅ |
| Financial Markets / Trading | ●●○ | ●●● | ✅ |
| Risk Management | ●●○ | ●●● | ✅ |
| AML/CTF & Sanctions | ●●○ | ●●● | ✅ |
| Legal / Regulatory (MiCA, DORA) | ●●○ | ●●● | ✅ |
| IT / Cybersecurity | ●●● | ●○○ | ✅ |
| Governance / Internal Control | ●●○ | ●●● | ✅ |
| Strategic Leadership | ●●● | ●●○ | ✅ |

*Scale: ●○○ Basic, ●●○ Proficient, ●●● Expert*

### 4.2 Responsibilities (Per Director)

| CEO (Chair) | CCO (Deputy) |
|-------------|--------------|
| Overall strategy & execution | Compliance, AML, regulatory affairs |
| Technology & product roadmap | Risk management framework |
| Business development & partnerships | Internal audit oversight |
| Investor & stakeholder relations | Outsourcing governance |
| Chair of Management Board | Deputy Chair; acts for CEO when absent |
| **Key Function Owner:** Technology, Business | **Key Function Owner:** Compliance, Risk, Internal Audit |

### 4.3 Decision-Making

| Matter | Authority | Quorum | Voting |
|--------|-----------|--------|--------|
| Strategy & budget | Management Board | 2/2 | Unanimous |
| Risk appetite & limits | Management Board | 2/2 | Unanimous |
| Key appointments (C-level) | Management Board | 2/2 | Unanimous |
| Outsourcing (critical) | Management Board | 2/2 | Unanimous |
| Regulatory notifications | Management Board | 2/2 | Unanimous |
| Day-to-day operations | Individual Director | 1/1 | N/A (delegated) |

**Conflict of Interest:** Directors must declare conflicts; conflicted director recuses; remaining director decides or escalates to General Meeting.

---

## 5. Board-Level Committees

### 5.1 Risk Committee

| Element | Detail |
|---------|--------|
| **Chair** | CCO |
| **Members** | CEO, CRO (invited), CISO (invited), Head of Engineering (invited) |
| **Frequency** | Monthly (ad-hoc for material risk events) |
| **Mandate** | Risk appetite, ICAAP/ILAAP, risk limits, new product risk, concentration risk, third-party risk |
| **Reporting** | Minutes to Management Board; quarterly risk report to General Meeting |
| **Key Policies Overseen** | `02_ICT_Risk_Management.md`, `00_Custody_Policy.md`, `03_Outsourcing_Register.md` |

### 5.2 Audit Committee

| Element | Detail |
|---------|--------|
| **Chair** | CCO (independent of operational management) |
| **Members** | CEO, External Auditor (invited), Head of Internal Audit |
| **Frequency** | Quarterly (aligned with audit cycle) |
| **Mandate** | Financial reporting, internal audit plan/results, external auditor independence, control effectiveness |
| **Reporting** | Minutes to Management Board; annual audit committee report to General Meeting |

### 5.3 Remuneration Committee

| Element | Detail |
|---------|--------|
| **Chair** | CEO |
| **Members** | CCO, External Advisor (invited) |
| **Frequency** | Semi-annual |
| **Mandate** | Remuneration policy, variable pay, MRT identification, gender pay gap, regulatory compliance |
| **Reporting** | Minutes to Management Board; annual remuneration report to General Meeting |

---

## 6. Key Functions (MiCA Art. 25–28)

| Function | Holder | Independence | Reporting Line | Policy Reference |
|----------|--------|--------------|----------------|------------------|
| **Risk Management** | CRO | Independent of business lines | CCO (Risk Committee) | `02_ICT_Risk_Management.md` |
| **Compliance** | CCO | Independent; direct Board access | Management Board | `00_AML_Policy.md`, `01_Travel_Rule_Policy.md` |
| **Internal Audit** | Head of IA | Independent; reports to Audit Committee | Audit Committee | Annual audit plan |
| **MLRO** | MLRO | Independent; direct FIU access | CCO | `03_Suspicious_Activity_Reporting.md` |
| **DPO** | DPO | Independent; reports to highest management | CEO | GDPR compliance |
| **Information Security** | CISO | Independent of IT operations | CCO / Risk Committee | `02_ICT_Risk_Management.md` |

**Key Function Holders — Fit & Proper:** All assessed per `03_Fit_Proper_Directors.md` criteria.

---

## 7. Conflicts of Interest Management

### 7.1 Policy Principles

| Principle | Implementation |
|-----------|----------------|
| **Identification** | Annual declaration; transaction-specific disclosure; register maintained |
| **Prevention** | Chinese walls (information barriers); segregated reporting lines; gift policy |
| **Management** | Recusal; independent review; client disclosure; Board oversight |
| **Monitoring** | Quarterly register review; ad-hoc for new relationships |

### 7.2 Conflict Register Categories

| Category | Examples | Mitigation |
|----------|----------|------------|
| **Personal** | Director shareholding in counterparty | Recusal; blind trust |
| **Family** | Spouse employed by vendor | Disclosure; independent procurement |
| **Business** | CASP investing in protocol it operates | Arm's length; independent valuation |
| **Cross-Role** | Director also serving on vendor board | Resignation or recusal |

### 7.3 Register Maintenance

- **Owner:** CCO (Compliance)
- **Review:** Quarterly by Risk Committee
- **Retention:** 10 years
- **Regulatory Access:** Available to HANFA on request

---

## 8. Remuneration Policy

### 8.1 Principles (MiCA Art. 25 / EBA Guidelines)

| Principle | Application |
|-----------|-------------|
| **Gender Neutral** | Job evaluation methodology; annual pay equity analysis |
| **Risk Alignment** | Variable pay capped; deferral; malus/clawback |
| **Performance Based** | Financial + non-financial KPIs (risk, compliance, customer) |
| **Transparency** | Annual remuneration report published |

### 8.2 Material Risk Takers (MRTs)

| Category | Identification Criteria | Count (Est.) |
|----------|------------------------|--------------|
| **Management Board** | All directors | 2 |
| **Key Function Holders** | CRO, CISO, MLRO, DPO, Head of IA | 5 |
| **Senior Business** | Heads of Trading, Custody, Engineering | 3 |
| **Control Functions** | Senior compliance, risk, audit staff | 4 |
| **Total MRTs** | | **~14** |

### 8.3 Variable Remuneration Structure

| Component | MRTs | Non-MRTs |
|-----------|------|----------|
| **Fixed / Variable Split** | Max 1:1 (1:2 with shareholder approval) | Market aligned |
| **Deferral** | 40% (60% if > €200k) over 4 years | N/A |
| **Instruments** | 50% in shares / share-linked | Cash |
| **Malus / Clawback** | 5 years from award | N/A |
| **Retention** | 12 months post-vesting | N/A |

---

## 9. Induction & Ongoing Training

### 9.1 Induction Program (New Directors / Key Function Holders)

| Module | Duration | Owner |
|--------|----------|-------|
| CASP Business Model & Strategy | 4h | CEO |
| MiCA / DORA / AML Regulatory Framework | 8h | CCO / Legal |
| Technology Architecture (DAG, Consensus, Node) | 8h | CTO |
| Risk Management Framework | 4h | CRO |
| AML/CTF, Travel Rule, Sanctions | 4h | MLRO |
| ICT & Cybersecurity | 4h | CISO |
| Governance, Conflicts, Remuneration | 2h | CCO / Legal |
| Internal Audit & Controls | 2h | Head of IA |
| **Total** | **36 hours** | |

### 9.2 Ongoing Training (Annual Minimum)

| Audience | Hours | Topics |
|----------|-------|--------|
| Management Board | 15h | Regulatory updates, emerging risks, strategy |
| Key Function Holders | 20h | Specialized (risk, compliance, audit, security) |
| All Staff | 8h | AML, security awareness, data protection, conduct |

---

## 10. Board Effectiveness Evaluation

| Evaluation | Frequency | Method | Owner |
|------------|-----------|--------|-------|
| **Collective Suitability** | Annual | Self-assessment + external facilitator (every 3 years) | Chair / CCO |
| **Individual Director** | Annual | Peer review + Chair assessment | Chair |
| **Committee Effectiveness** | Annual | Self-assessment | Committee Chairs |
| **Key Function Holders** | Annual | KPI review + 360° feedback | Management Board |

**Outcomes:** Development plans; composition changes; training needs; reported to General Meeting.

---

## 11. Information Flow & Reporting

### 11.1 Management Board Reporting Calendar

| Report | Frequency | Owner | Recipient |
|--------|-----------|-------|-----------|
| **CEO Dashboard** | Monthly | CEO | Management Board |
| **Risk Report** | Monthly | CRO | Risk Committee → Board |
| **Compliance Report** | Monthly | CCO | Management Board |
| **AML/CTF Report** | Monthly | MLRO | CCO → Board |
| **ICT / Security Report** | Monthly | CISO | Risk Committee → Board |
| **Internal Audit Report** | Quarterly | Head of IA | Audit Committee → Board |
| **Financial Report** | Monthly | CFO (outsourced) | Management Board |
| **Outsourcing Report** | Quarterly | COO | Risk Committee → Board |
| **Incident Summary** | Monthly | CISO / MLRO | Management Board |
| **Annual Governance Report** | Annual | CCO | General Meeting |

### 11.2 Escalation Thresholds

| Issue | Escalation To | Timeline |
|-------|---------------|----------|
| Regulatory breach / investigation | Management Board → HANFA | Immediate |
| Material risk limit breach | Risk Committee → Board | Same day |
| Critical outsourcing failure | Risk Committee → Board | Same day |
| Major security incident | CISO → CCO → Board | 1 hour (DORA) |
| SAR filing | MLRO → CCO → Board | Before filing |
| Media / reputational risk | CEO → Board | Immediate |

---

## 12. Succession Planning

| Role | Successor | Readiness | Development Plan |
|------|-----------|-----------|------------------|
| CEO | CCO | Ready (12 months) | External leadership program |
| CCO | Senior Compliance Manager | Ready (6 months) | ICA Diploma, Board exposure |
| CRO | Senior Risk Analyst | Developing (18 months) | FRM certification, committee chair |
| CISO | Senior Security Engineer | Developing (12 months) | CISSP, incident command training |
| MLRO | Deputy MLRO | Ready (3 months) | CAMS, FIU liaison |

---

## 13. Cross-References

| Document | Reference |
|----------|-----------|
| `00_Foundation/03_Fit_Proper_Directors.md` | Director assessments |
| `00_Foundation/04_Fit_Proper_Shareholders.md` | Shareholder assessments |
| `00_Foundation/01_Programme_of_Operations.md` | Org structure, key functions |
| `02_ICT_Risk_Management.md` | Risk management framework |
| `00_AML_Policy.md` | Compliance function scope |
| `03_Suspicious_Activity_Reporting.md` | MLRO role |
| `01_Capital_Requirements.md` | Capital adequacy governance |

---

## 14. Document Control

| Field | Value |
|-------|-------|
| **Document ID** | KOV-POL-GOV-000 |
| **Version** | 1.0 |
| **Classification** | Confidential |
| **Owner** | CEO / Chair of Management Board |
| **Approved By** | Management Board |
| **Approval Date** | [DATE] |
| **Next Review** | [DATE + 1 year] |
| **Distribution** | Management Board, General Meeting, HANFA (on request) |

### Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-15 | Compliance Team | Initial version for MiCA CASP application |

---

*End of Document*
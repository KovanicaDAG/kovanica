# Complaints Handling Policy

**Document ID:** KOV-POL-OPS-004
**Version:** 1.0
**Classification:** Confidential
**Owner:** MLRO / Head of Customer Support
**Review Cycle:** Annual (or upon regulatory change)
**Status:** Approved

---

## 1. Purpose & Regulatory Basis

This policy establishes the framework for handling client complaints at Kovanica Protocol d.o.o. ("the CASP") in compliance with:

| Regulation | Reference |
|------------|-----------|
| MiCA | Articles 76–77 (complaints handling, out-of-court dispute resolution) |
| EBA Guidelines | EBA/GL/2022/03 (complaints handling for CASPs) |
| Croatian Consumer Protection Act | Zakon o zaštiti potrošača |
| Croatian Financial Services Supervision Act | Zakon o nadzoru financijskih usluga |
| PSD2 / EMD2 (by analogy) | Complaints handling standards for payment services |

**Scope:** All complaints from **retail clients** and **professional clients** regarding CASP services (exchange, transfer, execution, custody).

---

## 2. Definitions

| Term | Definition |
|------|------------|
| **Complaint** | Any expression of dissatisfaction (oral or written) by a client regarding the provision of a CASP service, where a response is expected |
| **Retail Client** | Natural person acting for purposes outside their trade, business, or profession |
| **Professional Client** | Client meeting MiCA Art. 2(1)(c) criteria (large undertakings, financial institutions, etc.) |
| **Complaint Handler** | Designated staff member responsible for investigation and resolution |
| **MLRO** | Money Laundering Reporting Officer (escalation point for AML-related complaints) |
| **DPO** | Data Protection Officer (escalation for GDPR-related complaints) |
| **Resolution** | Final outcome communicated to client (acceptance, rejection, partial acceptance, compensation) |

---

## 3. Governance & Roles

| Role | Responsibility |
|------|----------------|
| **Management Board** | Ultimate accountability; approve policy; review quarterly complaints report |
| **Head of Customer Support** | Day-to-day management; resource allocation; SLA monitoring |
| **Complaint Handlers (L1)** | Initial receipt, categorization, acknowledgment, basic resolution |
| **Senior Complaint Handlers (L2)** | Complex investigations, regulatory liaison, compensation decisions |
| **MLRO** | AML-related complaints; SAR triggers; regulatory reporting |
| **DPO** | GDPR-related complaints; data subject rights |
| **Legal Counsel** | Legal assessment; litigation risk; regulatory correspondence |
| **Compliance** | Root cause analysis; policy updates; training; HANFA reporting |

---

## 4. Complaint Channels

| Channel | Availability | Recording |
|---------|--------------|-----------|
| **Web Portal** | 24/7 | Auto-ticket creation (Jira Service Management) |
| **Email** | `complaints@kovanica.protocol` | Auto-ticket creation |
| **In-App** | 24/7 (mobile/web) | Auto-ticket creation |
| **Post** | Business days | Manual entry within 1 business day |
| **Phone** | Business hours (09:00–17:00 CET) | Call recording + manual ticket |
| **Regulatory Referral** | Via HANFA / EBA | Priority handling |

**Language Support:** Croatian, English (mandatory); German, Italian, Slovenian (best effort).

---

## 5. Process Flow

```text
┌─────────────┐     ┌──────────────┐     ┌─────────────┐     ┌──────────────────┐
│  Receipt    │────▶│ Acknowledgment│────▶│ Investigation│────▶│   Resolution     │
│  (T+0)      │     │  (T+1 day)   │     │  (T+15 days) │     │  (T+15 days)     │
└─────────────┘     └──────────────┘     └─────────────┘     └────────┬─────────┘
                                                                       │
                                                                       ▼
                                                            ┌──────────────────┐
                                                            │  Client Response │
                                                            │  + Closure /     │
                                                            │  Escalation      │
                                                            └──────────────────┘
```

### 5.1 Step-by-Step Timeline

| Step | Action | Timeline | Owner |
|------|--------|----------|-------|
| **1. Receipt** | Log complaint; assign unique ID (`KOV-CMP-YYYYMMDD-NNNN`); categorize | T+0 (same business day) | L1 Handler |
| **2. Acknowledgment** | Send written acknowledgment to client (email/portal) with: complaint ID, handler name, expected timeline, contact details | **T+1 business day** (retail); **T+2 business days** (professional) | L1 Handler |
| **3. Categorization** | Classify by: service (exchange/transfer/execution/custody), type (fees, execution, access, data, AML, other), severity (Low/Medium/High/Critical) | T+1 business day | L1 Handler |
| **4. Triage** | - **Low/Medium**: L1 handles<br>- **High**: L2 + Compliance notified<br>- **Critical** (AML, safeguarding, systemic): L2 + MLRO + Legal + Management Board notified immediately | T+1 business day | L1/L2 Handler |
| **5. Investigation** | Gather evidence (logs, transaction records, communications, blockchain data); interview staff; assess regulatory implications | **T+15 business days** (standard); **T+30 business days** (complex) | L1/L2 Handler |
| **6. Resolution Decision** | Determine outcome: uphold, reject, partial uphold; calculate compensation if applicable | T+15 business days | L2 Handler + Compliance |
| **7. Response** | Written response to client: decision, reasoning, remedy, right to escalate | **T+15 business days** (standard); **T+30 business days** (complex) | L2 Handler |
| **8. Closure** | Client accepts → close ticket; Client rejects → escalate to internal review / external ADR | T+5 business days after response | L1 Handler |

---

## 6. Timelines by Client Type & Complexity

| Client Type | Acknowledgment | Standard Resolution | Complex Resolution | Extension Notice |
|-------------|----------------|---------------------|-------------------|------------------|
| **Retail** | 1 business day | 15 business days | 30 business days | At T+10 (standard) / T+25 (complex) |
| **Professional** | 2 business days | 15 business days | 30 business days | At T+10 (standard) / T+25 (complex) |

**Extension Rules:**
- Maximum one extension of 15 business days
- Must notify client before original deadline with reason and new deadline
- Complex cases: cross-border, multiple parties, regulatory investigation, blockchain forensics required

---

## 8. Categorization Matrix

| Category | Sub-Categories | Typical SLA | Escalation |
|----------|----------------|-------------|------------|
| **Fees & Pricing** | Undisclosed fees, incorrect charges, spread complaints | Standard | L1 → L2 |
| **Execution Quality** | Slippage, rejection, partial fill, latency | Standard | L1 → L2 |
| **Account Access** | Login issues, 2FA, API keys, KYC delays | Standard | L1 → L2 |
| **Transfers/Withdrawals** | Delays, failed txs, wrong address, unhosted wallet | Standard | L1 → L2 |
| **Custody/Safeguarding** | Asset segregation, staking rewards, corporate actions | High | L2 + Compliance |
| **Data/Privacy** | Access requests, erasure, breach notification | High | DPO + Legal |
| **AML/Sanctions** | Account freeze, SAR, enhanced due diligence | Critical | MLRO + Legal + Board |
| **Technical/Platform** | Downtime, bugs, API errors, data accuracy | Medium | L2 + Engineering |
| **Regulatory/Compliance** | MiCA disclosures, whitepaper, suitability | High | Compliance + Legal |

---

## 9. Remedies & Compensation

| Remedy Type | Applicability | Approval |
|-------------|---------------|----------|
| **Apology & Explanation** | All upheld complaints | L1 Handler |
| **Fee Refund** | Incorrect charges, execution failures | L2 Handler (≤ €1,000); Compliance (> €1,000) |
| **Financial Compensation** | Direct losses from CASP error | Compliance + Legal (≤ €10,000); Board (> €10,000) |
| **Service Credit** | Platform downtime, SLA breach | L2 Handler |
| **Corrective Action** | Process improvement, system fix | Compliance + Engineering |
| **Goodwill Gesture** | Discretionary, non-precedent | Compliance |

**Compensation Principles:**
- Restore client to position had error not occurred
- No compensation for market movements
- Documented calculation methodology
- Tax implications considered

---

## 10. Escalation Paths

### 10.1 Internal Escalation

| Level | Trigger | Participants | Timeline |
|-------|---------|--------------|----------|
| **L1 → L2** | Complexity, High/Critical severity, client request | L2 Handler, Compliance | Immediate |
| **L2 → Management Board** | > €10,000 compensation, regulatory risk, systemic issue, media risk | CEO, CCO, Legal, MLRO/DPO | T+1 business day |
| **Board → External Legal** | Litigation threat, regulatory enforcement | External counsel | As needed |

### 10.2 External Escalation (Client-Initiated)

| Body | Jurisdiction | When |
|------|--------------|------|
| **HANFA** | Croatia | After internal process exhausted; or directly for regulatory breaches |
| **FIN-NET / EBA** | Cross-border EU | Cross-border retail complaints |
| **Croatian Financial Arbitration** | Croatia | Binding arbitration (if agreed) |
| **Court** | Croatia / EU | Legal proceedings |

**Information to Client:** Every final response must include:
- Right to refer to HANFA (address, website, form)
- Right to FIN-NET (for cross-border)
- Time limits for external referral

---

## 11. Record Keeping

| Record | Retention | Format |
|--------|-----------|--------|
| Complaint register (all fields) | 10 years | Electronic (Jira + CRM) |
| Correspondence (inbound/outbound) | 10 years | Electronic (email, portal, call recordings) |
| Investigation notes & evidence | 10 years | Electronic |
| Resolution decision & reasoning | 10 years | Electronic |
| Compensation payments | 10 years | Electronic (accounting system) |
| Root cause analysis reports | 10 years | Electronic |
| Quarterly/Annual reports | 10 years | Electronic |

**Register Fields:** ID, date received, client type, client ID, channel, category, severity, handler, acknowledgment date, investigation start/end, resolution date, outcome, remedy, compensation, escalation, closure date, root cause, regulatory reportable (Y/N).

---

## 12. Reporting & Analytics

### 12.1 Internal Reporting

| Report | Frequency | Audience | Content |
|--------|-----------|----------|---------|
| **Complaints Dashboard** | Real-time | Support, Compliance | Volume, SLA compliance, aging, categories |
| **Weekly Summary** | Weekly | Head of Support, Compliance | New, resolved, breached SLAs, trends |
| **Monthly Analysis** | Monthly | Management Board, Compliance | Root causes, systemic issues, compensation, regulatory |
| **Quarterly Board Report** | Quarterly | Management Board, HANFA (summary) | KPIs, trends, regulatory actions, policy changes |

### 12.2 Key Performance Indicators

| KPI | Target | Alert Threshold |
|-----|--------|-----------------|
| **Acknowledgment within SLA** | 100% | < 100% |
| **Resolution within SLA (standard)** | > 95% | < 90% |
| **Resolution within SLA (complex)** | > 90% | < 85% |
| **Re-opened Complaints** | < 5% | > 10% |
| **Escalation to HANFA** | 0 (preventable) | > 0 |
| **Compensation as % of Revenue** | < 0.1% | > 0.5% |
| **Root Cause Completion** | 100% within 30 days | < 100% |

### 12.3 Regulatory Reporting (HANFA)

| Report | Frequency | Deadline | Content |
|--------|-----------|----------|---------|
| **Complaints Summary** | Quarterly | 20 business days post-quarter | Volume, categories, outcomes, compensation, systemic issues |
| **Annual Complaints Report** | Annual | 31 March (for prior year) | Full year analysis, trends, policy effectiveness, MiCA Art. 76 compliance |
| **Ad Hoc** | As needed | Immediate | Systemic issues, media risk, regulatory breaches |

---

## 13. Training & Awareness

| Audience | Content | Frequency |
|----------|---------|-----------|
| **All Staff** | Complaint recognition, logging, escalation | Annual + onboarding |
| **L1 Handlers** | Process, categorization, communication, tools | Quarterly |
| **L2 Handlers** | Investigation techniques, regulatory framework, compensation | Semi-annual |
| **Management Board** | Trends, regulatory expectations, governance | Annual |

---

## 14. Vulnerable Clients

**Identification:** Age > 75, cognitive impairment, language barrier, financial distress, bereavement, disability.

**Enhanced Handling:**
- Dedicated L2 handler
- Extended timelines (+50%)
- Proactive communication (phone preferred)
- Family/representative involvement (with consent)
- Referral to specialized support organizations

---

## 15. Cross-References

| Document | Reference |
|----------|-----------|
| `00_AML_Policy.md` | AML-related complaints, SAR triggers |
| `03_Suspicious_Activity_Reporting.md` | Complaints triggering SAR |
| `02_ICT_Risk_Management.md` | Technical complaints, incident overlap |
| `06_Incident_Response_Plan.md` | Incident-related complaints |
| `01_Programme_of_Operations.md` | Key functions, operational risk |
| `05_Transparency_Disclosure.md` | Fee disclosure, execution policy complaints |

---

## 16. Document Control

| Field | Value |
|-------|-------|
| **Document ID** | KOV-POL-OPS-004 |
| **Version** | 1.0 |
| **Classification** | Confidential |
| **Owner** | MLRO / Head of Customer Support |
| **Approved By** | Management Board |
| **Approval Date** | [DATE] |
| **Next Review** | [DATE + 1 year] |
| **Distribution** | Management Board, Compliance, Support, Legal, MLRO, DPO |

### Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-15 | Compliance Team | Initial version for MiCA CASP application |

---

## Appendix A: Complaint Acknowledgment Template

```
Subject: Acknowledgment of Your Complaint [KOV-CMP-YYYYMMDD-NNNN]

Dear [Client Name],

Thank you for contacting Kovanica Protocol. We have received your complaint and assigned it reference number **KOV-CMP-YYYYMMDD-NNNN**.

**Your Complaint Handler:** [Name], [Title]
**Contact:** complaints@kovanica.protocol / +385 1 XXXXXXX
**Expected Resolution Timeline:** [15/30] business days (by [DATE])

We take your concern seriously and will investigate thoroughly. You will receive a substantive response within the timeline above. If we need more time, we will notify you before the deadline with reasons and a new date.

If you have additional information, please reply to this email quoting your complaint reference.

Yours sincerely,
Kovanica Protocol Complaints Team
```

---

## Appendix B: Final Response Template

```
Subject: Final Response to Your Complaint [KOV-CMP-YYYYMMDD-NNNN]

Dear [Client Name],

Following our investigation into your complaint (ref: KOV-CMP-YYYYMMDD-NNNN), we write to inform you of our decision.

**Summary of Complaint:** [Brief description]
**Investigation Conducted:** [Evidence reviewed, parties consulted]
**Decision:** [Uphold / Reject / Partially Uphold]
**Reasoning:** [Detailed explanation referencing evidence and applicable terms/regulations]

**Remedy:** [Apology / Fee refund of €X / Compensation of €X / Service credit / Corrective action]

**Your Rights:**
If you are dissatisfied with this outcome, you may refer your complaint to:
- **HANFA (Croatian Financial Services Supervisory Agency):** Miramarska 24b, 10000 Zagreb; www.hanfa.hr; prituzbe@hanfa.hr
- **FIN-NET (Cross-border EU complaints):** https://ec.europa.eu/info/business-economy-euro/banking-and-finance/financial-supervision-and-risk-management/financial-dispute-resolution-network-fin-net_en
- **Croatian Financial Arbitration:** [Details if applicable]

Time limits apply for external referrals. Please consult the relevant body for details.

Yours sincerely,
[Name], [Title]
Kovanica Protocol Complaints Team
```

---

*End of Document*
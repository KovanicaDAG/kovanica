# Capital Requirements

**Document ID:** KOV-POL-GOV-001
**Version:** 1.0
**Classification:** Confidential
**Owner:** CFO / CRO
**Review Cycle:** Quarterly (aligned with regulatory reporting)
**Status:** Approved

---

## 1. Purpose & Regulatory Basis

This document defines the capital adequacy framework for Kovanica Protocol d.o.o. ("the CASP") under **MiCA Articles 55–61**, **RTS on Own Funds (Commission Delegated Regulation 2024/xxxx)**, **EBA Guidelines on Own Funds**, and **Croatian Companies Act**.

| Regulation | Key Requirements |
|------------|------------------|
| MiCA Art. 55 | Initial capital: €50,000 (Class 2 CASP — exchange, transfer, execution) |
| MiCA Art. 56 | Ongoing own funds: higher of €50k, ¼ fixed overheads, or K-factor requirement |
| MiCA Art. 57 | Own funds composition: CET1, AT1, T2 (CASP simplified: CET1 only) |
| MiCA Art. 58 | K-factor requirement (K-AUM, K-CMH, K-ASA, K-COH, K-DTF, K-CON) |
| MiCA Art. 59 | Liquidity requirement: 30% of fixed overheads in highly liquid assets |
| MiCA Art. 60 | Large exposure limits (25% of own funds per counterparty) |
| MiCA Art. 61 | Qualifying holdings notification |

**CASP Classification:** Class 2 (provides exchange, transfer, execution services — NOT portfolio management or custody)

---

## 2. Initial Capital

| Item | Amount | Source | Verification |
|------|--------|--------|--------------|
| **Share Capital** | €50,000 | Founders' cash contribution | Bank certificate, court registry |
| **Share Premium** | €0 | — | — |
| **Reserves** | €0 | — | — |
| **Total CET1 at Authorization** | **€50,000** | | |

**Confirmation:** €50,000 meets MiCA Art. 55 minimum for Class 2 CASP.

---

## 3. Ongoing Own Funds Requirement

The CASP must maintain own funds at all times ≥ the **highest** of:

| Requirement | Calculation | Current Estimate |
|-------------|-------------|------------------|
| **Pillar 1: Fixed Minimum** | €50,000 | €50,000 |
| **Pillar 2: Fixed Overheads** | ¼ × Annual Fixed Overheads | €37,500 (see below) |
| **Pillar 3: K-Factor** | Σ K-factors (see Section 4) | €12,000 (see below) |
| **Applicable Requirement** | **MAX of above** | **€50,000** |

### 3.1 Fixed Overheads Calculation (Annual)

| Cost Category | Annual Amount (€) | Notes |
|---------------|-------------------|-------|
| Personnel (14 FTEs) | 420,000 | Incl. MRTs, key functions |
| Premises & Utilities | 36,000 | Zagreb office, data center |
| Technology & Infrastructure | 120,000 | Cloud, nodes, monitoring, licenses |
| Professional Services (legal, audit, tax) | 80,000 | MiCA, DORA, statutory audit |
| Regulatory Fees (HANFA, FIU) | 15,000 | Authorization, ongoing supervision |
| Insurance (D&O, cyber, PI) | 25,000 | Per `06_Incident_Response_Plan.md` |
| Marketing & Acquisition | 50,000 | Digital, events, partnerships |
| **Total Fixed Overheads** | **746,000** | |
| **¼ Fixed Overheads** | **186,500** | |

> **Note:** Current estimate €186,500 > €50,000. The CASP must maintain ≥ €186,500 own funds once operational at scale. Initial €50k is sufficient for authorization; capital plan includes raise to €200k+ pre-launch.

### 3.2 K-Factor Requirement (MiCA Art. 58)

| K-Factor | Applicable? | Description | Est. Annual Volume | Capital Charge |
|----------|-------------|-------------|-------------------|----------------|
| **K-AUM** | No | Assets under management | N/A (no portfolio mgmt) | €0 |
| **K-CMH** | **Yes** | Client money held (fiat + crypto) | €5M avg daily | €8,000 |
| **K-ASA** | **Yes** | Assets safeguarded (crypto) | €10M avg daily | €3,000 |
| **K-COH** | No | Client orders handled (execution only) | N/A (matched principal) | €0 |
| **K-DTF** | **Yes** | Daily trading flow (exchange) | €2M daily | €1,000 |
| **K-CON** | No | Concentration risk | Diversified counterparties | €0 |
| **Total K-Factor** | | | | **€12,000** |

**K-Factor Formulas (Simplified):**
- K-CMH = 0.16% × avg daily client money
- K-ASA = 0.03% × avg daily assets safeguarded
- K-DTF = 0.05% × avg daily trading flow

---

## 4. Own Funds Composition

### 4.1 Current Structure (Authorization)

| Component | Amount (€) | % of Requirement | Eligibility |
|-----------|------------|------------------|-------------|
| **CET1 Capital** | | | |
| Share Capital | 50,000 | 100% | ✅ Fully eligible |
| Share Premium | 0 | — | ✅ |
| Retained Earnings | 0 | — | ✅ (subject to audit) |
| Other Reserves | 0 | — | ✅ |
| **Deductions** | | | |
| Intangible Assets | (5,000) | -10% | ❌ Deducted |
| Deferred Tax Assets | 0 | — | ❌ |
| **Total CET1** | **45,000** | **90%** | |
| **AT1 Capital** | 0 | — | Not issued |
| **T2 Capital** | 0 | — | Not issued |
| **Total Own Funds** | **45,000** | | |

> **Gap:** €45,000 < €50,000 minimum. **Action:** Founders to inject additional €10,000 before authorization (total €60,000 share capital).

### 4.2 Target Structure (Post-Launch, Month 6)

| Component | Amount (€) | Source |
|-----------|------------|--------|
| Share Capital | 200,000 | Series A raise (€150k new) |
| Share Premium | 300,000 | Series A premium |
| Retained Earnings | 50,000 | Operational profits |
| **Total CET1** | **550,000** | |
| **Requirement (Max)** | **186,500** | ¼ Fixed Overheads |
| **Surplus** | **363,500** | 195% coverage |

---

## 5. Liquidity Requirement (MiCA Art. 59)

| Requirement | Calculation | Amount (€) |
|-------------|-------------|------------|
| **30% of Fixed Overheads** | 0.30 × 746,000 | **223,800** |
| **Eligible Liquid Assets** | | |
| Cash (EUR) | Operational accounts | 150,000 |
| Central Bank Reserves | N/A (non-bank) | 0 |
| Government Bonds (AAA, <1yr) | Treasury bills | 100,000 |
| **Total Liquid Assets** | | **250,000** |
| **Coverage** | 250,000 / 223,800 | **112%** |

**Monitoring:** Daily liquidity dashboard; weekly reporting to Risk Committee.

---

## 6. Large Exposure Limits (MiCA Art. 60)

| Limit | Threshold | Current Exposures |
|-------|-----------|-------------------|
| **Single Counterparty** | 25% of own funds | Copper: €112,500 (25% of €450k target) |
| **Connected Counterparties** | 25% of own funds | Sumsub + AWS: < 10% |
| **Qualifying Holdings** | > 10% of own funds | None currently |

**Monitoring:** Real-time exposure tracking; daily report to CRO.

---

## 7. Capital Planning & Stress Testing

### 7.1 Capital Plan (24 Months)

| Milestone | Date | Share Capital | Total CET1 | Requirement | Coverage |
|-----------|------|---------------|------------|-------------|----------|
| Authorization | M0 | €60,000 | €55,000 | €50,000 | 110% |
| Launch | M3 | €60,000 | €55,000 | €50,000 | 110% |
| Series A Close | M6 | €200,000 | €550,000 | €186,500 | 295% |
| Break-even | M12 | €200,000 | €600,000 | €186,500 | 322% |
| Scale | M24 | €200,000 | €800,000 | €250,000 | 320% |

### 7.2 Stress Scenarios

| Scenario | Impact on Own Funds | Impact on Requirement | Mitigation |
|----------|---------------------|----------------------|------------|
| **Crypto Winter (-80% volumes)** | -€50k retained earnings | -30% K-factors | Cost reduction, liquidity buffer |
| **Major Security Incident** | -€100k (remediation, fines) | +20% overheads | Cyber insurance (€5M), incident reserves |
| **Regulatory Fine (MiCA Art. 94)** | -€200k (max 12.5% revenue) | — | Compliance investment, insurance |
| **Counterparty Default (Copper)** | -€50k (operational) | — | Diversification, ClearLoop netting |
| **Combined Severe** | -€250k | +10% | Capital raise trigger at 120% coverage |

**Recovery Plan:** If coverage < 120% → Board emergency meeting → capital call / cost reduction / asset sale within 30 days.

---

## 8. Portfolio Management Exclusion (€150k Gap)

| Service | MiCA Class | Capital Required | Status |
|---------|------------|------------------|--------|
| Exchange (fiat↔crypto, crypto↔crypto) | Class 2 | €50k base | ✅ Included |
| Transfer | Class 2 | €50k base | ✅ Included |
| Execution | Class 2 | €50k base | ✅ Included |
| **Portfolio Management** | **Class 3** | **€150k base** | ❌ **Excluded** |
| Custody | Class 2* | €50k base* | Partner (Copper) |

*Custody via Copper (outsourced) — CASP does not hold own custody license.

**Decision:** Portfolio management excluded from initial application. If added later:
- Minimum own funds: €150,000
- Additional K-factors: K-AUM (0.02% × AuM)
- Additional requirements: Investment committee, suitability assessment, client categorization

---

## 9. Regulatory Reporting

| Report | Frequency | Deadline | Recipient | Owner |
|--------|-----------|----------|-----------|-------|
| **Own Funds (COREP)** | Quarterly | 15 business days post-quarter | HANFA | CFO |
| **Large Exposures** | Quarterly | 15 business days post-quarter | HANFA | CRO |
| **Liquidity** | Monthly | 10 business days post-month | HANFA | CFO |
| **K-Factor Details** | Quarterly | 15 business days post-quarter | HANFA | CRO |
| **Annual Audited Financials** | Annual | 30 April | HANFA, Court Registry | CFO / Auditor |

---

## 10. Cross-References

| Document | Reference |
|----------|-----------|
| `00_Foundation/00_Business_Plan.md` | Financial projections, capital raise plan |
| `00_Foundation/01_Programme_of_Operations.md` | Cost structure, overheads |
| `02_ICT_Risk_Management.md` | ICT risk capital allocation |
| `00_Custody_Policy.md` | Copper custody, client asset segregation |
| `03_Outsourcing_Register.md` | Counterparty exposures |
| `06_Incident_Response_Plan.md` | Insurance coverage, incident reserves |

---

## 11. Document Control

| Field | Value |
|-------|-------|
| **Document ID** | KOV-POL-GOV-001 |
| **Version** | 1.0 |
| **Classification** | Confidential |
| **Owner** | CFO / CRO |
| **Approved By** | Management Board |
| **Approval Date** | [DATE] |
| **Next Review** | [DATE + 3 months] |
| **Distribution** | Management Board, Risk Committee, HANFA (on request) |

### Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-15 | Compliance Team | Initial version for MiCA CASP application |

---

*End of Document*
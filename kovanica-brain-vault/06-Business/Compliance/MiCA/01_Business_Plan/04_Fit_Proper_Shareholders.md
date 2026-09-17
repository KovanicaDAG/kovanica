# Fit & Proper Assessment — Qualifying Shareholders & UBOs

## 1. Purpose & Regulatory Basis

**Regulation**: MiCA Regulation (EU) 2023/1114, Article 17
**Guidelines**: EBA Guidelines on fit & proper assessments (EBA/GL/2021/06)
**National Law**: Croatian Act on Crypto-Asset Service Providers; AMLD6 (EU) 2018/843
**Supervisor**: HANFA (Hrvatska agencija za nadzor financijskih usluga)

**Scope**: All qualifying holders (≥10% direct/indirect) and Ultimate Beneficial Owners (UBOs ≥25% or control) per `02_Shareholder_Register.md`

## 2. Assessment Criteria (MiCA Art. 17)

| Criterion | Description | Evidence Required |
|-----------|-------------|-------------------|
| **Reputation** | Good repute, honesty, integrity; no criminal convictions, regulatory sanctions, adverse findings | Criminal record certificates, regulatory references, self-declaration, adverse media screen |
| **Financial Soundness** | Sufficient financial resources; no insolvency, over-indebtedness; source of funds verified | Audited financials (3y), bank references, source of wealth/funds declaration, credit reports |
| **No Sanctions/PEP Risk** | Not subject to EU/UN/OFAC sanctions; not a PEP or close associate without enhanced due diligence | World-Check/Dow Jones screen, PEP declaration, sanctions list verification |

## 3. Shareholder/UBO Assessment Template

*Complete one per qualifying holder and UBO identified in `02_Shareholder_Register.md`*

### 3.1 Entity / Individual Identification

| Field | Entry |
|-------|-------|
| **Assessment ID** | SH-[SEQ] / UBO-[SEQ] |
| **Legal Name** | [Full legal name] |
| **Legal Form** | Natural Person / Ltd. / GmbH / BV / Trust / Foundation / Other |
| **Jurisdiction of Incorporation/Residence** | [Country] |
| **Registered Office / Residential Address** | [Full address] |
| **Registration Number / ID Document** | [Commercial register no. / Passport / OIB] |
| **LEI (if legal entity)** | [20-char LEI] |
| **Role** | ☐ Qualifying Holder (≥10%) / ☐ UBO (≥25% or control) / ☐ Both |
| **% Holding (Direct)** | [X.X%] |
| **% Holding (Indirect)** | [X.X%] |
| **% Voting Rights** | [X.X%] |
| **Control Mechanism** | ☐ Shares / ☐ Voting Rights / ☐ Contract / ☐ Other: [Detail] |
| **Related Directors** | [Names from `03_Fit_Proper_Directors.md` if applicable] |

### 3.2 Reputation Assessment

| Check | Result | Evidence / Notes |
|-------|--------|------------------|
| **Criminal Record (Country of Residence)** | ☐ Clear / ☐ Adverse | Certificate < 3 months (natural persons) |
| **Criminal Record (Country of Incorporation)** | ☐ Clear / ☐ Adverse | Certificate < 3 months (entities: directors/UBOs) |
| **Criminal Record (Other Countries, 10y)** | ☐ Clear / ☐ Adverse | Certificates from each country of residence/operation |
| **Regulatory Sanctions (Financial Sector)** | ☐ None / ☐ Yes | HANFA, ECB, FCA, BaFin, AMF, CNMV, CONSOB, etc. |
| **Civil/Administrative Judgments (Financial)** | ☐ None / ☐ Yes | Court registers, credit bureaus |
| **Insolvency/Bankruptcy History (10y)** | ☐ None / ☐ Yes | Commercial court registers |
| **Professional Body Disciplinary** | ☐ None / ☐ Yes | Relevant professional bodies |
| **Adverse Media / Reputation Screen** | ☐ Clear / ☐ Adverse | World-Check, Dow Jones, LexisNexis, open source |
| **Self-Declaration Signed** | ☐ Yes / ☐ No | Template in Appendix A |

**Overall Reputation**: ☐ Fit / ☐ Not Fit — *Rationale: [Text]*

### 3.3 Financial Soundness Assessment

#### 3.3.1 Natural Persons (UBOs)

| Document | Provided? | Date | Key Findings |
|----------|-----------|------|--------------|
| **Source of Wealth Declaration** | ☐ Yes / ☐ No | [Date] | [Summary: business sale, inheritance, investments, etc.] |
| **Source of Funds for This Investment** | ☐ Yes / ☐ No | [Date] | [Bank statements, sale proceeds, loan agreement, etc.] |
| **Personal Financial Statement** | ☐ Yes / ☐ No | [Date] | Assets, liabilities, net worth |
| **Credit Report (Country of Residence)** | ☐ Yes / ☐ No | [Date] | No adverse findings |
| **Tax Residence Certificate** | ☐ Yes / ☐ No | [Date] | Current year |
| **Bank Reference Letter** | ☐ Yes / ☐ No | [Date] | Relationship > 2 years preferred |

#### 3.3.2 Legal Entities (Corporate Shareholders)

| Document | Provided? | Date | Key Findings |
|----------|-----------|------|--------------|
| **Audited Financial Statements (3 Years)** | ☐ Yes / ☐ No | [Dates] | Profitability, equity, cash flow |
| **Interim Financials (if >6 months old)** | ☐ Yes / ☐ No | [Date] | Current position |
| **Credit Rating / Agency Report** | ☐ Yes / ☐ No | [Date] | External rating if available |
| **Bank Reference Letters** | ☐ Yes / ☐ No | [Date] | Relationship > 2 years |
| **Regulatory Capital Adequacy (if regulated)** | ☐ Yes / ☐ No | [Date] | Solvency ratio, own funds |
| **Source of Funds for Subscription** | ☐ Yes / ☐ No | [Date] | Subscription agreement, bank trace |

**Financial Soundness Indicators**:

| Indicator | Value | Threshold | Assessment |
|-----------|-------|-----------|------------|
| **Net Worth / Equity** | [EUR] | > Investment Amount | ☐ Pass / ☐ Fail |
| **Leverage (Debt/Equity)** | [Ratio] | < 5:1 (entities) | ☐ Pass / ☐ Fail |
| **Liquidity (Current Ratio)** | [Ratio] | > 1.0 (entities) | ☐ Pass / ☐ Fail |
| **Profitability (3y Average)** | [EUR] | Positive (entities) | ☐ Pass / ☐ Fail |
| **Source of Funds Verified** | ☐ Yes / ☐ No | Mandatory | ☐ Pass / ☐ Fail |

**Overall Financial Soundness**: ☐ Fit / ☐ Not Fit — *Rationale: [Text]*

### 3.4 Sanctions & PEP Assessment

| Screen | Result | Date | Tool/Provider | Action if Match |
|--------|--------|------|---------------|-----------------|
| **EU Consolidated Sanctions List** | ☐ Clear / ☐ Match | [Date] | [World-Check/Dow Jones/Other] | ☐ Reject / ☐ Enhanced DD |
| **UN Security Council Sanctions** | ☐ Clear / ☐ Match | [Date] | [Provider] | ☐ Reject / ☐ Enhanced DD |
| **OFAC SDN List** | ☐ Clear / ☐ Match | [Date] | [Provider] | ☐ Reject / ☐ Enhanced DD |
| **UK Sanctions List** | ☐ Clear / ☐ Match | [Date] | [Provider] | ☐ Reject / ☐ Enhanced DD |
| **PEP Screen (Direct)** | ☐ Not PEP / ☐ PEP | [Date] | [Provider] | ☐ Standard / ☐ Enhanced DD |
| **PEP Screen (Close Associate)** | ☐ Not CA / ☐ CA | [Date] | [Provider] | ☐ Standard / ☐ Enhanced DD |
| **PEP Screen (Family Member)** | ☐ Not FM / ☐ FM | [Date] | [Provider] | ☐ Standard / ☐ Enhanced DD |
| **Adverse Media (Sanctions/Crime)** | ☐ Clear / ☐ Adverse | [Date] | [Provider] | ☐ Standard / ☐ Enhanced DD |

**Enhanced Due Diligence Required**: ☐ No / ☐ Yes — *If Yes, complete Section 3.5*

**Overall Sanctions/PEP**: ☐ Fit / ☐ Not Fit — *Rationale: [Text]*

### 3.5 Enhanced Due Diligence (If Triggered)

| EDD Measure | Completed? | Date | Findings |
|-------------|------------|------|----------|
| **Senior Management Approval** | ☐ Yes / ☐ No | [Date] | [Director A/B sign-off] |
| **Enhanced Source of Wealth/Funds** | ☐ Yes / ☐ No | [Date] | [Forensic trace, independent verification] |
| **Independent Background Investigation** | ☐ Yes / ☐ No | [Date] | [Third-party report] |
| **Ongoing Monitoring Enhancement** | ☐ Yes / ☐ No | [Date] | [Real-time alerts, quarterly reviews] |
| **Restricted Operations** | ☐ Yes / ☐ No | [Date] | [Limits, approvals, reporting] |

## 4. Consolidated Assessment Outcome

| Assessment ID | Name | Type | Reputation | Financial Soundness | Sanctions/PEP | **Overall** |
|---------------|------|------|------------|---------------------|---------------|-------------|
| SH-001 | [Founder A] | Natural Person / QH+UBO | ☐ Fit / ☐ Not Fit | ☐ Fit / ☐ Not Fit | ☐ Fit / ☐ Not Fit | ☐ **FIT** / ☐ **NOT FIT** |
| SH-002 | [Founder B] | Natural Person / QH+UBO | ☐ Fit / ☐ Not Fit | ☐ Fit / ☐ Not Fit | ☐ Fit / ☐ Not Fit | ☐ **FIT** / ☐ **NOT FIT** |
| SH-003 | [Investor Entity] | Legal Entity / QH | ☐ Fit / ☐ Not Fit | ☐ Fit / ☐ Not Fit | ☐ Fit / ☐ Not Fit | ☐ **FIT** / ☐ **NOT FIT** |
| UBO-001 | [Investor UBO 1] | Natural Person / UBO | ☐ Fit / ☐ Not Fit | ☐ Fit / ☐ Not Fit | ☐ Fit / ☐ Not Fit | ☐ **FIT** / ☐ **NOT FIT** |
| UBO-002 | [Investor UBO 2] | Natural Person / UBO | ☐ Fit / ☐ Not Fit | ☐ Fit / ☐ Not Fit | ☐ Fit / ☐ Not Fit | ☐ **FIT** / ☐ **NOT FIT** |

**Assessment Date**: [Date]
**Assessed By**: [Name, Role — e.g., CCO / AML Officer / External Assessor]
**Approved By**: [Director B (CCO) signature]
**Next Review Date**: [Date + 12 months] or upon trigger

## 5. Ongoing Monitoring

| Trigger | Action | Timeline | Responsible |
|---------|--------|----------|-------------|
| Annual re-assessment | Full re-assessment per this template | 12 months | CCO / AML Officer |
| Sanctions list update (new designation) | Immediate screen & escalate | 24 hours | AML Officer |
| PEP status change | Re-assess EDD requirements | 5 business days | AML Officer |
| Adverse media alert | Screen & escalate | 2 business days | AML Officer |
| Financial deterioration (entity) | Targeted financial review | 10 business days | CCO / CFO |
| Change in ownership/control | Full re-assessment | 10 business days | CCO / AML Officer |
| Regulatory action against holder | Immediate re-assessment | 5 business days | CCO |

## 6. Appendices

### Appendix A: Self-Declaration Template (Reputation & Sanctions)
```
I, [Full Name / Authorised Signatory for Entity], declare that:
1. I/We have not been convicted of any criminal offence related to financial crime, fraud, dishonesty...
2. I/We are not subject to any sanctions (EU, UN, OFAC, UK)....
3. I/We are not a Politically Exposed Person (PEP), close associate, or family member of a PEP...
4. The source of wealth/funds for this investment is: [Description]...
5. All information provided is true, complete, and not misleading.
Signature: _______________ Date: _______________
Capacity: [Individual / Director / Authorised Signatory]
```

### Appendix B: Source of Wealth / Funds Questionnaire
*Detailed questionnaire covering: origin of wealth, business activities, investment history, tax compliance, third-party verification*

### Appendix C: Corporate Structure Declaration (for Legal Entities)
```
We, [Entity Name], declare that:
1. Our full ownership chain to UBOs is disclosed in `02_Shareholder_Register.md` Section 5...
2. No undisclosed shareholders or beneficial owners exist...
3. We are not a shell company or nominee arrangement...
4. Our constitutional documents permit crypto-asset investment...
Signature: _______________ Date: _______________
Authorised Signatory: [Name, Title]
```

## 7. Document Control

| Element | Detail |
|---------|--------|
| **Version** | 1.0 |
| **Classification** | Confidential — Regulatory / AML |
| **Owner** | Director B (CCO) / AML Officer |
| **Approved By** | Management Board |
| **Review Cycle** | Annual or upon trigger (Section 5) |
| **Retention** | 10 years post-relationship (AMLD6) |
| **Related Documents** | `02_Shareholder_Register.md`, `03_Fit_Proper_Directors.md`, `01_AML/00_AML_Policy.md`, `01_AML/03_Sanctions_Policy.md` |

---

**End of Document**

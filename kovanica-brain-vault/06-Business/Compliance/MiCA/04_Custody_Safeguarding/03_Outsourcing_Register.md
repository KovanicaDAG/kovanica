# Outsourcing Register

**Document ID:** KOV-POL-OPS-003
**Version:** 1.0
**Classification:** Confidential
**Owner:** COO / CTO
**Review Cycle:** Quarterly (or upon material change)
**Status:** Approved

---

## 1. Purpose & Regulatory Basis

This register documents all outsourcing arrangements for Kovanica Protocol d.o.o. ("the CASP") in compliance with **MiCA Articles 30–31**, **DORA Articles 28–30**, **EBA Guidelines on Outsourcing (EBA/GL/2019/02)**, and **Croatian Law** transposing DORA.

| Regulation | Key Requirements |
|------------|------------------|
| MiCA Art. 30–31 | Register of outsourced functions; critical/material classification; notification to NCA |
| DORA Art. 28–30 | ICT third-party risk management; register of critical ICT providers; concentration risk |
| EBA/GL/2019/02 | Governance, due diligence, contract terms, monitoring, exit strategies |
| Croatian Law | Zakon o operativnoj otpornosti; HANFA reporting obligations |

**Classification Criteria:**
- **Critical Function:** Failure would materially impair CASP's ability to comply with MiCA, safeguard client assets, or maintain market integrity
- **Material Function:** Significant operational impact but not critical; or regulatory reporting dependency
- **Non-Material:** Standard vendor relationships with limited operational impact

---

## 2. Outsourcing Register Summary

| # | Provider | Service | Classification | Criticality | Contract Ref | Status | Last Review |
|---|----------|---------|----------------|-------------|--------------|--------|-------------|
| 1 | **Copper Technologies (UK) Ltd** | Custody, settlement, ClearLoop | **Critical (ICT + Business)** | Critical | KOV-CNT-001 | Active | 2026-01-15 |
| 2 | **Sumsub Ltd** | KYC/CDD, sanctions/PEP screening, adverse media | **Critical (ICT + Business)** | Critical | KOV-CNT-002 | Active | 2026-01-15 |
| 3 | **Amazon Web Services (AWS)** | Cloud infrastructure (compute, storage, network, KMS) | **Critical (ICT)** | Critical | KOV-CNT-003 | Active | 2026-01-15 |
| 4 | **Hetzner Online GmbH** | Bare metal / co-location (blockchain nodes) | **Critical (ICT)** | Critical | KOV-CNT-004 | Active | 2026-01-15 |
| 5 | **Cloudflare Inc.** | WAF, DDoS protection, CDN, DNS | **Material (ICT)** | High | KOV-CNT-005 | Active | 2026-01-15 |
| 6 | **GitHub Inc. (Microsoft)** | Source control, CI/CD, packages, Actions | **Material (ICT)** | High | KOV-CNT-006 | Active | 2026-01-15 |
| 7 | **Grafana Labs / Datadog** | Observability (metrics, logs, traces, alerting) | **Material (ICT)** | High | KOV-CNT-007 | Active | 2026-01-15 |
| 8 | **Chainalysis / TRM Labs** | Blockchain analytics, transaction monitoring | **Material (Business)** | High | KOV-CNT-008 | Pending | — |
| 9 | **Legal Counsel (External)** | Regulatory advisory, contract review, litigation | **Non-Material** | Medium | KOV-CNT-009 | Active | 2026-01-15 |
| 10 | **Audit Firm (Big 4)** | Financial audit, SOC 2 attestation, MiCA readiness | **Non-Material** | Medium | KOV-CNT-010 | Active | 2026-01-15 |
| 11 | **Insurance Broker** | Professional indemnity, cyber, D&O insurance | **Non-Material** | Low | KOV-CNT-011 | Active | 2026-01-15 |
| 12 | **HR / Payroll Provider** | Payroll, benefits, compliance | **Non-Material** | Low | KOV-CNT-012 | Active | 2026-01-15 |

---

## 3. Critical Function Details

### 3.1 Copper Technologies (UK) Ltd — Custody & Settlement

| Field | Detail |
|-------|--------|
| **Legal Entity** | Copper Technologies (UK) Ltd, 10 Queen Street Place, London EC4R 1BE, UK |
| **Regulatory Status** | FCA registered cryptoasset business (FRN: 928761); EEA passport via Croatia |
| **Service Description** | Institutional custody (MPC wallets), trade settlement (ClearLoop), staking, lending, reporting |
| **Criticality Rationale** | Sole custodian of client assets; settlement layer for all trades; failure = inability to safeguard assets or settle |
| **Contract Reference** | KOV-CNT-001 (MSA + DPA + SLA Addendum) |
| **Contract Term** | 3 years (2026-01-15 to 2029-01-14); auto-renew 1 year |
| **Key Contractual Terms** | |
| — SLA | 99.9% API uptime; < 500ms p95 latency; 24/7 support for P1 |
| — Liability | Cap: 12 months fees; Unlimited for: data breach, custody loss, regulatory fines |
| — Audit Rights | Annual SOC 2 Type II; on-site audit right; penetration test sharing |
| — Data Location | Primary: EU (Frankfurt); Backup: EU (Amsterdam); No US processing without consent |
| — Sub-processing | Prior written consent; sub-processor list maintained; 30-day notice for changes |
| — Termination | 90 days notice; 30 days for material breach; 180-day transition assistance |
| — Exit Strategy | Asset return in standard format (CSV/JSON); private key export (MPC shares); ClearLoop unwind |
| **Monitoring KPIs** | API uptime, latency, error rate, settlement success rate, ClearLoop cycle time, security incidents |
| **Concentration Risk** | Single custodian; evaluating secondary custodian for 2026 H2 |
| **Business Continuity** | Copper DR: multi-region (EU + UK); Kovanica fallback: manual signing via HSM |
| **Last Due Diligence** | 2026-01-15 (see `01_Custody_Partner_Due_Diligence.md`) |
| **Next Review** | 2026-04-15 (quarterly) |

### 3.2 Sumsub Ltd — KYC/CDD & Screening

| Field | Detail |
|-------|--------|
| **Legal Entity** | Sumsub Ltd, 3rd Floor, 86-90 Paul Street, London EC2A 4NE, UK |
| **Regulatory Status** | UK GDPR representative; EU processor via SCCs |
| **Service Description** | Identity verification (document + liveness), address proof, PEP/sanctions/adverse media screening, ongoing monitoring |
| **Criticality Rationale** | Regulatory requirement for CDD; onboarding gate; AML compliance dependency |
| **Contract Reference** | KOV-CNT-002 (MSA + DPA + SLA Addendum) |
| **Contract Term** | 2 years (2026-01-15 to 2028-01-14); auto-renew 1 year |
| **Key Contractual Terms** | |
| — SLA | 99.9% uptime; < 2s p95 verification; 24/7 support for P1 |
| — Liability | Cap: 12 months fees; Unlimited for: data breach, GDPR fines |
| — Audit Rights | Annual SOC 2 Type II; DPA audit clause; security questionnaire |
| — Data Location | Processing: EU (Frankfurt); Sub-processors: US (SCCs + supplementary measures) |
| — Sub-processing | Prior written consent; list: AWS, Google Cloud, specialized OCR vendors |
| — Termination | 60 days notice; 30 days for material breach; 90-day transition |
| — Exit Strategy | Applicant data export (JSON); webhook history; screening audit trail |
| **Monitoring KPIs** | Completion rate, processing time, false positive rate, webhook success, SDK error rate |
| **Concentration Risk** | Single KYC provider; manual fallback documented in `04_Sumsub_Integration_Policy.md` |
| **Business Continuity** | Sumsub multi-region (EU); Kovanica fallback: manual document review queue |
| **Last Due Diligence** | 2026-01-15 |
| **Next Review** | 2026-04-15 (quarterly) |

### 3.3 Amazon Web Services (AWS) — Cloud Infrastructure

| Field | Detail |
|-------|--------|
| **Legal Entity** | Amazon Web Services EMEA SARL, 38 Avenue John F. Kennedy, L-1855 Luxembourg |
| **Regulatory Status** | SOC 2 Type II, ISO 27001, ISO 27017, ISO 27018, C5, ENS High |
| **Service Description** | EC2, RDS, EKS, S3, KMS, CloudFront, Route53, WAF, Shield, IAM, CloudTrail, Config |
| **Criticality Rationale** | Hosts all critical systems: trading engine, custody integration, KYC, databases, monitoring |
| **Contract Reference** | KOV-CNT-003 (Enterprise Agreement + DPA) |
| **Contract Term** | 3 years (2026-01-15 to 2029-01-14) |
| **Key Contractual Terms** | |
| — SLA | Per service (EC2 99.99%, RDS 99.95%, S3 99.99%); Enterprise Support 15-min P1 |
| — Liability | Standard AWS terms; service credits for SLA breach |
| — Audit Rights | SOC reports via AWS Artifact; penetration testing permitted with notice |
| — Data Location | eu-central-1 (Frankfurt) primary; eu-north-1 (Stockholm) DR; GDPR-compliant |
| — Sub-processing | AWS sub-processor list; notification of changes |
| — Termination | Per EA; data export via standard APIs; 30-day retention post-termination |
| — Exit Strategy | Infrastructure as Code (Terraform) portable; multi-cloud DR evaluated (GCP) |
| **Monitoring KPIs** | Service health, cost, security findings, capacity utilization |
| **Concentration Risk** | Primary cloud; evaluating GCP/Azure for DR (see `02_ICT_Risk_Management.md`) |
| **Business Continuity** | Multi-AZ deployment; cross-region DR; automated failover for critical services |
| **Last Due Diligence** | 2026-01-15 |
| **Next Review** | 2026-04-15 (quarterly) |

### 3.4 Hetzner Online GmbH — Bare Metal / Co-location

| Field | Detail |
|-------|--------|
| **Legal Entity** | Hetzner Online GmbH, Industriestr. 25, 91710 Gunzenhausen, Germany |
| **Regulatory Status** | ISO 27001, DIN EN 50600 (datacenter availability) |
| **Service Description** | Dedicated servers (blockchain nodes), co-location racks, network, DDoS protection |
| **Criticality Rationale** | Hosts Kovanica validator nodes; consensus participation; block production |
| **Contract Reference** | KOV-CNT-004 (Dedicated Server Agreement + DPA) |
| **Contract Term** | Monthly rolling; 30-day notice |
| **Key Contractual Terms** | |
| — SLA | 99.9% network; hardware replacement 4h business hours |
| — Liability | Standard terms; hardware failure replacement |
| — Audit Rights | Datacenter tour (annual); security questionnaire |
| — Data Location | Nuremberg (DE) / Falkenstein (DE) / Helsinki (FI) — EU only |
| — Sub-processing | None for dedicated servers |
| — Termination | 30 days; hardware return; data wiping certification |
| — Exit Strategy | Node migration to AWS/GCP/Hetzner FI; automated via Ansible |
| **Monitoring KPIs** | Node uptime, peer count, block production rate, disk health, network latency |
| **Concentration Risk** | Single bare-metal provider; multi-datacenter (DE + FI); evaluating additional provider |
| **Business Continuity** | 3+ nodes across 2 datacenters; automated peer discovery; snapshot backup |
| **Last Due Diligence** | 2026-01-15 |
| **Next Review** | 2026-04-15 (quarterly) |

---

## 4. Material Function Details

### 4.1 Cloudflare Inc. — WAF, DDoS, CDN, DNS

| Field | Detail |
|-------|--------|
| **Classification** | Material (ICT) |
| **Criticality** | High |
| **Service** | WAF (OWASP Top 10 + custom rules), DDoS (L3/4/7), CDN, DNS, Bot Management, Rate Limiting |
| **Contract** | KOV-CNT-005 (Enterprise Plan) |
| **SLA** | 100% uptime SLA (Enterprise); < 50ms p99 DNS; DDoS mitigation < 10s |
| **Data Location** | Global Anycast; EU edge for EU traffic |
| **Exit Strategy** | DNS migration (TTL 60s); WAF rules export; Terraform-managed |
| **Monitoring** | Attack volume, WAF block rate, cache hit ratio, DNS resolution time |
| **Last Review** | 2026-01-15 |

### 4.2 GitHub Inc. — Source Control & CI/CD

| Field | Detail |
|-------|--------|
| **Classification** | Material (ICT) |
| **Criticality** | High |
| **Service** | GitHub Enterprise Cloud (repos, Actions, Packages, Advanced Security, Dependabot) |
| **Contract** | KOV-CNT-006 (Enterprise Cloud Agreement) |
| **SLA** | 99.9% uptime; 4h P1 support |
| **Data Location** | US (GitHub); code mirrored to self-hosted Gitea (EU) for DR |
| **Exit Strategy** | Git mirror to Gitea; Actions → GitLab CI / self-hosted runners; Packages → Nexus/Artifactory |
| **Monitoring** | Pipeline success rate, build time, security alerts, dependency freshness |
| **Last Review** | 2026-01-15 |

### 4.3 Grafana Labs / Datadog — Observability

| Field | Detail |
|-------|--------|
| **Classification** | Material (ICT) |
| **Criticality** | High |
| **Service** | Metrics (Prometheus/Grafana Cloud), Logs (Loki), Traces (Tempo), Alerting (Alertmanager), Dashboards |
| **Contract** | KOV-CNT-007 (Grafana Cloud Pro / Datadog Enterprise) |
| **SLA** | 99.9% ingestion; 99.9% query; 15-min P1 support |
| **Data Location** | EU (Frankfurt) for Grafana Cloud; EU for Datadog |
| **Exit Strategy** | Prometheus remote write to self-hosted; Loki/ Tempo self-hosted; dashboards as JSON |
| **Monitoring** | Ingestion latency, query performance, alert firing rate, data completeness |
| **Last Review** | 2026-01-15 |

### 4.4 Chainalysis / TRM Labs — Blockchain Analytics

| Field | Detail |
|-------|--------|
| **Classification** | Material (Business) |
| **Criticality** | High |
| **Service** | Transaction monitoring, risk scoring, sanctions screening, investigation tooling |
| **Contract** | KOV-CNT-008 (Pending — evaluation phase) |
| **SLA** | API 99.9%; risk score latency < 500ms |
| **Data Location** | EU processing (Chainalysis EU entity) |
| **Exit Strategy** | Risk model export; case data export; API migration to alternative |
| **Monitoring** | API latency, risk score distribution, alert volume, false positive rate |
| **Last Review** | Pending selection (Q1 2026) |

---

## 5. Non-Material Function Details

| Provider | Service | Contract | Review Cycle |
|----------|---------|----------|--------------|
| External Legal Counsel | Regulatory advisory, contracts, litigation | KOV-CNT-009 | Annual |
| Big 4 Audit Firm | Financial audit, SOC 2, MiCA readiness | KOV-CNT-010 | Annual (audit cycle) |
| Insurance Broker | PI, cyber, D&O policies | KOV-CNT-011 | Annual (renewal) |
| HR/Payroll Provider | Payroll, benefits, HR compliance | KOV-CNT-012 | Annual |

---

## 6. Governance & Lifecycle Management

### 6.1 Onboarding Process (Critical/Material)

```text
1. Business Case → 2. RFP / Vendor Selection → 3. Due Diligence → 4. Contract Negotiation
      ↓                    ↓                      ↓                    ↓
   Owner: COO         Owner: Procurement    Owner: CISO/Legal    Owner: Legal
   Approval: Board    Criteria: Security,   Scope: Questionnaire,  Terms: Liability,
   Criteria:          Financial,            SOC 2, DPA,          Audit, Exit,
   Strategic fit,     Concentration risk    Pen test, Ref checks  Sub-processing,
   Cost-benefit       assessment            Financial health      Data location
```

### 6.2 Ongoing Monitoring (Critical)

| Activity | Frequency | Owner | Tool |
|----------|-----------|-------|------|
| **SLA Dashboard Review** | Weekly | Service Owner | Grafana / Vendor Portal |
| **Security Rating** | Continuous | CISO | SecurityScorecard / BitSight |
| **Breach Monitoring** | Continuous | CISO | HaveIBeenPwned, vendor notifications |
| **Financial Health** | Quarterly | COO | Credit reports, public filings |
| **Concentration Risk** | Quarterly | CRO | Register analysis |
| **Contract Compliance** | Semi-annual | Legal | Contract management system |

### 6.3 Annual Reassessment (Critical)

| Component | Method |
|-----------|--------|
| Updated Security Questionnaire | CAIQ / SIG Lite + custom crypto questions |
| SOC 2 Type II Review | Report review + bridge letter |
| Penetration Test Results | Shared summary + remediation tracking |
| Financial Health | Audited accounts, credit rating |
| Service Performance | SLA compliance, incident history, client feedback |
| Regulatory Changes | Impact assessment (MiCA, DORA, AML) |
| Concentration Risk | Single-provider dependency analysis |

### 6.4 Offboarding / Exit Management

| Phase | Activities | Timeline |
|-------|------------|----------|
| **Decision** | Board approval; transition plan initiation | T+0 |
| **Notification** | Formal notice per contract; regulator notification (HANFA) if critical | T+7d |
| **Transition** | Parallel run; data migration; knowledge transfer; access handover | Per contract (90–180d) |
| **Validation** | Service cutover testing; data integrity verification; client communication | T+Transition |
| **Closure** | Final invoice; data deletion certification; access revocation; lessons learned | T+30d post-cutover |

---

## 7. Concentration Risk Analysis

| Provider | Critical Functions | % of Capacity | Mitigation |
|----------|-------------------|---------------|------------|
| **Copper** | Custody, Settlement | 100% | Evaluating secondary custodian (Fireblocks, BitGo, Coinbase Custody) for 2026 H2 |
| **AWS** | Compute, Storage, KMS, Network | ~80% | Multi-region (eu-central-1 + eu-north-1); GCP DR evaluation; Terraform portability |
| **Sumsub** | KYC/CDD, Screening | 100% | Manual fallback queue; evaluating secondary provider (Onfido, Veriff) |
| **Hetzner** | Blockchain Nodes | 100% | Multi-datacenter (DE + FI); evaluating additional provider (Equinix, Digital Realty) |
| **GitHub** | Source Control, CI/CD | 100% | Self-hosted Gitea mirror; GitLab CI evaluation; runner portability |

**Board-Approved Limits:**
- No single provider > 50% of any critical function capacity without Board exception
- Critical function dual-sourcing target: 2026 H2 for custody; 2027 for cloud

---

## 8. Regulatory Notifications (MiCA Art. 31 / DORA Art. 28)

| Trigger | Authority | Timeline | Content |
|---------|-----------|----------|---------|
| **New Critical Outsourcing** | HANFA | Before commencement | Provider, service, criticality, risk assessment, contract summary |
| **Material Change** | HANFA | 30 days before | Nature of change, impact assessment, updated risk assessment |
| **Termination of Critical** | HANFA | Immediately | Reason, transition plan, client impact, asset safeguarding |
| **Annual Register Submission** | HANFA | Annual (with MiCA reporting) | Full register with classifications |
| **ICT Concentration Risk** | HANFA / ECB | Quarterly (DORA) | Provider exposure, mitigation status |

---

## 9. Cross-References

| Document | Reference |
|----------|-----------|
| `00_Custody_Policy.md` | Copper custody arrangement, asset segregation |
| `01_Custody_Partner_Due_Diligence.md` | Copper detailed assessment |
| `02_ICT_Risk_Management.md` | ICT third-party risk framework, provider list |
| `04_Sumsub_Integration_Policy.md` | Sumsub integration, fallback, monitoring |
| `05_Business_Continuity_Plan.md` | Provider failure scenarios, DR procedures |
| `06_Incident_Response_Plan.md` | Third-party incident escalation |
| `01_Programme_of_Operations.md` | Key functions, operational risk management |

---

## 10. Document Control

| Field | Value |
|-------|-------|
| **Document ID** | KOV-POL-OPS-003 |
| **Version** | 1.0 |
| **Classification** | Confidential |
| **Owner** | COO / CTO |
| **Approved By** | Management Board |
| **Approval Date** | [DATE] |
| **Next Review** | [DATE + 3 months] |
| **Distribution** | Management Board, COO, CTO, CISO, Compliance, Legal, Procurement |

### Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-15 | Compliance Team | Initial version for MiCA CASP application |

---

## Appendix A: Outsourcing Register Template (Per Arrangement)

| Field | Description |
|-------|-------------|
| **Register ID** | KOV-OUT-XXX |
| **Provider Legal Name** | Full legal entity name |
| **Provider Address** | Registered office |
| **Service Description** | Detailed scope |
| **Classification** | Critical / Material / Non-Material |
| **Criticality (ICT/Business)** | Critical / High / Medium / Low |
| **Contract Reference** | KOV-CNT-XXX |
| **Contract Start/End** | Dates |
| **Key Contractual Terms** | SLA, liability, audit, data location, sub-processing, termination, exit |
| **Regulatory Status** | Licenses, registrations, passporting |
| **Criticality Rationale** | Why critical/material |
| **Monitoring KPIs** | Specific metrics with targets |
| **Concentration Risk** | % dependency, alternatives |
| **Business Continuity** | DR strategy, fallback |
| **Last Due Diligence** | Date, scope, findings |
| **Next Review Date** | Scheduled review |
| **HANFA Notification** | Date, reference, status |
| **Owner** | Internal service owner |
| **Status** | Active / Pending / Terminating / Terminated |

---

*End of Document*
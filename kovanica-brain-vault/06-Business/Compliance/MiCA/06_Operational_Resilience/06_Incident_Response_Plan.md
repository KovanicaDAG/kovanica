# Incident Response Plan

**Document ID:** KOV-POL-OPS-006
**Version:** 1.0
**Classification:** Confidential
**Owner:** CISO / CTO
**Review Cycle:** Semi-annual (or post-incident)
**Status:** Approved

---

## 1. Purpose & Regulatory Basis

This plan defines the incident response framework for Kovanica Protocol d.o.o. ("the CASP") covering **cybersecurity incidents**, **operational disruptions**, **data breaches**, and **regulatory incidents** in compliance with:

| Regulation | Key Requirements |
|------------|------------------|
| **DORA** (EU) 2022/2554 | Articles 17–23 (ICT incident management, reporting, testing) |
| **MiCA** | Articles 30–31 (outsourcing incidents), 66–67 (Travel Rule incidents) |
| **GDPR** | Articles 33–34 (personal data breach notification) |
| **Croatian Law** | Zakon o operativnoj otpornosti; Zakon o zaštiti osobnih podataka; HANFA reporting |
| **NIS2 Directive** | (If applicable) Incident reporting to CSIRT |
| **ISO 27035** | Information security incident management standard |

**Scope:** All CASP systems, data, personnel, and third-party services (critical/material per `03_Outsourcing_Register.md`).

---

## 2. Incident Classification

### 2.1 Severity Matrix

| Severity | Definition | Examples | Response Time | Escalation |
|----------|------------|----------|---------------|------------|
| **P1 — Critical** | Immediate threat to: client assets, regulatory compliance, market integrity, or life safety | Custody compromise; unauthorized withdrawal; ransomware on prod; sanctions violation; data breach > 1000 records | **< 15 min** | CISO → CTO → CEO → Board → HANFA (1h) |
| **P2 — High** | Significant degradation of critical services; potential regulatory impact | Trading engine down > 15 min; KYC provider down > 1h; blockchain node sync failure; PII exposure < 1000 records | **< 1 hour** | CISO → CTO → CEO → HANFA (4h if regulatory) |
| **P3 — Medium** | Partial service degradation; no immediate regulatory impact | Non-critical API latency; single node offline; internal tool failure; phishing attempt (no compromise) | **< 4 hours** | Service Owner → CISO |
| **P4 — Low** | Minor issue; no service impact | Cosmetic UI bug; log noise; scheduled maintenance overrun; vulnerability scan finding (non-exploitable) | **< 24 hours** | Service Owner |

### 2.2 Incident Categories

| Category | Code | Description | Typical Severity |
|----------|------|-------------|------------------|
| **Cyber Attack** | CYB | Malware, ransomware, DDoS, APT, supply chain compromise | P1–P2 |
| **Data Breach** | DAT | Unauthorized access/disclosure of personal/confidential data | P1–P2 |
| **Custody/Asset Loss** | CST | Unauthorized transfer, key compromise, settlement failure | P1 |
| **Regulatory Violation** | REG | Sanctions breach, Travel Rule failure, licensing issue | P1–P2 |
| **Service Outage** | OUT | Critical system unavailable (trading, KYC, custody, nodes) | P1–P3 |
| **Third-Party Incident** | TPI | Critical vendor breach, outage, or contract breach | P1–P3 |
| **Insider Threat** | INS | Malicious/negligent employee/contractor action | P1–P2 |
| **Blockchain/Protocol** | BCP | Chain halt, reorg > finality, consensus bug, smart contract exploit | P1–P2 |
| **Physical/Environmental** | PHY | Datacenter fire, power loss, hardware destruction | P1–P3 |

---

## 3. Roles & Responsibilities

### 3.1 Incident Response Team (IRT)

| Role | Primary | Backup | Responsibilities |
|------|---------|--------|------------------|
| **Incident Commander (IC)** | CISO | CTO | Overall command; severity declaration; stakeholder communication; Board/HANFA notification |
| **Technical Lead (TL)** | Lead Engineer (relevant domain) | Senior Engineer | Technical investigation; containment; eradication; recovery |
| **Communications Lead (CL)** | Head of Compliance / MLRO | Legal Counsel | Internal comms; client notification; regulator liaison; media (if needed) |
| **Legal/Regulatory Advisor** | External Counsel | Internal Legal | Legal obligations; privilege; regulatory filing; law enforcement coordination |
| **Business Continuity Lead** | COO | Head of Operations | BCP activation; service continuity; vendor coordination |
| **Scribe** | Rotating (Compliance/Engineering) | — | Timeline documentation; evidence preservation; action tracking |

### 3.2 Extended Stakeholders

| Stakeholder | Trigger | Contact Method |
|-------------|---------|----------------|
| **Management Board** | P1, P2 (regulatory) | Phone + encrypted email |
| **HANFA** | P1 (regulatory), P2 (regulatory), GDPR breach | Dedicated portal / email / phone per HANFA guidelines |
| **Croatian DPA (AZOP)** | GDPR personal data breach | Online form + phone (72h) |
| **CSIRT HR** | NIS2 / critical infrastructure | National CSIRT portal |
| **Copper** | Custody/settlement incident | Dedicated Slack + phone (24/7) |
| **Sumsub** | KYC/screening incident | Support portal + emergency contact |
| **AWS/Hetzner** | Infrastructure incident | Enterprise support ticket + phone |
| **Law Enforcement** | Criminal activity (theft, fraud, ransomware) | Police / USKOK / Europol (via Legal) |
| **Clients** | Service impact / data breach affecting them | In-app notification + email + website banner |

---

## 4. Incident Response Lifecycle

### 4.1 Phase 1: Detection & Reporting (T+0 to T+15 min)

```text
┌─────────────────┐
│   Detection     │  Sources: Monitoring alerts, vendor notification, client report,
│   (Automated/   │  internal report, threat intel, law enforcement, audit finding
│    Manual)      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Initial Triage │  1. Assign Incident ID: INC-YYYYMMDD-NNN
│  (First Responder)  2. Classify severity (P1–P4) & category
└────────┬────────┘  3. Notify IC immediately (P1/P2: phone + Slack)
         │
         ▼
┌─────────────────┐
│  IC Assessment  │  1. Confirm/upgrade severity
│  (T+15 min)     │  2. Activate IRT (P1/P2: full team; P3: TL + CL)
└────────┬────────┘  3. Establish war room (Slack channel + video bridge)
         │
         ▼
```

**Detection Sources:**
- **Automated:** Grafana alerts, AWS GuardDuty, Cloudflare WAF, CrowdStrike/Falcon, Sumsub webhooks, blockchain monitoring (Tenderly/Alchemy), node health checks
- **Manual:** Client support tickets, employee reports (security@kovanica.protocol), vendor notifications, threat intel feeds (MISP, AlienVault), law enforcement

**Reporting Channels:**
- **Internal:** `#incident-response` Slack channel; `security@kovanica.protocol`; PagerDuty for P1/P2
- **External (vendors):** Per vendor SLA (Copper: dedicated Slack; AWS: Enterprise Support; Sumsub: emergency email)

### 4.2 Phase 2: Containment (T+15 min to T+4h)

| Action | Owner | P1 Timeline | P2 Timeline |
|--------|-------|-------------|-------------|
| **Network Isolation** | TL + Infra | < 30 min | < 2h |
| **Credential Rotation** | TL + DevOps | < 1h | < 4h |
| **Service Degradation** | TL + Service Owner | Graceful shutdown if needed | Controlled failover |
| **Evidence Preservation** | TL + Scribe | Immediate (forensic images, logs) | Immediate |
| **Third-Party Notification** | CL | < 1h (critical vendors) | < 4h |
| **Regulatory Pre-Notification** | IC + CL | < 1h (HANFA) | < 4h (if required) |

**Containment Strategies by Category:**

| Category | Immediate Containment |
|----------|----------------------|
| **CYB (Malware/Ransomware)** | Network segment isolation; EDR quarantine; backup verification |
| **DAT (Data Breach)** | Access revocation; database read-only; encryption key rotation |
| **CST (Custody Loss)** | Emergency withdrawal halt; Copper emergency contact; blockchain monitoring |
| **REG (Sanctions/Travel Rule)** | Transaction blocking; counterparty notification; SAR filing |
| **OUT (Service Outage)** | Failover to DR; traffic routing; capacity scaling |
| **TPI (Vendor Incident)** | Activate fallback (manual KYC, secondary custodian, multi-cloud DR) |
| **INS (Insider)** | Access revocation; device seizure; forensic preservation |
| **BCP (Blockchain)** | Node pause; peer verification; consensus monitoring; exchange notification |
| **PHY (Physical)** | DR site activation; hardware replacement; data center coordination |

### 4.3 Phase 3: Eradication (T+4h to T+24h)

| Action | Owner | Verification |
|--------|-------|--------------|
| **Root Cause Analysis** | TL + IC | 5 Whys / Fishbone diagram documented |
| **Malware Removal** | TL + Security | Clean scans; IOC sharing |
| **Vulnerability Patching** | TL + DevOps | Patch deployed; regression test passed |
| **Access Remediation** | TL + IAM | Least privilege restored; MFA enforced |
| **Data Integrity Check** | TL + DBA | Checksums; blockchain state verification |
| **Configuration Hardening** | TL + Infra | CIS benchmarks; drift detection |

### 4.4 Phase 4: Recovery (T+24h to T+7d)

| Action | Owner | Acceptance Criteria |
|--------|-------|---------------------|
| **Service Restoration** | Service Owner | All health checks green; SLA metrics met |
| **Data Restoration** | DBA + TL | RPO/RTO per `05_Business_Continuity_Plan.md` met; integrity verified |
| **Client Communication** | CL | All affected clients notified; status page updated |
| **Regulatory Filing** | IC + CL | HANFA/DPA/CSIRT reports submitted per timelines |
| **Monitoring Enhancement** | TL + CISO | New detection rules deployed; alert tuning |
| **Post-Incident Review** | IC | Scheduled within 7 days |

### 4.5 Phase 5: Post-Incident Activity (T+7d to T+30d)

| Activity | Owner | Output |
|----------|-------|--------|
| **Post-Incident Review (PIR)** | IC + All | PIR report: timeline, root cause, gaps, action items |
| **Action Item Tracking** | IC | Jira epics with owners/dates; Board review |
| **Policy/Procedure Updates** | Compliance | Updated IRP, runbooks, playbooks |
| **Training/Drills** | CISO | Tabletop exercise; red team scenario |
| **Regulatory Follow-up** | CL | HANFA/DPA supplementary filings; closure confirmation |
| **Insurance Claim** | COO + Legal | Cyber insurance notification; evidence package |
| **Lessons Learned Publication** | IC | Internal memo; industry sharing (anonymized) |

---

## 5. Regulatory Reporting Timelines

| Regulation | Incident Type | Timeline | Authority | Channel |
|------------|---------------|----------|-----------|---------|
| **DORA Art. 19** | Major ICT incident | **1 hour** (early warning) | HANFA | Dedicated portal / email |
| | | **24 hours** (intermediate) | HANFA | Portal |
| | | **72 hours** (final) | HANFA | Portal |
| **GDPR Art. 33** | Personal data breach | **72 hours** | AZOP (DPA) | Online form + email |
| **GDPR Art. 34** | High-risk breach to data subjects | **Without undue delay** | Affected individuals | Email / in-app / post |
| **MiCA Art. 31** | Critical outsourcing incident | **Immediately** | HANFA | Portal / email |
| **TFR / AML** | Sanctions/Travel Rule breach | **Immediately** | HANFA / FIU | goAML / portal |
| **NIS2** (if applicable) | Significant cyber incident | **24 hours** | CSIRT HR | National portal |
| **Croatian AML Act** | Suspicious transaction | **Immediately** (TF) / **3 days** (ML) | FIU (Ured) | goAML |

### 5.1 Report Content Requirements

**DORA Major Incident Report (HANFA):**
- Description of incident, root cause (if known)
- Impact assessment (services, clients, market integrity)
- Containment measures taken
- Recovery status & timeline
- Cross-border impact (if any)
- Communication with other NCAs

**GDPR Breach Notification (AZOP):**
- Nature of breach (categories, records count)
- DPO contact details
- Likely consequences
- Measures taken/proposed
- DPIA reference (if applicable)

---

## 6. Communication Templates

### 6.1 Internal Incident Alert (Slack/Email)

```
🚨 INCIDENT ALERT — INC-20260115-001
Severity: P1 — Critical
Category: CST (Custody/Asset Loss)
Detected: 2026-01-15T10:30:00Z
Source: Copper webhook — unauthorized withdrawal detected
Impact: Client assets potentially compromised; trading halted
IRT Activated: IC=CISO, TL=Lead Backend, CL=MLRO
War Room: #inc-INC-20260115-001 (Slack) + meet.kovanica.protocol/inc-001
Next Update: T+30 min
```

### 6.2 HANFA Early Warning (1h)

```
Subject: DORA Art. 19 — Major ICT Incident Early Warning — Kovanica Protocol d.o.o.
Reference: INC-20260115-001
Date/Time: 2026-01-15T10:30:00Z
Severity: Major
Category: Custody/Asset Loss
Description: Unauthorized withdrawal detected via Copper custody integration. Trading halted. Investigation ongoing.
Impact: Potential client asset loss (scope TBD). All withdrawals suspended.
Actions: IRT activated; Copper emergency protocol engaged; blockchain monitoring enhanced.
Next Update: 2026-01-15T12:30:00Z (intermediate report)
Contact: CISO (ic@kovanica.protocol, +385-XX-XXX-XXX)
```

### 6.3 Client Notification (Data Breach)

```
Subject: Important Security Notice — Kovanica Protocol
Date: 2026-01-15

Dear [Client Name],

We are writing to inform you of a security incident that may have affected your personal data...

[Per GDPR Art. 34: nature of breach, likely consequences, measures taken, DPO contact, steps to protect yourself]
```

---

## 7. Playbooks (Reference)

| Playbook | Location | Category |
|----------|----------|----------|
| **Ransomware Response** | `playbooks/ransomware.md` | CYB |
| **Custody Compromise** | `playbooks/custody_compromise.md` | CST |
| **Data Breach (GDPR)** | `playbooks/data_breach_gdpr.md` | DAT |
| **Sanctions Violation** | `playbooks/sanctions_violation.md` | REG |
| **Trading Engine Outage** | `playbooks/trading_outage.md` | OUT |
| **KYC Provider Failure** | `playbooks/kyc_failure.md` | TPI |
| **Blockchain Reorg/Chain Halt** | `playbooks/blockchain_reorg.md` | BCP |
| **Insider Threat** | `playbooks/insider_threat.md` | INS |
| **DDoS Attack** | `playbooks/ddos.md` | CYB |
| **Supply Chain Compromise** | `playbooks/supply_chain.md` | CYB/TPI |

*Each playbook contains: trigger conditions, step-by-step runbook, decision trees, contact lists, evidence checklist, communication templates.*

---

## 8. Evidence Handling & Forensics

| Principle | Implementation |
|-----------|----------------|
| **Chain of Custody** | Evidence log (hash, collector, timestamp, storage location); tamper-evident bags for physical |
| **Preservation** | Forensic images (dd/FTK Imager); write-blockers; cloud snapshot (AWS/GCP) |
| **Integrity** | SHA-256 hashes recorded at collection; verified at each handoff |
| **Access Control** | Encrypted storage (VeraCrypt/LUKS); access limited to IC, TL, Legal |
| **Retention** | 7 years minimum (regulatory); legal hold overrides |
| **Law Enforcement Handoff** | Formal request via Legal; documented transfer; receipt obtained |

---

## 9. Testing & Exercising

| Exercise Type | Frequency | Participants | Scenario Examples |
|---------------|-----------|--------------|-------------------|
| **Tabletop** | Quarterly | Full IRT + Board observers | Ransomware; custody breach; GDPR breach; sanctions hit |
| **Technical Drill** | Semi-annual | Engineering + Security | Node failover; DR cutover; credential rotation; backup restore |
| **Red Team** | Annual | External + Internal | APT simulation; supply chain; insider; blockchain attack |
| **Regulatory Simulation** | Annual | IRT + Legal + Compliance | HANFA inspection; DPA audit; FIU inquiry |
| **Vendor Joint Exercise** | Annual | Copper, Sumsub, AWS | Coordinated failover; incident notification; data export |

**Metrics Tracked:**
- Mean Time to Detect (MTTD) — Target: < 15 min (P1)
- Mean Time to Respond (MTTR) — Target: < 1h (P1 containment)
- Mean Time to Recover (MTTRc) — Target: < 4h (P1 recovery)
- Regulatory reporting compliance — Target: 100% on-time

---

## 10. Third-Party Incident Coordination

| Vendor | Escalation Path | Joint Response | Contractual SLA |
|--------|-----------------|----------------|-----------------|
| **Copper** | Dedicated Slack → Emergency phone → CISO | Shared war room; coordinated containment; ClearLoop pause | 15-min P1 response; 1h containment |
| **Sumsub** | Emergency email → Support portal → Account manager | Shared investigation; screening re-run; applicant re-verification | 30-min P1 response; 4h resolution |
| **AWS** | Enterprise Support (15-min) → TAM → VP escalation | AWS Incident Manager; shared diagnostics; forensic support | 15-min P1; infrastructure-level |
| **Hetzner** | Support ticket → Emergency phone → Datacenter ops | Hardware replacement; network diagnostics; DC access | 4h hardware; network 30-min |
| **Cloudflare** | Enterprise support → SOC escalation | WAF rule deployment; DDoS mitigation tuning | Immediate (Enterprise) |

---

## 11. Insurance & Financial Impact

| Coverage | Policy | Limit | Trigger | Notification |
|----------|--------|-------|---------|--------------|
| **Cyber Liability** | [Insurer] | €5M | Data breach, ransomware, business interruption | Within 24h of P1/P2 |
| **Professional Indemnity** | [Insurer] | €2M | Regulatory fines, client claims | Within 48h |
| **D&O** | [Insurer] | €1M | Board liability from incident | Within 48h |
| **Crime/Fidelity** | [Insurer] | €1M | Insider theft, social engineering | Within 24h |

**Financial Impact Tracking:** All incident costs (forensics, legal, notification, regulatory fines, business loss) tracked in incident Jira epic for insurance/Board reporting.

---

## 12. Cross-References

| Document | Reference |
|----------|-----------|
| `02_ICT_Risk_Management.md` | ICT risk framework, threat landscape, vulnerability management |
| `03_Outsourcing_Register.md` | Third-party incident escalation, concentration risk |
| `04_Complaints_Handling.md` | Client complaints from incidents |
| `05_Business_Continuity_Plan.md` | BIA, RTO/RPO, DR procedures, crisis management |
| `00_AML_Policy.md` | SAR triggers from incidents |
| `01_Travel_Rule_Policy.md` | Travel Rule incident procedures |
| `02_Sanctions_Policy.md` | Sanctions breach response |

---

## 13. Document Control

| Field | Value |
|-------|-------|
| **Document ID** | KOV-POL-OPS-006 |
| **Version** | 1.0 |
| **Classification** | Confidential |
| **Owner** | CISO / CTO |
| **Approved By** | Management Board |
| **Approval Date** | [DATE] |
| **Next Review** | [DATE + 6 months] |
| **Distribution** | Management Board, CISO, CTO, COO, MLRO, Legal, Engineering Leads, Compliance |

### Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-15 | Compliance Team | Initial version for MiCA CASP application |

---

## Appendix A: Incident ID Format

`INC-YYYYMMDD-NNN` (e.g., `INC-20260115-001`)

Sequential per day. Reset daily.

---

## Appendix B: War Room Setup Checklist

- [ ] Slack channel created (`#inc-INC-XXXXXX-XXX`)
- [ ] Video bridge started (meet.kovanica.protocol/inc-XXX)
- [ ] Shared document (Notion/Google Doc) for timeline
- [ ] Evidence repository folder created (encrypted)
- [ ] Stakeholder notification sent (per severity)
- [ ] Regulatory pre-notification drafted (P1/P2)
- [ ] Scribe assigned and logging

---

## Appendix C: PIR Report Template

| Section | Content |
|---------|---------|
| **Incident Summary** | ID, dates, severity, category, one-paragraph summary |
| **Timeline** | Detailed chronological log (detection → closure) |
| **Root Cause** | Technical & organizational (5 Whys) |
| **Impact Assessment** | Clients, assets, revenue, reputation, regulatory |
| **Response Effectiveness** | What worked; what didn't; MTTD/MTTR/MTTRc |
| **Gaps Identified** | Detection, containment, communication, recovery |
| **Action Items** | ID, description, owner, due date, priority |
| **Regulatory Outcomes** | Reports filed, feedback received, ongoing obligations |
| **Lessons Learned** | Process, technology, people, vendor improvements |

---

*End of Document*
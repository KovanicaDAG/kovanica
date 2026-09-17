# ICT Risk Management Policy

**Document ID:** KOV-POL-OPS-002
**Version:** 1.0
**Classification:** Confidential
**Owner:** CTO / CISO
**Review Cycle:** Annual (or upon material change)
**Status:** Approved

---

## 1. Purpose & Regulatory Basis

This policy establishes the ICT risk management framework for Kovanica Protocol d.o.o. ("the CASP") in compliance with **DORA (Regulation (EU) 2022/2554)**, **MiCA Articles 30–31, 66–67**, and **EBA Guidelines on ICT Risk (EBA/GL/2019/04)**.

| Regulation | Key Articles |
|------------|--------------|
| DORA | Articles 5–12 (ICT risk management), 13–17 (incident reporting), 18–27 (testing), 28–30 (third-party risk) |
| MiCA | Articles 30–31 (outsourcing), 66–67 (operational resilience) |
| EBA/GL/2019/04 | ICT risk management guidelines |
| Croatian Law | Zakon o operativnoj otpornosti (transposing DORA) |

**Scope:** All ICT assets supporting CASP services: trading engine, custody integration, KYC/AML systems, blockchain nodes, APIs, databases, networks, and third-party services.

---

## 2. Governance & Roles

| Role | Responsibility |
|------|----------------|
| **Management Board** | Ultimate accountability; approve ICT risk appetite, budget, strategy |
| **CTO / CISO** | Day-to-day ICT risk management; policy implementation; reporting |
| **ICT Risk Committee** | Quarterly review of risk profile, incidents, testing results |
| **ICT Operations** | Asset management, change management, vulnerability management |
| **Compliance / MLRO** | Regulatory alignment, incident reporting to HANFA/ECB |
| **Third-Party Risk Owner** | Vendor risk assessments, contract management, monitoring |

**Reporting Lines:**
- CTO → Management Board (monthly)
- CISO → ICT Risk Committee (quarterly)
- Major incidents → Management Board + HANFA (per DORA Art. 13–17)

---

## 3. ICT Asset Register

### 3.1 Critical Asset Classification

| Asset Category | Examples | Criticality | Recovery Time Objective (RTO) | Recovery Point Objective (RPO) |
|----------------|----------|-------------|-------------------------------|--------------------------------|
| **Core Trading** | Matching engine, order book, settlement | **Critical** | 1 hour | 0 (synchronous replication) |
| **Custody Integration** | Copper API, ClearLoop, HSM interfaces | **Critical** | 2 hours | 0 |
| **KYC/AML** | Sumsub integration, sanctions screening, SAR engine | **Critical** | 4 hours | 1 hour |
| **Blockchain Infrastructure** | Kovanica nodes, P2P mesh, RPC endpoints | **Critical** | 2 hours | 0 |
| **Data Layer** | PostgreSQL (ledger), Redis (cache), TimescaleDB (metrics) | **Critical** | 1 hour | 0 |
| **API Gateway** | Public REST/WebSocket, partner APIs | **High** | 4 hours | 1 hour |
| **Monitoring/Alerting** | Prometheus, Grafana, Alertmanager, Loki | **High** | 8 hours | 4 hours |
| **CI/CD Pipeline** | GitHub Actions, build agents, artifact storage | **Medium** | 24 hours | 4 hours |
| **Corporate IT** | Email, HR, finance, documentation | **Low** | 72 hours | 24 hours |

### 3.2 Asset Inventory Requirements

- **Automated discovery** via CMDB (Configuration Management Database)
- **Updated** on every change (CI/CD pipeline integration)
- **Reviewed** quarterly by ICT Risk Committee
- **Attributes tracked:** Owner, criticality, dependencies, data classification, location, backup status, patch level

---

## 4. Risk Assessment Methodology

### 4.1 Risk Identification

| Source | Frequency | Method |
|--------|-----------|--------|
| **Threat Intelligence** | Continuous | CERT-EU, ENISA, vendor advisories, ISAC feeds |
| **Vulnerability Scanning** | Weekly (automated), Monthly (authenticated) | Nessus / OpenVAS + container scanning (Trivy) |
| **Penetration Testing** | Annual (external), Semi-annual (internal) | CREST-certified provider |
| **Red Team Exercise** | Annual | Simulated APT targeting custody/trading |
| **Incident Analysis** | Per incident | Root cause → risk register update |
| **Change Risk Assessment** | Per change | Pre-deployment risk scoring |

### 4.2 Risk Scoring (DORA-Aligned)

| Likelihood | Impact: Critical (4) | Impact: High (3) | Impact: Medium (2) | Impact: Low (1) |
|------------|----------------------|------------------|--------------------|-----------------|
| **Very High (4)** | 16 — **Extreme** | 12 — **High** | 8 — **Medium** | 4 — **Low** |
| **High (3)** | 12 — **High** | 9 — **High** | 6 — **Medium** | 3 — **Low** |
| **Medium (2)** | 8 — **Medium** | 6 — **Medium** | 4 — **Low** | 2 — **Low** |
| **Low (1)** | 4 — **Low** | 3 — **Low** | 2 — **Low** | 1 — **Low** |

**Risk Appetite:**
- **Extreme (16):** Unacceptable — immediate mitigation required
- **High (9–12):** Tolerable only with compensating controls + Board approval
- **Medium (4–8):** Acceptable with treatment plan + timeline
- **Low (1–3):** Acceptable — monitor only

### 4.3 Top Inherent Risks (Pre-Mitigation)

| Risk ID | Description | Likelihood | Impact | Score | Treatment |
|---------|-------------|------------|--------|-------|-----------|
| **ICT-001** | Custody partner (Copper) API outage | High (3) | Critical (4) | 12 | Multi-region, circuit breakers, manual fallback |
| **ICT-002** | Blockchain node consensus failure / fork | Medium (2) | Critical (4) | 8 | Multi-node deployment, automated fork detection |
| **ICT-003** | KYC provider (Sumsub) data breach | Low (1) | Critical (4) | 4 | DPA, encryption, minimal data sharing |
| **ICT-004** | Ransomware on corporate IT | Medium (2) | High (3) | 6 | Immutable backups, network segmentation, EDR |
| **ICT-005** | Insider threat (privileged access abuse) | Low (1) | Critical (4) | 4 | PAM, MFA, audit logging, separation of duties |
| **ICT-006** | Supply chain attack (CI/CD compromise) | Low (1) | Critical (4) | 4 | SLSA Level 3, signed artifacts, dependency scanning |
| **ICT-007** | DDoS on public APIs | High (3) | High (3) | 9 | WAF, rate limiting, CDN, auto-scaling |
| **ICT-008** | Data corruption in ledger database | Low (1) | Critical (4) | 4 | Replication, point-in-time recovery, checksums |

---

## 5. Protection & Prevention Controls

### 5.1 Network Security

| Control | Implementation | Verification |
|---------|----------------|--------------|
| **Network Segmentation** | VPCs: public (API), private (trading, custody, data), management | Quarterly network topology review |
| **Zero Trust** | mTLS between all services; SPIFFE/SPIRE for identity | Continuous verification via service mesh (Istio/Linkerd) |
| **WAF / DDoS** | Cloudflare / AWS Shield Advanced on public endpoints | Monthly rule review; quarterly DDoS simulation |
| **Egress Control** | Allow-list for external API calls (Copper, Sumsub, blockchain peers) | Weekly firewall log review |

### 5.2 Identity & Access Management

| Control | Implementation |
|---------|----------------|
| **Privileged Access Management (PAM)** | CyberArk / HashiCorp Boundary; session recording; just-in-time access |
| **MFA** | Hardware tokens (YubiKey) for all admin access; TOTP for user access |
| **Least Privilege** | RBAC with quarterly access reviews; automated deprovisioning on role change |
| **Service Accounts** | Short-lived tokens (Vault); no long-lived keys in code/config |

### 5.3 Data Protection

| Control | Implementation |
|---------|----------------|
| **Encryption at Rest** | AES-256 (AWS KMS / HashiCorp Vault); per-database keys |
| **Encryption in Transit** | TLS 1.3 everywhere; mTLS for service-to-service |
| **Key Management** | HashiCorp Vault with auto-rotation (90 days); HSM for root keys |
| **Data Classification** | Public / Internal / Confidential / Restricted (PII, keys) |
| **Data Loss Prevention** | Nightfall / custom regex scanning on egress points |

### 5.4 Vulnerability & Patch Management

| Asset Type | Patch SLA (Critical) | Patch SLA (High) | Patch SLA (Medium/Low) |
|------------|----------------------|------------------|------------------------|
| **OS / Firmware** | 72 hours | 14 days | 30 days |
| **Container Base Images** | 24 hours (rebuild) | 7 days | 14 days |
| **Application Dependencies** | 48 hours (Dependabot auto-PR) | 14 days | 30 days |
| **Network Devices** | 7 days | 30 days | 90 days |

**Exceptions:** Documented, approved by CISO, compensating controls, max 90 days.

---

## 6. Detection & Monitoring

### 6.1 Security Monitoring Stack

| Layer | Tool | Coverage |
|-------|------|----------|
| **Infrastructure** | Prometheus Node Exporter, cAdvisor | Host metrics, container resources |
| **Application** | OpenTelemetry (traces, metrics, logs) | Distributed tracing, business metrics |
| **Security** | Falco (runtime), Suricata (network), CrowdStrike (EDR) | Runtime threats, network anomalies, endpoint |
| **Log Aggregation** | Loki + Promtail | Structured logs, 13-month retention |
| **SIEM** | Elastic Security / Splunk (TBD) | Correlation rules, UEBA |

### 6.2 Key Detection Rules (MITRE ATT&CK Mapped)

| Rule ID | MITRE Technique | Description | Severity |
|---------|-----------------|-------------|----------|
| **DET-001** | T1190 (Exploit Public-Facing App) | WAF block + error spike on API | High |
| **DET-002** | T1078 (Valid Accounts) | Impossible travel / geo-velocity on admin accounts | Critical |
| **DET-003** | T1486 (Data Encrypted for Impact) | Mass file encryption / ransomware indicators | Critical |
| **DET-004** | T1059 (Command & Scripting) | Unexpected shell in containers | High |
| **DET-005** | T1567 (Exfiltration Over Web) | Large egress to unknown destinations | High |
| **DET-006** | T1068 (Exploitation for Privilege Escalation) | Sudo/su anomalies, kernel exploit patterns | Critical |
| **DET-007** | T1133 (External Remote Services) | VPN/SSH brute force, credential stuffing | Medium |
| **DET-008** | T1505 (Server Software Component) | Unauthorized webhook / API registration | High |

### 6.3 Alerting & Escalation

| Severity | Response Time | Escalation |
|----------|---------------|------------|
| **Critical (P1)** | 15 minutes | On-call → CISO → Management Board (1h) |
| **High (P2)** | 1 hour | On-call → CTO (4h) |
| **Medium (P3)** | 4 hours | Team lead → CTO (24h) |
| **Low (P4)** | 24 hours | Sprint backlog |

---

## 7. Incident Response (DORA Art. 13–17)

### 7.1 Incident Classification

| Category | Criteria | Examples |
|----------|----------|----------|
| **Major (DORA Reportable)** | Significant impact on services, clients, or market integrity | Trading outage > 1h, custody freeze, data breach > 1000 records, ransomware |
| **Significant** | Degraded service, no client fund loss | API latency > 500ms p95, partial KYC outage |
| **Minor** | No service impact, contained quickly | Single failed deployment, false positive alert |

### 7.2 DORA Reporting Timelines

| Event | Authority | Timeline |
|-------|-----------|----------|
| **Major Incident** | HANFA (NCA) + ECB (if significant) | **4 hours** initial notification |
| **Interim Update** | HANFA | **24 hours** |
| **Final Report** | HANFA | **1 month** (root cause, lessons learned) |
| **Significant Cyber Threat** | HANFA | **24 hours** |

### 7.3 Incident Response Phases

| Phase | Activities | Owner | Timeline |
|-------|------------|-------|----------|
| **1. Detection** | Alert triage, initial classification | On-call Engineer | T+0 |
| **2. Containment** | Isolate affected systems, preserve evidence | Incident Commander | T+1h |
| **3. Eradication** | Remove threat, patch vulnerabilities | Engineering + Security | T+4h |
| **4. Recovery** | Restore services, verify integrity | Operations | T+RTO |
| **5. Post-Incident** | Root cause analysis (5 Whys), update runbooks | CISO + Team | T+7d |

**Runbooks:** Maintained in GitOps repo; tested quarterly via tabletop exercises.

---

## 8. Digital Operational Resilience Testing (DORA Art. 18–27)

### 8.1 Testing Programme

| Test Type | Frequency | Scope | Provider |
|-----------|-----------|-------|----------|
| **Vulnerability Assessment** | Monthly (auto), Quarterly (manual) | All internet-facing + internal | Internal + External |
| **Penetration Testing** | Annual | Full scope (web, API, mobile, network, physical) | CREST-certified |
| **Advanced Testing (TLPT)** | Every 3 years (or post-major change) | Threat-led penetration test (red team) | TIBER-EU accredited |
| **Business Continuity Test** | Semi-annual | Failover to DR site, data restore | Internal |
| **Tabletop Exercise** | Quarterly | Scenario: ransomware, custody outage, fork, insider | ICT Risk Committee |
| **Chaos Engineering** | Monthly (GameDays) | Dependency failure injection (Copper, Sumsub, nodes) | Engineering |

### 8.2 Test Results Management

- **Findings tracked** in Jira with SLA per severity
- **Remediation verified** before closure
- **Trend analysis** presented to Management Board quarterly
- **Regulatory reporting** of TLPT results per DORA Art. 26

---

## 9. Third-Party ICT Risk Management (DORA Art. 28–30)

### 9.1 Critical ICT Providers

| Provider | Service | Criticality | Contract Status | Last Assessment |
|----------|---------|-------------|-----------------|-----------------|
| **Copper** | Custody, settlement, ClearLoop | **Critical** | MSA + DPA executed | 2026-01-15 |
| **Sumsub** | KYC/CDD, sanctions/PEP screening | **Critical** | MSA + DPA executed | 2026-01-15 |
| **AWS / Hetzner** | Cloud infrastructure (compute, storage, network) | **Critical** | Enterprise Agreement | 2026-01-15 |
| **Cloudflare** | WAF, DDoS, CDN, DNS | **High** | Enterprise Plan | 2026-01-15 |
| **GitHub** | Source control, CI/CD, packages | **High** | Enterprise Cloud | 2026-01-15 |
| **Datadog / Grafana Cloud** | Observability (metrics, logs, traces) | **High** | SaaS Agreement | 2026-01-15 |

### 9.2 Third-Party Risk Lifecycle

| Phase | Activities | Frequency |
|-------|------------|-----------|
| **Onboarding** | Security questionnaire, SOC 2 review, DPA, contract negotiation | Per vendor |
| **Ongoing Monitoring** | Continuous: security ratings (SecurityScorecard), breach monitoring (HaveIBeenPwned), SLA dashboards | Continuous |
| **Annual Reassessment** | Updated questionnaire, SOC 2 Type II review, financial health, concentration risk | Annual |
| **Offboarding** | Data return/deletion certification, access revocation, transition plan | Per exit |

### 9.3 Concentration Risk

- **No single provider** > 50% of critical function capacity
- **Copper:** Primary custody; evaluating secondary custodian for 2026 H2
- **AWS:** Multi-region (eu-central-1, eu-north-1); evaluating GCP/Azure for DR
- **Sumsub:** Single KYC provider; manual fallback documented

---

## 10. Business Continuity & Disaster Recovery

### 10.1 Recovery Objectives

| Service | RTO | RPO | DR Strategy |
|---------|-----|-----|-------------|
| **Trading Engine** | 1 hour | 0 | Active-active (eu-central-1 + eu-north-1) |
| **Custody Integration** | 2 hours | 0 | Multi-region Copper endpoints; manual signing fallback |
| **KYC/AML** | 4 hours | 1 hour | Sumsub multi-region; manual review queue |
| **Blockchain Nodes** | 2 hours | 0 | 3+ nodes across 2 AZs; automated peer discovery |
| **Databases** | 1 hour | 0 | Synchronous replication + PITR |
| **Monitoring** | 8 hours | 4 hours | Cross-region Prometheus federation |

### 10.2 Backup Strategy

| Data Type | Frequency | Retention | Storage | Encryption | Test Frequency |
|-----------|-----------|-----------|---------|------------|----------------|
| **Ledger Database** | Continuous (WAL) + Daily snapshot | 13 months | S3 (IA + Glacier) | AES-256 (KMS) | Monthly restore test |
| **Configuration/Secrets** | On change (GitOps) | Indefinite | Git (signed commits) | GPG + Vault | Quarterly DR test |
| **Application Code** | On commit | Indefinite | GitHub + S3 mirror | N/A | N/A |
| **Logs/Metrics** | Continuous | 13 months | Loki + S3 | AES-256 | N/A |

---

## 11. Training & Awareness

| Audience | Training | Frequency |
|----------|----------|-----------|
| **All Staff** | Security awareness (phishing, social engineering, data handling) | Annual + onboarding |
| **Engineering** | Secure coding (OWASP Top 10), container security, supply chain | Semi-annual |
| **Operations** | Incident response, runbook execution, DR procedures | Quarterly |
| **Management Board** | Cyber risk oversight, regulatory obligations | Annual |
| **Privileged Users** | PAM procedures, emergency access, audit responsibilities | Semi-annual |

**Phishing Simulations:** Quarterly; results tracked, repeat clickers → enhanced training.

---

## 12. Metrics & KPIs

| KPI | Target | Reporting Frequency |
|-----|--------|---------------------|
| **Critical Vulnerability Remediation** | 100% within SLA | Monthly |
| **Patch Compliance** | > 95% | Monthly |
| **Incident Detection Time (MTTD)** | < 15 min (P1) | Monthly |
| **Incident Response Time (MTTR)** | < RTO per service | Monthly |
| **Availability (Critical Services)** | > 99.9% | Monthly |
| **Backup Restore Success Rate** | 100% (tested) | Monthly |
| **Third-Party SLA Compliance** | > 99.5% | Quarterly |
| **Phishing Click Rate** | < 5% | Quarterly |
| **Access Review Completion** | 100% on schedule | Quarterly |

---

## 13. Cross-References

| Document | Reference |
|----------|-----------|
| `00_Custody_Policy.md` | Custody integration resilience, Copper SLA |
| `01_Custody_Partner_Due_Diligence.md` | Copper ICT risk assessment |
| `03_Outsourcing_Register.md` | Full outsourcing register with ICT providers |
| `05_Business_Continuity_Plan.md` | Detailed BCP/DR procedures |
| `06_Incident_Response_Plan.md` | Detailed IR procedures, runbooks |
| `00_AML_Policy.md` | KYC/AML system availability requirements |
| `01_Programme_of_Operations.md` | Key functions, operational risk management |

---

## 14. Document Control

| Field | Value |
|-------|-------|
| **Document ID** | KOV-POL-OPS-002 |
| **Version** | 1.0 |
| **Classification** | Confidential |
| **Owner** | CTO / CISO |
| **Approved By** | Management Board |
| **Approval Date** | [DATE] |
| **Next Review** | [DATE + 1 year] |
| **Distribution** | Management Board, CTO, CISO, ICT Risk Committee, Compliance |

### Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-15 | Compliance Team | Initial version for MiCA CASP application |

---

*End of Document*
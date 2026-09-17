# Business Continuity Plan

**Document ID:** KOV-POL-OPS-005
**Version:** 1.0
**Classification:** Confidential
**Owner:** COO / CISO
**Review Cycle:** Annual (or upon material change)
**Status:** Approved

---

## 1. Purpose & Regulatory Basis

This Business Continuity Plan (BCP) ensures Kovanica Protocol d.o.o. ("the CASP") can maintain critical operations during and after disruptive events, in compliance with:

| Regulation | Reference |
|------------|-----------|
| MiCA | Articles 30–31 (operational resilience), Article 75 (safeguarding) |
| DORA | Articles 11–12 (business continuity management), Articles 28–30 (ICT third-party risk) |
| EBA Guidelines | EBA/GL/2021/05 (ICT and security risk management), EBA/GL/2019/02 (outsourcing) |
| ISO 22301 | Business Continuity Management Systems (reference standard) |
| Croatian Law | Zakon o operativnoj otpornosti (DORA transposition) |

**Scope:** All critical business functions, ICT systems, and outsourced services supporting CASP activities (exchange, transfer, execution, custody).

---

## 2. Business Impact Analysis (BIA)

### 2.1 Critical Functions & Recovery Objectives

| Critical Function | Description | RTO | RPO | MTPD | Dependencies |
|-------------------|-------------|-----|-----|------|--------------|
| **Custody & Asset Safeguarding** | Client asset segregation, private key management, staking | 1 hour | 0 (real-time) | 4 hours | Copper, HSM, Hetzner nodes |
| **Trade Execution & Settlement** | Order matching, trade confirmation, ClearLoop settlement | 2 hours | Near-zero | 8 hours | Trading engine, Copper, AWS |
| **Client Onboarding & KYC** | Identity verification, CDD, sanctions screening | 4 hours | 1 hour | 24 hours | Sumsub, AWS, Database |
| **Transfers & Withdrawals** | On-chain transactions, unhosted wallet handling, Travel Rule | 2 hours | Near-zero | 8 hours | Blockchain nodes, Copper, TRP |
| **Blockchain Consensus** | Block production, validation, peer gossip, DAG sync | 30 min | 0 (real-time) | 2 hours | Hetzner nodes, AWS, P2P mesh |
| **Regulatory Reporting** | MiCA, AML, transaction reports to HANFA/FIU | 24 hours | 1 hour | 72 hours | Data warehouse, reporting engine |
| **Client Support & Complaints** | Ticket handling, escalation, communication | 4 hours | 1 hour | 24 hours | Jira, Email, Phone |
| **Monitoring & Alerting** | Infrastructure, application, security, business metrics | 15 min | 5 min | 1 hour | Grafana, Datadog, PagerDuty |

**Definitions:**
- **RTO (Recovery Time Objective):** Maximum acceptable downtime
- **RPO (Recovery Point Objective):** Maximum acceptable data loss
- **MTPD (Maximum Tolerable Period of Disruption):** Beyond which survival is threatened

### 2.2 Resource Requirements at MTPD

| Resource | Minimum for MTPD | Target for RTO |
|----------|------------------|----------------|
| **Personnel** | 3 engineers + 1 compliance + 1 support | Full team (15+) |
| **Compute (AWS)** | 2x EKS nodes (eu-central-1) | Full capacity (multi-AZ) |
| **Bare Metal (Hetzner)** | 3 validator nodes (2 DE + 1 FI) | 5+ nodes (3 DE + 2 FI) |
| **Custody (Copper)** | API read-only + manual signing | Full API + ClearLoop |
| **KYC (Sumsub)** | Manual review queue | Full automated + manual |
| **Data** | Last consistent backup (RPO) | Real-time replication |
| **Network** | VPN + SSH access | Full SD-WAN + DDoS protection |

---

## 3. Disruption Scenarios & Response Strategies

### 3.1 Scenario Matrix

| Scenario | Likelihood | Impact | Primary Strategy | Fallback |
|----------|------------|--------|------------------|----------|
| **AWS Region Failure (eu-central-1)** | Low | Critical | Failover to eu-north-1 (Stockholm) | GCP eu-west-1 (manual) |
| **Hetzner Datacenter Outage (Nuremberg)** | Medium | High | Failover to Falkenstein + Helsinki nodes | AWS EKS validator nodes |
| **Copper API Outage** | Low | Critical | Manual signing via HSM; queue settlements | Secondary custodian (eval) |
| **Sumsub Outage** | Low | High | Manual KYC queue; document upload portal | Secondary KYC provider (eval) |
| **Blockchain Network Partition** | Medium | High | Wait for healing; manual peer reconnect | Emergency checkpoint sync |
| **Ransomware / Cyber Attack** | Medium | Critical | Isolate; restore from immutable backups | Forensic + rebuild |
| **Key Personnel Unavailability** | Medium | Medium | Cross-training; documented runbooks | External contractor retainer |
| **Regulatory Action (License Suspension)** | Low | Critical | Legal response; orderly wind-down | Pre-approved wind-down plan |
| **Power/Internet Outage (Office)** | Medium | Low | Remote work (full capability) | Co-working space / cloud shell |

### 3.2 Detailed Response: AWS Region Failure

**Trigger:** AWS Health Dashboard + CloudWatch alarms (EC2, RDS, EKS unavailable > 5 min)

**Automated Response (Terraform + AWS):**
1. Route53 health checks fail → DNS failover to eu-north-1 (TTL 60s)
2. EKS cluster in eu-north-1 scales from 0 → target nodes (ASG)
3. RDS read replica in eu-north-1 promoted to primary
4. S3 Cross-Region Replication (CRR) already syncing
5. KMS multi-region keys active in both regions

**Manual Response (Runbook: `RUN-AWS-001`):**
1. On-call engineer confirms failover completion (Slack #incidents)
2. Verify critical services: API, trading engine, custody integration, KYC
3. Update Copper/Sumsub webhook endpoints if needed
4. Notify clients via status page + email (template `COM-AWS-001`)
5. Monitor for 2 hours; declare "stable" or escalate

**Recovery:** When eu-central-1 restored, controlled failback (off-peak)

### 3.3 Detailed Response: Copper Custody Outage

**Trigger:** Copper API 5xx > 5% for 10 min; or Copper status page incident

**Response (Runbook: `RUN-COP-001`):**
1. **Immediate (T+0):** Switch to read-only mode; queue all withdrawals/settlements
2. **T+15 min:** Activate HSM manual signing procedure (2-of-3 MPC shares)
3. **T+30 min:** Notify clients: "Withdrawals delayed; funds safe in custody"
4. **T+1 hour:** If Copper unavailable → execute emergency withdrawal to cold storage (pre-signed)
5. **T+4 hours (MTPD):** If still down → Board decision on secondary custodian activation

**Communication:** Status page, email, in-app notification every 30 min

### 3.4 Detailed Response: Blockchain Network Partition

**Trigger:** Peer count < 3; block production stopped > 10 min; sync height stalled

**Response (Runbook: `RUN-BC-001`):**
1. **Automated:** Nodes attempt peer reconnect (exponential backoff)
2. **T+5 min:** Alert on-call engineer (PagerDuty)
3. **T+15 min:** Engineer assesses: network partition vs. node failure
4. **If partition:** Wait for healing (GHOSTDAG tolerates); monitor peer diversity
5. **If node failure:** Spin up replacement nodes on Hetzner/AWS (Ansible playbook)
6. **T+30 min:** If > 50% network affected → coordinate with other validators (Signal group)
7. **T+2 hours (MTPD):** Emergency checkpoint from trusted peer (seed nodes)

---

## 4. Crisis Management Structure

### 4.1 Crisis Management Team (CMT)

| Role | Primary | Alternate | Responsibilities |
|------|---------|-----------|------------------|
| **Crisis Lead** | CEO | COO | Overall command; Board liaison; external comms |
| **Technical Lead** | CTO | Lead Engineer | Technical recovery; infrastructure; engineering |
| **Operations Lead** | COO | Head of Ops | Business process continuity; vendor coordination |
| **Compliance Lead** | CCO/MLRO | Compliance Officer | Regulatory obligations; HANFA/FIU communication |
| **Legal Lead** | General Counsel | External Counsel | Legal assessment; contracts; litigation risk |
| **Communications Lead** | Head of Marketing | CEO | Client comms; status page; media; social |
| **Security Lead** | CISO | Lead Security | Forensics; containment; evidence preservation |

### 4.2 Activation Protocol

| Severity | Criteria | Activation | Notification |
|----------|----------|------------|--------------|
| **Level 1 (Minor)** | Single non-critical service degraded; RTO not threatened | On-call engineer handles | Slack #incidents |
| **Level 2 (Major)** | Critical function at risk; RTO threatened; client impact | CMT Lead activates | PagerDuty → CMT + Slack #crisis |
| **Level 3 (Crisis)** | Multiple critical functions down; MTPD threatened; regulatory/media risk | CEO activates | PagerDuty → CMT + Board + Slack #crisis + status page |

### 4.3 Communication Plan

| Audience | Channel | Frequency | Owner | Template |
|----------|---------|-----------|-------|----------|
| **CMT** | Slack #crisis + Bridge call | Continuous | Crisis Lead | — |
| **All Staff** | Slack #general + Email | Every 2 hours | Comms Lead | `COM-STAFF-XXX` |
| **Clients (Retail)** | Status page + Email + In-app | Every 30 min (L2/3) | Comms Lead | `COM-CLIENT-XXX` |
| **Clients (Professional)** | Email + Phone (dedicated) | Every 30 min (L2/3) | Account Manager | `COM-PRO-XXX` |
| **HANFA** | Email + Phone (dedicated) | Immediate + updates | Compliance Lead | `COM-HANFA-XXX` |
| **Copper/Sumsub** | Dedicated Slack/Email | As needed | Ops Lead | `COM-VENDOR-XXX` |
| **Media** | Press release + Holding statement | As needed | Comms Lead | `COM-MEDIA-XXX` |

**Status Page:** `status.kovanica.protocol` (hosted on Cloudflare Workers, independent of AWS)

---

## 5. Backup & Recovery Procedures

### 5.1 Backup Strategy

| Data Type | Method | Frequency | Retention | Location | Encryption |
|-----------|--------|-----------|-----------|----------|------------|
| **PostgreSQL (Primary DB)** | pgBackRest (full + incremental) | Daily full / 15-min WAL | 30 days / 1 year (monthly) | S3 (eu-central-1 + eu-north-1) | AES-256 (KMS) |
| **Redis (Cache/Session)** | RDB + AOF | Hourly | 7 days | S3 (eu-central-1) | AES-256 |
| **Blockchain Data (DAG)** | Snapshot + incremental log | Daily | 90 days | S3 + Hetzner (off-site) | AES-256 |
| **KYC/AML Records** | Sumsub export + internal DB | Daily | 10 years (regulatory) | S3 (eu-central-1 + Glacier) | AES-256 |
| **Configuration (Terraform/Ansible)** | Git (GitHub + Gitea mirror) | Continuous | Indefinite | GitHub + Self-hosted | GPG signed |
| **Secrets (Vault)** | Vault snapshots | Daily | 90 days | S3 (eu-central-1) | AES-256 + Shamir |

### 5.2 Recovery Procedures

| System | Recovery Steps | Validation |
|--------|----------------|------------|
| **Primary Database** | 1. Provision RDS in target region<br>2. Restore from pgBackRest (latest + WAL)<br>3. Verify schema + row counts<br>4. Update DNS / connection strings | `pg_verifybackup`; application health checks |
| **Kubernetes (EKS)** | 1. Apply Terraform (cluster + node groups)<br>2. ArgoCD sync (GitOps)<br>3. Verify all pods Running<br>4. Run smoke tests | `kubectl get pods`; integration tests |
| **Validator Nodes** | 1. Provision Hetzner servers (Ansible)<br>2. Restore DAG snapshot<br>3. Start kovanica-node<br>4. Verify peer sync + block production | `dag.tips()`; `node.block_count()` |
| **Custody Integration** | 1. Verify Copper API health<br>2. Re-establish webhook endpoints<br>3. Reconcile pending settlements | Test deposit/withdrawal |
| **KYC Integration** | 1. Verify Sumsub API health<br>2. Re-establish webhook<br>3. Process manual queue | Test applicant flow |

---

## 6. Testing & Exercise Program

### 6.1 Test Schedule

| Test Type | Frequency | Scope | Participants | Success Criteria |
|-----------|-----------|-------|--------------|------------------|
| **Tabletop Exercise** | Quarterly | CMT decision-making; comms; escalation | CMT + observers | Decisions logged; comms templates used; < 30 min to activate |
| **Component Failover** | Monthly (rotating) | Single system (DB, EKS, Nodes, Copper, Sumsub) | On-call + SRE | RTO/RPO met; automated failover works |
| **Full DR Drill** | Semi-annual | Complete region failover (AWS eu-central-1 → eu-north-1) | Full engineering + CMT | All critical functions restored within RTO |
| **Cyber Attack Simulation** | Annual | Ransomware / data exfiltration / DDoS | CMT + Security + External red team | Containment < 1 hour; recovery < 4 hours |
| **Vendor Failure** | Annual | Copper / Sumsub / AWS simultaneous outage | CMT + Vendor managers | Manual procedures executed; client comms sent |

### 6.2 Test Documentation

Each test produces:
- **Test Plan** (pre-approved by CISO)
- **Execution Log** (timestamped actions, decisions, issues)
- **After-Action Report** (gaps, improvements, action items)
- **Action Item Tracking** (Jira epics, assigned, due dates)

---

## 7. Third-Party Dependency Continuity

| Provider | Critical Function | Provider BCP | Our Fallback | Contractual Commitment |
|----------|-------------------|--------------|--------------|------------------------|
| **Copper** | Custody, Settlement | Multi-region (EU/UK); SOC 2; DR tested | HSM manual signing; cold storage withdrawal | 99.9% SLA; 180-day transition |
| **Sumsub** | KYC/CDD | Multi-region (EU); SOC 2 | Manual review queue; document portal | 99.9% SLA; 90-day transition |
| **AWS** | Cloud Infrastructure | Multi-AZ + Multi-Region; 99.99% | GCP (Terraform portable); Hetzner bare metal | Enterprise Support; SLA credits |
| **Hetzner** | Validator Nodes | 3 EU datacenters; hardware SLA | AWS EKS validators; additional provider | 99.9% network; 4h hardware replace |
| **Cloudflare** | WAF/DDoS/DNS | Global Anycast; 100% SLA | AWS Shield + Route53; manual DNS | Enterprise SLA; 15-min P1 |

---

## 8. Regulatory Obligations During Disruption

| Obligation | Requirement | Continuity Measure |
|------------|-------------|-------------------|
| **MiCA Safeguarding (Art. 75)** | Client assets always segregated | Copper MPC + HSM backup; cold storage keys offline |
| **Transaction Reporting** | No gaps in AML/CTF reporting | Buffered queue; manual filing if automated down |
| **Complaints Handling** | Acknowledgment within 1 day | Manual logging; email acknowledgment templates |
| **Capital Requirements** | Own funds monitoring | Daily calculation; offline spreadsheet backup |
| **HANFA Notification** | Material incidents within 24h | Pre-drafted notification templates; secure channel |
| **Data Protection (GDPR)** | Breach notification 72h | Incident response plan; DPO on-call |

---

## 9. Insurance Coverage

| Policy | Coverage | Limit | Relevance to BCP |
|--------|----------|-------|------------------|
| **Cyber Liability** | Data breach, ransomware, business interruption | €5M | Covers recovery costs, notification, legal |
| **Professional Indemnity** | Errors & omissions in CASP services | €2M | Covers client losses from disruption |
| **Directors & Officers** | Management liability | €2M | Covers Board decisions during crisis |
| **Property / Equipment** | Hardware, office | €500k | Covers Hetzner colo equipment |
| **Business Interruption** | Lost revenue during downtime | €1M (12-month indemnity) | Covers revenue loss during MTPD |

---

## 10. Cross-References

| Document | Reference |
|----------|-----------|
| `02_ICT_Risk_Management.md` | ICT risk framework, incident management, DR |
| `06_Incident_Response_Plan.md` | Incident classification, containment, forensics |
| `03_Outsourcing_Register.md` | Vendor BCPs, concentration risk, exit strategies |
| `00_Custody_Policy.md` | Asset safeguarding continuity |
| `04_Complaints_Handling.md` | Complaints continuity during disruption |
| `01_Programme_of_Operations.md` | Key functions, operational risk |

---

## 11. Document Control

| Field | Value |
|-------|-------|
| **Document ID** | KOV-POL-OPS-005 |
| **Version** | 1.0 |
| **Classification** | Confidential |
| **Owner** | COO / CISO |
| **Approved By** | Management Board |
| **Approval Date** | [DATE] |
| **Next Review** | [DATE + 1 year] |
| **Distribution** | Management Board, CMT, COO, CTO, CISO, Compliance, Engineering, Support |

### Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-15 | Compliance Team | Initial version for MiCA CASP application |

---

## Appendix A: Runbook Index

| Runbook ID | Title | System | Last Tested |
|------------|-------|--------|-------------|
| `RUN-AWS-001` | AWS Region Failover (eu-central-1 → eu-north-1) | AWS/EKS/RDS | 2026-01-10 |
| `RUN-COP-001` | Copper Custody Outage | Copper/API | 2026-01-12 |
| `RUN-BC-001` | Blockchain Network Partition | kovanica-node/P2P | 2026-01-08 |
| `RUN-SUM-001` | Sumsub KYC Outage | Sumsub/API | 2026-01-11 |
| `RUN-DB-001` | Database Restore (pgBackRest) | PostgreSQL/RDS | 2026-01-09 |
| `RUN-K8S-001` | EKS Cluster Rebuild | Kubernetes/ArgoCD | 2026-01-10 |
| `RUN-HET-001` | Hetzner Node Replacement | Bare Metal/Ansible | 2026-01-11 |
| `RUN-CYBER-001` | Ransomware Response | All Systems | 2025-12-15 |

---

## Appendix B: Emergency Contact List

| Role | Name | Phone | Email | Backup |
|------|------|-------|-------|--------|
| CEO / Crisis Lead | [NAME] | +385 9X XXX XXX | ceo@kovanica.protocol | COO |
| CTO / Technical Lead | [NAME] | +385 9X XXX XXX | cto@kovanica.protocol | Lead Eng |
| COO / Operations Lead | [NAME] | +385 9X XXX XXX | coo@kovanica.protocol | Head of Ops |
| CCO/MLRO / Compliance Lead | [NAME] | +385 9X XXX XXX | compliance@kovanica.protocol | Compliance Off |
| CISO / Security Lead | [NAME] | +385 9X XXX XXX | security@kovanica.protocol | Lead Sec |
| General Counsel / Legal Lead | [NAME] | +385 9X XXX XXX | legal@kovanica.protocol | External Counsel |
| Head of Marketing / Comms Lead | [NAME] | +385 9X XXX XXX | marketing@kovanica.protocol | CEO |
| HANFA Contact | [NAME] | +385 1 XXXXXXX | nadzor@hanfa.hr | — |
| Copper Support | [NAME] | +44 20 XXXXXXX | support@copper.co | TAM |
| Sumsub Support | [NAME] | +44 20 XXXXXXX | support@sumsub.com | TAM |
| AWS Enterprise Support | — | — | AWS Console / Phone | TAM |

---

*End of Document*
# Programme of Operations — Kovanica Payments d.o.o.

> **MiCA Article 14(2)(b)** | **Version**: v0.1-draft | **Status**: 📋 Planned  
> **Parent**: [[Business_Plan]]  

---

## 1. Scope of Services (Detailed)

### 1.1 Custody and Administration (Service 1)
**Description**: Safekeeping of crypto-assets and related cryptographic keys on behalf of clients.

| Sub-service | Implementation | Status |
|-------------|----------------|--------|
| **Retail custody** | Mobile light-node (FFI) — user-held keys, multisig option | ✅ SPV + FFI; ⚠️ Multisig UI |
| **Institutional custody** | M-of-N P2SH (RFC-001), HSM-backed, geographic distribution | ✅ Consensus; ⚠️ Node/FFI exposure |
| **Staking custody** | Bonded stake (frozen UTXOs), validator key management | ✅ Hybrid staking |
| **Key backup/recovery** | Shamir secret sharing, encrypted cloud backup (opt-in) | 📋 Design phase |

**Safeguarding Measures** (per Art. 75–76):
- Asset segregation: Client keys ≠ Company keys (separate derivation paths)
- Reconciliation: Daily UTXO set reconciliation vs. internal ledger
- Insurance: Crime + cyber policy covering custody losses
- Audit: Annual SOC2 Type II + crypto-asset specific audit

### 1.2 Exchange: Fiat ↔ Crypto (Service 2)
**Description**: Purchase/sale of crypto-assets for fiat currency.

| Channel | Partner | Status |
|---------|---------|--------|
| **On-ramp** | MoonPay / Banxa / local bank | 📋 Negotiation |
| **Off-ramp** | SEPA Instant / TARGET2 via partner | 📋 Negotiation |
| **FX Engine** | Internal (mid-market + spread) | 📋 Design |

**Limits**: Tiered by KYC level (L1: €1k/day, L2: €10k/day, L3: €100k/day)

### 1.3 Exchange: Crypto ↔ Crypto (Service 3)
**Description**: Exchange between different crypto-assets.

| Approach | Detail | Status |
|----------|--------|--------|
| **Native only** | KVNC only (no other assets issued) | ✅ Current |
| **DEX aggregation** | 1inch / Paraswap integration | 📋 Future |
| **Atomic swaps** | HTLC on Kovanica (future upgrade) | ❌ Not designed |

### 1.4 Transfer (Service 4)
**Description**: Transfer of crypto-assets on behalf of clients.

| Feature | Implementation |
|---------|----------------|
| **Single-sig** | `send_from(secret, amount, to)` — FFI |
| **Multi-sig** | `build_multisig_spend` + `combine_multisig_sigs` — RFC-001 |
| **Batch** | Multiple outputs in single tx (UTXO model) |
| **Fee estimation** | p90 mempool fee rate + RBF support |
| **Confirmation** | SPV proof (KVLSv1) or full node verification |

### 1.5 Execution of Orders (Service 5)
**Description**: Executing buy/sell orders for crypto-assets.

| Component | Implementation |
|-----------|----------------|
| **Order types** | Market, Limit, Stop-limit (via mempool v2) |
| **Matching** | Price-time priority in mempool; RBF for replacement |
| **Settlement** | On-chain (GHOSTDAG finality) |
| **Reporting** | Execution confirmations + MiCA trade reports |

### 1.6 Placing (Service 6)
**Description**: Placing of crypto-assets (primary distribution).

| Scenario | Status |
|----------|--------|
| **KVNC native** | No placing — fair launch, mining/staking only |
| **Future tokens** | Whitepaper + MiCA Art. 17–24 compliance required |
| **Tokenized assets** | Not in scope (requires separate authorization) |

### 1.7 Advice (Service 7)
**Description**: Providing advice on crypto-assets.

| Scope | Implementation |
|-------|----------------|
| **Suitability** | Risk profiling questionnaire → product recommendation |
| **Disclosure** | MiCA Art. 66 disclosures (risks, costs, conflicts) |
| **Record-keeping** | All advice logged, timestamped, retained 5+ years |

### 1.8 Portfolio Management (Service 8)
**Description**: Managing portfolios of crypto-assets on discretionary basis.

| Feature | Implementation |
|---------|----------------|
| **Staking portfolio** | Auto-bond/unbond, validator selection, reward compounding |
| **Rebalancing** | Not applicable (single native asset) |
| **Mandates** | Discretionary (staking params) vs. advisory |
| **Reporting** | Quarterly + on-demand, MiCA Art. 68 compliant |

---

## 2. Client Onboarding & Classification

### 2.1 Client Categories (MiCA Art. 66)
| Category | Criteria | Protections |
|----------|----------|-------------|
| **Retail** | Default | Full disclosures, suitability, complaints, ADR |
| **Professional** | Qualifies per MiCA Annex II | Reduced disclosures, no suitability test |
| **Eligible Counterparty** | Credit institutions, VASPs, etc. | Minimal protections |

### 2.2 Onboarding Flow
```
1. Registration (email/phone) → 2. KYC/KYB (provider) → 3. Risk Profiling 
→ 4. Client Classification → 5. Suitability (if retail + advice/portfolio) 
→ 6. Terms Acceptance → 7. Account Activation → 8. Ongoing Monitoring
```

### 2.3 Ongoing Monitoring
- **KYC refresh**: Annual (retail), semi-annual (high-risk)
- **Transaction monitoring**: Real-time + batch (see AML Policy)
- **Sanctions screening**: Every transaction, daily batch re-screen
- **PEP/adverse media**: Monthly re-check

---

## 3. Operational Processes

### 3.1 Transaction Lifecycle
```
Initiation → Validation (structure + AML) → Fee Estimation → Signing 
→ Broadcast → Mempool → Inclusion → Finality → Confirmation → Settlement
```

### 3.2 Error & Exception Handling
| Scenario | Process | SLA |
|----------|---------|-----|
| **Failed tx (OOM, nonce)** | Auto-retry with fee bump (RBF) | < 5 min |
| **Stuck tx (mempool eviction)** | Resubmit with higher fee | < 15 min |
| **Double-spend attempt** | Reject, flag, SAR if suspicious | Immediate |
| **Custody key unavailable** | Fallback to multisig quorum | < 1 hr |
| **Partner API down** | Queue, retry with exponential backoff | < 4 hr |

### 3.3 Settlement & Reconciliation
- **On-chain**: GHOSTDAG finality (default 100 blue blocks ≈ ~2 hrs at 1 min/block)
- **Off-chain ledger**: Daily reconciliation vs. DAG UTXO set
- **Fiat settlement**: T+1 via partner (SEPA Instant target)

---

## 4. Technology & Infrastructure

### 4.1 Core Protocol Stack
| Layer | Technology | Redundancy |
|-------|------------|------------|
| **Consensus** | GHOSTDAG (k=3) | Multi-node DAG |
| **P2P** | Kademlia DHT + DNS seeds | 3+ seed nodes |
| **Sync** | Headers-first + payload pruning | Multi-source |
| **RPC** | Line protocol + JSON/HTTP (explorer) | Load balanced |
| **Light Client** | KVLSv1 (SPV) + Merkle proofs | Filter-based sync |

### 4.2 Deployment Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                    EU Region (Primary)                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ Node A   │  │ Node B   │  │ Node C   │  │ Explorer │    │
│  │ (Validator)│  │ (Full)   │  │ (Full)   │  │ (API)    │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
│         │           │           │           │                │
│         └───────────┴───────────┴───────────┘                │
│                         │                                    │
│              ┌──────────▼──────────┐                         │
│              │  Prometheus +       │                         │
│              │  Grafana + Alerting │                         │
│              └─────────────────────┘                         │
└─────────────────────────────────────────────────────────────┘
                         │
         ┌───────────────┼───────────────┐
         ▼               ▼               ▼
   ┌─────────┐     ┌─────────┐     ┌─────────┐
   │ Seed 1  │     │ Seed 2  │     │ Seed 3  │
   │(Hostinger)    │ (VPS)   │     │ (AWS)   │
   └─────────┘     └─────────┘     └─────────┘
```

### 4.3 Security Controls
- **Network**: VPC, security groups, TLS 1.3 everywhere
- **Access**: SSH keys only, bastion host, MFA for console
- **Secrets**: HashiCorp Vault / AWS Secrets Manager
- **Logging**: Structured JSON, centralized (Loki/ELK), 1yr retention
- **Monitoring**: 15 Prometheus alerts + 9 recording rules (armed)

---

## 5. Outsourcing Arrangements (Material)

| Function | Provider | Critical? | Contract Status | Review Cycle |
|----------|----------|-----------|-----------------|--------------|
| **Hot Custody** | Fireblocks/Copper/BitGo | Yes | 📋 Negotiation | Annual |
| **KYC/AML** | Sumsub/Veriff | Yes | 📋 Evaluation | Annual |
| **Travel Rule** | TRISA/OpenVASP | Yes | 📋 Evaluation | Annual |
| **Cloud** | AWS/Hetzner | Yes | 📋 Standard | Annual |
| **Pen Testing** | [Firm] | Yes | 📋 RFP | Per engagement |
| **Legal Counsel** | [MiCA Firm] | Yes | 📋 Retainer | Ongoing |
| **DPO** | [External] | Yes | 📋 Contract | Annual |
| **Audit** | [Big 4 / Crypto] | Yes | 📋 RFP | Annual |

**Outsourcing Policy** (per Art. 30):
- All material outsourcing registered in Outsourcing Register
- Contractual audit rights, data localization (EU), termination clauses
- Concentration risk monitored (no single provider >50% critical functions)

---

## 6. Business Continuity Summary

| Metric | Target | Test Frequency |
|--------|--------|----------------|
| **RTO (Critical)** | 4 hours | Quarterly |
| **RPO (Critical)** | 1 hour | Quarterly |
| **RTO (Non-critical)** | 24 hours | Semi-annual |
| **Backup Integrity** | 100% verified | Monthly |
| **Failover Drill** | Successful | Quarterly |

**Key Dependencies**: Seed nodes, custody partner API, KYC provider, cloud region

---

## 7. Complaints & Dispute Resolution

| Channel | SLA | Escalation |
|---------|-----|------------|
| **In-app / Email** | Acknowledge 24h, resolve 15 business days | CCO → MD |
| **Phone** | Business hours, callback 4h | CCO → MD |
| **ADR** | Croatian Financial Ombudsman / EU ODR | Post-internal exhaustion |

---

## 8. Record Keeping

| Record Type | Retention | Format | Location |
|-------------|-----------|--------|----------|
| **Transaction records** | 5+ years | Immutable (DAG) + indexed DB | Node + archive |
| **KYC/AML files** | 5+ years post-relationship | Encrypted PDF + metadata | Vault + S3 (EU) |
| **Advice/portfolio records** | 5+ years | Structured + PDF | CRM + archive |
| **Complaints** | 5+ years | Case management | Helpdesk + archive |
| **Board/Committee minutes** | 10+ years | Signed PDF | Governance portal |
| **Audit logs (ICT)** | 3+ years | Immutable (append-only) | SIEM |

---

## 9. Fees & Charges Schedule (Indicative)

| Service | Fee | Disclosure |
|---------|-----|------------|
| **Network tx fee** | Dynamic (p90 mempool) | Real-time in app |
| **Custody (retail)** | Free (self-custody light-node) | Terms |
| **Custody (institutional)** | 0.1–0.5% p.a. AUM | Custody agreement |
| **Staking commission** | 5–10% of rewards | Staking terms |
| **FX spread** | 0.5–1.5% | Pre-trade disclosure |
| **Merchant acquiring** | 1–2% + €0.10 | Merchant agreement |
| **API Enterprise** | €500–5000/mo + volume | Enterprise contract |

---

## Document Control

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| v0.1-draft | 2026-09-06 | [Author] | Initial template |
| v1.0-submission | TBD | [Author] | Legal review complete |

---

**Cross-References**:
- `[[Business_Plan]]`
- `[[../04_Custody_Safeguarding/Custody_Policy]]`
- `[[../05_AML_CTF/AML_Policy]]`
- `[[../07_Consumer_Protection/Consumer_Policy]]`
- `[[../08_Market_Abuse/Market_Abuse_Policy]]`
- `[[../12_Contracts_Policies/Terms_of_Service]]`

# Travel Rule Policy — Kovanica Protocol CASP

## 1. Purpose & Regulatory Basis

**Regulation**:
- **Transfer of Funds Regulation (TFR)** — Regulation (EU) 2023/1113, Articles 3–18 (full application to crypto-asset transfers)
- **MiCA Regulation (EU) 2023/1114**, Articles 66–67 (AML obligations for CASPs, explicit TFR cross-reference)
- **FATF Recommendation 16** (2012, updated 2023) — "Travel Rule" for Virtual Assets & VASPs
- **Croatian Act on Prevention of Money Laundering and Terrorist Financing** (Zakon o sprječavanju pranja novca i financiranja terorizma), Articles 33–35 (TFR transposition)
- **EU AML Directive (AMLD6)** — Directive (EU) 2018/843, Article 13 (CDD) & Article 32 (wire transfers)

**Supervisors**:
- **HANFA** — CASP authorization & ongoing supervision (TFR compliance)
- **Ured za sprječavanje pranja novca** (Croatian FIU) — SAR recipient, TFR breach reporting
- **HNB** (Croatian National Bank) — Prudential oversight, operational resilience

**Scope**: All crypto-asset transfer services provided by Kovanica Protocol Ltd. (custodial transfers, non-custodial transfers, exchange-related transfers, OTC transfers) for retail and professional clients.

---

## 2. Scope & Applicability

### 2.1 Threshold
| Regulation | Threshold | Kovanica Application |
|------------|-----------|---------------------|
| **TFR Art. 3(1)** | €0 (no de minimis) | **All transfers** — every crypto-asset transfer triggers Travel Rule obligations |
| **FATF Rec. 16** | $1,000 / €1,000 | **Not applied** — EU TFR €0 threshold is stricter and binding |

### 2.2 Covered Transfers
| Transfer Type | Originator VASP | Beneficiary VASP | Travel Rule Required |
|---------------|-----------------|------------------|---------------------|
| Custodial → Custodial (VASP-to-VASP) | Yes | Yes | **Yes** (full data) |
| Custodial → Self-hosted (unhosted) | Yes | No (natural person) | **Yes** (originator data + beneficiary name/address) |
| Self-hosted → Custodial | No (natural person) | Yes | **Yes** (beneficiary data + originator name/address) |
| Self-hosted → Self-hosted | No | No | **No** (outside VASP scope) |
| Internal (same VASP, same client) | N/A | N/A | **Record only** (no transmission) |
| Internal (same VASP, different clients) | Yes | Yes | **Yes** (full data, internal routing) |

### 2.3 Roles
| Role | Definition | Kovanica Function |
|------|------------|-------------------|
| **Originator VASP** | VASP initiating transfer on behalf of originator | Kovanica when client sends |
| **Beneficiary VASP** | VASP receiving transfer for beneficiary | Kovanica when client receives |
| **Intermediary VASP** | VASP in payment chain (not originator/beneficiary) | Not applicable (direct settlement) |

---

## 3. Required Information — Data Fields

### 3.1 Originator Information (TFR Art. 4 / MiCA Art. 66)

#### 3.1.1 Natural Persons (Retail & Professional)
| Field | TFR Reference | Format | Mandatory | Verification |
|-------|---------------|--------|-----------|--------------|
| **Full Legal Name** | Art. 4(1)(a) | Text (max 140 chars) | **Yes** | CDD-verified (Sumsub) |
| **Account Number** (Wallet Address) | Art. 4(1)(b) | Blockchain address (KVNC: `kvnc...dag`, ETH: `0x...`, BTC: `bc1...`) | **Yes** | On-chain derivation |
| **Address** (Residential) | Art. 4(1)(c) | Structured: Street, City, Postal Code, Country (ISO 3166-1 alpha-2) | **Yes** | CDD-verified (≤3 months) |
| **Official Personal Document Number** | Art. 4(1)(d) | Document type + number (e.g., `PASSPORT:12345678`, `ID_CARD:HR1234567`) | **Yes** | CDD-verified |
| **Customer ID** (Internal) | Art. 4(1)(e) | UUID v4 (Kovanica client ID) | **Yes** | System-generated |
| **Date of Birth** | Art. 4(1)(f) | ISO 8601 (`YYYY-MM-DD`) | **Yes** | CDD-verified |
| **Place of Birth** | Art. 4(1)(f) | City, Country (ISO 3166-1 alpha-2) | **Yes** | CDD-verified |

#### 3.1.2 Legal Entities (Corporate/Institutional)
| Field | TFR Reference | Format | Mandatory | Verification |
|-------|---------------|--------|-----------|--------------|
| **Legal Name** | Art. 4(2)(a) | Text (max 140 chars) | **Yes** | Commercial register |
| **Account Number** (Wallet Address) | Art. 4(2)(b) | Blockchain address | **Yes** | On-chain derivation |
| **Address** (Registered Office) | Art. 4(2)(c) | Structured: Street, City, Postal Code, Country (ISO 3166-1 alpha-2) | **Yes** | Commercial register |
| **LEI** (Legal Entity Identifier) | Art. 4(2)(d) | 20-char alphanumeric (ISO 17442) | **Yes** | GLEIF database |
| **Customer ID** (Internal) | Art. 4(2)(e) | UUID v4 (Kovanica client ID) | **Yes** | System-generated |
| **Registration Number** | Art. 4(2)(d) | Jurisdiction-specific | **Yes** | Commercial register |

> **Note**: For legal entities, LEI is mandatory per TFR Art. 4(2)(d). If LEI not available (e.g., non-EU entity not in GLEIF), registration number + jurisdiction serves as equivalent identifier.

### 3.2 Beneficiary Information (TFR Art. 5 / MiCA Art. 66)

| Field | TFR Reference | Format | Mandatory | Notes |
|-------|---------------|--------|-----------|-------|
| **Full Legal Name** | Art. 5(1)(a) | Text (max 140 chars) | **Yes** | Natural or legal person |
| **Account Number** (Wallet Address) | Art. 5(1)(b) | Blockchain address | **Yes** | Destination address |

> **Note**: Beneficiary address, document number, date/place of birth are **not required** for VASP-to-VASP transfers per TFR Art. 5. Required only for unhosted wallet transfers (Section 8).

### 3.3 Transfer Metadata (TFR Art. 6)
| Field | Format | Mandatory |
|-------|--------|-----------|
| **Transfer Amount** | Decimal (18 dp) + Asset ID (e.g., `KVNC`, `ETH`, `USDC`) | **Yes** |
| **Transfer Date/Time** | ISO 8601 UTC (`YYYY-MM-DDTHH:MM:SSZ`) | **Yes** |
| **Transfer Reference** | UUID v4 (Kovanica transfer ID) | **Yes** |
| **Originator VASP LEI/ID** | LEI or VASP registration number | **Yes** |
| **Beneficiary VASP LEI/ID** | LEI or VASP registration number | **Yes** |

---

## 4. Dual Implementation: IVMS101 + TRP

Kovanica Protocol implements **both** messaging standards to ensure maximum interoperability with counterparties.

### 4.1 IVMS101 (2023) — InterVASP Messaging Standard
| Aspect | Specification |
|--------|---------------|
| **Version** | IVMS101 2023 (latest stable) |
| **Transport** | HTTPS/REST (mutual TLS) + Async message queue (RabbitMQ/Kafka) |
| **Schema** | JSON Schema (strict validation) |
| **Message Types** | `IVMS101_Originator`, `IVMS101_Beneficiary`, `IVMS101_Transfer`, `IVMS101_Acknowledgement`, `IVMS101_Rejection` |
| **Encoding** | UTF-8, Base64 for binary fields |
| **Validation** | Schema validation + business rule engine (Section 7) |

#### 4.1.1 IVMS101 Field Mapping (Originator — Natural Person)
```json
{
  "originator": {
    "naturalPerson": {
      "name": {
        "nameIdentifier": [
          { "primaryIdentifier": "LEGAL_NAME", "name": "John Doe" }
        ]
      },
      "nationalIdentification": [
        { "nationalIdType": "PASSPORT", "nationalIdNumber": "12345678", "countryOfIssue": "HR" }
      ],
      "dateAndPlaceOfBirth": {
        "dateOfBirth": "1990-01-15",
        "placeOfBirth": "Zagreb",
        "countryOfBirth": "HR"
      },
      "address": {
        "addressLine": "Ilica 10",
        "city": "Zagreb",
        "postCode": "10000",
        "country": "HR"
      },
      "customerNumber": "550e8400-e29b-41d4-a716-446655440000"
    },
    "accountNumber": "kvnc1q2w3e4r5t6y7u8i9o0p"
  }
}
```

#### 4.1.2 IVMS101 Field Mapping (Originator — Legal Entity)
```json
{
  "originator": {
    "legalPerson": {
      "name": "Acme Corp d.o.o.",
      "legalForm": "LLC",
      "registration": {
        "registrationId": "12345678901",
        "registrationAuthority": "Croatian Court Register",
        "countryOfRegistration": "HR"
      },
      "lei": "529900ABCDEFGHIJKL12",
      "address": {
        "addressLine": "Ilica 10",
        "city": "Zagreb",
        "postCode": "10000",
        "country": "HR"
      },
      "customerNumber": "550e8400-e29b-41d4-a716-446655440001"
    },
    "accountNumber": "kvnc1q2w3e4r5t6y7u8i9o0p"
  }
}
```

### 4.2 TRP (Travel Rule Protocol) — OpenVASP / TRISA
| Aspect | Specification |
|--------|---------------|
| **Version** | TRP 2.0 (OpenVASP Association) / TRISA 1.1 |
| **Transport** | gRPC (mutual TLS) + HTTPS fallback |
| **Protocol** | Protobuf (`.proto` definitions) |
| **Message Types** | `TravelRuleRequest`, `TravelRuleResponse`, `TravelRuleAck`, `TravelRuleError` |
| **Discovery** | VASP Directory (GLEIF LEI-based) + DNS-based discovery |
| **Validation** | Protobuf schema + business rule engine (Section 7) |

#### 4.2.1 TRP Field Mapping (Originator — Natural Person)
```protobuf
message Originator {
  NaturalPerson natural_person = 1;
  string account_number = 2;  // wallet address
}

message NaturalPerson {
  string legal_name = 1;
  repeated NationalId national_id = 2;
  DateOfBirth date_of_birth = 3;
  PostalAddress address = 4;
  string customer_id = 5;
}

message NationalId {
  string type = 1;      // PASSPORT, ID_CARD, DRIVING_LICENSE
  string number = 2;
  string country = 3;   // ISO 3166-1 alpha-2
}

message DateOfBirth {
  string date = 1;      // YYYY-MM-DD
  string place = 2;     // City
  string country = 3;   // ISO 3166-1 alpha-2
}

message PostalAddress {
  string address_line = 1;
  string city = 2;
  string postal_code = 3;
  string country = 4;   // ISO 3166-1 alpha-2
}
```

### 4.3 Protocol Selection Logic
| Counterparty Capability | Protocol Used | Fallback |
|------------------------|---------------|----------|
| TRP (gRPC) + IVMS101 | **TRP preferred** (lower latency, streaming) | IVMS101 |
| IVMS101 only (REST) | IVMS101 | — |
| TRP only (gRPC) | TRP | — |
| Neither | **Reject transfer** — non-compliant counterparty | Manual process (Section 7) |

> **Implementation Note**: Kovanica Node exposes both endpoints:
> - `POST /api/travel-rule/ivms101` (REST/JSON)
> - `grpc.kovanica.protocol:443` (TRP/gRPC)
> Counterparty capability advertised via VASP Directory (LEI-based).

---

## 5. Copper-Specific Counterparty Onboarding Flow

Copper Technologies (UK) Ltd. ("Copper") is Kovanica Protocol's **primary custody partner** and **primary TRP/IVMS101 counterparty**. This section defines the dedicated onboarding and operational flow.

### 5.1 Copper Profile
| Attribute | Value |
|-----------|-------|
| **Legal Entity** | Copper Technologies (UK) Ltd. |
| **LEI** | `213800X5Q9LZ8Y7V6W43` |
| **Regulator** | FCA (UK) — Cryptoasset Business Registration |
| **VASP Status** | Registered VASP (FCA Register) |
| **Protocols Supported** | TRP (gRPC primary), IVMS101 (REST fallback) |
| **Custody Model** | MPC-based institutional custody (ClearLoop) |
| **Settlement** | On-chain (KVNC, ETH, BTC, ERC-20) + Off-chain (ClearLoop) |

### 5.2 Onboarding Process

```mermaid
flowchart TD
    A[Initiate Copper Onboarding] --> B[Exchange Legal/Compliance Contacts]
    B --> C[Execute Data Processing Agreement (DPA)]
    C --> D[Exchange Technical Specifications]
    D --> E[TRP/gRPC Certificate Exchange]
    E --> F[IVMS101 Endpoint Configuration]
    F --> G[Test Message Exchange (Sandbox)]
    G --> H{Interoperability Tests Pass?}
    H -->|No| I[Remediate & Retest]
    I --> G
    H -->|Yes| J[Production Certificate Exchange]
    J --> K[Go-Live Approval (CCO)]
    K --> L[Add to VASP Directory]
    L --> M[Monitoring Dashboard Activation]
```

### 5.3 Onboarding Checklist

| Step | Description | Owner | Timeline | Evidence |
|------|-------------|-------|----------|----------|
| 1 | Legal entity verification (LEI, FCA register) | Compliance | Day 1 | Register screenshots |
| 2 | DPA execution (GDPR Art. 28) | Legal/CCO | Day 1–3 | Signed DPA |
| 3 | Technical spec exchange (API specs, protobuf, schemas) | Engineering | Day 1–5 | Spec documents |
| 4 | mTLS certificate generation & exchange | Engineering/Security | Day 3–7 | Cert fingerprints |
| 5 | Sandbox environment provisioning | Engineering | Day 5–10 | Sandbox URLs |
| 6 | IVMS101 test vectors (10+ scenarios) | Engineering/Compliance | Day 7–14 | Test results log |
| 7 | TRP test vectors (10+ scenarios) | Engineering/Compliance | Day 7–14 | Test results log |
| 8 | Error handling & retry logic validation | Engineering | Day 10–14 | Test results log |
| 9 | Sanctions screening integration test | Compliance | Day 10–14 | Screening logs |
| 10 | Go-live approval (CCO sign-off) | CCO | Day 14+ | Approval record |
| 11 | Production monitoring alerts configured | Engineering/Compliance | Day 14+ | Alert rules |
| 12 | Quarterly re-certification scheduled | Compliance | Ongoing | Calendar invite |

### 5.4 Operational Procedures (Copper-Specific)

#### 5.4.1 Outbound Transfer (Kovanica → Copper)
| Step | Action | Protocol | Timeout | Retry |
|------|--------|----------|---------|-------|
| 1 | Client initiates withdrawal to Copper custody address | — | — | — |
| 2 | Kovanica constructs Travel Rule payload (originator = client, beneficiary = Copper) | TRP (preferred) | 5s | 3× (exponential backoff) |
| 3 | Send `TravelRuleRequest` to Copper TRP endpoint | TRP/gRPC | 5s | 3× |
| 4 | Receive `TravelRuleResponse` (ACK/NACK) | TRP/gRPC | — | — |
| 5 | If TRP fails → fallback to IVMS101 REST | IVMS101/HTTPS | 10s | 2× |
| 6 | On ACK: Broadcast on-chain transaction | — | — | — |
| 7 | On NACK/Timeout: Hold transfer, alert Compliance | — | — | Manual |

#### 5.4.2 Inbound Transfer (Copper → Kovanica)
| Step | Action | Protocol | Timeout | Retry |
|------|--------|----------|---------|-------|
| 1 | Copper sends `TravelRuleRequest` to Kovanica TRP endpoint | TRP/gRPC | 5s | N/A (server) |
| 2 | Kovanica validates payload (Section 7) | — | <1s | — |
| 3 | If valid → `TravelRuleResponse` (ACK) + credit client | TRP/gRPC | — | — |
| 4 | If invalid → `TravelRuleError` (reject code) | TRP/gRPC | — | — |
| 5 | If TRP unavailable → IVMS101 webhook | IVMS101/HTTPS | 10s | N/A |

#### 5.4.3 ClearLoop Off-Chain Settlement
| Aspect | Treatment |
|--------|-----------|
| **Travel Rule Required** | **Yes** — ClearLoop transfers are "transfers" per TFR Art. 3 |
| **Protocol** | TRP (real-time) — IVMS101 not used for ClearLoop |
| **Beneficiary Data** | Copper provides full beneficiary data (their client) |
| **Settlement Finality** | Travel Rule ACK required before ClearLoop credit |
| **Reconciliation** | Daily batch match: Travel Rule messages ↔ ClearLoop ledger |

### 5.5 Copper-Specific Escalation Matrix
| Issue | Level 1 (Ops) | Level 2 (Engineering) | Level 3 (CCO/Legal) | SLA |
|-------|---------------|----------------------|---------------------|-----|
| TRP connection failure | Restart client, check certs | Debug gRPC logs, cert rotation | Notify Copper, invoke fallback | 15 min |
| IVMS101 schema validation error | Log & alert | Fix payload construction | — | 1 hour |
| Data mismatch (originator/beneficiary) | Hold transfer, request clarification | — | Joint investigation with Copper | 4 hours |
| Sanctions hit on Copper transfer | Auto-block, SAR | — | CCO + Legal notify regulators | Immediate |
| ClearLoop reconciliation break | Match ledgers | Root cause analysis | CCO sign-off on resolution | 24 hours |

---

## 6. Missing / Incomplete Information Procedures

### 6.1 Originator VASP Obligations (TFR Art. 7)
| Scenario | Action | Timeline | Escalation |
|----------|--------|----------|------------|
| **Missing mandatory field** (originator) | **Reject transfer** — do not initiate on-chain | Immediate | Client notification (generic) |
| **Incomplete field** (e.g., address missing postal code) | **Request completion** from client via app/email | 24 hours | Auto-reject if not resolved in 5 business days |
| **Unverifiable data** (document number format invalid) | **Request re-verification** via Sumsub | 24 hours | Enhanced CDD if repeated |
| **Sanctions match on originator** | **Block + SAR** | Immediate | CCO + FIU |

### 6.2 Beneficiary VASP Obligations (TFR Art. 8)
| Scenario | Action | Timeline | Escalation |
|----------|--------|----------|------------|
| **Missing mandatory field** (beneficiary name/address) | **Reject transfer** — return funds to originator VASP | 24 hours | Originator VASP notification |
| **Incomplete field** | **Request completion** from originator VASP | 5 business days | Reject if not resolved |
| **Unverifiable beneficiary** (address format invalid) | **Request clarification** from originator VASP | 5 business days | Reject if not resolved |
| **Sanctions match on beneficiary** | **Block + SAR** | Immediate | CCO + FIU |

### 6.3 Inter-VASP Communication (Missing Info)
| Message Type | IVMS101 | TRP | Purpose |
|--------------|---------|-----|---------|
| **Request for Information (RFI)** | `IVMS101_RequestInformation` | `TravelRuleRequest` (type=RFI) | Request missing/incomplete data |
| **Response to RFI** | `IVMS101_InformationResponse` | `TravelRuleResponse` (type=INFO) | Provide requested data |
| **Rejection** | `IVMS101_Rejection` | `TravelRuleError` | Reject transfer with reason code |

### 6.4 Rejection Reason Codes (TFR Art. 9)
| Code | Description | Action |
|------|-------------|--------|
| `MISSING_ORIGINATOR_DATA` | Mandatory originator field absent | Originator VASP must resend |
| `MISSING_BENEFICIARY_DATA` | Mandatory beneficiary field absent | Beneficiary VASP must request |
| `INVALID_FORMAT` | Field format invalid (address, LEI, date) | Resend with corrected format |
| `SANCTIONS_MATCH` | Party on sanctions list | Block — no retry |
| `UNHOSTED_WALLET_THRESHOLD` | Unhosted wallet > €1,000 without EDD | Require EDD or reject |
| `COUNTERPARTY_NON_COMPLIANT` | Counterparty VASP not TRP/IVMS101 capable | Manual process only |
| `DUPLICATE_TRANSFER` | Duplicate transfer reference | Investigate — do not retry |

---

## 7. Unhosted Wallet Handling (Self-Hosted Wallets)

### 7.1 Definition
**Unhosted Wallet** (TFR Recital 15): A wallet where the private key is controlled solely by the natural/legal person (not a VASP). Includes hardware wallets, software wallets, paper wallets, smart contract wallets (non-custodial).

### 7.2 Thresholds & Requirements
| Transfer Direction | Amount | Travel Rule Data Required | Enhanced Due Diligence (EDD) |
|--------------------|--------|---------------------------|------------------------------|
| **VASP → Unhosted** | Any (€0) | Originator full data + Beneficiary name & address | **≥ €1,000**: EDD mandatory |
| **Unhosted → VASP** | Any (€0) | Beneficiary full data + Originator name & address | **≥ €1,000**: EDD mandatory |
| **Unhosted ↔ Unhosted** | N/A | Not applicable (no VASP) | N/A |

### 7.3 EDD Requirements for Unhosted Wallets (≥ €1,000)
| Requirement | Description | Verification |
|-------------|-------------|--------------|
| **Proof of Control** | Client signs message with wallet private key | Cryptographic verification (nonce challenge) |
| **Source of Funds** | Declaration + supporting docs (bank statement, mining receipt, etc.) | Document review |
| **Beneficial Owner Confirmation** | Self-declaration of beneficial ownership | Cross-check with CDD |
| **Purpose of Transfer** | Detailed narrative | Consistency with client profile |
| **Senior Approval** | Director B (CCO) sign-off | Recorded approval |

### 7.4 Unhosted Wallet Verification Flow
```mermaid
flowchart TD
    A[Client initiates transfer to/from unhosted wallet] --> B{Amount ≥ €1,000?}
    B -->|No| C[Standard Travel Rule: name + address only]
    B -->|Yes| D[Trigger EDD Workflow]
    D --> E[Request Proof of Control (signature challenge)]
    E --> F[Request Source of Funds Declaration]
    F --> G[Request Purpose of Transfer]
    G --> H[CCO Review & Approval]
    H --> I{Approved?}
    I -->|Yes| J[Execute Transfer + Travel Rule]
    I -->|No| K[Reject Transfer]
    C --> J
```

### 7.5 Address Ownership Verification (Technical)
| Method | Assets Supported | Implementation |
|--------|------------------|----------------|
| **Message Signing (EIP-191 / BIP-137)** | ETH, EVM-compatible, KVNC | `personal_sign` / `eth_sign` with nonce |
| **Bitcoin Message Signing (BIP-137)** | BTC | `signmessage` with nonce |
| **Solana Wallet Adapter Sign** | SOL, SPL | `wallet.signMessage` |
| **KVNC Native** | KVNC | `kovanica-cli sign-message --address <addr> --nonce <nonce>` |

> **Nonce Format**: `KOVANICA_TRAVEL_RULE_<timestamp>_<random16>` — single-use, 10-min expiry.

### 7.6 Record Keeping for Unhosted Wallets
| Record | Retention | Storage |
|--------|-----------|---------|
| Proof of control (signature + nonce) | 10 years | Encrypted database |
| Source of funds declaration | 10 years | Encrypted database |
| EDD approval record | 10 years | Encrypted database (restricted access) |
| Transfer metadata | 10 years | Immutable ledger + database |

---

## 8. Data Protection (GDPR Compliance)

### 8.1 Legal Basis (GDPR Art. 6)
| Processing Activity | Legal Basis | TFR Reference |
|---------------------|-------------|---------------|
| Originator/beneficiary data collection | **Art. 6(1)(c)** — Legal obligation (TFR) | TFR Art. 4, 5 |
| Data transmission to counterparty VASP | **Art. 6(1)(c)** — Legal obligation (TFR) | TFR Art. 7, 8 |
| Data retention (10 years) | **Art. 6(1)(c)** — Legal obligation (AMLD6 Art. 40) | AMLD6 Art. 40 |
| Sanctions screening | **Art. 6(1)(c)** + **Art. 9(2)(g)** — Substantial public interest | EU Sanctions Regs |
| Blockchain analytics | **Art. 6(1)(f)** — Legitimate interest (AML) | FATF Rec. 15 |

### 8.2 Data Minimization (GDPR Art. 5(1)(c))
| Principle | Implementation |
|-----------|----------------|
| **Purpose Limitation** | Travel Rule data used **only** for TFR compliance, sanctions screening, SAR — never for marketing |
| **Data Minimization** | Only mandatory TFR fields collected (Section 3) — no optional fields |
| **Storage Limitation** | 10-year retention (Section 8.4) — auto-deletion after expiry |
| **Access Control** | Role-based: Compliance (full), Engineering (pseudonymized), Support (masked) |
| **Pseudonymization** | Client ID (UUID) used in analytics — no direct identifiers in monitoring systems |

### 8.3 International Transfers (GDPR Ch. V)
| Transfer | Mechanism | Safeguards |
|----------|-----------|------------|
| **EU → EU VASP** | No transfer (same jurisdiction) | — |
| **EU → UK (Copper)** | **Adequacy Decision** (EU 2021/680) | DPA + Standard Contractual Clauses (backup) |
| **EU → US VASP** | **Standard Contractual Clauses** (2021/914) + Transfer Impact Assessment | DPA + Supplementary measures |
| **EU → Other Third Country** | SCCs + TIA + Supplementary measures | Case-by-case CCO approval |

### 8.4 Retention Schedule
| Data Category | Retention Period | Deletion Method |
|---------------|------------------|-----------------|
| Travel Rule message payloads (IVMS101/TRP) | 10 years from transfer date | Cryptographic erasure (key deletion) |
| Originator/beneficiary personal data | 10 years from end of relationship | Cryptographic erasure |
| Sanctions screening logs | 10 years | Cryptographic erasure |
| EDD records (unhosted wallets) | 10 years | Cryptographic erasure |
| Audit logs (access, modifications) | 10 years | Append-only — no deletion |
| Backups | 10 years (aligned) | Backup rotation policy |

### 8.5 Data Subject Rights (GDPR Ch. III)
| Right | Travel Rule Context | Procedure |
|-------|---------------------|-----------|
| **Access (Art. 15)** | Client requests copy of their Travel Rule data | Provide within 30 days (redact counterparty data) |
| **Rectification (Art. 16)** | Correct inaccurate data | Update + notify counterparty VASP if transmitted |
| **Erasure (Art. 17)** | **Not applicable** — legal obligation override (Art. 17(3)(b)) | Inform client of retention requirement |
| **Restriction (Art. 18)** | Pending verification dispute | Restrict processing until resolved |
| **Portability (Art. 20)** | Not applicable — not automated decision-making | N/A |
| **Objection (Art. 21)** | **Not applicable** — legal obligation override | Inform client |

### 8.6 Data Processing Agreement (DPA) Requirements
All counterparty VASPs (including Copper) must execute a DPA covering:
- GDPR Art. 28 processor obligations
- Purpose limitation: Travel Rule compliance only
- Security measures (encryption in transit/rest, access controls)
- Sub-processor management (prior written consent)
- Data breach notification (72 hours)
- Audit rights (annual)
- International transfer safeguards

---

## 9. Quality Assurance & Testing

### 9.1 Interoperability Testing Program

#### 9.1.1 Test Categories
| Category | Frequency | Scope | Success Criteria |
|----------|-----------|-------|------------------|
| **Protocol Conformance** | Quarterly | IVMS101 schema, TRP protobuf | 100% valid messages |
| **Counterparty Integration** | Per counterparty onboarding + annual | End-to-end with each VASP | 100% ACK rate, <5s latency |
| **Error Handling** | Quarterly | All rejection codes (Section 6.4) | Correct error codes, no data loss |
| **Load/Performance** | Semi-annual | 10× peak volume | <2s p99 latency, 0% message loss |
| **Failover/Resilience** | Semi-annual | TRP→IVMS101 fallback, cert rotation | <30s failover, zero data loss |
| **Sanctions Screening** | Monthly | Real-time + batch | 0 false negatives on test vectors |

#### 9.1.2 Test Vectors (Minimum Set)
| Vector ID | Description | Protocol | Expected Result |
|-----------|-------------|----------|-----------------|
| TR-001 | Valid originator (natural person) → VASP | TRP + IVMS101 | ACK |
| TR-002 | Valid originator (legal entity) → VASP | TRP + IVMS101 | ACK |
| TR-003 | Missing originator address | TRP + IVMS101 | REJECT (`MISSING_ORIGINATOR_DATA`) |
| TR-004 | Invalid LEI format | TRP + IVMS101 | REJECT (`INVALID_FORMAT`) |
| TR-005 | Sanctioned originator | TRP + IVMS101 | REJECT (`SANCTIONS_MATCH`) |
| TR-006 | Unhosted wallet ≥ €1,000 without EDD | TRP + IVMS101 | REJECT (`UNHOSTED_WALLET_THRESHOLD`) |
| TR-007 | Duplicate transfer reference | TRP + IVMS101 | REJECT (`DUPLICATE_TRANSFER`) |
| TR-008 | TRP timeout → IVMS101 fallback | Both | ACK via IVMS101 |
| TR-009 | Copper ClearLoop settlement | TRP | ACK + on-chain/off-chain match |
| TR-010 | Large payload (max fields) | TRP + IVMS101 | ACK |

### 9.2 Monitoring & Alerting

#### 9.2.1 Key Metrics
| Metric | Target | Alert Threshold | Dashboard |
|--------|--------|-----------------|-----------|
| **TRP Success Rate** | ≥ 99.9% | < 99.5% (5 min) | Grafana: `travel_rule_trp_success_rate` |
| **IVMS101 Success Rate** | ≥ 99.9% | < 99.5% (5 min) | Grafana: `travel_rule_ivms101_success_rate` |
| **End-to-End Latency (p99)** | < 2s | > 5s (5 min) | Grafana: `travel_rule_latency_p99` |
| **Fallback Rate (TRP→IVMS101)** | < 0.1% | > 1% (1 hour) | Grafana: `travel_rule_fallback_rate` |
| **Rejection Rate** | < 0.5% | > 2% (1 hour) | Grafana: `travel_rule_rejection_rate` |
| **Missing Info Requests** | < 10/day | > 50/day | Grafana: `travel_rule_rfi_count` |
| **Sanctions Hits** | 0 (expected) | ≥ 1 | Grafana: `travel_rule_sanctions_hits` + PagerDuty |

#### 9.2.2 Log Retention
| Log Type | Retention | Format |
|----------|-----------|--------|
| Travel Rule message payloads (full) | 10 years | Encrypted JSON Lines (S3/GCS) |
| Protocol metadata (timing, errors, retries) | 3 years | Structured JSON (Elasticsearch) |
| Metrics time-series | 2 years | Prometheus TSDB |
| Audit trail (access, config changes) | 10 years | Append-only (immutable) |

### 9.3 Incident Response
| Severity | Definition | Response Time | Escalation |
|----------|------------|---------------|------------|
| **SEV-1** | Travel Rule completely down (both protocols) | 15 min | CCO + Engineering Lead + Copper |
| **SEV-2** | One protocol down, fallback active | 1 hour | Engineering Lead |
| **SEV-3** | Elevated error rate (>2% rejections) | 4 hours | Compliance + Engineering |
| **SEV-4** | Single counterparty issue | 24 hours | Compliance |

---

## 10. Cross-References

| Policy | Reference | Relationship |
|--------|-----------|--------------|
| **AML/CTF Policy** | `00_AML_Policy.md` | Parent policy — Travel Rule is a control within AML framework |
| **Sanctions Policy** | `03_Sanctions_Policy.md` | Sanctions screening integrated in Travel Rule flow (Section 6) |
| **Custody Policy** | `02_Ops/01_Custody_Policy.md` | Custody transfers trigger Travel Rule; Copper ClearLoop procedures |
| **Suspicious Activity Reporting** | `04_Suspicious_Activity_Reporting.md` | Travel Rule mismatches → SAR triggers |
| **Sumsub Integration Policy** | `04_Sumsub_Integration_Policy.md` | Originator data sourced from Sumsub CDD |
| **Data Protection Policy** | `05_Data_Protection_Policy.md` | GDPR compliance for Travel Rule data (Section 8) |
| **Outsourcing Register** | `02_Ops/02_Outsourcing_Register.md` | Copper, Sumsub, blockchain analytics as critical outsourcing |

---

## 11. Document Control

| Element | Detail |
|---------|--------|
| **Document ID** | `COMP-MICA-AML-TRP-001` |
| **Version** | 1.0 |
| **Classification** | Confidential — Regulatory / AML / Travel Rule |
| **Owner** | Director B (CCO) / AML Officer |
| **Approved By** | Management Board |
| **Effective Date** | [Upon CASP Authorization] |
| **Review Cycle** | Semi-annual (aligned with TFR regulatory updates) or upon trigger |
| **Triggers for Review** | TFR/MiCA amendment, FATF guidance update, Copper protocol change, SEV-1 incident, new counterparty onboarding |
| **Retention** | 10 years (policy versions) |
| **Distribution** | Management Board, Compliance, Engineering, Legal, Copper (under NDA) |
| **Related Documents** | `00_AML_Policy.md`, `03_Sanctions_Policy.md`, `02_Ops/01_Custody_Policy.md`, `04_Suspicious_Activity_Reporting.md`, `04_Sumsub_Integration_Policy.md`, `05_Data_Protection_Policy.md` |

### Revision History
| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-09-06 | Compliance Team | Initial version for CASP application |

---

**End of Document**

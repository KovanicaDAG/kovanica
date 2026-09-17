# Travel Rule IVMS101 & TRP Field Mapping

**Document ID:** KOV-POL-AML-005
**Version:** 1.0
**Classification:** Confidential
**Owner:** MLRO / CTO
**Review Cycle:** Annual (or upon protocol update)
**Status:** Approved

---

## 1. Purpose & Regulatory Basis

This document specifies the technical field mapping between **TFR (EU) 2023/1113** requirements, **IVMS101 (2023)** messaging standard, and **TRP (Travel Rule Protocol)** transport layer for Kovanica Protocol CASP.

| Regulation / Standard | Version | Role |
|----------------------|---------|------|
| TFR (Transfer of Funds Regulation) | (EU) 2023/1113 | Legal requirement (Art. 4–6) |
| FATF Recommendation 16 | 2012 (updated 2021) | International standard |
| IVMS101 | 2023 (v1.0.0) | Message format standard |
| TRP | 2023 (v1.0) | Transport protocol (REST + Protobuf) |
| MiCA | Art. 66–67 | CASP obligations |
| Croatian AML Act | Zakon o sprječavanju pranja novca | National implementation |

**Dual-Protocol Strategy:** Kovanica implements **both** IVMS101 (messaging) and TRP (transport) for maximum counterparty coverage. Protocol selection per counterparty capability.

---

## 2. IVMS101 Field Mapping Tables

### 2.1 Originator — Natural Person (TFR Art. 4(1)(a))

| TFR Requirement | IVMS101 Element (Originator) | Kovanica Field | Mandatory | Format / Validation |
|-----------------|------------------------------|----------------|-----------|---------------------|
| Name | `originator.person.name.nameIdentifier[0].primaryIdentifier` | `originator.name` | **YES** | Max 140 chars, Latin/Cyrillic |
| | `originator.person.name.nameIdentifier[0].secondaryIdentifier` | `originator.name_secondary` | No | Max 140 chars |
| Account Number (Wallet Address) | `originator.accountNumber` | `originator.wallet_address` | **YES** | Blockchain address format (KVNC: `kvnc...dag` or hex) |
| Address | `originator.person.address.addressLine1` | `originator.address.line1` | **YES** | Max 70 chars |
| | `originator.person.address.addressLine2` | `originator.address.line2` | No | Max 70 chars |
| | `originator.person.address.city` | `originator.address.city` | **YES** | Max 35 chars |
| | `originator.person.address.country` | `originator.address.country` | **YES** | ISO 3166-1 alpha-2 |
| | `originator.person.address.postCode` | `originator.address.postal_code` | No | Max 16 chars |
| Official Personal Document Number | `originator.person.nationalIdNumber` | `originator.id_number` | **YES** | Country-specific format |
| Customer ID | `originator.person.customerIdNumber` | `originator.customer_id` | **YES** | Internal: `KOV-{YYYYMMDD}-{SEQ}` |
| Date of Birth | `originator.person.dateOfBirth` | `originator.dob` | **YES** | ISO 8601 (YYYY-MM-DD) |
| Place of Birth | `originator.person.placeOfBirth` | `originator.pob` | **YES** | City, Country (ISO 3166-1 alpha-2) |

### 2.2 Originator — Legal Entity (TFR Art. 4(1)(b))

| TFR Requirement | IVMS101 Element (Originator) | Kovanica Field | Mandatory | Format / Validation |
|-----------------|------------------------------|----------------|-----------|---------------------|
| Name | `originator.legalEntity.name` | `originator.entity_name` | **YES** | Max 140 chars |
| Account Number (Wallet Address) | `originator.accountNumber` | `originator.wallet_address` | **YES** | Blockchain address format |
| Address | `originator.legalEntity.address.addressLine1` | `originator.address.line1` | **YES** | Max 70 chars |
| | `originator.legalEntity.address.city` | `originator.address.city` | **YES** | Max 35 chars |
| | `originator.legalEntity.address.country` | `originator.address.country` | **YES** | ISO 3166-1 alpha-2 |
| | `originator.legalEntity.address.postCode` | `originator.address.postal_code` | No | Max 16 chars |
| LEI / Customer ID | `originator.legalEntity.lei` OR `originator.legalEntity.customerIdNumber` | `originator.lei` / `originator.customer_id` | **YES** (one) | LEI: 20-char alphanumeric; Customer ID: internal format |
| Registration Number | `originator.legalEntity.registrationNumber` | `originator.registration_number` | **YES** | Country-specific |
| Registration Country | `originator.legalEntity.registrationCountry` | `originator.registration_country` | **YES** | ISO 3166-1 alpha-2 |

### 2.3 Beneficiary — Natural Person (TFR Art. 4(2)(a))

| TFR Requirement | IVMS101 Element (Beneficiary) | Kovanica Field | Mandatory | Format / Validation |
|-----------------|-------------------------------|----------------|-----------|---------------------|
| Name | `beneficiary.person.name.nameIdentifier[0].primaryIdentifier` | `beneficiary.name` | **YES** | Max 140 chars |
| Account Number (Wallet Address) | `beneficiary.accountNumber` | `beneficiary.wallet_address` | **YES** | Blockchain address format |

*Note: TFR requires only name + account for beneficiary. Additional fields collected for EDD.*

### 2.4 Beneficiary — Legal Entity (TFR Art. 4(2)(b))

| TFR Requirement | IVMS101 Element (Beneficiary) | Kovanica Field | Mandatory | Format / Validation |
|-----------------|-------------------------------|----------------|-----------|---------------------|
| Name | `beneficiary.legalEntity.name` | `beneficiary.entity_name` | **YES** | Max 140 chars |
| Account Number (Wallet Address) | `beneficiary.accountNumber` | `beneficiary.wallet_address` | **YES** | Blockchain address format |

---

### 2.5 Transaction Details (Both Parties)

| Field | IVMS101 Element | Kovanica Field | Mandatory | Format |
|-------|-----------------|----------------|-----------|--------|
| Amount | `transaction.amount` | `tx.amount` | **YES** | Decimal string (18 decimals max) |
| Asset / Currency | `transaction.currency` | `tx.asset` | **YES** | Asset code (KVNC, EURC, etc.) |
| Timestamp | `transaction.timestamp` | `tx.timestamp` | **YES** | ISO 8601 UTC (YYYY-MM-DDTHH:mm:ssZ) |
| Blockchain Tx Hash | `transaction.blockchainTransactionId` | `tx.hash` | **YES** | 64-char hex (BLAKE3) |
| Network | `transaction.network` | `tx.network` | **YES** | `kovanica-mainnet` / `kovanica-testnet` |
| Transfer Type | `transaction.transferType` | `tx.type` | **YES** | `exchange`, `transfer`, `execution` |

---

## 3. TRP Message Flows

### 3.1 Counterparty Discovery

```text
┌─────────────┐     1. GET /vasp/directory     ┌──────────────────┐
│  Originator │──────────────────────────────▶│  VASP Directory  │
│    VASP     │◀──────────────────────────────│  (TRP/GLEIF)     │
└─────────────┘     2. VASP List + Endpoints   └──────────────────┘
       │
       │ 3. Resolve beneficiary VASP by wallet address prefix / domain
       ▼
┌─────────────┐     4. GET /.well-known/trp     ┌──────────────────┐
│  Originator │──────────────────────────────▶│  Beneficiary VASP│
│    VASP     │◀──────────────────────────────│  (TRP Endpoint)  │
└─────────────┘     5. TRP Config (version,    └──────────────────┘
                    protocols, public key)
```

### 3.2 Pre-Transaction Verification (TRP `POST /transactions/verify`)

```text
Originator VASP                          Beneficiary VASP
     │                                           │
     │  POST /transactions/verify                │
     │  {                                        │
     │    "originator": {...},                   │
     │    "beneficiary": {...},                  │
     │    "transaction": {...}                   │
     │  }                                        │
     │─────────────────────────────────────────▶│
     │                                           │
     │  200 OK                                   │
     │  {                                        │
     │    "status": "verified",                  │
     │    "beneficiaryVerified": true,           │
     │    "referenceId": "trp-abc123"            │
     │  }                                        │
     │◀─────────────────────────────────────────│
     │                                           │
```

**Request Payload (IVMS101-wrapped in TRP envelope):**
```json
{
  "protocol": "IVMS101",
  "version": "1.0.0",
  "messageId": "msg-20260115-000001",
  "timestamp": "2026-01-15T10:30:00Z",
  "originatorVasp": "did:web:kovanica.protocol",
  "beneficiaryVasp": "did:web:copper.co",
  "payload": {
    "originator": { ... IVMS101 originator object ... },
    "beneficiary": { ... IVMS101 beneficiary object ... },
    "transaction": { ... IVMS101 transaction object ... }
  }
}
```

**Response Codes:**
| Code | Meaning | Action |
|------|---------|--------|
| `verified` | Beneficiary confirmed, proceed | Execute on-chain |
| `rejected` | Beneficiary unknown / sanctions / policy | Abort, notify originator |
| `pending` | Manual review required | Wait for callback / poll |
| `error` | Technical error | Retry with backoff |

### 3.3 Post-Transaction Confirmation (TRP `POST /transactions/confirm`)

```text
Originator VASP                          Beneficiary VASP
     │                                           │
     │  POST /transactions/confirm               │
     │  {                                        │
     │    "referenceId": "trp-abc123",           │
     │    "blockchainTxHash": "0xabc...",        │
     │    "blockHeight": 12345,                  │
     │    "timestamp": "2026-01-15T10:31:00Z",   │
     │    "status": "settled"                    │
     │  }                                        │
     │─────────────────────────────────────────▶│
     │                                           │
     │  200 OK                                   │
     │  { "status": "acknowledged" }             │
     │◀─────────────────────────────────────────│
     │                                           │
```

### 3.4 Error Handling & Fallback

| Error Scenario | TRP Response | Fallback |
|----------------|--------------|----------|
| Beneficiary VASP unreachable | Timeout / 5xx | Retry 3x (1m, 5m, 15m) → IVMS101 email |
| Beneficiary rejects | `rejected` | Abort, notify originator, log |
| Invalid payload | `error` (400) | Log, alert engineering, manual review |
| Version mismatch | `error` (426) | Negotiate version, retry |
| Signature verification fail | `error` (401) | Reject, security alert |

**IVMS101 Email Fallback:** If TRP unavailable, send IVMS101 JSON via encrypted email (PGP) to beneficiary VASP compliance address.

---

## 4. Copper-Specific Flows

### 4.1 Copper as Primary Counterparty

| Parameter | Value |
|-----------|-------|
| **VASP DID** | `did:web:copper.co` |
| **TRP Endpoint** | `https://api.copper.co/trp/v1` |
| **IVMS101 Email** | `compliance@copper.co` (PGP key: `0xABCDEF...`) |
| **Supported Protocols** | TRP v1.0 (REST + Protobuf), IVMS101 2023 |
| **Test Environment** | `https://test-api.copper.co/trp/v1` |

### 4.2 Copper Onboarding Checklist

| Step | Action | Owner | Verification |
|------|--------|-------|--------------|
| 1 | Exchange VASP DIDs & TRP endpoints | Compliance | Mutual `.well-known/trp` resolution |
| 2 | Exchange PGP keys for email fallback | Security | Key fingerprint verified out-of-band |
| 3 | Configure shared watchlists (sanctions, PEP) | MLRO | Test screening hit on test vector |
| 4 | Execute TRP interoperability test suite | Engineering | All 10 test vectors pass |
| 5 | Execute IVMS101 email fallback test | Compliance | Email received, parsed, acknowledged |
| 6 | Configure ClearLoop settlement integration | Operations | Test settlement round-trip |
| 7 | Sign Data Processing Addendum | Legal | DPA executed |
| 8 | Production go-live approval | Management Board | Signed off |

### 4.3 Copper Operational Procedures

| Procedure | Trigger | Action |
|-----------|---------|--------|
| **Pre-trade verification** | Every outbound transfer to Copper | TRP `verify` → wait `verified` |
| **Post-trade confirmation** | On-chain settlement detected | TRP `confirm` with tx hash |
| **Sanctions hit on Copper side** | Copper webhook / API alert | Immediate freeze, SAR if needed |
| **ClearLoop settlement** | Batch settlement window | Net settlement via Copper ClearLoop |
| **Dispute resolution** | Mismatched confirmation | Escalation to Copper compliance + Kovanica MLRO |

---

## 5. Unhosted Wallet Handling (Self-Hosted)

### 5.1 TFR Requirements (Art. 5–6)

| Threshold | Requirement |
|-----------|-------------|
| **All transfers** | Collect originator info (full), beneficiary name + wallet address |
| **≥ €1,000** | Enhanced verification of beneficiary wallet control |

### 5.2 Kovanica Implementation

| Step | Description |
|------|-------------|
| 1 | Client initiates withdrawal to external address |
| 2 | System checks if address is in known VASP directory (TRP) |
| 3 | **If VASP found** → Standard TRP flow (Section 3) |
| 4 | **If unhosted** → Require self-declaration form: |
|   | - Beneficiary name (matches originator KYC or new) |
|   | - Wallet address |
|   | - Proof of control (sign message with private key) |
|   | - Source of funds declaration (if ≥ €1,000) |
| 5 | If ≥ €1,000: Blockchain analytics risk score (Chainalysis / TRM / internal) |
| 6 | If risk score > threshold → Manual review (Tier 3 EDD) |
| 7 | Proceed with on-chain transfer; log as unhosted in Travel Rule log |

### 5.3 Self-Declaration Form Fields

| Field | Mandatory | Validation |
|-------|-----------|------------|
| Beneficiary full name | **YES** | Match against sanctions/PEP |
| Wallet address | **YES** | Valid Kovanica address format |
| Signature (message: "Kovanica withdrawal authorization {timestamp}") | **YES** | Ed25519 verify against address |
| Source of funds description | If ≥ €1,000 | Free text + supporting doc upload |
| Relationship to originator | No | Free text |

---

## 6. Data Validation Rules

| Field | Format | Length | Character Set | Example |
|-------|--------|--------|---------------|---------|
| Name (person) | Free text | 1–140 | Latin, Cyrillic, spaces, hyphens, apostrophes | `Ivan Horvat` |
| Name (entity) | Free text | 1–140 | Latin, Cyrillic, spaces, punctuation | `Kovanica Protocol d.o.o.` |
| Wallet address | KVNC / Hex | 32–64 | Base58 (KVNC) or hex | `kvnc1abc...dag` / `0xabc...` |
| Address line | Free text | 1–70 | Latin, Cyrillic, numbers, punctuation | `Ilica 10` |
| City | Free text | 1–35 | Latin, Cyrillic | `Zagreb` |
| Country | ISO 3166-1 alpha-2 | 2 | Uppercase | `HR` |
| Postal code | Alphanumeric | 1–16 | Alphanumeric, spaces, hyphens | `10000` |
| ID number | Country-specific | 1–50 | Alphanumeric | `12345678901` |
| LEI | ISO 17442 | 20 | Alphanumeric | `549300ABCDEFGHIJKL12` |
| Customer ID | Internal | 22 | `KOV-YYYYMMDD-NNNNNN` | `KOV-20260115-000042` |
| Date of birth | ISO 8601 | 10 | `YYYY-MM-DD` | `1990-05-15` |
| Place of birth | `City, CC` | 2–70 | Latin, Cyrillic | `Zagreb, HR` |
| Amount | Decimal string | 1–38 | Digits, optional decimal point | `1234.56` |
| Asset code | Alphanumeric | 3–12 | Uppercase, numbers | `KVNC`, `EURC` |
| Timestamp | ISO 8601 UTC | 20–24 | `YYYY-MM-DDTHH:mm:ssZ` | `2026-01-15T10:30:00Z` |
| Tx hash | Hex | 64 | Lowercase hex | `abcdef123456...` |
| Network | Enum | - | `kovanica-mainnet`, `kovanica-testnet` | `kovanica-mainnet` |

---

## 7. Error Codes & Handling

### 7.1 Standard TRP Error Codes

| Code | HTTP | Message | Retry | Action |
|------|------|---------|-------|--------|
| `INVALID_PAYLOAD` | 400 | Payload validation failed | No | Log, alert engineering |
| `UNAUTHORIZED` | 401 | Signature verification failed | No | Security alert |
| `FORBIDDEN` | 403 | VASP not authorized | No | Check VASP directory |
| `NOT_FOUND` | 404 | Beneficiary not found | No | Reject, notify originator |
| `VERSION_MISMATCH` | 426 | Unsupported protocol version | Yes (negotiate) | Upgrade/downgrade |
| `RATE_LIMITED` | 429 | Too many requests | Yes (backoff) | Exponential backoff |
| `INTERNAL_ERROR` | 500 | Server error | Yes (3x) | Alert on-call |
| `SERVICE_UNAVAILABLE` | 503 | Maintenance / overload | Yes (3x) | Fallback to email |

### 7.2 Kovanica Custom Error Codes

| Code | Meaning | Resolution |
|------|---------|------------|
| `KOV_UNHOSTED_EDD_REQUIRED` | Transfer ≥ €1,000 to unhosted wallet | Collect self-declaration + proof of control |
| `KOV_SANCTIONS_MATCH` | Counterparty on sanctions list | Reject, freeze, SAR |
| `KOV_PEP_MATCH` | Counterparty is PEP/RCA | Enhanced review, senior approval |
| `KOV_BLOCKCHAIN_RISK_HIGH` | Analytics score > threshold | Manual review, possible reject |
| `KOV_INSUFFICIENT_TRAVEL_RULE_DATA` | Missing mandatory fields | Request missing data (RFI) |

---

## 8. Testing & Certification

### 8.1 IVMS101 Conformance Test Vectors

| Test ID | Description | Expected Result |
|---------|-------------|-----------------|
| `IVMS-001` | Valid natural person originator + beneficiary | `verified` |
| `IVMS-002` | Valid legal entity originator + beneficiary | `verified` |
| `IVMS-003` | Missing mandatory originator field | `error` (400) |
| `IVMS-004` | Invalid wallet address format | `error` (400) |
| `IVMS-005` | Sanctions list match (test vector) | `rejected` |
| `IVMS-006` | PEP match (test vector) | `pending` → manual review |
| `IVMS-007` | Unhosted wallet ≥ €1,000 | `verified` with EDD flag |
| `IVMS-008` | Maximum field lengths | `verified` |
| `IVMS-009` | Special characters in names (Cyrillic) | `verified` |
| `IVMS-010` | Round-trip: verify → confirm | `acknowledged` |

### 8.2 TRP Interoperability Test Vectors

| Test ID | Description | Expected Result |
|---------|-------------|-----------------|
| `TRP-001` | Counterparty discovery via `.well-known/trp` | Endpoint resolved |
| `TRP-002` | Pre-transaction verify (happy path) | `verified` |
| `TRP-003` | Pre-transaction verify (beneficiary unknown) | `rejected` |
| `TRP-004` | Post-transaction confirm | `acknowledged` |
| `TRP-005` | Retry on 5xx (3 attempts) | Success on 2nd/3rd |
| `TRP-006` | Fallback to IVMS101 email on TRP failure | Email sent, acknowledged |
| `TRP-007` | Signature verification (valid) | `verified` |
| `TRP-008` | Signature verification (invalid) | `error` (401) |
| `TRP-009` | Rate limiting handling | Backoff, retry, success |
| `TRP-010` | Version negotiation (v1.0 ↔ v1.1) | Negotiated, success |

### 8.3 Production Go-Live Criteria

- [ ] All IVMS101 test vectors pass
- [ ] All TRP test vectors pass (with Copper test environment)
- [ ] Email fallback tested end-to-end
- [ ] Unhosted wallet flow tested (≥ €1,000 and < €1,000)
- [ ] Sanctions/PEP screening integrated in verify flow
- [ ] Monitoring dashboards live (success rate, latency, errors)
- [ ] Incident runbook tested (tabletop)
- [ ] Management Board sign-off

---

## 9. Monitoring & Metrics

| Metric | Target | Alert Threshold | Dashboard |
|--------|--------|-----------------|-----------|
| **TRP Verify Success Rate** | > 99% | < 98% (5m) | Grafana: `trp_verify_success` |
| **TRP Verify Latency (p95)** | < 2s | > 5s (5m) | Grafana: `trp_verify_latency_p95` |
| **TRP Confirm Success Rate** | > 99.5% | < 99% (5m) | Grafana: `trp_confirm_success` |
| **Email Fallback Rate** | < 0.1% | > 1% (1h) | Grafana: `trp_email_fallback_rate` |
| **Counterparty Coverage** | > 95% of volume | < 90% (daily) | Grafana: `trp_counterparty_coverage` |
| **Error Rate by Code** | < 0.5% each | > 1% (1h) | Grafana: `trp_errors_by_code` |
| **Unhosted Wallet EDD Rate** | Tracked | Spike > 2x baseline | Grafana: `trp_unhosted_edd` |

---

## 10. Cross-References

| Document | Reference |
|----------|-----------|
| `01_Travel_Rule_Policy.md` | Policy framework, governance, unhosted wallet thresholds |
| `00_AML_Policy.md` | Risk ratings, CDD tiers, ongoing monitoring |
| `02_Sanctions_Policy.md` | Sanctions screening in verify flow |
| `03_Suspicious_Activity_Reporting.md` | SAR triggers from Travel Rule failures |
| `00_Custody_Policy.md` | Copper custody integration, ClearLoop |
| `01_Custody_Partner_Due_Diligence.md` | Copper counterparty assessment |

---

## 11. Document Control

| Field | Value |
|-------|-------|
| **Document ID** | KOV-POL-AML-005 |
| **Version** | 1.0 |
| **Classification** | Confidential |
| **Owner** | MLRO / CTO |
| **Approved By** | Management Board |
| **Approval Date** | [DATE] |
| **Next Review** | [DATE + 1 year] |
| **Distribution** | Management Board, MLRO, CTO, Compliance, Engineering |

### Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-15 | Compliance Team | Initial version for MiCA CASP application |

---

## Appendix A: IVMS101 JSON Schema Reference (Originator Natural Person)

```json
{
  "originator": {
    "person": {
      "name": {
        "nameIdentifier": [
          {
            "primaryIdentifier": "Ivan",
            "secondaryIdentifier": "Horvat"
          }
        ]
      },
      "address": {
        "addressLine1": "Ilica 10",
        "addressLine2": "",
        "city": "Zagreb",
        "country": "HR",
        "postCode": "10000"
      },
      "nationalIdNumber": "12345678901",
      "customerIdNumber": "KOV-20260115-000042",
      "dateOfBirth": "1990-05-15",
      "placeOfBirth": "Zagreb, HR"
    },
    "accountNumber": "kvnc1abcdefghijklmnopqrstuvwxyz123456dag"
  }
}
```

---

## Appendix B: TRP Envelope Structure

```protobuf
message TravelRuleMessage {
  string protocol = 1;           // "IVMS101"
  string version = 2;            // "1.0.0"
  string message_id = 3;         // UUID
  string timestamp = 4;          // RFC3339
  string originator_vasp = 5;    // DID
  string beneficiary_vasp = 6;   // DID
  bytes payload = 7;             // IVMS101 JSON (UTF-8)
  bytes signature = 8;           // Ed25519 over payload
}
```

---

*End of Document*
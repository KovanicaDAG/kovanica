# Kovanica Mainnet — Legal Jurisdiction Analysis Memo (Template)

> **Status**: DRAFT — For legal counsel review
> **Prepared**: 2026-09-26
> **Classification**: ATTORNEY-CLIENT PRIVILEGED

---

## Executive Summary

Kovanica Protocol plans mainnet genesis Q4 2026. This memo analyzes legal/regulatory requirements for launching **KVNC** (native token, 90.2M max supply) across target jurisdictions.

**Key questions**:
1. Token classification: Security? Commodity? Utility? Currency?
2. AML/KYC obligations for: protocol, validators, exchanges, wallets
3. Securities law: Genesis distribution, treasury vaults, founder premine
4. Corporate structure: Foundation, DAO, or other entity
5. Ongoing compliance: Reporting, audits, travel rule

---

## 1. Token Classification Analysis

### 1.1 KVNC Characteristics
| Attribute | Value | Regulatory Implication |
|-----------|-------|------------------------|
| Native to L1 blockchain | Yes | Commodity-like (CFTC) |
| Fixed max supply | 90.2M KVNC | Scarcity = investment contract risk |
| Founder premine | 0.2M KVNC (0.22%) | Promoter compensation → Howey factor |
| Treasury | 10M KVNC (11.1%) | Managed by multisig → common enterprise |
| Curve emission | 80M KVNC over ~20 years | Ongoing distribution → expectation of profit |
| Fee burn | 75% of fees | Deflationary → value accrual mechanism |
| Staking/PoA rewards | None (PoW mainnet) | No yield from protocol |

### 1.2 Howey Test Assessment (US)

| Prong | Analysis | Risk |
|-------|----------|------|
| Investment of money | Users buy KVNC on secondary markets | ✅ Yes |
| Common enterprise | Treasury + founder + protocol devs | ⚠️ Likely |
| Expectation of profit | Deflationary + scarcity narrative | ⚠️ Likely |
| Efforts of others | Core team + validators + governance | ⚠️ Likely |

**Preliminary**: **High risk of security classification under US law** without structural mitigations.

### 1.3 Mitigation Strategies
- [ ] **Decentralized launch**: No central team controls >50% supply at genesis
- [ ] **No promoter promises**: No roadmap tokens, no "buy now" marketing
- [ ] **Utility-first**: KVNC required for fees, multisig, HTLC, vaults, assets
- [ ] **Foundation structure**: Non-profit (Cayman/Guernsey/Swiss) holds treasury
- [ ] **Governance tokens separate**: If needed, issue distinct governance token

---

## 2. Target Jurisdictions

| Jurisdiction | Priority | Key Regulations | Notes |
|--------------|----------|-----------------|-------|
| **United States** | High | SEC (securities), CFTC (commodity), FinCEN (AML), State MTL | Largest market; highest risk |
| **European Union** | High | MiCA (2024), AMLR, TFR | MiCA "asset-referenced token" vs "utility token" |
| **Singapore** | Medium | PSA, SFA, MAS guidelines | Clear framework; requires VASP license |
| **Switzerland** | Medium | FINMA, DLT Act | Foundation-friendly; "payment token" category |
| **UAE (ADGM/DIFC)** | Medium | FSRA, DFSA | Growing hub; clear VASP regime |
| **Cayman Islands** | Medium | VASP Act, Foundation Law | Common for token foundations |
| **British Virgin Islands** | Low | VASP Act | Alternative to Cayman |

---

## 3. Specific Obligations by Category

### 3.1 Securities Law
- [ ] **Registration or exemption**: Reg D, Reg S, Reg A+?
- [ ] **Accredited investor restrictions** for genesis allocation?
- [ ] **Lock-up agreements** for founder/team/treasury?
- [ ] **SAFT vs direct sale** — SAFT may itself be security
- [ ] **Ongoing reporting** if security (10-K, 10-Q equivalents)

### 3.2 AML/KYC (FinCEN / FATF Travel Rule)
- [ ] **VASP classification**: Protocol = not VASP; Wallet/Exchange = VASP
- [ ] **Validator obligations**: Block production = not VASP (likely)
- [ ] **Travel Rule**: $1,000 threshold for VASP-to-VASP
- [ ] **OFAC screening**: RPC endpoints, explorer, seed nodes
- [ ] **Suspicious Activity Reports**: Thresholds, timing

### 3.3 Commodity/Derivatives (CFTC)
- [ ] KVNC as "commodity" — self-certification?
- [ ] Futures/options on KVNC — exchange registration
- [ ] Manipulation/enforcement risk

### 3.4 Consumer Protection
- [ ] Disclosure requirements (whitepaper = prospectus?)
- [ ] Refund rights / cooling-off periods
- [ ] Marketing restrictions (no "guaranteed returns")

### 3.5 Tax
- [ ] Token classification for tax: property, currency, security?
- [ ] Genesis allocation: income vs capital gains
- [ ] Staking rewards (if any): income at receipt
- [ ] Fee burn: taxable event?

---

## 4. Corporate/Entity Structure

### 4.1 Recommended: Multi-Entity Structure

```
┌─────────────────────────────────────────────────────────────┐
│  Kovanica Foundation (Cayman/Guernsey/Swiss non-profit)    │
│  • Holds treasury multisig keys                             │
│  • Grants ecosystem funding                                 │
│  • Owns IP/trademarks                                       │
└─────────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│ Core Dev Co   │   │ Validator Co  │   │ OpCo (Services)│
│ (DevAgreement)│   │ (Staking/Infra)│   │ (Explorer,API) │
│ Delaware LLC  │   │ Local entities │   │ Delaware LLC  │
└───────────────┘   └───────────────┘   └───────────────┘
```

### 4.2 Key Agreements Needed
- [ ] **Development Agreement** — Foundation → Core Dev Co
- [ ] **Validator Agreements** — Foundation ↔ Validators
- [ ] **IP Assignment** — Contributors → Foundation
- [ ] **Treasury Management Policy** — Multisig governance
- [ ] **Grant Agreements** — Foundation → Ecosystem projects

---

## 5. Genesis Distribution Compliance

| Allocation | Amount | Compliance Approach |
|------------|--------|---------------------|
| Founder premine | 0.2M KVNC | Lock-up 1yr (CSV), vesting 4yr, no transfer |
| Treasury | 10M KVNC | 10 vaults, staggered unlock, multisig governance |
| Curve emission | 80M KVNC | Algorithmic, no central control |
| Community/launch | 0 | Fair launch — no pre-sale, no ICO |

**Critical**: No public sale, no SAFT, no private placement. Pure fair launch + treasury.

---

## 6. Action Items for Counsel

| # | Task | Priority | Due |
|---|------|----------|-----|
| 1 | Formal token classification memo (US, EU, SG, CH) | 🔴 Critical | 2026-10-10 |
| 2 | Foundation jurisdiction recommendation + setup | 🔴 Critical | 2026-10-15 |
| 3 | Genesis distribution legal opinion | 🔴 Critical | 2026-10-15 |
| 4 | VASP/non-VASP analysis for each participant type | 🟡 High | 2026-10-20 |
| 5 | Travel Rule compliance design | 🟡 High | 2026-10-20 |
| 6 | IP assignment + contributor agreements | 🟡 High | 2026-10-25 |
| 7 | Ongoing compliance calendar (filings, audits) | 🟢 Medium | 2026-11-01 |

---

## 7. Budget Estimate

| Service | Notes |
|---------|-------|
| Token classification (4 jurisdictions) | Scope-dependent |
| Foundation setup (Cayman/Guernsey) | Scope-dependent |
| Genesis distribution opinion | Scope-dependent |
| VASP/Travel Rule design | Scope-dependent |
| Agreements/templates | Scope-dependent |

---

## 8. Next Steps

1. **Engage counsel** (specialized in crypto/token law)
2. **Execute engagement letter** with scope above
3. **Provide**: Whitepaper, RFC-006, AUTHORITY_KEY_CEREMONY.md, GENESIS_PLAYBOOK.md
4. **Weekly check-ins** until deliverables complete

---

*This template should be reviewed and completed by qualified crypto/securities counsel in each target jurisdiction.*
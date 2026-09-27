# Governance Election Materials

## 1. Election Announcement Template

**Subject:** Kovanica Mainnet Genesis — Authority Election (7 Signers, 7 Validators, 3 Auditors)

---

The Kovanica Protocol is approaching mainnet genesis in Q4 2026. Per the Authority Key Ceremony specification, we are conducting a governance election to select:

| Role | Count | Term | Responsibility |
|------|-------|------|----------------|
| **Genesis Signers** | 7 (5-of-7 threshold) | One-time | Sign genesis transaction (multisig) |
| **Initial Validators** | 7 (5-of-7 threshold) | 6 months | Produce blocks during PoA bootstrap (T+0 to T+14) |
| **Witness Auditors** | 3 | Ceremony only | Observe key generation, attest to process |

**Election Timeline:**
- **Nominations Open:** 2026-09-26
- **Nominations Close:** 2026-10-10
- **Voting Period:** 2026-10-11 to 2026-10-15
- **Results Announced:** 2026-10-16
- **Key Generation:** 2026-10-17 to 2026-10-24
- **Ceremony (T-0):** 2026-10-25

**Eligibility:**
- **Signers/Validators:** Proven technical competence, geographic diversity, no single entity controls >1 key
- **Auditors:** Recognized security firms with blockchain audit experience

**How to Participate:**
1. Review `AUTHORITY_KEY_CEREMONY.md` for full requirements
2. Submit nomination via multisig vote at: [governance.kovanica.online]
3. Vote using KVNC-weighted voting (1 KVNC = 1 vote)

**Key Documents:**
- `GENESIS_PLAYBOOK.md` — Full ceremony flow
- `AUTHORITY_KEY_CEREMONY.md` — Key generation & distribution
- `PREREQUISITES_TRACKER.md` — Current status tracker

---

## 2. Candidate Nomination Form

```
NOMINATION FOR: [ ] Genesis Signer  [ ] Initial Validator  [ ] Witness Auditor

CANDIDATE INFORMATION:
- Name/Organization: _________________________
- Contact (Signal/Email): _________________________
- Geographic Jurisdiction: _________________________
- Relevant Experience: _________________________
  (e.g., "Ran testnet validator since 2024", "Audited 3 L1 protocols", etc.)
- Conflict of Interest Disclosure: _________________________
  (Any relationship with other nominees, Kovanica team, etc.)
- Commitment Statement: _________________________
  (e.g., "I commit to 6-month validator term, 99.9% uptime, key rotation at T+14d")

NOMINATED BY: _________________________
SECONDED BY: _________________________
DATE: _________________________
```

---

## 3. Voting Procedure (Multisig-Based)

### Voting Smart Contract / Off-Chain Process

**Option A: On-Chain (Preferred)**
1. Deploy voting multisig (KVP-101, threshold = majority of eligible voters)
2. Each eligible voter = 1 vote (or 1 KVNC = 1 vote if token-weighted)
3. Vote = transaction signing `CandidatePubkey + Role`
4. Results tallied on-chain at close

**Option B: Off-Chain (Fallback)**
1. Coordinator collects signed votes via secure channel
2. Each vote = `VoterPubkey || CandidatePubkey || Role || Timestamp || Signature`
3. Results published with all signatures for verification

### Vote Format
```json
{
  "election": "kovanica-mainnet-genesis-2026",
  "voter": "voter_pubkey_hex",
  "candidate": "candidate_pubkey_hex",
  "role": "genesis_signer" | "validator" | "auditor",
  "timestamp": "2026-10-12T14:30:00Z",
  "signature": "ed25519_signature_hex"
}
```

### Tally Rules
- **Genesis Signers:** Top 7 by vote count (minimum 5 votes each to win)
- **Validators:** Top 7 by vote count (minimum 5 votes each)
- **Auditors:** Top 3 by vote count
- Ties broken by: 1) geographic diversity score, 2) earliest nomination

### Verification
All votes published with signatures → anyone can verify tally independently.

---

## 4. Election Timeline Checklist

| Date | Milestone | Owner | Status |
|------|-----------|-------|--------|
| 2026-09-26 | Announcement published | Governance Chair | ⬜ |
| 2026-09-26 | Nomination form live | Governance Chair | ⬜ |
| 2026-10-10 | Nominations close | Governance Chair | ⬜ |
| 2026-10-11 | Voting opens | Governance Chair | ⬜ |
| 2026-10-15 | Voting closes (23:59 UTC) | Governance Chair | ⬜ |
| 2026-10-16 | Tally + results announced | Governance Chair | ⬜ |
| 2026-10-17 | Key generation begins | Elected participants | ⬜ |
| 2026-10-24 | Key generation complete | Elected participants | ⬜ |
| 2026-10-25 | Genesis ceremony (T-0) | All participants | ⬜ |

---

## 5. Post-Election: Key Generation Instructions

### For Genesis Signers (7)
```bash
# On air-gapped machine:
cargo run --release --example generate_authority_keys -- \
  --out-dir ./genesis-keys --count 1 --threshold 1
# Output: genesis-signer-N.pub (share), genesis-signer-N.key (keep offline)
# Send .pub to coordinators
```

### For Validators (7)
```bash
# On production server (or air-gapped then transfer):
cargo run --release --example generate_authority_keys -- \
  --out-dir /etc/kovanica/authority-keys --count 1 --threshold 1 --slot-duration 120000
# Output: authority-1.env (KOVANICA_AUTHORITY_KEY), authorities.conf (pubkey)
# Send authorities.conf to coordinators
```

### For Coordinators
```bash
# Collect all 7 signer pubkeys
cat genesis-signer-*.pub > genesis-signers.pub

# Compute multisig
kovanica-cli multisig --threshold 5 --pubkeys genesis-signers.pub --output genesis-multisig.hash

# Collect all 7 validator pubkeys
cat validator-*.pub > validators.pub
# Build authority set config
```

---

## 6. Witness Auditor Attestation Template

```
ATTESTATION: Kovanica Mainnet Genesis Ceremony

Auditor: [Name/Firm]
Date: [ISO 8601]
Phase Observed: [Key Generation / Multisig Computation / Genesis Signing / Network Boot]

Observed:
- [ ] 7 genesis signers generated keys offline (air-gapped)
- [ ] 7 validators generated authority keys
- [ ] Multisig hash computed: <hash>
- [ ] All participants verified their keys present
- [ ] Genesis block signed by ≥5 signers
- [ ] 3 seed nodes booted with identical genesis
- [ ] Block production at ~120s slots verified

Signature: [PGP signature of this document]

Published to: GitHub, transparency log, mailing list
```
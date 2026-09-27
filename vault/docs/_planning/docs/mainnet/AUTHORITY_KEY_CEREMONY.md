# Kovanica Mainnet Authority Key Ceremony

> **Status**: DRAFT — For governance review
> This document specifies the key generation, distribution, and governance process for mainnet genesis authorities and signers.

---

## 1. Roles & Key Types

| Key Type | Purpose | Holder | Count | Threshold |
|----------|---------|--------|-------|-----------|
| **Genesis signer** | Signs genesis transaction (multisig) | Governance-elected | 7 | 5 of 7 |
| **Initial validator (PoA)** | Signs blocks during PoA bootstrap | Reputable operators | 7 | 5 of 7 |
| **Witness auditor** | Observes ceremony, publishes attestation | Security firms | 3 | N/A |
| **Treasury vault** | Time-locked RFC-005 vaults | Multisig governance | 10 | 5 of 7 |

**Separation rule**: Genesis signers ≠ Initial validators. Different keys, different people.

---

## 2. Genesis Signer Ceremony (5-of-7 Multisig)

### 2.1 Participant Selection

Governance elects 7 signers meeting:
- Geographic distribution (≥3 jurisdictions)
- No entity controls >1 key
- Public reputation, no anonymous keys
- Commitment to 6-month availability

### 2.2 Key Generation (Offline, Air-Gapped)

Each signer on dedicated offline machine:

```bash
# 1. Generate entropy (dice rolls / hardware RNG)
# 2. Create Ed25519 keypair
kovanica-cli keygen --offline --output genesis-signer-N.key

# 3. Extract public key
kovanica-cli pubkey --input genesis-signer-N.key --output genesis-signer-N.pub

# 4. Verify key format
kovanica-cli verify-key --input genesis-signer-N.pub
```

**Output**: `genesis-signer-N.pub` (32 bytes, 64 hex chars) — safe to share.

**Secret**: `genesis-signer-N.key` — NEVER leaves offline machine.

### 2.3 Public Key Collection

Coordinators collect all 7 pubkeys → `genesis-signers.pub` (one per line).

All signers verify:
```bash
cat genesis-signers.pub
# Each signer confirms their key is present and correct
```

### 2.4 Multisig Script Hash

```bash
kovanica-cli multisig --threshold 5 --pubkeys genesis-signers.pub --output genesis-multisig.hash
```

Produces: P2SH script hash (32 bytes) for genesis transaction output.

All signers independently verify the hash matches.

---

## 3. Initial Validator Ceremony (7 Validators, 5-of-7 Threshold)

### 3.1 Participant Selection

7 operators meeting:
- Proven testnet operation (≥6 months)
- Geographic/jurisdictional diversity
- Hardware: dedicated server, 99.9% uptime SLA
- Commitment to key rotation at T+14 days (PoA→PoW)

### 3.2 Key Generation

Each validator on their production server (or air-gapped, then transferred via encrypted channel):

```bash
# On production server (or offline then scp)
cargo run --release --example generate_authority_keys \
  --out-dir /etc/kovanica/authority-keys \
  --count 1 --threshold 1 --slot-duration 120000
```

Outputs:
- `authority-1.env` (KOVANICA_AUTHORITY_KEY, mode 0600)
- `authorities.conf` (public key)

Validator sends **only** `authorities.conf` to coordinators.

### 3.3 Authority Set Assembly

Coordinators concatenate 7 public keys → `mainnet-validators.conf`:

```bash
KOVANICA_AUTHORITIES=pk1,pk2,pk3,pk4,pk5,pk6,pk7
KOVANICA_AUTHORITY_THRESHOLD=5
KOVANICA_SLOT_DURATION=120000
```

Published to all validators and seed nodes.

---

## 4. Treasury Vault Ceremony (10 Vaults, 1M KVNC Each)

### 4.1 Vault Script Generation

Using RFC-005 time-lock vaults with staggered unlocks:

| Vault | Unlock Height | Approx Date | Purpose |
|-------|---------------|-------------|---------|
| 1 | +52,560 | ~1 year | Operational reserve |
| 2 | +105,120 | ~2 years | Ecosystem grants |
| 3 | +157,680 | ~3 years | Development fund |
| ... | ... | ... | ... |
| 10 | +525,600 | ~10 years | Long-term treasury |

Script template (per vault):
```rust
// RFC-005 vault script
OP_IF
  <unlock_height> OP_CHECKSEQUENCEVERIFY OP_DROP
  <multisig_5of7_pubkeys> OP_CHECKMULTISIG
OP_ELSE
  <emergency_multisig_5of7_pubkeys> OP_CHECKMULTISIG
OP_ENDIF
```

### 4.2 Vault Key Management

- Same 5-of-7 multisig as genesis signers (or designated treasury multisig)
- Vault seeds derived from `KOVANICA_TREASURY_SEED` (64 hex chars, set at genesis)
- Emergency multisig: separate 5-of-7 set for recovery

---

## 5. Ceremony Timeline

| Phase | Date | Action | Output |
|-------|------|--------|--------|
| T-30d | Governance vote | Elect 7 genesis signers, 7 validators | Public announcement |
| T-14d | Key gen | Signers/validators generate keys offline | Pubkeys collected |
| T-7d | Code freeze | Tag `mainnet-genesis-rc1`, reproducible builds | SHA256 hashes published |
| T-1d | Dry run | Test genesis construction on testnet | Verified genesis hash |
| T-0 | Ceremony | Coordinated genesis signing | `genesis-block.json` |
| T+0 | Network boot | Seeds + validators start | Live mainnet |
| T+14d | PoA→PoW | Governance vote, key burn | PoW consensus |
| T+180d | Vault 1 unlock | First treasury spend | Verified on-chain |

---

## 6. Verification & Attestation

### 6.1 Witness Auditor Role

3 independent auditors:
- Observe key generation (remote screen share or in-person)
- Verify multisig hash computation
- Attest: "I witnessed N key generations, all offline, no key material leaked"

### 6.2 Public Attestation Format

```markdown
# Attestation: Kovanica Mainnet Genesis Ceremony

Auditor: [Name/Firm]
Date: [ISO 8601]
Phase: [Key Gen / Multisig / Genesis Signing / Network Boot]

Observed:
- [ ] 7 genesis signers generated keys offline
- [ ] 7 validators generated authority keys
- [ ] Multisig hash: <hash>
- [ ] Genesis block hash: <hash>
- [ ] All participants verified their keys

Signature: [PGP sig of this document]
```

Published to: GitHub, transparency log, mailing list.

---

## 7. Key Rotation Policy (Post-Genesis)

| Key Type | Rotation | Process |
|----------|----------|---------|
| Genesis signer | One-time | Destroyed after genesis |
| Validator (PoA) | T+14d (PoA→PoW) | Burn PoA keys, enable PoW |
| Validator (PoW) | N/A | No authority keys in PoW |
| Treasury | Per vault unlock | Multisig spend, new vault created |

---

## 8. Emergency Procedures

| Scenario | Response |
|----------|----------|
| Signer key compromised pre-ceremony | Replace signer, regenerate key |
| Validator key compromised during PoA | Coordinator calls emergency rotation (multisig) |
| >2 validators offline | Pause ceremony, recruit replacements |
| Genesis hash mismatch | Abort, rollback, new ceremony |

---

## 9. References

- **GENESIS_PLAYBOOK.md** — Overall ceremony flow
- **RFC-006** — Tokenomics (supply, emission, fees)
- **KVP-101** — Multisig (P2SH, threshold signatures)
- **KVP-105** — Time-lock vaults (RFC-005, CSV)
- **TESTNET_AUTHORITY_KEYS.md** — Testnet ceremony record (burned)

---

*Version: 1.0*  
*Author: Kovanica Core Team*  
*Classification: CONFIDENTIAL — Governance review required*
# Authority Key Ceremony Kit

> **Complete runbook for offline key generation, multisig setup, and ceremony execution.**

---

## 1. Prerequisites Checklist

### Hardware Requirements
- [ ] 1 dedicated air-gapped laptop per participant (no network, no Bluetooth, no WiFi)
- [ ] 1 USB drive per participant (for pubkey export only, formatted FAT32)
- [ ] 1 coordinator laptop (online, for collecting pubkeys only)
- [ ] 1 printer (for paper backups)
- [ ] Tamper-evident bags for key storage

### Software Requirements
- [ ] Rust toolchain (1.75+) installed on all laptops
- [ ] `kovanica-cli` built from `fix/secret-scan-working-tree` branch
- [ ] GPG/PGP installed for attestations
- [ ] `sha256sum` for binary verification

### Binary Verification
```bash
# On coordinator machine:
cd kovanica/protocol
cargo build --release --locked -p kovanica-cli
sha256sum target/release/kovanica-cli > kovanica-cli.sha256

# Distribute binary + hash to all participants
# Each participant verifies:
sha256sum -c kovanica-cli.sha256
```

---

## 2. Genesis Signer Key Generation (7 participants, offline)

### Step-by-Step (Per Signer)

```bash
# 1. Verify binary
sha256sum -c kovanica-cli.sha256
# Must show: kovanica-cli: OK

# 2. Generate Ed25519 keypair (offline)
kovanica-cli keygen --offline --output genesis-signer-N.key
# Output: genesis-signer-N.key (32 bytes, 64 hex chars)

# 3. Extract public key
kovanica-cli pubkey --input genesis-signer-N.key --output genesis-signer-N.pub
# Output: genesis-signer-N.pub (32 bytes, 64 hex chars)

# 4. Verify key format
kovanica-cli verify-key --input genesis-signer-N.pub
# Output: OK

# 5. Export public key to USB
cp genesis-signer-N.pub /media/usb/

# 6. Secure private key
# Option A: Print paper backup (QR code + hex), store in tamper-evident bag
# Option B: Encrypt with passphrase, store on encrypted USB
# NEVER: copy to networked machine, cloud, email, chat

# 7. Verify public key on coordinator machine
# Coordinator collects all 7 .pub files
```

### Verification (All Signers Together)
```bash
# Coordinator combines all pubkeys
cat genesis-signer-*.pub > genesis-signers.pub

# Compute multisig hash
kovanica-cli multisig --threshold 5 --pubkeys genesis-signers.pub --output genesis-multisig.hash

# All 7 signers independently verify:
cat genesis-signers.pub
# Each confirms their key is present and correct

kovanica-cli verify-multisig --threshold 5 --pubkeys genesis-signers.pub --hash genesis-multisig.hash
# All confirm hash matches
```

---

## 3. Validator Key Generation (7 participants, on production servers)

### Step-by-Step (Per Validator)

```bash
# 1. On production server (or offline then secure transfer)
mkdir -p /etc/kovanica/authority-keys
chmod 700 /etc/kovanica/authority-keys

# 2. Generate authority key
cargo run --release --example generate_authority_keys -- \
  --out-dir /etc/kovanica/authority-keys \
  --count 1 --threshold 1 --slot-duration 120000

# Output:
# - authority-1.env (KOVANICA_AUTHORITY_KEY, mode 0600) — KEEP SECURE
# - authorities.conf (public key only) — safe to share

# 3. Export public key
cp authorities.conf /media/usb/

# 4. Verify
cat authorities.conf
# Should show: KOVANICA_AUTHORITIES=<pubkey>, KOVANICA_AUTHORITY_THRESHOLD=1, KOVANICA_SLOT_DURATION=120000
```

### Validator Key Security
- `authority-1.env` mode 0600, owned by root
- Never commit to git, never share in chat/email
- Backup: print paper copy, store in tamper-evident bag
- Server: dedicated hardware, 99.9% uptime SLA, monitoring

---

## 4. Multisig Setup & Genesis Transaction

### Coordinator Actions

```bash
# 1. Collect all 7 genesis signer pubkeys
# 2. Collect all 7 validator pubkeys

# 3. Build genesis transaction
cat > genesis-tx.json << 'EOF'
{
  "network": "kovanica-mainnet",
  "timestamp": "2026-10-25T00:00:00Z",
  "outputs": [
    {
      "amount": "2000000000000000",
      "script": "<founder-pubkey> OP_CHECKSIG",
      "timelock": 525600
    },
    {
      "amount": "1000000000000000",
      "script": "<vault-1-script>",
      "vault_id": 1
    },
    ...
    {
      "amount": "1000000000000000",
      "script": "<vault-10-script>",
      "vault_id": 10
    }
  ],
  "multisig": "genesis-multisig.hash"
}
EOF

# 3. Distribute genesis-tx.json to all 7 signers
# Each signer signs offline:
kovanica-cli sign --multisig --input genesis-tx.json --key genesis-signer-N.key --output genesis-sig-N.sig

# 4. Collect all 7 signatures
# Coordinator aggregates (need ≥5):
kovanica-cli multisig-combine --threshold 5 --signatures genesis-sig-*.sig --output genesis-tx-signed.json

# 5. Verify final genesis
kovanica-cli verify-genesis --input genesis-tx-signed.json --expected-hash <expected-genesis-hash>
```

---

## 5. Ceremony Day Checklist (T-0)

| Time | Action | Participants | Verification |
|------|--------|--------------|--------------|
| T-2h | All coordinators online, comms tested | Coordinators | Signal group ✅ |
| T-1h | All 7 signers confirm readiness | Signers | Signal ✅ |
| T-30m | All 7 validators confirm server readiness | Validators | SSH + health check ✅ |
| T-15m | Witness auditors join observation | Auditors | Screen share ✅ |
| T-0 | Coordinators distribute genesis block | Coordinators | Hash broadcast ✅ |
| T+0 | 3 seed nodes boot with genesis | Coordinators | `api/head` genesis match ✅ |
| T+2m | 7 validators connect & sync | Validators | P2P connected ✅ |
| T+5m | First block produced (slot 0) | All | Height=1 ✅ |
| T+30m | Stable production (5+ blocks) | All | Consecutive slots ✅ |
| T+1h | Public announcement | Coordinators | Blog/twitter ✅ |

---

## 6. Post-Ceremony

### Key Rotation (T+14 days)
```bash
# Governance vote to disable PoA
# All validators update:
KOVANICA_CONSENSUS=pow
# Remove KOVANICA_AUTHORITY_KEY
# Authority keys destroyed (burn ceremony)

# PoA keys burned per policy
shred -n 3 -z -u /etc/kovanica/authority-keys/authority-1.env
# Paper backups destroyed
```

### Emergency Contacts
| Role | Contact | Channel |
|------|---------|---------|
| Lead Coordinator | [Name] | Signal + Email |
| Security Lead | [Name] | Signal + Email |
| Infrastructure | [Name] | Signal + PagerDuty |
| Legal | [Name] | Email |

---

## 7. Attestation Templates

### Signer Attestation
```
I, [Name], attest that on [Date] I generated my genesis signer keypair
on an air-gapped machine, never shared the private key, and verified
my public key in the genesis signer set (hash: <genesis-signers.pub hash>).

Signature: [PGP sig]
```

### Validator Attestation
```
I, [Name/Org], attest that on [Date] I generated my validator authority
keypair on a dedicated server, the private key never left the server,
and the public key matches the validator set committed in genesis.

Signature: [PGP sig]
```

### Auditor Attestation
```
I, [Name/Firm], attest that I observed the [Phase] of the Kovanica
mainnet genesis ceremony on [Date]. All participants followed the
prescribed offline key generation and verification procedures.

Signature: [PGP sig]
```

---

## 8. Emergency Procedures

| Scenario | Response |
|----------|----------|
| Signer key compromised pre-ceremony | Replace signer, regenerate key, restart |
| Validator offline at T+0 | Standby validator recruited (pre-identified) |
| Genesis hash mismatch | Abort, rollback, new ceremony |
| >2 validators offline | Pause, recruit replacements |
| Legal injunction | Activate legal response plan |

---

## 9. File Inventory

| File | Purpose | Security |
|------|---------|----------|
| `genesis-signer-N.key` | Signer private key | OFFLINE ONLY, paper backup |
| `genesis-signer-N.pub` | Signer public key | Public, shared |
| `genesis-signers.pub` | All 7 pubkeys | Public |
| `genesis-multisig.hash` | 5-of-7 multisig hash | Public |
| `genesis-tx-signed.json` | Signed genesis tx | Public (post-ceremony) |
| `authority-1.env` | Validator signing key | 0600, server only |
| `authorities.conf` | Validator pubkey | Public |
| `kovanica-cli.sha256` | Binary hash | Verified by all |

---

**Version:** 1.0  
**Last Updated:** 2026-09-26  
**Classification:** CONFIDENTIAL — Ceremony use only
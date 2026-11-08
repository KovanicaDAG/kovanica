# Kovanica Genesis Key Distribution Ceremony Protocol

> **Network ID:** `1` (`MAINNET`)  
> **Native Ticker:** **KUNA** (9 decimals: $1\text{ KUNA} = 10^9\text{ atoms}$)  
> **Target Date:** Mainnet Genesis  
> **Status:** Normative Cryptographic & Operational Security Protocol  
> **Consensus Impact:** Client and Operational only; defines genesis ceremony execution and multi-party key generation.

---

## 1. Objectives and Security Principles

The Kovanica Genesis Key Ceremony guarantees that all genesis cryptographic material—including validator identity keys, verifiable random function (VRF) leader election keys, founder allocations, and the 8,000,000 KUNA multi-signature treasury keys—are generated with verifiable entropy, zero electronic leakage, and cryptographic proofs of possession.

### Core Security Guarantees
1. **Zero Pre-Existing Exposure:** All private keys and seeds are generated on air-gapped, volatile-memory hardware boots.
2. **Entropy Mixing:** Combines hardware TRNGs, physical ambient entropy (dice/coin tosses), and cryptographic hashing (BLAKE3 + Argon2id).
3. **Proof of Possession (PoP):** Every validator proves knowledge of their private key without revealing the secret, eliminating rogue-key attacks.
4. **Threshold Custody (Shamir's Secret Sharing):** Master keys for the Treasury ($8\text{M KUNA}$) and governance emergency vaults are partitioned using an $M$-of-$N$ threshold scheme (3-of-5) distributed across independent geographic custodians.
5. **Irreversible Sanitization:** All transient media, RAM buffers, and staging drives are cryptographically sanitized post-ceremony.

---

## 2. Participants and Roles

| Role | Count | Responsibilities |
|---|---|---|
| **Master of Ceremonies (MC)** | 1 | Coordinates timing, distributes deterministic challenge strings, collects public keys and PoP signatures, executes `ops/genesis/ceremony.sh`. |
| **Genesis Validators** | 15–20 | Generate Ed25519 validator keys and Ristretto255 VRF keys on air-gapped machines; submit public artifacts; sign ceremony transcript. |
| **Treasury Custodians** | 5 | Participate in the Shamir's Secret Sharing ceremony for treasury multisig recovery; accept physical metal custody plates. |
| **Independent Auditors / Witnesses** | 2–3 | Verify air-gap integrity, oversee entropy mixing, inspect hashes, co-sign public ceremony audit report. |

---

## 3. Hardware & Environmental Air-Gap Requirements

All key generation steps must take place in an electromagnetically and physically controlled air-gapped environment.

### 3.1 Workstation Specification
- **Hardware:** Commercial off-the-shelf laptop/PC with physical Wi-Fi/Bluetooth cards physically removed or hardware-disabled via switch.
- **Power:** Battery power or filtered uninterruptible power supply (UPS) without networked telemetry.
- **Peripherals:** Wired USB keyboard and mouse only; no wireless transmitters.
- **Operating System:** Ephemeral Linux Live OS (Debian Live or Tails OS) booted from a write-protected, read-only USB flash drive.
- **Verification:** Detached GPG signature of the Live ISO verified on an independent machine prior to flashing.

### 3.2 Storage & Transfer Media
- **Air-Gap USB Drives:** Factory-sealed, single-use USB 3.0 flash drives used strictly for unidirectional export of public keys and signatures.
- **Strict Prohibition:** No private key, unencrypted keystore, or mnemonic seed phrase may ever be written to an external USB drive.
- **Tamper-Evident Security Bags:** Used for transporting public export drives and physical key shares.

---

## 4. Cryptographic Key Generation Procedures

### 4.1 Ambient Entropy Collection & Mixing

To guard against compromised hardware pseudo-random number generators (PRNGs), operators must collect multiple independent sources of physical entropy:

1. **Hardware RNG:** `/dev/random` kernel entropy pool.
2. **Physical Dice Rolls:** 100 rolls of high-precision casino dice recorded as decimal values.
3. **Keyboard & Pointer Latency:** Timing jitter from user keystrokes.

The entropy sources are mixed using BLAKE3 to seed the cryptographic keystore generator:

```bash
# Combine hardware random and operator manual entropy
(
  head -c 64 /dev/urandom
  echo -n "Manual-Entropy-Dice-String: "
  read -s MANUAL_ENTROPY
  echo -n "$MANUAL_ENTROPY"
  date +%s%N
) | b3sum | cut -d ' ' -f 1 > /tmp/ceremony_seed.hex
```

### 4.2 Validator Keystore Generation

Each genesis validator generates an encrypted Keystore v2 containing:
1. **Ed25519 Validator Key:** Used for block signing and consensus voting.
2. **Ristretto255 VRF Key:** Used for verifiable random leader selection (96-byte proof; Ristretto255 group with SHA3-512 hash-to-curve).

Execution on the air-gapped machine:

```bash
# Generate encrypted keystore with Argon2id + XChaCha20-Poly1305
kovanica key generate \
  --output validator.keystore \
  --with-vrf \
  --kdf-memory 65536 \
  --kdf-iterations 4

# Extract canonical public address and VRF public key
kovanica key inspect --keystore validator.keystore > public_credentials.txt
```

### 4.3 Address Format Verification

Verify that the derived address strictly adheres to the canonical **PRO-21 base32** format:
- Must begin with prefix `kvnc`.
- Must end with suffix `dag`.
- Exactly **64 characters** in total.
- Lowercase bech32 alphabet (`qpzry9x8gf2tvdw0s3jn54khce6mua7l`).
- Validated via `kovanica-types::Address::from_str`.

Example:
```text
kvnc1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqdag
```

---

## 5. Shamir's Secret Sharing (SSS) for Treasury & Founder Governance

The 8,000,000 KUNA Treasury allocation (held in linear 8-year vesting vaults) and the 200,000 KUNA Founder allocation are secured with $M$-of-$N$ threshold governance.

### 5.1 Scheme Architecture: 3-of-5 Threshold

A master 256-bit entropy seed is split into $N=5$ Shamir shares with a recovery threshold of $M=3$:
- **Any 3 shares** can reconstruct the master recovery seed.
- **Any 2 shares** reveal mathematically zero information about the secret.
- Field: Galois Field $\text{GF}(2^8)$ / Shamir polynomial interpolation.

### 5.2 Splitting & Engraving Procedure

1. **Air-Gap Generation:**
   ```bash
   kovanica-cli key sss-split \
     --secret /tmp/treasury_master.seed \
     --threshold 3 \
     --shares 5 \
     --output-dir /tmp/shares/
   ```
2. **Physical Medium:**
   Each share is stamped onto a fireproof, corrosion-resistant stainless steel or titanium plate using mechanical alphanumeric letter punches. Paper notes are prohibited.
3. **Custody Distribution:**
   - **Share 1:** European Institutional Custody Vault (Switzerland).
   - **Share 2:** North American Institutional Custody Vault (USA).
   - **Share 3:** Asia-Pacific Institutional Custody Vault (Singapore).
   - **Share 4:** Kovanica Foundation Lead Legal Trustee.
   - **Share 5:** Core Developer Protocol Escrow Safe.
4. **Packaging:**
   Each metal plate is sealed inside a numbered, tamper-evident security envelope with a registered serial number signed by the Master of Ceremonies and witnessed by an auditor.

---

## 6. Proof of Possession (PoP) & Public Submission

To prevent rogue-key and key-substitution attacks, each validator submits a cryptographic Proof of Possession.

### 6.1 Challenge Specification

The Master of Ceremonies issues a deterministic challenge:
```text
Domain: KUNA/genesis/ceremony/v1
Chain ID: 1
Timestamp: 1764072000
Validator Nonce: <random-32-byte-hex>
P2P Ingress: <public-ipv4>:8000
```

### 6.2 Signing the Challenge

On the air-gapped machine:

```bash
kovanica sign-message \
  --keystore validator.keystore \
  --message "KUNA/genesis/ceremony/v1:chain_id=1:ts=1764072000:nonce=${NONCE}:p2p=${IP}:8000" \
  --output pop_signature.sig
```

### 6.3 Public Artifact Bundle

Each validator exports **only** the following bundle via the air-gap USB drive:
```json
{
  "validator_address": "kvnc1...",
  "ed25519_public_key": "4a7b...",
  "vrf_public_key": "89ef...",
  "p2p_endpoint": "198.51.100.10:8000",
  "challenge_nonce": "c3a8...",
  "pop_signature": "7d91..."
}
```

---

## 7. Verification, Genesis Compilation & Attestation

### 7.1 Automated Ingestion & Validation

The Master of Ceremonies aggregates all validator submissions into `validators.json` and runs the verification suite:

```bash
# Verify all PoP signatures and VRF keys
kovanica key verify-ceremony --validators validators.json

# Compile Genesis bundle
./ops/genesis/ceremony.sh \
  --network mainnet \
  --validators validators.json \
  --treasury-address kvnc1...dag \
  --founder-address kvnc1...dag \
  --output ./genesis-artifacts
```

### 7.2 Publication & Cryptographic Checksums

The resulting public files are hashed and published to the canonical ceremony repository and distributed out-of-band:

```bash
sha256sum genesis-artifacts/genesis.json \
          genesis-artifacts/genesis_validators.toml \
          genesis-artifacts/bootnodes.toml > SHA256SUMS

b3sum genesis-artifacts/genesis.json \
      genesis-artifacts/genesis_validators.toml \
      genesis-artifacts/bootnodes.toml > B3SUMS
```

### 7.3 Multi-Party Attestation

All 15–20 validator operators, 5 custodians, and independent auditors sign the final `SHA256SUMS` file with their respective GPG keys:

```bash
gpg --armor --detach-sign SHA256SUMS
```

The resulting signatures are archived in `docs/ceremony/mainnet-attestations/`.

---

## 8. Post-Ceremony Sanitization Protocol

To ensure no remnant secret data persists:

1. **Memory Flushing:** All air-gapped workstations are powered off completely for a minimum of 5 minutes to discharge volatile DRAM capacitors.
2. **Ephemeral Media Wipe:** USB drives used during intermediate staging are wiped according to DoD 5220.22-M:
   ```bash
   shred -v -n 3 -z /dev/sdX
   ```
3. **Physical Inspection:** Metal engraving punches and waste materials are inspected and secured.

---

## 9. Operator Sign-Off Checklist

Before mainnet node activation, each validator operator must confirm:

- [ ] Validator identity key and VRF key generated on an air-gapped live boot.
- [ ] Keystore secured with high-entropy passphrase (minimum 16 characters).
- [ ] Backup keystore file stored on encrypted off-line media.
- [ ] Address matches canonical PRO-21 format (`kvnc`...`dag`, 64 characters).
- [ ] Proof of Possession verified by the Master of Ceremonies.
- [ ] `genesis.json` and `genesis_validators.toml` checksums match published `SHA256SUMS`.
- [ ] `funded_accounts.toml` is absent from node data directory.
- [ ] Node configuration passes pre-flight checks (`kovanica-node --check-config`).

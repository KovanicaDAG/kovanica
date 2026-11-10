# Kovanica Genesis Key Distribution Ceremony Protocol

> **Network ID:** `1` (`MAINNET`)  
> **Native Ticker:** **KVNC** (9 decimals: $1\text{ KVNC} = 10^9\text{ base units / atoms}$)  
> **Consensus:** **Hybrid PoW + GHOSTDAG** (NO stake for consensus; target block 4 s, GHOSTDAG $K=5$; PoW Hash: KovanicaHash)  
> **Status:** DRAFT, not normative, being rewritten for PoW + GHOSTDAG no-stake  

---

## 1. Objectives and Architecture

Under Kovanica's **Hybrid PoW + GHOSTDAG** consensus architecture, block production is driven by Proof-of-Work (miners solving KovanicaHash targets) and block ordering is driven by GHOSTDAG ($K=5$). **Consensus has strictly NO stake in the consensus path.**

Consequently:
1. **No BFT Committee Quorum:** The legacy Proof-of-Stake requirement of 15–20 active voting validators is retired. The network does not rely on validator vote signatures or rounds to reach consensus.
2. **No VRF Keys:** PRO-7 (VRF leader election) is **FROZEN**; Ristretto255 VRF keys (`--with-vrf`) are not used or required.
3. **Open Mining:** Any node may mine blocks immediately upon chain launch by configuring a standard PRO-21 address (`KOVANICA_MINING_AUTHOR`). Miners do NOT register keys or participate in the ceremony.
4. **Ceremony Focus:** The Genesis Key Ceremony is strictly focused on securing:
   - **The Treasury Multisig:** 8,000,000 KVNC linear 8-year vesting allocation partitioned via a 3-of-5 threshold scheme.
   - **The Founder Premine:** 200,000 KVNC cold storage allocation.
   - **Bootstrap Seed Nodes:** 3–5 initial public seed node identities to bootstrap initial P2P mesh discovery.

---

## 2. Participants and Roles

| Role | Count | Responsibilities |
|---|---|---|
| **Master of Ceremonies (MC)** | 1 | Coordinates timing, distributes challenge strings, compiles public artifacts, executes `ops/genesis/ceremony.sh`. |
| **Treasury Custodians** | 5 | Participate in the Shamir's Secret Sharing (3-of-5) ceremony for Treasury vaults; accept physical metal custody plates. |
| **Founder** | 1 | Generates the 200,000 KVNC cold storage key on an air-gapped machine. |
| **Bootstrap Seed Node Operators** | **3–5** | Generate Ed25519 node identities and PRO-21 addresses for initial P2P seed nodes (`seed1`, `seed2`, `seed3.kovanica.online`). |
| **Independent Witnesses** | 2–3 | Verify air-gap physical integrity, oversee entropy mixing, co-sign public ceremony audit report. |

---

## 3. Hardware & Environmental Air-Gap Requirements

All critical key generation steps must occur in an electromagnetically and physically isolated air-gapped environment.

### 3.1 Workstation Specification
- **Hardware:** Commercial laptop/PC with physical Wi-Fi/Bluetooth hardware physically disconnected or removed.
- **Power:** Battery power or filtered uninterruptible power supply (UPS) with no external network telemetry.
- **Peripherals:** Wired USB keyboard and mouse only; no wireless transmitters.
- **Operating System:** Ephemeral Linux Live OS (Debian Live or Tails OS) booted from a write-protected, read-only USB flash drive.
- **Verification:** Detached GPG signature of the Live ISO verified on an independent workstation prior to flashing.

### 3.2 Storage & Transfer Media
- **Air-Gap USB Drives:** Single-use USB flash drives used strictly for unidirectional export of public keys and signatures.
- **Strict Prohibition:** Private keys, seed phrases, or unencrypted keystores may never be written to an external USB drive.
- **Security Envelopes:** Tamper-evident bags for transporting physical key shares and public export drives.

---

## 4. Cryptographic Key Generation Procedures

### 4.1 Ambient Entropy Collection & Mixing

Operators collect multiple independent sources of physical entropy to guard against compromised hardware PRNGs:
1. **Hardware RNG:** Kernel entropy pool (`/dev/random`).
2. **Physical Dice Rolls:** 100 rolls of precision casino dice recorded as decimal values.
3. **Keyboard & Pointer Latency:** High-resolution timing jitter.

Mixed via BLAKE3 to seed the cryptographic keystore generator:
```bash
(
  head -c 64 /dev/urandom
  echo -n "Manual-Entropy-Dice-String: "
  read -s MANUAL_ENTROPY
  echo -n "$MANUAL_ENTROPY"
  date +%s%N
) | b3sum | cut -d ' ' -f 1 > /tmp/ceremony_seed.hex
```

### 4.2 Bootstrap Seed Node Keystore Generation

Each bootstrap seed node operator generates an encrypted Keystore v2:
```bash
# Generate encrypted keystore (Argon2id + XChaCha20-Poly1305)
kovanica key generate \
  --output seed_node.keystore \
  --kdf-memory 65536 \
  --kdf-iterations 4

# Extract canonical PRO-21 address
kovanica key inspect --keystore seed_node.keystore > public_credentials.txt
```

### 4.3 Address Format Verification (PRO-21)

Verify that all derived addresses strictly adhere to canonical **PRO-21 base32** formatting:
- Prefix: `kvnc`
- Suffix: `dag`
- Total length: exactly **64 characters**
- Character set: lowercase bech32 (`qpzry9x8gf2tvdw0s3jn54khce6mua7l`)
- Checksum: 29-bit BLAKE3 checksum appended to the 32-byte Ed25519 public key.
- Legacy hex addresses are strictly rejected on Mainnet (`chain_id = 1`).

---

## 5. Shamir's Secret Sharing (SSS) for Treasury Custody

The 8,000,000 KVNC Treasury allocation (governed by linear 8-year vesting) is secured via a 3-of-5 threshold scheme.

### 5.1 Scheme Architecture: 3-of-5 Threshold
- A master 256-bit entropy seed is split into $N=5$ Shamir shares with a recovery threshold of $M=3$.
- **Any 3 shares** reconstruct the master recovery seed.
- **Any 2 shares** reveal zero information about the secret.
- Field: Galois Field $\text{GF}(2^8)$ / Shamir polynomial interpolation.

### 5.2 Splitting & Metal Engraving Procedure
1. **Air-Gap Generation:**
   ```bash
   kovanica-cli key sss-split \
     --secret /tmp/treasury_master.seed \
     --threshold 3 \
     --shares 5 \
     --output-dir /tmp/shares/
   ```
2. **Physical Medium:** Each share is stamped onto a fireproof, corrosion-resistant stainless steel or titanium plate using mechanical alphanumeric letter punches. Paper notes are prohibited.
3. **Geographic Custody Distribution:**
   - **Share 1:** European Institutional Custody Vault (Switzerland).
   - **Share 2:** North American Institutional Custody Vault (USA).
   - **Share 3:** Asia-Pacific Institutional Custody Vault (Singapore).
   - **Share 4:** Kovanica Foundation Lead Legal Trustee.
   - **Share 5:** Core Developer Protocol Escrow Safe.

---

## 6. TODO: rewrite for PoW + GHOSTDAG

---

## 7. Verification & Genesis Compilation

### 7.1 Automated Ingestion

TODO: rewrite for PoW + GHOSTDAG

### 7.2 Cryptographic Hashes & Attestation
```bash
sha256sum genesis-artifacts/genesis.json \
          genesis-artifacts/bootnodes.toml > SHA256SUMS

b3sum genesis-artifacts/genesis.json \
      genesis-artifacts/bootnodes.toml > B3SUMS
```

All participants sign `SHA256SUMS` with their detached GPG keys for publication in `docs/ceremony/mainnet-attestations/`.

---

## 8. Post-Ceremony Sanitization

1. **Memory Discharge:** Air-gapped workstations powered off completely for ≥5 minutes to discharge volatile DRAM capacitors.
2. **Staging Media Wipe:** Ephemeral USB drives wiped per DoD 5220.22-M:
   ```bash
   shred -v -n 3 -z /dev/sdX
   ```
3. **Physical Inspection:** Metal punches and residual materials verified and secured.

---

## 9. Operator Sign-Off Checklist

- [ ] Node identity keys generated on an air-gapped live boot.
- [ ] Keystore secured with high-entropy passphrase (minimum 16 characters).
- [ ] Address matches canonical PRO-21 format (`kvnc`...`dag`, 64 characters).
- [ ] Treasury 3-of-5 Shamir shares engraved onto metal plates and distributed.
- [ ] Genesis bundle checksums match published `SHA256SUMS` and `B3SUMS`.
- [ ] `funded_accounts.toml` is absent from data directory (strictly prohibited on Mainnet).
- [ ] Pre-flight config check succeeds: `kovanica-node --check-config`.

# Kovanica BIP32 / BIP44 Hierarchical Deterministic (HD) Wallet Specification

- **Status:** Implemented
- **Standard:** SLIP-0010 (Ed25519), BIP-39, BIP-44
- **Registered Coin Type:** `3007` (`3007'`)
- **Purpose:** `44` (`44'`)
- **Relevant Crates:** `crates/kovanica-wallet-core`, `crates/kovanica-keystore`, `crates/kovanica-cli`

---

## 1. Overview

Kovanica uses **Ed25519** signing keys and canonical PRO-21 base32 addresses (`kvnc<57 base32>dag`, 64 characters).

Standard BIP-32 defines hierarchical deterministic derivation for ECDSA curves (`secp256k1`). Because Ed25519 points cannot be added to scalars in a manner that permits non-hardened public child key derivation without severe security vulnerabilities, the industry-standard specification for Ed25519 HD derivation is **SLIP-0010** ("Universal private key derivation from master private key").

This specification documents Kovanica's SLIP-0010 Ed25519 implementation, canonical BIP-44 derivation paths, and the backward-compatibility architecture preserving legacy non-HD mnemonic phrases.

---

## 2. Key Derivation Architecture

### 2.1. Seed Generation from BIP-39 Mnemonic
Standard BIP-39 mnemonic phrases (typically 24 English words) produce a 512-bit (64-byte) binary seed using PBKDF2:
$$\text{Seed} = \text{PBKDF2-HMAC-SHA512}(\text{password} = \text{phrase}, \text{salt} = \text{"mnemonic"} \mathbin{\Vert} \text{passphrase}, \text{iterations} = 2048, \text{len} = 64)$$

In `crates/kovanica-keystore`:
```rust
pub fn bip39_seed_from_mnemonic(phrase: &str, passphrase: &str) -> Result<Zeroizing<[u8; 64]>>;
```

### 2.2. Master Key Generation (SLIP-0010 Master Node)
The master node is derived from the 64-byte seed:
$$I = \text{HMAC-SHA512}(\text{Key} = \text{b"ed25519 seed"}, \text{Data} = \text{Seed})$$
- Master private key: $k_L = I[0..32]$ (32-byte Ed25519 private key seed)
- Master chain code: $c = I[32..64]$ (32-byte chain code)

### 2.3. Hardened Child Key Derivation
For Ed25519, SLIP-0010 strictly enforces that **only hardened child keys** ($i \ge 2^{31} = 0x8000\_0000$) can be derived:
$$\text{Data} = 0x00 \mathbin{\Vert} \text{ser}_{256}(k_{\text{par}}) \mathbin{\Vert} \text{ser}_{32}(i)$$
$$I = \text{HMAC-SHA512}(\text{Key} = c_{\text{par}}, \text{Data})$$
- Child private key: $k_i = I[0..32]$
- Child chain code: $c_i = I[32..64]$

Any attempt to derive a non-hardened child key ($i < 0x8000\_0000$) is mathematically undefined for Ed25519 under SLIP-0010 and is strictly rejected with `WalletError::Derivation`.

---

## 3. Canonical Kovanica BIP-44 Derivation Path

Kovanica adheres to the BIP-44 hierarchy:
$$\text{m} / 44' / 3007' / \text{account}' / \text{change}' / \text{address\_index}'$$

All five path components are hardened per SLIP-0010 rules:
- `purpose`: `44'` (`0x8000_002C`)
- `coin_type`: `3007'` (`0x8000_0BBF`) — Kovanica Coin Type
- `account`: `account'` (`account + 0x8000_0000`), default `0'`
- `change`: `change'` (`change + 0x8000_0000`), default `0'` (external / payout)
- `address_index`: `index'` (`index + 0x8000_0000`), default `0'`

Example canonical path for Account 0, Address 0:
`m/44'/3007'/0'/0'/0'`

---

## 4. Backward Compatibility Requirement (Critical)

Historically, Kovanica's mnemonic import/export mapped 24-word English phrases directly to their 32-byte raw entropy (`Mnemonic::to_entropy()`) and used those 32 bytes directly as the Ed25519 seed (`SigningKey::from_bytes`).

To prevent breaking existing user wallets:
1. **Raw-Entropy Default:**
   When neither `--hd`, `--account`, nor `--index` is supplied on the CLI, mnemonic import continues to use raw 32-byte entropy. Existing phrases produce the exact same addresses and keys.
2. **Opt-in HD Derivation:**
   HD derivation is strictly opt-in, triggered when `--hd`, `--account`, or `--index` is explicitly specified.
3. **Keystore Compatibility:**
   Both modes produce standard v2 encrypted keystores storing the final 32-byte Ed25519 seed.

---

## 5. CLI Usage

### 5.1. Non-HD Legacy Import (Default)
```bash
# Uses raw 32-byte entropy directly (backward compatible)
kovanica import-key --format mnemonic --out wallet.json
```

### 5.2. Opt-in HD Derivation
```bash
# Derives Account 0 at m/44'/3007'/0'/0'/0'
kovanica import-key --format mnemonic --hd --out wallet_hd.json

# Derives Account 1, Index 5 at m/44'/3007'/1'/0'/5'
kovanica import-key --format mnemonic --hd --account 1 --index 5 --out wallet_acc1.json
```

### 5.3. Address Inspection
```bash
# Offline address inspection with HD derivation
kovanica address --key-file phrase.txt --allow-raw-hex --hd --account 0 --index 0
```
Output includes the canonical `derivation_path`:
```
Address:         kvnc1...dag
Public key:      0123...
Derivation path: m/44'/3007'/0'/0'/0'
```

---

## 6. Official SLIP-0010 Test Vectors

The implementation is verified against the official SLIP-0010 Ed25519 test vectors in `crates/kovanica-wallet-core/src/hd.rs`:

### Test Vector 1
- **Seed:** `000102030405060708090a0b0c0d0e0f`
- **Master private key:** `2b4be7f19ee27bbf30c667b642d5f4aa69fd169872f8fc3059c08ebae2eb19e7`
- **Master chain code:** `90046a93de5380a72b5e45010748567d5ea02bbf6522f979e05c0d8d8ca9fffb`
- **Child `m/0'` private key:** `68e0fe46dfb67e368c75379acec591dad19df3cde26e63b93a8e704f1dade7a3`
- **Child `m/0'/1'` private key:** `b1d0bad404bf35da785a64ca1ac54b2617211d2777696fbffaf208f746ae84f2`

### Test Vector 2
- **Seed:** `fffcf9f6f3f0edeae7e4e1dedbd8d5d2cfccc9c6c3c0bdbab7b4b1aeaba8a5a29f9c999693908d8a8784817e7b7875726f6c696663605d5a5754514e4b484542`
- **Master private key:** `171cb88b1b3c1db25add599712e36245d75bc65a1a5c9e18d76f9f2b1eab4012`
- **Master chain code:** `ef70a74db9c3a5af931b5fe73ed8e1a53464133654fd55e7a66f8570b8e33c3b`
- **Child `m/0'` private key:** `1559eb2bbec5790b0c65d8693e4d0875b1747f4970ae8b650486ed7470845635`

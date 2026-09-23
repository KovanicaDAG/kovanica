# Address & Sighash Spec (from kovanica-node)

**Source:** `kovanica-state` keys.rs + `tx.rs`  
**Captured:** 2026-09-23  
**Status:** Canonical for SDK alignment

> Node does **not** use bech32. Bech32 appears only as an unused transitive npm dependency in the web lockfile.

---

## 1. Address format

### Human-readable

```text
kvnc + base58(versioned_bytes) + dag
```

- Parse is **case-insensitive** on the string.
- `to_kvnc()` returns this form; `Debug` / display often shows **66-hex**.

### Versioned bytes

| Layout | Meaning |
|--------|---------|
| `byte[0]` | Version |
| `byte[1..33]` | 32-byte payload |

### Version table

| Version | Type | Payload |
|---------|------|---------|
| `0x00` | P2PK | Ed25519 public key (32B) |
| `0x01` | P2SH | BLAKE3 script hash (multisig) |
| `0x02` | Script v2 | script-related payload |
| `0x03` | Stealth | Special: 65B on wire = `0x03 ‖ scan_pk(32) ‖ spend_pk(32)` → `kvnc+base58(65B)+dag` or 130-hex |
| `0x04` | HTLC | HTLC script payload |
| `0x05` | Vault / CSV | Vault payload |

### Also accepted by node

| Form | Meaning |
|------|---------|
| 66-hex | 33 versioned bytes |
| 64-hex legacy | 32B → treated as P2PK (`0x00`) |

---

## 2. Sighash

```rust
// node tx.rs
pub fn sighash(&self) -> [u8; 32] {
    let mut buf = Vec::new();
    self.encode_into(&mut buf, false); // witness-free
    *blake3::hash(&buf).as_bytes()
}
```

### Rules

- **BLAKE3** over canonical encoding **without witness** (signatures omitted).
- Preimage includes: input count + outpoints (txid + index), output count + each output (value, asset flag+id, stealth/owner fields), `n_lock_time` (4B), `sequence` (4B), tag (len+bytes).
- Covers BIP-65-style lock-time and BIP-112-style sequence binding.
- Result: **32 bytes** → 64 hex chars.
- **Signature:** Ed25519 over those **32 bytes** → 64B / 128 hex.
- Node verifies with **`verify_strict`** (rejects malleable curves).

### TxId vs sighash

| Function | Hash input |
|----------|------------|
| `Transaction::sighash()` | `encode_into(..., witness=false)` → BLAKE3 |
| `Transaction::id()` (TxId) | full encoding **with** witness → BLAKE3 |

Same logic exists in both workspaces (`node/crates/` and `protocol/crates/` mirror).

---

## 3. SDK implementation status

| Item | Status |
|------|--------|
| P2PK `kvnc…dag` encode | **Done** in `kovanica-keys` |
| Decode `kvnc…dag` / 66-hex / 64-hex | **Done** |
| Version constants 0x00–0x05 | **Done** |
| Stealth 65B encode path | Pending (builder) |
| Sighash = BLAKE3(witness-free) | **Algorithm locked**; simplified preimage buffer until full `encode_into` is ported |
| Ed25519 over 32-byte sighash | **Done** in signing path |
| Byte-identical `encode_into` | **Open** — requires port from node `tx.rs` |

---

## 4. Tasks

- **S-03b:** Address codec — **largely complete** for P2PK; remaining = parity tests against node vectors + stealth/HTLC/vault helpers.
- **S-03c:** Sighash — algorithm correct; remaining = port exact `encode_into` for byte-identical hashes.

---

*Keep this file next to the SDK; treat node source as ultimate authority if this note drifts.*
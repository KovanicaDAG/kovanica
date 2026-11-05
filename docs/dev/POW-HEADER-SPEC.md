# PoW Block Header Specification (P1–P2)

- **Status:** Implemented (P1–P2, Toni 2026-11-05, ADR 0003)
- **Scope:** Block header primitives, preimage serialization, compact difficulty target conversion, and Chain ID 5 registry wiring.

---

## 1. Chain ID 5 Registry Wiring

Chain ID 5 is registered for the planned Hybrid PoW + GHOSTDAG testnet:
- Constant: `CHAIN_ID_HYBRID_TESTNET: u64 = 5` in `kovanica-params` and `kovanica-types::signing::chain_id`.
- Network tag: `hybrid-testnet` (`kovanica_params::NETWORK_TAG_HYBRID_TESTNET`).
- Activation intent: Gated under feature flag `hybrid-pow`. Under hybrid consensus, block authoring uses PoW mining (KovanicaHash) and block ordering uses GHOSTDAG (k=5, 4 s block time) without stake weighting.
- Address acceptance: Canonical PRO-21 (`kvnc...dag`, 64 characters) is standard. Legacy hex addresses are permitted for testnet development convenience (`legacy_address_allowed(5) == true`).

---

## 2. Block Header Data Structure

The `BlockHeader` (aliases `PowHeader`, `HeaderV2`) lives in `kovanica_types::block`:

```rust
pub struct BlockHeader {
    pub chain_id: u64,
    pub round: Round,
    pub author: AuthorityIndex,
    pub parents: Vec<BlockReference>,
    pub merkle_root: Hash,
    pub timestamp: u64,
    pub target_bits: u32,
    pub nonce: u64,
    pub pow_hash: Option<Hash>,
}
```

### Preimage Format & Serialization

The preimage serialization for PoW evaluation uses domain separation tag `KVNC-POW-HDR-v1` (`0x00`-padded to 16 bytes via `POW_HEADER_DOMAIN_TAG` / `DOMAIN_POW_HEADER`):

| Offset | Size (bytes) | Field | Encoding |
|-------:|-------------:|:------|:---------|
| 0 | 16 | Domain Tag | `DomainTag16("KVNC-POW-HDR-v1")` |
| 16 | 8 | `chain_id` | `u64` LE |
| 24 | 8 | `round` | `u64` LE (DAG height / round) |
| 32 | 2 | `author` | `u16` LE (`AuthorityIndex`) |
| 34 | 4 | `parents_count` | `u32` LE |
| 38 | `parents_count * 42` | Parents array | For each parent: `author` (2 B LE), `round` (8 B LE), `digest` (32 B) |
| 38 + 42 * P | 32 | `merkle_root` | `[u8; 32]` transaction Merkle root |
| 70 + 42 * P | 8 | `timestamp` | `u64` LE (unix milliseconds) |
| 78 + 42 * P | 4 | `target_bits` | `u32` LE (compact target representation) |
| 82 + 42 * P | 8 | `nonce` | `u64` LE (mining nonce) |

Preimage length with 1 parent is exactly 132 bytes.

---

## 3. Compact Target Representation (`nBits`)

The compact format converts between a 32-bit compact unsigned integer (`bits`) and a 256-bit big-endian target:
- Exponent: `(bits >> 24) as usize`
- Mantissa: `bits & 0x007f_ffff`
- Negative sign bit: `0x0080_0000` (if set, target is zero / invalid)
- Formula: $\text{target} = \text{mantissa} \times 256^{(\text{exponent} - 3)}$

Functions in `kovanica_types::block`:
- `compact_to_target(bits: u32) -> [u8; 32]`
- `target_to_compact(target: &[u8; 32]) -> u32`
- `check_pow_hash_meets_target(hash: &Hash, target_bits: u32) -> bool` (checks `hash.0 <= target` lexicographically as big-endian 256-bit unsigned integers)

# PoW Block Header & Mining Specification (P1–P4)

- **Status:** Implemented (P1–P4, Toni 2026-11-05, ADR 0003)
- **Scope:** Block header primitives, preimage serialization, compact difficulty target conversion, KovanicaHash PoW algorithm, header verification wiring, mining worker, and difficulty adjustment algorithm (DAA) for Chain ID 5.

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

---

## 4. KovanicaHash PoW Algorithm (P3)

KovanicaHash is a memory-hard proof-of-work hasher combining BLAKE3 with a 2 MiB pseudo-random memory mix buffer. It is designed to be lightweight on 2–4 GB RAM VPS nodes while resisting ASIC/SRAM parallelization.

### Algorithm Constants
- Memory Size: `KOVANICA_HASH_MEMORY_SIZE = 2 * 1024 * 1024` (2,097,152 bytes)
- Chunk Size: `KOVANICA_HASH_CHUNK_SIZE = 32` bytes
- Total Chunks: `KOVANICA_HASH_NUM_CHUNKS = 65,536` ($2^{16}$)
- Mix Rounds: `KOVANICA_HASH_MIX_ROUNDS = 1,024`
- Seed Domain Tag: `KVNC-POW-SEED-v1`
- Finalize Domain Tag: `KVNC-POW-FINL-v1`

### Phases
1. **Seed Phase:**
   The 132+ byte canonical header preimage is hashed using BLAKE3 keyed mode under `KVNC-POW-SEED-v1` to generate a 32-byte initial seed.
2. **Memory Mix Phase:**
   - **Generation:** BLAKE3 in extendable output (XOF) mode keyed with the seed fills the 2 MiB buffer deterministically (65,536 chunks of 32 bytes).
   - **Wandering & Mutation:** Running state starts at seed. For 1,024 rounds:
     - Unbiased chunk index: `u16::from_le_bytes([state[0], state[1]])`.
     - Chunk is read and mixed into state using non-linear 64-bit wrapping arithmetic, XORs, and bit-rotations (`rotate_left(17, 29, 37, 43)`).
     - State is written back to the chunk offset, creating a read-write memory dependency.
     - Step index is XOR-folded into state.
3. **Finalize Phase:**
   A final keyed BLAKE3 compression under `KVNC-POW-FINL-v1` digests the seed, mixed state, and boundary memory samples (first 32 bytes and last 32 bytes) to produce the final 32-byte `Hash`.

### Golden Test Vectors
- Empty Preimage (`b""`):
  `05cca3939da23b1b771692875380dd70d953c98ba5498468a683add6aaa2a623`
- 132-byte Preimage (`[0x5a; 132]`):
  `cce556a21d6efee5463d39911877bab564d5fbe70f6b07fea1dfad2c1ccb2db4`
- Fixed String (`b"KVNC-CHAIN5-HYBRID-POW-TEST-PREIMAGE"`):
  `ba2bbe92ef378bdbf970bdde68a2d5cffaaea4c5aa02c9146ddd9b1765b8b262`

---

## 5. Header Verification & Timestamps (P3)

Header verification in `kovanica_types::block` enforces:
1. `pow_hash` must be present (`HeaderError::MissingPowHash`).
2. Declared `pow_hash` must match `compute_pow_hash()` (`HeaderError::InvalidPowHash`).
3. Declared `pow_hash` must meet `target_bits` via `check_pow_hash_meets_target` (`HeaderError::TargetNotMet`).
4. `target_bits` must be non-zero and valid (`HeaderError::InvalidTargetBits`).
5. Timestamp drift: header timestamp cannot exceed local wall-clock time plus `MAX_FUTURE_TIMESTAMP_DRIFT_MS = 30_000` ms (`HeaderError::TimestampDrift`).
6. Monotonicity: header timestamp must be strictly greater than parent timestamps (`HeaderError::TimestampNotMonotonic`).

---

## 6. Mining Worker & DAA Primitives (P4)

Implemented in `kovanica-consensus`:
- `mine_header(mut header: BlockHeader, max_iterations: u64) -> Option<BlockHeader>`: Synchronous nonce search reusing a single 2 MiB context.
- `mine_header_async(header: BlockHeader, max_iterations: u64) -> Option<BlockHeader>`: Async background mining.
- `MiningWorker`: Reusable mining worker struct with preallocated memory buffer.
- `NodeMiner`: Node-level mining wrapper with integrated DAA calculations.

### Difficulty Adjustment Algorithm (DAA)
- Target block time: `TARGET_BLOCK_MS = 4000` ms (4 s).
- Adjustment window: `DAA_WINDOW_BLOCKS = 60` blocks.
- Max adjustment factor: 4x / 0.25x clamping per window.
- Ceiling: `MAX_TARGET_BITS = 0x1f00_ffff`.
- Floor: `MIN_TARGET_BITS = 0x0100_0001`.
- Math helper: `mul_div_u256(val, num, den)` provides exact 320-bit intermediate multi-limb arithmetic.

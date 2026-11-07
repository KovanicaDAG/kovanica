# PoW Block Header & Mining Specification (P1–P8)

- **Status:** Implemented (P1–P8, Toni 2026-11-05, ADR 0003, Milestone M8 Ready)
- **Scope:** Block header primitives, preimage serialization, compact difficulty target conversion, KovanicaHash PoW algorithm, header verification wiring, mining worker, difficulty adjustment algorithm (DAA), P2P anti-DoS network ingress, GHOSTDAG committer with reward minting, and node mining authoring loop for Chain ID 5.

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
- Floor: `MIN_TARGET_BITS = 0x0101_0000`.
- Math helper: `mul_div_u256(val, num, den)` provides exact 320-bit intermediate multi-limb arithmetic.

### Consensus Derivation & Admission Hardening Rules
1. **Ancestry & Window Rules:**
   - For every block $B$ on Chain 5 (`HYBRID_TESTNET`), the required DAA target difficulty is derived strictly from its declared parent DAG past cone ($B.\text{parents}$).
   - Traversal evaluates up to $N = 60$ ancestor PoW blocks (`DAA_WINDOW_BLOCKS`).
   - Ancestors are traversed deterministically from $B.\text{parents}$ back up to 60 ancestor blocks.
2. **Ordering Rules:**
   - Ancestor timestamps are collected deterministically and sorted in strictly ascending chronological order ($t_0 \le t_1 \le \dots \le t_{k-1}$).
   - Because $B$'s ancestry graph is immutable and uniquely identified by block references within $B$'s signed header, the resulting sorted timestamp sequence is completely identical across all nodes regardless of local block arrival order or current node DAG tips.
3. **Bootstrap Rules:**
   - For round 0 (canonical genesis, $B.\text{parents} = \emptyset$), the required target is `MAX_TARGET_BITS` (`0x1f00_ffff`, or `KOVANICA_TEST_TARGET_BITS` when configured for test environments).
   - If the ancestry window contains fewer than 2 timestamps ($k < 2$, e.g. round 1 blocks whose only parent is round 0 genesis with no PoW header), the target defaults to `current_target_bits` which is `MAX_TARGET_BITS`.
   - For $k \ge 2$, `current_target_bits` is derived from the primary parent ($B.\text{parents}[0]$).
   - The new target is scaled by:
     $$\text{new\_target} = \text{current\_target} \times \frac{\text{clamped\_actual\_duration}}{\text{target\_duration}}$$
     where $\text{actual\_duration} = \max(1, t_{k-1} - t_0)$, $\text{target\_duration} = (k - 1) \times 4000\text{ ms}$, clamped to $[0.25, 4.0] \times \text{target\_duration}$.
     The target is clamped to $[\text{MIN\_TARGET\_BITS}, \text{MAX\_TARGET\_BITS}]$.
4. **Missing-History & Orphan Rules:**
   - If any parent in $B.\text{parents}$ is not present in accepted consensus storage (`dag_store`), block $B$ cannot be admitted into accepted consensus state, cannot be placed in `GhostdagCommitter`, and cannot execute or mint emission rewards.
   - Block $B$ is retained in the bounded `OrphanBuffer` (bounded by `max_orphans = 1024`, `max_bytes = 32 MiB`, `max_per_peer = 256`, `ttl = 60s`, `max_in_flight = 256`).
   - The node issues deduplicated `BlockSyncRequest::ByHash` requests to the peer for missing parents.
   - When missing parents arrive and are successfully admitted to `dag_store`, `orphans.parent_accepted` iteratively releases ready blocks for full revalidation and admission.
   - Malformed parent references (such as `parents.len() > GHOSTDAG_K = 5`, duplicate parent digests, self-parenting references, non-decreasing rounds `parent.round >= block.round`, or round gaps exceeding `MAX_PARENT_ROUND_GAP = 50`) are rejected immediately as invalid blocks and never buffered.
5. **Legacy vs Hybrid submitWork Isolation:**
   - RPC method `kovanica_submitWork` immediately rejects any submission if the serving node is running on a non-hybrid chain (e.g. `chain_id != HYBRID_TESTNET`), returning `RpcError::InvalidRequest`.
   - Matching `header.chain_id == node.chain_id` is NOT sufficient; only registered hybrid networks may process `submitWork`.
   - No block or consensus storage writes (`consensus_store.put_block`) occur before validation passes.



---

## 7. P2P Network Ingress & PoW Anti-DoS Boundary (P5)

Implemented in `crates/kovanica-network`:
- **Validation Pipeline:** In `validate_block` / `verify_block`, incoming blocks on Chain 5 (`HYBRID_TESTNET`) are validated against PoW requirements before being accepted or forwarded:
  1. Merkle root integrity and block digest verification.
  2. Header extraction via `StatementBlock::extract_pow_header(chain_id)`. If missing or malformed, the block is rejected with `Rejection::MissingPow` or `Rejection::BadPow`.
  3. Header PoW verification via `header.verify_pow()` (enforcing valid `pow_hash`, target requirement check against `target_bits`, and timestamp sanity).
  4. Skip committee authority checks: Chain 5 operates without validator signatures or committee authority lookups (NO stake in consensus path per ADR 0003).
- **Anti-DoS Protection:**
  - Blocks failing PoW verification are rejected immediately at the network ingress boundary.
  - Invalid blocks are neither propagated to gossipsub peers nor stored in the DAG memory/disk stores.
  - Sending peers are penalized with `invalid_message_weight` in gossipsub peer scoring.
- **Message Parsing:** `NetworkService::on_block_message` admits both standard `StatementBlock` messages and raw `BlockHeader` structures (automatically converted to `StatementBlock` with empty transaction lists).

---

## 8. GHOSTDAG Committer & Execution Wiring (P6)

Implemented in `crates/kovanica-consensus` and `crates/kovanica-execution`:
- **GHOSTDAG Committer (`GhostdagCommitter`):**
  - Parameter: `GHOSTDAG_K = 5` (ADR 0003).
  - Parent Selection: Identifies the selected parent from candidate tips by maximizing:
    1. Cumulative PoW work (sum of block work where `work = 2^256 / (target + 1)`).
    2. Blue score (number of blue ancestors in the chosen parent DAG).
    3. Lexicographical tie-breaking by block digest.
  - Mergeset Construction: mergeset(B) = past(B) \ past(selected_parent(B)).
  - Blue/Red Colouring: Evaluates each block in the mergeset topologically to verify whether adding it preserves the k=5 anti-chain limit against existing blue blocks.
  - Sub-DAG Linearization: Commits the selected parent chain into `CommittedSubDag` containing blue-ordered blocks, plus uncommitted red blocks recorded in `non_blue_blocks`.
  - Stake Isolation: Consensus operates completely free of stake weighting, vote quorums, committee signatures, or VRF tickets.
- **Mining Subsidy & Reward Minting:**
  - In `kovanica-execution::execute_committed_subdag_inner`:
    - Chain 5 detects `chain_id::HYBRID_TESTNET` and bypasses epoch committee lookups.
    - Each committed block in the linearized sequence mints `kovanica_params::INITIAL_BLOCK_REWARD` (2.5 KVNC = 2,500,000,000 atoms) to the block author address derived from `header.author`.
    - No validator commission, delegator cuts, or light-client escrow deductions (`escrow_part = 0`).
    - Total mining subsidy is tracked against the 81,000,000 KVNC mining budget.

---

## 9. Node Block Authoring & Miner Loop (P6)

Implemented in `crates/kovanica-node`:
- **Configuration:**
  - `[mining]` section in `config.toml`:
    ```toml
    [mining]
    enabled = false
    threads = 1
    ```
  - Overridable via `KOVANICA_MINING_ENABLED` and `KOVANICA_MINING_THREADS`.
- **DAG Tip & Ancestor Selection:**
  - `select_parents_from_tips(dag_store, max_parents = 5)`: selects up to k=5 candidate parent tips from the DAG store at the highest DAG round.
  - `extract_ancestor_timestamps`: collects past timestamps to evaluate the 60-block DAA window for difficulty calculation.
- **Autonomous Authoring Loop (`run_mining_loop`):**
  - Periodically pulls pending transactions from the mempool and computes the transaction Merkle root.
  - Queries DAA to compute `target_bits` for the new round.
  - Assembles `BlockHeader` and invokes `NodeMiner::mine_block` with multi-threaded KovanicaHash worker.
  - Upon successful nonce discovery:
    1. Converts `BlockHeader` to `StatementBlock` and saves to local `dag_store`.
    2. Feeds block into `GhostdagCommitter` to produce a `CommittedSubDag`.
    3. Submits committed sub-DAG to `kovanica-execution` via execution channel.
    4. Broadcasts block to network peers via gossipsub `NetworkCommand::PublishBlock`.

---

## 10. JSON-RPC Mining & DAG Inspection Endpoints (P7)

Implemented in crates/kovanica-rpc (chain_methods.rs):
- kovanica_getWork:
  - Returns mining template preimage, target compact bits (	arget_bits), target 256-bit hex, candidate DAG parent tips, and round for external miners or authoring loops.
- kovanica_submitWork:
  - Accepts a mined header (as JSON object or hex-encoded preimage + pow_hash).
  - Validates PoW via header.verify_pow() and timestamp monotonicity.
  - Ingests into local dag_store and broadcasts to P2P peers via gossip channel.
- kovanica_getMiningInfo:
  - Returns mining status (mining_enabled, mining_threads), target bits, target block time (4000 ms), GHOSTDAG parameter (=5$), difficulty, and network hashrate estimate.
- kovanica_getDagTips:
  - Returns active DAG tip digests, total tip count, max round, and GHOSTDAG =5$.

---

## 11. CLI Commands & Observability Metrics (P7)

Implemented in crates/kovanica-cli and crates/kovanica-consensus:
- **CLI Commands:**
  - kovanica mining info: Displays formatted table (or --json) of mining difficulty, target bits, target block time, active tips, and estimated network hashrate.
  - kovanica mining status: Displays node mining state (running vs disabled), worker threads, and current target bits.
  - kovanica dag tips: Displays active DAG tips, tip count, max round, and GHOSTDAG =5$.
- **Prometheus Metrics:**
  - pow_hashrate_estimate (Gauge): Estimated network hashrate derived from current DAA target bits.
  - pow_blocks_mined_total (Counter): Total PoW blocks mined by local worker.
  - ghostdag_blue_blocks_total (Counter): Total blue blocks committed in the GHOSTDAG main chain.
  - ghostdag_red_blocks_total (Counter): Total red (non-blue) blocks excluded from blue scores.
  - ghostdag_tips_count (Gauge): Current count of active DAG tips.

---

## 12. Multi-Node Simulation Test Suite (P8)

Implemented in crates/kovanica-consensus/tests/hybrid_pow_simulation.rs:
- 	est_multi_miner_ghostdag_dag_convergence:
  - Simulates 3 concurrent independent miners authoring blocks off shared tips across rounds.
  - Validates DAG convergence under GHOSTDAG =5$, topological mergeset ordering, and linear reward minting of 2.5 KVNC (INITIAL_BLOCK_REWARD = 2_500_000_000 atoms) per block author without stake commission.
- 	est_ghostdag_k5_blue_red_categorization:
  - Generates 8 concurrent sibling blocks referencing the same parent (DAG width 8 > =5$).
  - Validates that upon merging, exactly 5 siblings remain blue while 2 siblings are classified as red (
on_blue), preserving the $-cluster anti-chain bound.
- 	est_daa_difficulty_adaptation:
  - Tests DAA response: fast block intervals (1,000 ms) increase difficulty (target decreases); slow intervals (10,000 ms) decrease difficulty (target increases).
  - Enforces boundary clamping against MAX_TARGET_BITS (ceiling) and MIN_TARGET_BITS (floor).
- 	est_pow_anti_dos_rejection:
  - Verifies that tampered nonces, unmet difficulty targets, or blocks lacking PoW statements are rejected at ingress before entering consensus.


# Implementation Plan: Hybrid PoW + GHOSTDAG Engine (P5–P6)

P5/P6 scope: Integrate PoW verification at the network boundary (P2P anti-DoS ingress), wire the GHOSTDAG engine (k=5) for block sequencing on Chain 5 without stake weighting/certificates, and wire the node block authoring loop with the miner.

## Proposed Changes

### 1. `crates/kovanica-types` (Block & PoW Statements)
- In `crates/kovanica-types/src/block.rs`:
  - Define `POW_STATEMENT_MAGIC` (`b"KVNC-POW1"`, 9 bytes) and `POW_STATEMENT_LEN` (61 bytes).
  - Add `is_pow_statement(statements: &[u8]) -> bool`.
  - Add `encode_pow_statement(timestamp: u64, target_bits: u32, nonce: u64, pow_hash: Option<Hash>) -> Vec<u8>`.
  - Add `decode_pow_statement(statements: &[u8]) -> Option<(u64, u32, u64, Option<Hash>)>`.
  - Add `StatementBlock::extract_pow_header(&self, chain_id: u64) -> Result<BlockHeader, HeaderError>`.
  - Add `StatementBlock::from_pow_header(header: &BlockHeader, transactions: Vec<Transaction>) -> StatementBlock`.
  - Add `BlockHeader::to_statement_block(&self, transactions: Vec<Transaction>) -> StatementBlock`.
  - Ensure `compute_digest_statements` covers statements when `is_pow_statement(s)` is true.

### 2. `crates/kovanica-network` (P5: Network Ingress & Anti-DoS Boundary)
- In `crates/kovanica-network/src/validation.rs`:
  - Extend `Rejection` with `BadPow(HeaderError)` and `MissingPow`.
  - In `verify_block`:
    - For `chain_id == CHAIN_ID_HYBRID_TESTNET` (Chain 5):
      - Check statement/block digest.
      - Check transaction Merkle root.
      - Extract `BlockHeader` via `block.extract_pow_header(chain_id)`.
      - Verify PoW: `pow_header.verify_pow()`.
      - Reject if missing or invalid PoW, or target not met.
      - Do NOT require committee keys or Ed25519 author signatures under committee keys.
    - For all other chains (1, 2, 3, 4, 1337): keep existing committee signature validation unchanged.
- In `crates/kovanica-network/src/service.rs`:
  - In `on_block_message`:
    - Support deserializing `StatementBlock` or raw `BlockHeader` for gossip.
    - If `chain_id == CHAIN_ID_HYBRID_TESTNET`, do not drop unforwarded when committee keys are not set; proceed to PoW verification.
    - Reject and score down peers sending invalid PoW blocks.
  - In `BlockSyncResponse::Block`:
    - Ensure sync blocks on Chain 5 undergo `verify_block` PoW verification before acceptance.

### 3. `crates/kovanica-consensus` (P6: GHOSTDAG Committer)
- Create `crates/kovanica-consensus/src/ghostdag.rs`:
  - Define `GhostdagCommitter`:
    - Parameter `k = 5` (`GHOSTDAG_K`).
    - Parent selection: select parent maximizing cumulative PoW work / blue score / hash tie-break.
    - Mergeset calculation relative to selected parent.
    - GHOSTDAG mergeset coloring using `colour_mergeset(mergeset, 5, previous_tips)`.
    - Selected parent chain tracking.
    - Linearization into `CommittedSubDag` with blue-ordered blocks and red `non_blue` references.
- In `crates/kovanica-consensus/src/lib.rs`:
  - Export `ghostdag` module and `GhostdagCommitter`.

### 4. `crates/kovanica-execution` (P6: Block Reward Minting without Stake Commission)
- In `crates/kovanica-execution/src/lib.rs`:
  - In `execute_committed_subdag_inner`:
    - If `self.signing_ctx.chain_id == chain_id::HYBRID_TESTNET` (Chain 5):
      - Mint `INITIAL_BLOCK_REWARD` (2.5 KVNC = 2_500_000_000 atoms) directly to the author of each confirmed block in `subdag.blocks` and `subdag.leader`.
      - 100% of the block reward goes to the miner: no validator commission, no delegator split, no light-node escrow deduction (`0` bps).

### 5. `crates/kovanica-node` (Miner Loop & Config Wiring)
- In `crates/kovanica-node/src/config.rs`:
  - Add `MiningConfig { enabled: bool, threads: usize }`.
  - Add `pub mining: MiningConfig` to `NodeConfig`.
  - Add env overrides: `KOVANICA_MINING_ENABLED`, `KOVANICA_MINING_THREADS`.
- In `crates/kovanica-node/src/miner.rs`:
  - Add DAG tip parent selection.
  - Add ancestor timestamp collection for DAA.
  - Add `run_mining_loop` task or authoring helper.
- In `crates/kovanica-node/src/main.rs`:
  - When `config.chain_id == CHAIN_ID_HYBRID_TESTNET` and `config.mining.enabled`:
    - Spawn `run_mining_loop`.

### 6. Documentation & State
- Update `docs/dev/POW-HEADER-SPEC.md` with P5/P6 specifications.
- Update `STATE.md`.

## Verification Plan
1. `cargo fmt --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace --locked --no-fail-fast`
4. `python scripts/lint_docs.py`

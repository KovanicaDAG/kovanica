# Canonical Testnet 2 (Hybrid PoW + GHOSTDAG) Migration Design

- **Status:** Proposal / Design Document (Documentation Only)
- **Target Branch:** `docs/canonical-testnet-2-plan`
- **Reference Main:** `ee5db899e8b5e2f2d922fbe5034fc2dc1e7fab12`
- **Related ADR / PRs:** [ADR 0003: Hybrid PoW + GHOSTDAG Consensus](../adr/0003-hybrid-pow-ghostdag.md), PR #182, PR #183, PR #184, PR #186, PR #188, PR #189, PR #190, PR #191

---

## Executive Summary

On 2026-11-05, project owner Toni locked the product consensus architecture to **Hybrid PoW + GHOSTDAG** with **NO stake in the consensus path**, a 4-second block target (`TARGET_BLOCK_MS = 4000`), GHOSTDAG $k=5$ (`GHOSTDAG_K = 5`), and the memory-hard PoW algorithm **KovanicaHash** ([ADR 0003](../adr/0003-hybrid-pow-ghostdag.md)).

Initial implementation and milestone M8 harnesses were introduced under **Chain ID 5** (`HYBRID_TESTNET`) as an isolated test network. This proposal provides the complete source-backed design for transitioning the canonical public testnet—traditionally designated as **Chain ID 2** (`TESTNET`)—to the Hybrid PoW + GHOSTDAG architecture, retiring the legacy Mysticeti consensus path for public testnet operations.

> [!IMPORTANT]
> **No Code Changes in This PR:** This document is strictly an architectural proposal and migration inventory. It does not modify consensus dispatch, edit chain IDs, create genesis state, touch running seed nodes, or alter existing data directories.

---

## 1. Full Inventory of Chain-ID-Dependent Behaviour

A thorough audit of the tip codebase (`ee5db899e8b5e2f2d922fbe5034fc2dc1e7fab12`) reveals all locations where node behaviour diverges based on `chain_id`. Simply setting `KOVANICA_CHAIN_ID=2` today does **not** run hybrid consensus; it falls back to the legacy Mysticeti validator pipeline and rejects hybrid blocks.

### 1.1. Hybrid Consensus Dispatch
- **`crates/kovanica-types/src/signing.rs:32-34`**:
  ```rust
  pub fn is_hybrid(id: u64) -> bool {
      id == HYBRID_TESTNET // Only returns true for Chain ID 5
  }
  ```
  `is_hybrid` is the central dispatch predicate for hybrid behaviour. Currently, `TESTNET` (`2`) returns `false`.

### 1.2. Node Startup & Engine Wiring
- **Validator Key Requirement (`crates/kovanica-node/src/main.rs:954-955`)**:
  ```rust
  let is_hybrid = kovanica_types::signing::chain_id::is_hybrid(config.chain_id);
  if config.run_validator && !is_hybrid { ... }
  ```
  On non-hybrid chains, running as a validator (`run_validator = true`) requires a validator signing key seed (`validator.key`). On hybrid chains, consensus block production is driven by PoW mining rather than validator keys.
- **Mysticeti Background Tasks (`crates/kovanica-node/src/main.rs:1029-1037`)**:
  ```rust
  if !is_hybrid {
      // Spawns Mysticeti propose loop and commit ticker
  }
  if !is_hybrid {
      // Spawns Mysticeti vote processing loop
  }
  ```
  When `is_hybrid` is false (as on Chain 2 currently), the node spawns Mysticeti proposal and vote loops.
- **Mining Worker Loop Activation (`crates/kovanica-node/src/main.rs:1076`)**:
  ```rust
  let _mining_task = if config.chain_id == kovanica_types::signing::chain_id::HYBRID_TESTNET
      && config.mining.enabled
  ```
  **Hardcoded to Chain ID 5**. Even if `is_hybrid(2)` were made true, `main.rs:1076` would not launch the background PoW mining loop on Chain 2 unless updated.

### 1.3. Block Ingestion & Admission Hot Paths
- **Hybrid Ingestion (`crates/kovanica-node/src/main.rs:2534`)**:
  ```rust
  if io.chain_id == kovanica_types::signing::chain_id::HYBRID_TESTNET {
      // 1. Extract PoW header
      // 2. Cheap structural parent validation (<= K=5, no duplicates, no self-reference)
      // 3. Round constraints & drift checks
      // 4. Memory-hard PoW verification
      // 5. Parent presence in DagStore
      // 6. Required DAA target derivation
  }
  ```
  **Hardcoded to Chain ID 5**. On Chain 2, `ingest_block` currently skips PoW extraction, parent structural validation, and DAA target derivation, falling through to legacy Mysticeti committee admission.
- **Main Event Loop Hot Path (`crates/kovanica-node/src/main.rs:4239-4256`)**:
  ```rust
  let is_chain5 = mempool.signing_context().chain_id
      == kovanica_types::signing::chain_id::HYBRID_TESTNET;
  if !is_chain5 {
      // Hot-path validation: batch verify Ed25519 signatures
      kovanica_crypto::verify_batch(...)
  } else if let Err(e) = block
      .extract_pow_header(mempool.signing_context().chain_id)
      .and_then(|h| h.verify_pow())
  ```
  **Hardcoded to Chain ID 5**. Gossip events on Chain 2 are currently subjected to Ed25519 validator signature checks instead of PoW verification.

### 1.4. Network Layer & Gossipsub Pre-Checks
- **Block Gossip Service Filter (`crates/kovanica-network/src/service.rs:1525-1530`)**:
  ```rust
  if self.config.network_id.chain_id != kovanica_types::signing::chain_id::HYBRID_TESTNET
      && !self.validator.is_configured()
  {
      debug!("committee keys not set; block gossip dropped unforwarded");
      return Verdict::Ignore;
  }
  ```
  On non-Chain 5 nodes without committee keys configured, block gossip is dropped unforwarded.
- **Gossip Validation (`crates/kovanica-network/src/validation.rs:254`)**:
  ```rust
  if chain_id == kovanica_types::signing::chain_id::HYBRID_TESTNET {
      // PoW header extraction and validation before gossip forwarding
  }
  ```
  **Hardcoded to Chain ID 5**.

### 1.5. Consensus GHOSTDAG Committer
- **Work Extraction (`crates/kovanica-consensus/src/ghostdag.rs:88`)**:
  ```rust
  let work = if let Ok(header) =
      block.extract_pow_header(kovanica_params::CHAIN_ID_HYBRID_TESTNET)
  {
      calculate_work_from_target(header.target_bits)
  } else {
      1u128
  };
  ```
  **Hardcoded to Chain ID 5**. If a block header declares `chain_id = 2`, `extract_pow_header(5)` fails with `HeaderError::ChainMismatch`, causing `GhostdagCommitter` to assign a fallback work weight of `1u128` instead of the DAA difficulty weight.

### 1.6. Execution Engine & Block Reward Minting
- **Committee Resolution (`crates/kovanica-execution/src/lib.rs:395`)**:
  ```rust
  let epoch_committee = if self.signing_ctx.chain_id == chain_id::HYBRID_TESTNET {
      Vec::new()
  } else {
      ensure_epoch_committee(storage, &self.staking, leader_epoch)?
  };
  ```
  Skips Mysticeti epoch committee resolution only when `chain_id == 5`.
- **Block Reward Minting (`crates/kovanica-execution/src/lib.rs:458`)**:
  ```rust
  if self.signing_ctx.chain_id == chain_id::HYBRID_TESTNET {
      if emission_policy == EmissionPolicy::NoStake82M {
          // Mint 2.5 KVNC directly to eligible block authors
      }
  }
  ```
  PoW block reward minting directly to block authors is hardcoded to `chain_id == 5`. On Chain 2, it falls through to legacy Mysticeti staking commission splits.
- **Light Escrow Staking (`crates/kovanica-execution/src/lib.rs:613`)**:
  ```rust
  if self.signing_ctx.chain_id != chain_id::HYBRID_TESTNET {
      self.pay_light_escrow(...)?;
  }
  ```
  On non-Chain 5 networks, execution attempts to pay light escrow rewards to legacy staking pools.

### 1.7. Cryptographic Signing Domains & Preimages
- **Transaction Preimage (`TX_DOMAIN_TAG = "KUNA/tx/v1"`, `crates/kovanica-types/src/signing.rs:59`)**:
  Every transaction signature preimage embeds `chain_id` as an 8-byte little-endian integer (`u64::to_le_bytes`). A transaction signed for Chain 5 cannot be replayed on Chain 2.
- **PoW Header Preimage (`POW_HEADER_DOMAIN_TAG = "KVNC-POW-HDR-v1"`, `crates/kovanica-types/src/block.rs:133-145`)**:
  PoW header preimage strictly binds `chain_id` LE:
  `POW_HEADER_DOMAIN_TAG (16B) ‖ chain_id (8B) ‖ round (8B) ‖ author (2B) ‖ parents_hash (32B) ‖ merkle_root (32B) ‖ timestamp (8B) ‖ target_bits (4B) ‖ nonce (8B)`.
  A PoW solution mined for Chain 5 is cryptographically invalid for Chain 2.
- **Consensus Domains**: `BLOCK_DOMAIN_TAG ("KUNA/block/v1")`, `COMMIT_DOMAIN_TAG ("KUNA/commit/v1")`, and `VOTE_DOMAIN_TAG ("KUNA/vote/v2")` all bind `chain_id` LE.

### 1.8. Address Format & Legacy Hex Policy
- **Address Validation Policy (`crates/kovanica-types/src/address.rs:35-43`)**:
  ```rust
  pub fn legacy_address_allowed(chain_id: u64) -> bool {
      matches!(
          chain_id,
          crate::signing::chain_id::DEVNET
              | crate::signing::chain_id::LOCAL
              | crate::signing::chain_id::TESTNET_V2
              | crate::signing::chain_id::HYBRID_TESTNET
      )
  }
  ```
  - **Chain 2 (`TESTNET`):** `legacy_address_allowed(2) == false`. **Strictly requires canonical 64-character PRO-21 base32 addresses (`kvnc...dag`)**. Legacy hex (`kvnc<hex>dag`, bare 64-hex, `0x`-prefixed 64-hex) is rejected immediately with `AddressError::MissingAffix` or `AddressError::InvalidBase32`.
  - **Chain 5 (`HYBRID_TESTNET`):** `legacy_address_allowed(5) == true`.
  - **Mainnet (`MAINNET = 1`):** `legacy_address_allowed(1) == false`.
  Canonical Testnet 2 strictly matches Mainnet address rules.

### 1.9. Genesis Configuration Digest & Storage Guards
- **Configured Genesis Digest Calculation (`crates/kovanica-node/src/main.rs:1986-2105`)**:
  The `genesis_config_digest` is a keyed BLAKE3 hash over:
  1. Emission policy (`Pno-stake-82m` if set).
  2. Canonical round-0 genesis block digest (`canonical_genesis_digest()`).
  3. **Signed `chain_id` as 8-byte LE** (`config.chain_id.to_le_bytes()`).
  4. Treasury address (parsed via `parse_address_hex(..., config.chain_id)`).
  5. Parsed entries of `validators.json`, `founder_premine.hex`, `funded_accounts.toml`, and `genesis_validators.toml`.
- **Address Parsing in Genesis Files (`crates/kovanica-node/src/main.rs:5749-5755`)**:
  All addresses in genesis files are parsed via `Address::parse_for_chain(value, config.chain_id)`.
  **Critical implication for Chain 2:** Any address in `funded_accounts.toml` or `genesis_validators.toml` formatted in legacy hex will cause `parse_address_hex` to fail on Chain 2, crashing node startup! All Chain 2 genesis allocations must use canonical `kvnc...dag` addresses.
- **Persisted Genesis Digest Mismatch Guard (`crates/kovanica-node/src/main.rs:2117-2178`)**:
  `verify_persisted_genesis_digest` loads the stored genesis digest from `state.redb`. If `stored != *fresh`, startup is refused. The override `KOVANICA_ALLOW_GENESIS_MISMATCH` is **strictly forbidden on non-dev chains** (`allow_mismatch && is_dev(chain_id)` where `is_dev(2) == false`).
- **Consensus Genesis Marker Guard (`crates/kovanica-node/src/main.rs:2193-2220`)**:
  `verify_consensus_genesis_marker` checks `consensus.redb`. If the stored marker does not match `fresh`, the node hard-aborts to prevent reopening another chain's DAG.

### 1.10. Network Layer Identity & Protocol Multiplexing
- **Network Tag (`crates/kovanica-network/src/network_id.rs:50-55`)**:
  `<tag>` is `<chain_id>-<first 16 hex chars of genesis_config_digest>`.
  - Protocol streams: `/kovanica/<tag>/kad/1.0.0`, `/kovanica/<tag>/block-sync/1.1.0`, `/kovanica/<tag>/state-sync/2.0.0`.
  - Libp2p Identify: `/kovanica/<tag>/1.0.0`.
  - Gossipsub topics: `kovanica/<tag>/blocks`, `kovanica/<tag>/transactions`.
  Peers advertising a mismatched tag are disconnected during identify. No wire traffic or DHT records are exchanged between different chain IDs or different genesis digests.

### 1.11. RPC Mining Endpoints
- **RPC Availability Gates (`crates/kovanica-rpc/src/chain_methods.rs:1921, 2047, 2245`)**:
  `kovanica_getWork`, `kovanica_getDagTips`, `kovanica_getMiningInfo`, and `kovanica_submitWork` all execute:
  ```rust
  if !kovanica_types::signing::chain_id::is_hybrid(chain_id) {
      return Err(RpcError::InvalidRequest(format!(
          "... only available on hybrid PoW networks; node is running on chain_id {chain_id}"
      )));
  }
  ```

### 1.12. Standalone Solo Miner CLI
- **`crates/kovanica-cli/src/miner.rs:579`**:
  Defaults to `HYBRID_TESTNET` (`5`).
  Validates payout address with `Address::parse_for_chain(&args.address, chain_id)` (line 56), which enforces canonical `kvnc...dag` addresses on Chain 2.

### 1.13. Operations & Docker Compose
- **`docker-compose.yml`**: Currently sets `KOVANICA_CHAIN_ID: "4"` for public testnet nodes.
- **`ops/docker/docker-compose.chain5.yml`**: Configures 3-node cluster + observer on `KOVANICA_CHAIN_ID: "5"`.

### 1.14. External Ecosystem Consumers
- **`kovanica-sdk`**:
  - `src/chains.ts`: Defines `CHAIN_IDS.TESTNET = 2` as Mysticeti (`allowsLegacyAddress: false`) and `CHAIN_IDS.HYBRID_TESTNET = 5` as Hybrid PoW (`allowsLegacyAddress: true`).
  - `src/address.ts:173`: Enforces `isLegacyAddressAllowed(chainId)`.
- **`kovanica-playground`**:
  - `src/App.tsx:479-484`: Offers Chain 5 for Hybrid PoW and Chain 2 as "Testnet (Chain 2 - Legacy)".
- **`kovanica-apps` (Android Wallet, Android Comms, Android Light)**:
  - `ChainIds.kt`: `ChainIds.TESTNET` is currently mapped to `4`, and `ChainIds.TESTNET_V1` to `2`.
  - `KvncAddress.kt`: `legacyAllowed(2) == false`.
- **`kovanica-web`**:
  - `dispatch.server.ts`: Defaults to `chainId = 4`.

---

## 2. Migration Options & Evaluation

### Option A: Chain 2 Becomes the Canonical Hybrid Testnet via Explicit Network Parameters (Recommended)

In this option, Chain ID 2 is formally promoted to be the canonical Hybrid PoW + GHOSTDAG testnet. Chain ID 5 is either retained concurrently for isolated local integration tests or deprecated.

- **Required Code Modifications (Core L1):**
  1. `crates/kovanica-types/src/signing.rs`:
     Update `is_hybrid(id: u64)`:
     ```rust
     pub fn is_hybrid(id: u64) -> bool {
         matches!(id, TESTNET | HYBRID_TESTNET)
     }
     ```
  2. `crates/kovanica-node/src/main.rs`:
     - Line 1076: Replace `config.chain_id == chain_id::HYBRID_TESTNET` with `is_hybrid`.
     - Line 2534: Replace `io.chain_id == chain_id::HYBRID_TESTNET` with `kovanica_types::signing::chain_id::is_hybrid(io.chain_id)`.
     - Line 4240: Replace `== chain_id::HYBRID_TESTNET` with `kovanica_types::signing::chain_id::is_hybrid(chain_id)`.
  3. `crates/kovanica-network/src/service.rs:1525` & `validation.rs:254`:
     Replace `== HYBRID_TESTNET` with `kovanica_types::signing::chain_id::is_hybrid(chain_id)`.
  4. `crates/kovanica-consensus/src/ghostdag.rs:88`:
     Parameterize `GhostdagCommitter` with `chain_id` and replace hardcoded `CHAIN_ID_HYBRID_TESTNET` with the committer's `chain_id`.
  5. `crates/kovanica-execution/src/lib.rs:395, 458, 613`:
     Replace `self.signing_ctx.chain_id == chain_id::HYBRID_TESTNET` with `kovanica_types::signing::chain_id::is_hybrid(self.signing_ctx.chain_id)`.
  6. `crates/kovanica-cli/src/miner.rs:579`:
     Default `chain_id` to `TESTNET` (`2`) or make it an explicit flag.
- **Consensus & Activation Impact:**
  - Complete, clean break from legacy Mysticeti. Not a soft fork or state migration; a brand new genesis deployment.
  - Aligns naming: `TESTNET` (Chain 2) becomes the sole canonical public testnet.
- **Address-Format Decision:**
  - Chain 2 **already enforces** `legacy_address_allowed(2) == false`.
  - Maintain this strict invariant: all genesis accounts, RPC calls, faucet transactions, and mining payouts on Chain 2 **must use canonical 64-character PRO-21 base32 (`kvnc...dag`)**. This ensures public testnet perfectly mirrors Mainnet address hygiene.
- **Test Plan:**
  - Unit tests asserting `is_hybrid(2) == true`.
  - Ingestion and admission regressions executing with `chain_id: 2`.
  - Cluster acceptance test parameterization to verify two nodes mining and observing on Chain 2.

### Option B: Structured `ChainSpec` Architecture in `kovanica-params`

Instead of multiple `is_hybrid` checks scattered across 11 files, introduce a formal `ChainSpec` registry in `kovanica-params`.

- **Concept:**
  ```rust
  pub enum ConsensusKind {
      MysticetiClassic,
      HybridGhostdagPow,
  }

  pub struct ChainSpec {
      pub chain_id: u64,
      pub consensus: ConsensusKind,
      pub allows_legacy_address: bool,
      pub default_target_ms: u64,
  }
  ```
- **Evaluation:**
  Architecturally cleanest, but constitutes a larger refactoring footprint across crates. Can be performed as a second-phase refinement after Option A establishes functional parity.

---

## 3. Fresh Genesis & Fresh Dedicated Storage Requirements

### 3.1. Why Relabeling or Renaming an Existing Database Is Technically Impossible
An operator might wonder if an existing Chain 5 database (`data/chain5`) or legacy Chain 2 database from older seeds can simply be renamed or pointed to with `KOVANICA_CHAIN_ID=2`. **This is fundamentally impossible and will cause the node to immediately crash with an unrecoverable startup error:**

1. **Cryptographic Genesis Digest Guard (`state.redb`):**
   - The genesis transaction writes `state_storage.state().set_genesis_digest(fresh)`.
   - `genesis_config_digest` hashes the little-endian bytes of `chain_id` into the digest.
   - If a directory was created under `chain_id = 5`, its persisted digest was computed with `5`. Booting with `chain_id = 2` computes a different digest.
   - `verify_persisted_genesis_digest` checks `persisted != fresh`. Because `is_dev(2) == false`, the override `KOVANICA_ALLOW_GENESIS_MISMATCH` is rejected, and the node exits:
     ```
     Error: genesis digest mismatch: this data dir belongs to genesis ... but this build and config give ...; refusing to start to avoid mixing chains. The fix is a wipe
     ```
2. **Consensus DAG Marker Guard (`consensus.redb`):**
   - `verify_consensus_genesis_marker` checks the genesis marker table in `consensus.redb`.
   - A mismatched marker bails with:
     ```
     Error: consensus.redb belongs to genesis ... but this build and config give ...; refusing to reopen another chain's DAG and commit cursor.
     ```
3. **PoW Statement Absence in Legacy Data:**
   - If an old seed1 Docker volume from legacy Chain 2 is attached, its DAG blocks lack `Statement::PowHeader`.
   - The hybrid DAA ancestry crawler and GHOSTDAG committer will fail to extract PoW headers from history, resulting in immediate panic or rejection.

### 3.2. Dedicated Storage Specification for Canonical Testnet 2
- **Data Directory:** Must be a fresh, uninitialized directory (e.g. `/var/lib/kovanica/testnet2` or `./data/testnet2`).
- **Genesis Bundle:** Every node in the testnet mesh must receive the exact same byte-identical genesis files:
  - `genesis_validators.toml` declaring `chain_id = 2` and canonical `kvnc...dag` addresses.
  - `funded_accounts.toml` declaring canonical `kvnc...dag` addresses.
- **Node Refusal Verification:** When deploying, nodes must verify that any preexisting data directory with an old digest is refused at startup.

---

## 4. Disposition of Chain-5 Dev Data & Ecosystem Follow-Ups

### 4.1. Chain 5 Test Suites & Local Cluster
- **Coexistence:** Chain ID 5 can remain in `crates/kovanica-types/src/signing.rs` as a registered dev/test chain.
- **Automated Tests:** Existing regression suites (`hybrid_pow_simulation`, `isolated_hybrid_acceptance`, `hybrid_node_cluster_integration`) can remain on Chain 5 or be parameterized to run on Chain 2.
- **No Production Chain 5 Data Exists:** Chain 5 was strictly an internal testnet; no external user funds, validator stakes, or permanent contracts exist on Chain 5.

### 4.2. Downstream Ecosystem Follow-Up Inventory (Non-L1 Repos)
These tasks must be tracked for implementation across the workspace and downstream repositories:

1. **`kovanica-sdk` (`/root/Projects/kovanica-sdk`):**
   - Update `src/chains.ts`: Change `CHAIN_IDS.TESTNET` (2) config from `consensus: "Mysticeti"` to `consensus: "Hybrid PoW + GHOSTDAG"`, `targetBlockTimeMs: 4000`, `ghostdagK: 5`.
   - Ensure `isLegacyAddressAllowed(2)` remains `false`.
   - Add RPC client bindings for `kovanica_getWork`, `kovanica_submitWork`, `kovanica_getMiningInfo`, `kovanica_getDagTips` against Chain 2.
2. **`kovanica-apps` (`/root/Projects/kovanica-apps`):**
   - Update `android-chain/src/main/kotlin/io/kovanica/chain/ChainIds.kt`: Update `TESTNET` constant from 4 to 2 (or define `CANONICAL_TESTNET = 2`).
   - Update `Network.kt` across `android-wallet`, `android-comms`, and `android-light` to point the Testnet profile to Chain 2 RPC endpoints.
3. **`kovanica-playground` (`/root/Projects/kovanica-playground`):**
   - Update `src/App.tsx` network selector: Set default testnet to `Chain 2 (Canonical Hybrid Testnet)`.
4. **`kovanica-web` (`/root/kovanica-web`):**
   - Update `src/lib/api/dispatch.server.ts` default chain ID from 4 to 2.
5. **Infrastructure & Seed Deployment:**
   - Update `ops/docker/docker-compose.testnet2.yml`.
   - Coordinate wipe of disposable testnet volumes on seed nodes and deploy fresh Chain 2 genesis.

---

## 5. Explicit Decisions Toni Must Make

The following numbered architectural decisions require explicit sign-off from Toni before code implementation begins:

1. **Dual-Chain Coexistence vs. Hard Cutover:**
   Should `is_hybrid(id)` accept both `TESTNET` (2) and `HYBRID_TESTNET` (5) to allow CI and local test harnesses to continue running on Chain 5 while deploying Chain 2 publicly, or should Chain 5 be deprecated entirely in favour of Chain 2?
   *(Recommendation: Dual acceptance in `is_hybrid`—`matches!(id, TESTNET | HYBRID_TESTNET)`—for zero disruption to existing M8 test harnesses).*

2. **Emission Policy for Canonical Testnet 2:**
   Should Chain 2 launch under `EmissionPolicy::NoStake82M` (82,000,000 KVNC mining budget, 0 genesis validator allocation per Toni 2026-11-05) or `EmissionPolicy::Legacy81M` (81,000,000 KVNC mining budget + 1,000,000 KVNC genesis validator allocation)?
   *(Recommendation: `no-stake-82m`, directly matching ADR 0003 pure PoW tokenomics).*

3. **Address Policy Confirmation:**
   Confirm that Chain 2 will strictly maintain `legacy_address_allowed(2) == false`, requiring all genesis accounts, RPC queries, and wallet users to use 64-character PRO-21 base32 addresses (`kvnc...dag`), rejecting bare hex and legacy `kvnc<hex>dag` addresses.
   *(Recommendation: Strictly confirm `false`; public testnet should enforce the exact same address standards as Mainnet).*

4. **Decommissioning Plan for Chain 4 (`TESTNET_V2`):**
   Confirm the timeline for sunsetting the legacy Mysticeti Chain 4 seed nodes once the Canonical Testnet 2 mesh achieves stable block production and observer synchronization.

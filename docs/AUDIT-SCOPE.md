# Kovanica External Security Audit — Scope Document

**Version:** 2.0  
**Date:** 2026-11-09  
**Codebase:** `kovanica` (Hybrid PoW + GHOSTDAG, account-based, Wasmi execution)  
**Repository:** `https://github.com/KovanicaDAG/kovanica`  
**License:** GPL-3.0-or-later  
**Native Coin:** **KVNC** (9 decimals; `ONE_KVNC = 10^9`)  

---

## 1. Executive Summary

Kovanica is a clean, from-scratch Layer-1 blockchain engineered for decentralized, secure, and low-latency value transfer and smart contracts.

Key architectural pillars (confirmed Toni 2026-11-05):
- **Consensus:** Hybrid Proof-of-Work (PoW) + GHOSTDAG. **Strictly NO stake in the consensus path.**
- **PoW Hash:** **KovanicaHash** (BLAKE3 + 2 MiB memory-hard mix scratchpad; deterministic verification).
- **Consensus Defaults:** Target block time `TARGET_BLOCK_MS = 4000` (4 s) and GHOSTDAG parameter `GHOSTDAG_K = 5`.
- **Execution:** Wasmi (deterministic WebAssembly engine with strict fuel-based gas metering).
- **Account Model:** Account-based state transitions with nonces and cryptographic replay protection.
- **Resource Footprint:** Optimized to run comfortably on standard 2–4 GB RAM VPS infrastructure.
- **Address Standard:** PRO-21 canonical base32 address: `kvnc` + 57 bech32 characters + `dag` = **64 characters** (32-byte Ed25519 pubkey ‖ 29-bit BLAKE3 checksum). Legacy hex addresses are strictly prohibited on Mainnet (`chain_id = 1`).

This scope document specifies the components, invariants, threat models, and priority review tracks for an external security audit preceding the Kovanica Mainnet $T_0$ launch.

---

## 2. In-Scope Components

### 2.1 Consensus & PoW — `kovanica-consensus`, `kovanica-dag`, `kovanica-types` (CRITICAL)

| Module / Component | Risk | Description |
|---|---|---|
| `kovanica-crypto::pow` | **Critical** | **KovanicaHash** algorithm: 2 MiB memory scratchpad initialization from seed hash, sequential pseudo-random memory lookups, and final BLAKE3 compression. Memory-hardness against ASIC acceleration; deterministic evaluation across all target architectures. |
| `kovanica-types::header` | **Critical** | PoW header format: `nonce` (u64), `mix_hash` (Hash), `target` / `bits` (u32 compact representation), mining pre-image serialization, and target verification (`target_from_compact_bits`, `meets_target`). |
| `ghostdag.rs` / `ghostdag_scoped.rs` | **Critical** | GHOSTDAG consensus implementation with `K = 5`: blue set selection, mergeset evaluation, topological DAG ordering, past/future reachability queries, and tips election. |
| `daa.rs` (Difficulty Adjustment) | **Critical** | Difficulty adjustment algorithm targeting 4 s block time (`TARGET_BLOCK_MS = 4000`). Adjustment dampening, min/max target clamping, timestamp median calculations, and resistance against timestamp warping / selfish mining manipulation. |
| `block_manager.rs` (DAG Store) | **Critical** | Ingestion pipeline, parent validation, duplicate detection, topological commit cursor advancement, and pruning bounds. |
| Anti-DoS Edge Ingress | **Critical** | Header PoW verification before gossip relay (`validate_pow_target`). Ensures un-mined or low-difficulty blocks cannot flood the P2P mesh or exhaust memory before verification. |
| Legacy Mysticeti Transition | **High** | Note that PRO-7 (Mysticeti VRF leader selection) is **FROZEN** under Hybrid PoW + GHOSTDAG. Verify that legacy leader/stake logic does not interfere with the PoW + GHOSTDAG execution path on the hybrid networks (Chain 1 and Chain 2). |

**Key Consensus Invariants to Verify:**
1. **Deterministic DAG Ordering:** All honest nodes processing the same set of DAG blocks derive an identical GHOSTDAG linear sequence and identical state root.
2. **KovanicaHash Determinism:** Given any header pre-image and nonce, `kovanica_hash(preimage, nonce)` produces identical output and `mix_hash` across x86_64, aarch64, and other Tier-1 platforms.
3. **No Stake in Consensus:** Validator stakes and delegations do NOT grant block proposal or commit rights; block production is purely PoW-mined, and block ordering is purely GHOSTDAG K=5.
4. **Anti-DoS Relay Bound:** Blocks failing `meets_target(header.pow_hash, header.target)` are dropped immediately at the network edge with peer penalty, without entering the DAG store or mempool.
5. **Target Monotonicity & Clamping:** Target adjustments stay strictly within protocol bounds (`MAX_TARGET` limit, max factor change per adjustment window).

---

### 2.2 Execution & WASM Runtime — `kovanica-execution`, `kovanica-runtime` (CRITICAL)

| Module / Component | Risk | Description |
|---|---|---|
| `Runtime::execute` (`kovanica-runtime`) | **Critical** | Wasmi bytecode engine: fuel deduction per opcode, deterministic trap handling, stack depth limits, linear memory isolation. |
| Resource Ceilings | **Critical** | Maximum memory: 16 WASM pages (1 MiB per instance). Gas limit: default 10,000,000 gas. Fuel-to-gas ratio calibration. |
| Host ABI Imports (`wasm.rs`) | **Critical** | Host functions imported by WASM: `kovanica_caller`, `kovanica_contract_address`, `kovanica_block_height`, `kovanica_timestamp`, `kovanica_balance_of`, `kovanica_storage_get`, `kovanica_storage_set`, `kovanica_transfer`, `kovanica_emit_log`, `kovanica_alloc`, `kovanica_dealloc`. Verification of memory safety across host/guest boundary and protection against reentrancy. |
| Native Contracts | **High** | Built-in smart contracts: HTLC, Vault, Multisig, Token. Packed error codes, state rollback on abort, host trait compliance. |
| State Application & State Root | **Critical** | Application of committed block transactions to the account state tree, balance deduction, nonces increment, deterministic Merkle/Radix state root calculation. |

**Key Execution Invariants to Verify:**
1. **Determinism:** Identical input transactions executed against the same pre-state yield identical post-state, identical gas consumption, and identical state roots regardless of host OS or architecture.
2. **Memory Sandboxing:** A WASM contract cannot read or write beyond its allocated 16 linear memory pages, cannot cause host panics, and cleans up completely on out-of-gas traps.
3. **Reentrancy Protection:** Host state transitions prevent reentrant exploitation during cross-contract or native contract invocations.

---

### 2.3 Tokenomics & Emission — `kovanica-staking`, `kovanica-params` (CRITICAL)

| Area | Risk | Description |
|---|---|---|
| Hard Cap & Total Supply | **Critical** | Total supply is strictly capped at **90 200 000 KVNC** (`90_200_000 * 10^9` base units). |
| Mining Subsidy Emission | **Critical** | Initial block reward $s_0 = \mathbf{2,5\text{ KVNC}}$ per committed block, minted to the block's `author` address. |
| Subsidy Decay Curve | **Critical** | Decay factor $\times \frac{3}{4}$ applied every **8 200 000** rewarded blocks; total mining subsidy budget is exactly **82 000 000 KVNC** (`NoStake82M`). |
| Treasury Allocation | **High** | **8 000 000 KVNC** vested linearly over 8 years (1 000 000 KVNC/year), claimed via `claim_treasury_vesting(committed_height)`. |
| Founder Premine | **High** | **200 000 KVNC** allocated at genesis. |
| Genesis Validator Allocation | **High** | **0 KVNC** on chains 1 and 2 (`NoStake82M`); genesis bootstrap seeds (1..=20) hold no stake, and the former 1 000 000 KVNC is folded into the mining budget. |
| Mainnet `funded_accounts.toml` Prohibition | **Critical** | Arbitrary pre-funding via `funded_accounts.toml` is **strictly prohibited on Mainnet (`chain_id = 1`)**. Code rejects node boot if `funded_accounts.toml` is present on Chain 1. |

**Key Supply Invariant:**
$$\text{TOTAL\_SUPPLY} = \text{PREMINE } (0.2\text{M}) + \text{TREASURY } (8\text{M}) + \text{GENESIS\_ALLOC } (0) + \text{MINING\_BUDGET } (82\text{M}) = \mathbf{90\text{ }200\text{ }000\text{ KVNC}}$$

---

### 2.4 Mempool — `kovanica-mempool` (HIGH)

| Pipeline Step | Risk | Description |
|---|---|---|
| Admission Filtering | **High** | Transaction size bound (`MAX_TX_BYTES = 512 KiB`), minimum gas floor, fee floor, Ed25519 strict signature verification, hash binding (`tx.hash == tx.signing_hash()`). |
| Nonce & Balance Accounting | **High** | Cumulative balance checks across all pending transactions for a sender; sequential nonce enforcement with sliding window. |
| Replace-By-Fee (RBF) | **High** | Strict 10% minimum fee rate bump required to replace an existing in-flight or mempool transaction. |
| Capacity & Eviction | **Medium** | Eviction of strictly cheaper transactions under congestion; anti-DoS bounds preventing memory exhaustion. |

---

### 2.5 Network & P2P — `kovanica-network` (HIGH)

| Component | Risk | Description |
|---|---|---|
| Libp2p Swarm & Protocols | **High** | Transport: TCP on port 8000, authenticated encryption via Noise, multiplexing via Yamux. Identification: `/kovanica/<network-tag>/1.0.0`. |
| Gossipsub Mesh | **High** | Topics: `kovanica/<network-tag>/{blocks, transactions, sync}`. Message framing capped at `MAX_TRANSMIT_SIZE = 1 MiB` (64 KiB above the `MAX_BLOCK_BYTES = 960 KiB` consensus limit). |
| Edge Validation & Scoring | **Critical** | Pre-gossip edge validation: PoW target check (`validate_pow_target`), transaction signature and fee validation. Peer scoring, topic mesh penalties, and automatic disconnection/banning of misbehaving nodes. |
| Seed Bootstrap | **Medium** | Bootstrap via DNS seeds (`testnet.kovanica.online`, `seed1/2/3.kovanica.online`). Verification that no third-party infrastructure is baked into code. |

---

### 2.6 Cryptography & Keystore — `kovanica-crypto`, `kovanica-keystore` (CRITICAL)

| Primitive | Risk | Description |
|---|---|---|
| Ed25519 Signatures | **Critical** | Uses `ed25519-dalek` v2.1 in strict mode: verification mandates canonical $S$ values and rejects weak/small-order public keys. |
| Address Derivation (PRO-21) | **Critical** | Canonical PRO-21 address encoding: 32-byte Ed25519 public key concatenated with a 29-bit BLAKE3 checksum, encoded in lowercase bech32 base32, prefixed with `kvnc` and suffixed with `dag` (64 characters total). Hex addresses rejected on Mainnet. |
| Keystore v2 | **High** | Argon2id key derivation (64 MiB memory, 3 iterations, 4 parallelism), XChaCha20-Poly1305 AEAD symmetric encryption, atomic file writes with `0600` permissions. Memory zeroization of private keys on drop. |
| Domain Separation | **High** | Explicit domain separation tags for transaction signing, mining pre-images, and cryptographic hashes. |

---

### 2.7 RPC Interface — `kovanica-rpc` (MEDIUM)

| Endpoint Group | Risk | Description |
|---|---|---|
| Mining Interface | **High** | `kovanica_getWork`, `kovanica_submitWork`, `kovanica_getMiningInfo`, `kovanica_getDagTips`. Concurrency control, rate limiting, and input sanitization on submitted work. |
| Standard Client RPC | **Medium** | `kovanica_sendRawTransaction`, `kovanica_getBlock`, `kovanica_getTransactionReceipt`, `kovanica_call` (WASM state simulation). |
| Security Middleware | **Medium** | Per-IP token-bucket rate limiting, Bearer token authentication support, CORS configuration, request body size limits, 10s request execution timeout. |

---

## 3. Out of Scope

- Client applications and frontends (`packages/api-client`, `packages/console`, `packages/signer`, `packages/web`).
- Operational deployment scripts and Docker compose files (`ops/`).
- Local integration test runners and benchmarks.
- Documentation and prose files.

---

## 4. Prioritized Risk Areas for Audit Attention

| Rank | Area | Target Vulnerabilities / Invariants |
|---|---|---|
| **0** | **KovanicaHash PoW Verification** | Memory-hard algorithm soundness, scratchpad initialization collision resistance, deterministic cross-platform execution, ASIC resistance analysis. |
| **1** | **GHOSTDAG K=5 Committer** | DAG ordering determinism, blue-set selection correctness, handling of deep DAG splits, anti-front-running/reorg stability. |
| **2** | **Difficulty Adjustment Algorithm (DAA)** | Stability under wild hash-rate swings, anti-timestamp-manipulation guards, dampening factor correctness for 4 s block time. |
| **3** | **Wasmi WASM Sandbox & Gas Metering** | Fuel exhaustion handling, infinite loop / stack overflow protection, host boundary memory safety, reentrancy guards. |
| **4** | **P2P Ingress Anti-DoS** | PoW target verification before gossip relay (`validate_pow_target`), memory exhaustion protection against forged block graphs. |
| **5** | **Tokenomics & Supply Cap** | Preservation of the 90 200 000 KVNC hard cap, exact 2.5 KVNC block reward, era decay arithmetic, mainnet pre-funding prohibition. |
| **6** | **Cryptographic Signatures & Addresses** | Strict Ed25519 canonical verification, PRO-21 address derivation and checksum validation, private key zeroization in memory. |

---

## 5. Recommended Audit Schedule (4 Weeks)

1. **Week 1: Consensus & PoW**
   - KovanicaHash memory scratchpad and hashing algorithm.
   - GHOSTDAG K=5 ordering, DAG store traversal, and DAA difficulty adjustment.
2. **Week 2: Execution Engine & Wasmi Runtime**
   - WASM fuel metering, opcode gas accounting, 16-page memory boundary.
   - Host function imports, native contracts, and transaction state application.
3. **Week 3: Mempool, P2P & Anti-DoS Ingress**
   - Pre-gossip PoW validation and edge rate limiting.
   - Mempool RBF, nonce management, and P2P gossipsub mesh security.
4. **Week 4: Cryptography, Tokenomics & RPC**
   - Ed25519 strict verification and PRO-21 address checksums.
   - 90.2M KVNC supply cap arithmetic and subsidy decay.
   - RPC mining endpoints and authentication middleware. Final report collation.

---

## 6. Repository & Build Pinning

- **Repository:** `https://github.com/KovanicaDAG/kovanica`
- **Branch:** `main`
- **Build Instructions:**
  ```bash
  cargo build --workspace --release
  cargo test --workspace
  ```
- **Key Pinned Dependencies:**
  - `wasmi = "0.40.0"`
  - `ed25519-dalek = "2.1.0"`
  - `blake3 = "1.5.1"`
  - `libp2p = "0.57.0"`
  - `redb = "2.1.0"`
  - `argon2 = "0.5"`
  - `chacha20poly1305 = "0.10"`

---

*This document defines the formal scope of external security assessment for Kovanica before Mainnet $T_0$.*

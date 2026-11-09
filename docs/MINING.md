# Kovanica Proof-of-Work Mining Manual

This document is the authoritative operator and developer guide for mining on the **Kovanica** Layer-1 blockchain (native ticker **KVNC**). It details the consensus invariants for the Hybrid PoW + GHOSTDAG network (Chain ID 5), the memory-hard **KovanicaHash** algorithm, local benchmarking via `kovanica miner bench`, solo mining operations, mining pool and Stratum protocol integration, JSON-RPC APIs, and VPS sizing guidelines.

---

## Table of Contents

1. [Consensus Overview & Invariants](#1-consensus-overview--invariants)
2. [KovanicaHash PoW Technical Specification](#2-kovanicahash-pow-technical-specification)
3. [Local Hashrate Benchmarking (`kovanica miner bench`)](#3-local-hashrate-benchmarking-kovanica-miner-bench)
4. [Solo Mining Operations](#4-solo-mining-operations)
5. [Stratum Mining & Pool Integration](#5-stratum-mining--pool-integration)
6. [Mining JSON-RPC Interfaces](#6-mining-json-rpc-interfaces)
7. [Hardware Guidelines & Troubleshooting](#7-hardware-guidelines--troubleshooting)
8. [Document References](#8-document-references)

---

## 1. Consensus Overview & Invariants

Kovanica utilizes a **Hybrid PoW + GHOSTDAG** consensus architecture for decentralized block production and transaction ordering.

- **Chain ID:** `5` (`HYBRID_TESTNET` / `hybrid-testnet`).
- **Consensus Model:** Hybrid Proof-of-Work block authoring combined with GHOSTDAG DAG ordering. **There is strictly NO stake required or weighted for consensus** (architectural decision confirmed 2026-11-05).
- **PoW Algorithm:** **KovanicaHash** (BLAKE3 + 2 MiB pseudo-random memory mix scratchpad).
- **Target Block Time:** 4 seconds (`TARGET_BLOCK_MS = 4000`).
- **GHOSTDAG Parameter $K$:** $K = 5$ (`GHOSTDAG_K = 5`).
- **Block Reward:** **2.5 KVNC** per committed block author, credited directly to the miner's payout address.
- **Subsidy Halving / Decay:** Initial subsidy era of 8,100,000 blocks with $\times 3/4$ decay each subsequent era, drawing from an 81,000,000 KVNC mining subsidy budget.
- **Payout Address Format:** Canonical PRO-21 base32 address (`kvnc` + 57 base32 chars + `dag` = 64 characters). Legacy hex addresses (`0x...` or 64 hex characters) are also supported on Chain ID 5 for testnet tooling compatibility.

For further architectural context, see the [Hybrid PoW + GHOSTDAG ADR](adr/0003-hybrid-pow-ghostdag.md), the [Block Header Specification](dev/POW-HEADER-SPEC.md), and the [Tokenomics Specification](TOKENOMICS.md).

---

## 2. KovanicaHash PoW Technical Specification

KovanicaHash is a memory-hard proof-of-work algorithm engineered to resist specialized ASIC/FPGA monopolization by enforcing memory-latency and bandwidth dependency while remaining comfortable on commodity multi-core CPUs and low-cost (2–4 GB RAM) VPS instances.

```
       +------------------------------------+
       |  Canonical Block Header Preimage   |
       |  (132+ bytes, nonce at tail 8 B)   |
       +-----------------+------------------+
                         |
                         v
       +------------------------------------+
       |  1. Seed Phase (BLAKE3 Keyed)      |
       |  Domain: "KVNC-POW-SEED-v1"        |
       +-----------------+------------------+
                         | 32-byte seed
                         v
       +------------------------------------+
       |  2. Memory Mix Phase (2 MiB)       |
       |  - BLAKE3 XOF initializes 65,536   |
       |    chunks of 32 bytes (2,097,152 B)|
       |  - 1,024 Pseudo-Random Rounds      |
       |  - Non-linear state wandering      |
       |  - Word rotations & write-backs    |
       +-----------------+------------------+
                         | mixed state + memory samples
                         v
       +------------------------------------+
       |  3. Finalize Phase (BLAKE3 Keyed)  |
       |  Domain: "KVNC-POW-FINL-v1"        |
       +-----------------+------------------+
                         |
                         v
                Final 32-byte Hash
            (check <= target_bits threshold)
```

### Execution Phases

1. **Seed Phase:**
   The canonical block header preimage (serialized per [PoW Header Spec](dev/POW-HEADER-SPEC.md) with domain tag `KVNC-POW-HDR-v1`) is hashed via BLAKE3 in keyed mode using domain `KVNC-POW-SEED-v1` into a 32-byte seed.
2. **Memory Mix Phase (2 MiB):**
   - A 2 MiB (`2,097,152` bytes) scratchpad allocated once per worker thread context is initialized from the seed using BLAKE3 in extendable output (XOF) mode.
   - The state traverses 1,024 pseudo-random mixing rounds. In each round, state words select chunks from the 2 MiB scratchpad via non-linear indexing, apply bitwise rotations (`rotate_left(19)` and `rotate_left(41)`), compute wrapping additions, and write back mutated states into memory.
3. **Finalize Phase:**
   Keyed BLAKE3 compression under domain `KVNC-POW-FINL-v1` ingests the initial seed, mutated mix state, and scratchpad boundary samples, producing the final 32-byte solution hash.

### Zero-Allocation Hot Path

`KovanicaHasher` allocates its 2 MiB scratchpad on the heap during initialization. Subsequent evaluations reuse the preallocated buffer, incurring **zero heap allocations** in the hashing loop.

---

## 3. Local Hashrate Benchmarking (`kovanica miner bench`)

Before deploying mining rigs or participating in testnet rounds, operators can evaluate local CPU throughput using the offline `kovanica miner bench` command.

The benchmark runs completely **offline and standalone** — no running node or RPC connection is needed.

### CLI Syntax & Flags

```bash
kovanica miner bench [OPTIONS]
```

Or using the alias:

```bash
kovanica mining bench [OPTIONS]
```

| Flag | Short | Default | Description |
|---|---|---|---|
| `--threads <N>` | `-t` | Logical CPU cores | Number of parallel worker threads (1 to 256). |
| `--duration <SECS>` | `-d` | `5` | Benchmark duration in seconds ($\ge 1$). |
| `--json` | | `false` | Output machine-readable JSON instead of tabular summary. |

### Human-Readable Output Example

Running a 5-second benchmark on 4 CPU threads:

```bash
kovanica miner bench --threads 4 --duration 5
```

Output:

```text
============================================================
            KovanicaHash PoW Benchmark Results             
============================================================
 Algorithm:          KovanicaHash (BLAKE3 + 2 MiB mix)
 Worker Threads:     4
 Target Duration:    5 s
 Elapsed Time:       5.002 s
 Total Hashes:       6280
 Total Hashrate:     1.26 kH/s (1255.50 H/s)
 Per-Thread Rate:    313.87 H/s
============================================================
```

### JSON Output Example (`--json`)

When invoked with `--json`, the command emits structured JSON suitable for monitoring scripts and CI pipelines:

```bash
kovanica --json miner bench --threads 2 --duration 3
```

```json
{
  "algorithm": "KovanicaHash (BLAKE3 + 2 MiB mix)",
  "threads": 2,
  "target_duration_secs": 3,
  "elapsed_secs": 3.001,
  "total_hashes": 1892,
  "hashrate_hps": 630.46,
  "hashrate_formatted": "630.46 H/s",
  "per_thread_hps": 315.23,
  "thread_hashes": [
    946,
    946
  ]
}
```

### Hardware Hashrate Reference (KovanicaHash)

| CPU Tier | Physical / Logical Cores | Memory Config | Expected Total Hashrate |
|---|---|---|---|
| VPS Entry (2 vCPU) | 2 vCPU | DDR4 (shared host) | 400 – 650 H/s |
| VPS Mid (4 vCPU) | 4 vCPU | DDR4 (dedicated) | 1.0 – 1.4 kH/s |
| Modern Desktop (AMD Ryzen 7 7800X3D) | 8C / 16T | DDR5-6000 (96 MB L3 Cache) | 4.5 – 6.2 kH/s |
| Server Node (AMD EPYC 7763) | 32 Threads allocated | 8-Channel DDR4-3200 | 12.0 – 16.5 kH/s |

---

## 4. Solo Mining Operations

Solo miners connect directly to a Kovanica node, fetch work templates via JSON-RPC, compute PoW solutions locally, and submit solved blocks to the network.

### Step 1: Generate or Select Payout Address

Mining rewards (2.5 KVNC per block) are paid directly to your account address. Generate a fresh keypair using the CLI:

```bash
kovanica keygen --out ~/.kovanica/miner.json
```

Display your canonical PRO-21 address (`kvnc...dag`):

```bash
kovanica import --keystore ~/.kovanica/miner.json
```

Example payout address:
`kvnca0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5dag`

### Step 2: Launch Mining Worker

#### Mode A: Standalone CLI Solo Miner (`kovanica miner run`)

The standalone CLI solo miner connects to any local or remote Kovanica node running Chain ID 5:

```bash
kovanica miner run \
  --address kvnca0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5dag \
  --threads 4 \
  --poll-interval-ms 1000 \
  --rpc-url http://127.0.0.1:8545
```

Terminal Output:

```text
============================================================
       Kovanica Standalone Solo CPU Miner (Chain 5)         
============================================================
 Chain ID:        5
 RPC Endpoint:    http://127.0.0.1:8545
 Payout Address:  kvnca0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5dag
 Worker Threads:  4
 Poll Interval:   1000 ms
 PoW Hash:        KovanicaHash (BLAKE3 + 2 MiB mix)
 Press Ctrl-C to stop mining gracefully.
============================================================
[INFO] Fetched candidate work: round 42, target_bits 0x207fffff, parents: 1
[INFO] Solved block! round 42, nonce 192840, hash 00000abc...
[INFO] Block submitted successfully! Total accepted: 1
```

Press `Ctrl-C` at any time to shut down the worker gracefully; aggregate hash counts, runtime duration, and acceptance statistics will be summarized upon exit.

#### Mode B: In-Node Built-in Miner

Nodes can author blocks natively by enabling internal mining in their configuration file (`config.chain5.toml`):

```toml
[network]
chain_id = 5

[mining]
enabled = true
author = 0
threads = 4
payout_address = "kvnca0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5a0b1c2d3e4f5dag"
```

Or passing flags to the node binary:

```bash
kovanica-node --config config.chain5.toml --mining.enabled --mining.threads 4
```

---

## 5. Stratum Mining & Pool Integration

Mining pools aggregate hashrate across multiple mining rigs and distribute block rewards proportionally among participants.

### Pool Topology

```
   +--------------------+     +--------------------+
   |   Miner Rig #1     |     |   Miner Rig #2     |
   | (Stratum Client)   |     | (Stratum Client)   |
   +---------+----------+     +---------+----------+
             | Stratum TCP              | Stratum TCP
             +------------+-------------+
                          |
                          v
             +--------------------------+
             |   Stratum Pool Bridge    |
             | (e.g. kovanica-stratum)  |
             +------------+-------------+
                          | JSON-RPC (HTTP)
                          | kovanica_getWork / kovanica_submitWork
                          v
             +--------------------------+
             |   Kovanica Node (RPC)    |
             |   (Chain ID 5 / Testnet) |
             +--------------------------+
```

### Stratum V1 Protocol Mapping

The Stratum pool bridge translates standard Stratum V1 TCP socket communications into Kovanica JSON-RPC calls.

#### 1. Connection & Subscription (`mining.subscribe`)

The miner connects via TCP and requests session parameters:

```json
{"id": 1, "method": "mining.subscribe", "params": ["kovanica-miner/1.0.0"]}
```

Pool response:

```json
{
  "id": 1,
  "result": [
    [["mining.set_difficulty", "session_sub_id"], ["mining.notify", "notify_sub_id"]],
    "01a4b8", 
    4
  ],
  "error": null
}
```
- `01a4b8`: Allocated `extranonce1` (assigned uniquely per client connection).
- `4`: `extranonce2_size` in bytes.

#### 2. Worker Authorization (`mining.authorize`)

```json
{"id": 2, "method": "mining.authorize", "params": ["kvnc1...dag.worker1", "x"]}
```

Pool response:

```json
{"id": 2, "result": true, "error": null}
```

#### 3. Share Difficulty Adjustment (`mining.set_difficulty`)

The pool sends difficulty targets suited for worker hashrates so workers submit frequent proof-of-work shares (e.g., 5–15 shares/minute):

```json
{"id": null, "method": "mining.set_difficulty", "params": [0.05]}
```

#### 4. Job Notification (`mining.notify`)

When new DAG tips arrive at the node (detected via `kovanica_getWork`), the pool broadcasts a new job:

```json
{
  "id": null,
  "method": "mining.notify",
  "params": [
    "job_42a",
    "000000050000002a...",
    "207fffff",
    true
  ]
}
```
- Parameter 1: `job_id`.
- Parameter 2: Serialized block header preimage template prefix.
- Parameter 3: Network compact difficulty bits (`target_bits`).
- Parameter 4: `clean_jobs` (boolean flag instructing workers to immediately abort previous work upon true).

#### 5. Share Submission (`mining.submit`)

When a worker finds a nonce meeting the assigned pool share target, it submits:

```json
{
  "id": 4,
  "method": "mining.submit",
  "params": [
    "kvnc1...dag.worker1",
    "job_42a",
    "00000001",
    "000000000002e412"
  ]
}
```
- Param 3: `extranonce2` (hex).
- Param 4: 64-bit `nonce` (hex).

### Share Validation & Network Block Submission

1. **Pool Share Verification:**
   The pool reconstructs the header preimage using the worker's `extranonce1 + extranonce2 + nonce`, computes `KovanicaHash`, and verifies that the resulting hash satisfies the worker's assigned share difficulty. Valid shares are recorded for miner reward accounting (PPLNS or PPS).
2. **Network Solution Check:**
   If the solution hash additionally satisfies the full network target (`check_pow_hash_meets_target(&hash, target_bits) == true`), the pool immediately packages the header and submits it to the local node via `kovanica_submitWork`.

---

## 6. Mining JSON-RPC Interfaces

The Kovanica node exposes standard JSON-RPC 2.0 endpoints for mining operations:

### `kovanica_getWork`

Fetches candidate work templates for the next DAG block.

- **Parameters:** `[payout_address: string]`
- **Response:**
  ```json
  {
    "chain_id": 5,
    "round": 42,
    "author": 0,
    "parents": ["0x55aa..."],
    "parent_refs": [{"digest": "0x55aa...", "round": 41}],
    "merkle_root": "0x3344...",
    "timestamp": 1700000000000,
    "target_bits": "0x207fffff",
    "target": "0x000007ffff...",
    "preimage": "0x4b564e432d504f572d4844522d7631..."
  }
  ```

### `kovanica_submitWork`

Submits a solved block header containing a valid nonce and PoW solution.

- **Parameters:** `[solved_header: object]`
- **Response:** `true` on successful acceptance and network broadcast; error object if rejected.

### `kovanica_getMiningInfo`

Returns network difficulty, current target bits, estimated network hashrate, and active tips.

- **Parameters:** `[]`
- **Response:**
  ```json
  {
    "chain_id": 5,
    "mining_enabled": true,
    "current_round": 42,
    "target_bits": "0x207fffff",
    "difficulty": 1.0,
    "network_hashrate_estimate_hps": 12500.0,
    "active_tips_count": 3
  }
  ```

### `kovanica_getDagTips`

Inspects the current frontier tips of the GHOSTDAG block DAG.

For detailed RPC schemas and error handling, refer to [RPC Documentation](RPC.md).

---

## 7. Hardware Guidelines & Troubleshooting

### Memory Sizing & VPS Resource Constraints

- **Scratchpad Footprint:** Exactly 2 MiB per worker thread context.
  - 1 thread = 2 MiB RAM.
  - 16 threads = 32 MiB RAM.
  - 64 threads = 128 MiB RAM.
- **Node Memory Footprint:** The Kovanica node and GHOSTDAG state comfortably fit within a **2–4 GB RAM** VPS footprint.
- **Thread Allocation Strategy:** Set worker threads to physical CPU cores or logical cores. Because KovanicaHash repeatedly accesses a 2 MiB memory buffer, CPUs with large L3 caches (e.g. AMD 3D V-Cache with 96 MB L3) can host dozens of threads simultaneously in fast cache without DDR bus saturation.

### Minimizing DAG Orphan & Stale Rates

With a 4-second block time:
1. **Network Latency:** Ensure your mining node maintains high-speed peer connections to testnet seed nodes (see [Testnet Seed Nodes](TESTNET-SEED-NODES.md)).
2. **Polling Frequency:** When running the standalone miner, configure `--poll-interval-ms` between `500` and `1000` ms to ensure work templates switch promptly when peers produce blocks.
3. **Time Synchronization:** Run `systemd-timesyncd` or `chrony` on your VPS to ensure timestamps remain within the allowable DAG tolerance.

### Common Error Codes & Diagnostics

| Symptom / Error | Root Cause | Remediation |
|---|---|---|
| `invalid payout address for chain 5` | Payout address is corrupted or invalid checksum. | Verify address was generated via `kovanica keygen` and starts with `kvnc` and ends with `dag`. |
| `stale block header / parent not in DAG` | Work template was solved after a new round committed. | Lower `--poll-interval-ms` (e.g. 500 ms) and optimize node P2P connectivity. |
| `pow hash does not meet target` | Nonce or preimage mismatch between miner and node. | Ensure miner uses `KovanicaHasher` and does not mutate preimage bytes other than the tail 8-byte nonce. |
| `requested thread count exceeds maximum` | `--threads` exceeds 256. | Reduce thread allocation below 256. |

---

## 8. Document References

- [Architecture & System Overview](ARCHITECTURE.md)
- [Hybrid PoW + GHOSTDAG Architectural Decision Record (ADR 0003)](adr/0003-hybrid-pow-ghostdag.md)
- [PoW Block Header & Mining Specification](dev/POW-HEADER-SPEC.md)
- [Hybrid Testnet M8 Milestone Spec](dev/HYBRID-TESTNET-M8.md)
- [JSON-RPC API Documentation](RPC.md)
- [Tokenomics Specification](TOKENOMICS.md)
- [Testnet Seed Nodes Runbook](TESTNET-SEED-NODES.md)

---
title: "How to Mine KVNC on Kovanica Testnet — SUPERSEDED"
category: 30-Operations
source: protocol/HOWTO_MINE.md
synced: 2026-09-26
---
# How to Mine KVNC on Kovanica Testnet — SUPERSEDED

> **Status:** SUPERSEDED (marked 2026-09-25). **Consensus impact:** the
> PoA-only transition it describes is **consensus-breaking** (a hard fork — see
> RFC-POA-Migration §0.6); this tombstone itself changes no protocol rule.
> This document is a **tombstone**. Mining no longer exists as a concept in the
> Kovanica protocol; the instructions below are kept verbatim as a historical
> record of the pre-PoA testnet only. **Do not follow them.**
>
> > **Consensus decision (ratified 2026-09-25): Kovanica is PoA-only.**
> > Proof-of-Work is being removed from the protocol. See
> > `protocol/docs/RFC-POA-Migration.md` §0 (canonical). Items marked `[TARGET]`
> > are ratified but not yet implemented; `[CURRENT]` items describe shipped code.

---

## Why this document is superseded

**Kovanica is PoA-only. There is nothing left to mine.**

- `[TARGET]` Mining ceases to exist entirely. The `pow` module
  (`mine`/`mine_from`/`meets_target`), the `difficulty` module
  (`Retarget`/`TimedWork`/`next_work`), `Dag::set_proof_of_work`,
  `Dag::set_difficulty`, `Dag::proof_of_work_enabled`,
  `DagError::InsufficientProofOfWork`, `DagError::DifficultyMismatch`, the node
  mining loop, `KOVANICA_POW` / `KOVANICA_MINE` / `KOVANICA_MINE_SECS` /
  `KOVANICA_MINER_ADDRESS`, the RPC/explorer `kind: "pow"`, FFI `BlockKind::Pow`,
  the `pow`/`pow-vrf` cargo features and `protocol/mine-kvnc.sh` are all
  **removed**. Full removal table with line references: RFC-POA-Migration §0.1.
- `[CURRENT]` This is **not yet done in code.** PoW, difficulty and the mining
  loop are all still in the tree and still reachable, and `KOVANICA_POW` is
  still read (`explorer.rs` ~874, ~981). The rest of this page describes that
  legacy, pre-reset state.
- The transition is **consensus-breaking and requires a genesis reset.** A
  legacy PoW chain and a PoA chain cannot be reconciled (§0.6): under PoA every
  block must carry `work == POA_NOMINAL_WORK = 1`, and a real PoW block's work
  is orders of magnitude higher, so it would out-compete every PoA block in the
  GHOSTDAG blue-work fold. PoA genesis also commits the authority set as a
  `KVA1` coinbase output, so the genesis block id differs.

## What replaces mining: running an authority

There is no drop-in replacement, and there should not be: an authority is a
**permissioned** role, not a purchasable one. PoW is what made admission
permissionless — anyone could spend CPU to enter the DAG. PoA removes that
property deliberately (RFC-POA-Migration §0.5).

| | PoW miner (obsolete) | PoA authority operator (`[TARGET]`) |
|---|---|---|
| Admission | Permissionless — buy hash power | **Permissioned** — must be in the on-chain authority set |
| Key | any key; reward address chosen by you | an authority **Ed25519 signing key**, issued at the key ceremony |
| Schedule | whenever you solve a target | one block per slot, only when **scheduled**: `authorities[slot % n]` over the canonical (sorted) key order |
| Config | `KOVANICA_MINE=1`, `KOVANICA_POW=1` | `KOVANICA_CONSENSUS=poa`, `KOVANICA_AUTHORITIES`, `KOVANICA_AUTHORITY_THRESHOLD`, `KOVANICA_SLOT_DURATION` |
| Key delivery | n/a | `Node::set_authority_signing_key` — **never** a seed you pick yourself |

An authority's signing key is **not** a PoW key and is **not** derived from a
seed you choose. Under the current testnet fallback the set is the
deterministic, **publicly derivable** placeholder derived from
`AUTHORITY_PLACEHOLDER_BASE = 9001` (`explorer.rs` ~196) — mirroring the
placeholder treasury keys, and testnet-only. `kovanica-mainnet` **refuses to
boot** without an explicit `KOVANICA_AUTHORITIES`.

`[OPEN]` **How anyone becomes an authority is not settled.** The on-chain
mechanism exists (`AuthorityUpdateTx`, RFC-POA §3) but the selection, rotation
and governance policy, and the key ceremony, do not. Do not publish a
"become an authority" guide until that is decided — see RFC-POA-Migration §0.7.2.

## What replaces "mining rewards"

Under PoW, whoever solved a target collected the coinbase. Under PoA, **the
block subsidy accrues to the scheduled authority for that slot.** If nobody is
scheduled and online, the slot is simply **empty** — there is no gap-fill and no
retarget; the next scheduled authority continues on the 3000 ms clock
(`SLOT_DURATION_MS`). So the "how do I get KVNC" question changes shape
fundamentally. See `protocol/HOWTO_GET_KVNC.md` for the current, honest
mechanisms and the `[TARGET]`/`[OPEN]` split.

**Tokenomics did not change.** Removing PoW does not change supply math: the
emission curve is height-indexed and `cumulative_minted` is hard-capped at
`MAX_SUPPLY` in `apply_block`. MAX_SUPPLY **90.2M KVNC**, s₀ **10 KVNC/block**,
era **2,000,000 blocks**, α **3/4**, maturity **100 blocks**, fee split
**75% burned / 25% producer**, **1 KVNC = 100_000_000 atoms** — all unchanged.
What changes is the *pace* of emission in wall-clock terms, not the cap.

---

## Historical record: the pre-PoA mining guide

> Everything below this line described the **pre-reset PoW testnet** and is kept
> for provenance. Several parts were already inaccurate when written — see the
> "Known inaccuracies" list at the very bottom. **Do not use.**

### Overview

Kovanica uses a hybrid Proof-of-Work (PoW) + VRF-staked block production system. Mining is opt-in and can be done via CPU mining on the testnet.

### Prerequisites

- Linux or macOS (Windows via WSL)
- Rust toolchain installed (via rustup)
- Git
- At least 2GB RAM recommended

### Step 1: Clone the Repository

```bash
git clone https://github.com/KovanicaDAG/kovanica-protocol.git
cd kovanica-protocol
```

### Step 2: Build the Node

```bash
# Build in release mode for better mining performance
cargo build --release --bin kovanica-node
```

The binary will be available at `target/release/kovanica-node`.

### Step 3: Configure Environment Variables

Create a `.env` file or set environment variables:

```bash
# Enable mining (set to 1 to mine, 0 to run as light node)
export KOVANICA_MINE=1

# Optional: Set mining interval in seconds (default: 120s)
export KOVANICA_MINE_SECS=60

# Optional: Disable hybrid mode for pure PoW (default: enabled)
# export KOVANICA_POW=1

# Optional: Set custom data directory
# export KOVANICA_DATA=/path/to/your/data

# Optional: Set miner address (if empty, uses ephemeral key)
# export KOVANICA_MINER_ADDRESS=your_address_here
```

### Step 4: Run the Node

```bash
# Run with environment variables
KOVANICA_MINE=1 KOVANICA_MINE_SECS=60 ./target/release/kovanica-node
```

Or if you set them in your shell:

```bash
export KOVANICA_MINE=1
export KOVANICA_MINE_SECS=60
./target/release/kovanica-node
```

### Step 5: Verify Mining is Working

Check the logs for lines like:
```
[INFO] kovanica_node: Produced block <hash> (height: <height>, txs: <count>)
[INFO] kovanica_node: Mined nonce <nonce> for work target <target>
```

You can also monitor your hashrate via:
```bash
# In another terminal, assuming default data directory
watch -n 1 "grep -c 'Produced block' ~/.kovanica/logs/info.log"
```

### CPU Mining Recommendations

- For meaningful testnet participation: Use at least 2 CPU cores
- The testnet difficulty is low, so even modest hardware can mine blocks
- To mine continuously, keep the node running
- Mining rewards go to the address specified by `KOVANICA_MINER_ADDRESS` or an ephemeral key

### Stopping Mining

To run as a light node (no mining):
```bash
export KOVANICA_MINE=0
./target/release/kovanica-node
```

### Monitoring Your Rewards

Check your balance via:
1. The wallet UI: https://wallet.kovanica.online
2. Or via the node's RPC API:
```bash
curl -s http://127.0.0.1:8080/api/address/<your_address>/balance
```

### Troubleshooting

#### "No peers" messages
- This is normal on first startup - peer discovery takes time
- Ensure ports 30303-30304 are open for TCP/UDP
- Check your firewall settings

#### Mining not producing blocks
- Verify `KOVANICA_MINE=1` is set
- Check that `target/release/kovanica-node` is actually running
- Look for error messages in the logs
- Ensure you're synced with the network (check tip height vs explorer)

#### High CPU usage
- This is expected when mining
- To reduce CPU usage, increase `KOVANICA_MINE_SECS` or set `KOVANICA_MINE=0`

### Testnet Faucet

If you need test KVNC for transactions while waiting to mine:
- Visit: https://faucet.testnet.kovanica.online
- Or use the API: `POST /api/faucet?to=<address>&amount=<atoms>`

### Mainnet Considerations

Mainnet mining will require:
- Updated software release
- Different network configuration
- Potentially higher difficulty requiring specialized hardware
- Refer to announcements on https://kovanica.online for mainnet launch details

---

## Known inaccuracies in the historical text

Found while marking this document superseded. Left in place for provenance, but
they were wrong before the PoA-only decision too:

1. **Ports 30303-30304 are not Kovanica's.** P2P is plaintext **TCP 9000 only**
   (libp2p was removed). No UDP. `NETWORK.md` is canonical.
2. **`KOVANICA_MINER_ADDRESS` is not read by any Rust code.** It is read only by
   the shell script `protocol/mine-kvnc.sh` (also `[TARGET]`-for-removal). The
   node itself has no such env var.
3. **`KOVANICA_POW=1` is not "pure PoW".** `KOVANICA_HYBRID` is a separate flag
   (default `0`); with it on, the PoW path is only one of two admission paths.
4. **"Ports 30303-30304 open for TCP/UDP"** — there is no UDP transport.
5. **Monorepo paths changed.** `cargo build --release --bin kovanica-node` from
   the protocol workspace still works, but the deployable node binary is built
   from `node/` (a thin `kovanica-node-bin` wrapper over the protocol crates via
   path deps); `protocol/crates/` is the single source of truth for the five
   crates.
6. **The mainnet "higher difficulty requiring specialized hardware" bullet is
   now meaningless** — there is no difficulty and no specialized hardware under
   PoA.

---

*Originally last updated September 2026. Marked SUPERSEDED 2026-09-25 by the
PoA-only consensus decision; see `protocol/docs/RFC-POA-Migration.md` §0.*

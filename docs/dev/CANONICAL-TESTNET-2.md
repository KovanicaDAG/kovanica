# Canonical Testnet 2 — Hybrid PoW + GHOSTDAG

- **Status:** Implemented (shipped; this is the reference for the canonical public testnet)
- **Chain ID:** `2` (`TESTNET`)
- **Related:** [ADR 0003: Hybrid PoW + GHOSTDAG Consensus](../adr/0003-hybrid-pow-ghostdag.md), `docs/MINING.md`, `docs/TOKENOMICS.md`

---

## Executive Summary

Chain ID `2` is the canonical public testnet and runs the same consensus design as
Mainnet: **Hybrid PoW + GHOSTDAG** with **no stake in the consensus path**. Block
production is KovanicaHash mining; GHOSTDAG orders the block DAG with $k = 5$ and a
4-second block target (`TARGET_BLOCK_MS = 4000`). Emission follows
**`NoStake82M`**, and addresses are **strictly canonical `kvnc…dag`**.

The isolated hybrid test network that was previously registered as Chain ID `5` is
**retired**: `HYBRID_TESTNET` is now only an alias for Chain `2`
(`CHAIN_ID_HYBRID_TESTNET = TESTNET = 2`), and Chain `4` (`TESTNET_V2`) no longer
exists. The registered chain set is `1` (mainnet), `2` (testnet), `3` (devnet) and
`1337` (local).

Chain 2 is a **fresh genesis deployment**. No pre-hybrid Chain 2 data directory and
no retired Chain 5 data directory can be reused or relabeled — see
[§5](#5-genesis--storage-binding).

---

## 1. Invariants

| Property | Value | Source of truth |
|---|---|---|
| Chain ID | `2` (`TESTNET`) | `crates/kovanica-types/src/signing.rs` |
| Consensus | Hybrid PoW + GHOSTDAG, no stake for consensus | `chain_id::is_hybrid(2) == true` |
| Block target | 4 000 ms | `kovanica_params::TARGET_BLOCK_MS` |
| GHOSTDAG parameter | $k = 5$ | `kovanica_params::GHOSTDAG_K` |
| PoW function | KovanicaHash (BLAKE3 + 2 MiB memory mix) | ADR 0003 |
| Emission policy | `NoStake82M` (82 000 000 KVNC mining budget) | `ensure_emission_policy_matches_chain` |
| Initial block reward | 2.5 KVNC, minted to the committed block author | `docs/TOKENOMICS.md` |
| Address format | Canonical `kvnc…dag` only | `legacy_address_allowed(2) == false` |
| Legacy Chain 5 | Retired; alias of Chain 2 | `chain_id::HYBRID_TESTNET` |
| Legacy Chain 4 | Removed | — |

## 2. Consensus Dispatch

`kovanica_types::signing::chain_id::is_hybrid(id)` is the single dispatch predicate
for hybrid behaviour and returns true for exactly `MAINNET` (1) and `TESTNET` (2):

```rust
pub fn is_hybrid(id: u64) -> bool {
    matches!(id, MAINNET | TESTNET)
}
```

Every chain-id-dependent branch that used to compare against a hardcoded hybrid
chain ID now asks `is_hybrid` (or the equivalent params constant), so Chain 2 and
Mainnet cannot drift apart:

- **Node wiring** — the background PoW mining loop runs on hybrid chains, and the
  Mysticeti proposal/vote loops do not.
- **Block ingestion** — hybrid admission extracts the PoW header, validates parent
  structure (≤ $k$ parents, no duplicates, no self-reference), enforces round
  constraints and drift checks, verifies the memory-hard PoW, requires parents in
  the DAG store, and derives the DAA target.
- **Gossip** — blocks are PoW-verified on hybrid chains instead of batch-verified
  with Ed25519 committee signatures, and block gossip is not dropped for want of
  committee keys.
- **GHOSTDAG committer** — work weight is derived from the PoW header target.
- **Execution** — no epoch committee resolution, no staking commission split, no
  light-escrow payout; the block subsidy is minted to eligible block authors.

Consensus and wire formats bind `chain_id`, so a PoW solution, block, vote or
transaction produced for any other chain ID is invalid here:

- PoW header preimage: `POW_HEADER_DOMAIN_TAG ‖ chain_id ‖ round ‖ author ‖
  parents_hash ‖ merkle_root ‖ timestamp ‖ target_bits ‖ nonce`.
- Transaction preimages carry `chain_id` as 8-byte little-endian.
- `BLOCK_DOMAIN_TAG`, `COMMIT_DOMAIN_TAG` and `VOTE_DOMAIN_TAG` all bind `chain_id`.

## 3. Emission: `NoStake82M` Only

Chain 2 is a no-stake hybrid chain, so `NoStake82M` is the only permitted emission
policy. This is enforced **fail-closed** at startup, before genesis state is created
or opened (`crates/kovanica-node/src/main.rs`):

```rust
if is_hybrid(chain_id) && policy != EmissionPolicy::NoStake82M {
    bail!("chain {chain_id} is a hybrid PoW + GHOSTDAG chain and requires the \
           no-stake-82m emission policy, not {}; ...");
}
```

A node configured with `Legacy81M` (or any other policy) on Chain 2 refuses to
start. `ops/docker/docker-compose.chain2.yml` sets
`KOVANICA_EMISSION_POLICY: "no-stake-82m"` for every service.

Emission math is unchanged from the locked tokenomics: 90 200 000 KVNC total,
8 200 000 rewarded blocks per era, initial reward 2.5 KVNC, × 3/4 decay per era,
8 000 000 KVNC linear treasury over 8 years, 200 000 KVNC founder premine and zero
genesis validator allocation. See `docs/TOKENOMICS.md`.

## 4. Address Policy

Chain 2 mirrors Mainnet address hygiene: `legacy_address_allowed(2) == false`, so
every address that reaches the node as **user, RPC or network input** must be the
canonical 64-character PRO-21 base32 form `kvnc…dag` (32-byte key ‖ first 29 bits of
`blake3(key)`, bech32 alphabet, lowercase). Legacy `kvnc<hex>dag`, bare 64-hex and
`0x`-prefixed hex are rejected.

Consequences:

- Genesis files (`funded_accounts.toml`, `genesis_validators.toml`,
  `validators.json`, treasury and founder addresses) must contain canonical
  addresses; a legacy hex entry makes `parse_address_hex` fail and aborts startup.
- Mining payout addresses (`kovanica_submitWork`, CLI miner) must be canonical.
- Faucet and wallet flows must submit canonical addresses.

Legacy hex is still accepted on devnet (`3`) and local (`1337`) only, where
`legacy_address_allowed` returns true.

### 4.1 Stored keystore labels are not user input

Keystore and wallet files written by older builds may store an address label in a
legacy hex form. Those labels are read with a chain-independent parser that
deliberately does **not** take a chain ID:

```rust
// crates/kovanica-types/src/address.rs
Address::parse_stored_label(&label)
```

It accepts the canonical form plus the legacy hex forms (same length and checksum
validation as the chain-aware parser) and exists **only** for reading labels that are
already on disk. It must never be used for user input, RPC input or network input —
those go through `Address::parse_for_chain(value, chain_id)`, whose behaviour is
unchanged on every chain. Using it does not bind the key to any chain: a keystore is
portable and `kovanica-keystore` passes no chain ID.

## 5. Genesis & Storage Binding

An existing data directory cannot be relabeled onto Chain 2. `genesis_config_digest`
is a keyed BLAKE3 hash over:

1. the emission policy (`P` + `no-stake-82m` when set),
2. the canonical round-0 genesis block digest,
3. `chain_id` as 8-byte little-endian,
4. the treasury address (parsed for the configured chain),
5. parsed entries of `validators.json`, `founder_premine.hex`,
   `funded_accounts.toml` and `genesis_validators.toml`.

Because `chain_id` is in the digest, a directory created under a different chain ID
has a different digest. `verify_persisted_genesis_digest` (in `state.redb`) and
`verify_consensus_genesis_marker` (in `consensus.redb`) both refuse to start, and the
`KOVANICA_ALLOW_GENESIS_MISMATCH` override is rejected on non-dev chains
(`is_dev(2) == false`).

A retired hybrid data directory is also unusable for a second reason: blocks from a
different chain ID fail the header's chain binding, and pre-hybrid blocks carry no
`Statement::PowHeader` at all, so the DAA ancestry crawl and GHOSTDAG work weight
cannot be computed.

Deployment rule: give Chain 2 a **fresh, empty data directory** and a byte-identical
genesis bundle on every node. There is no carry-over legacy seed set or data
directory for Chain 2; the supported cluster is
`ops/docker/docker-compose.chain2.yml` (node1/node2/node3, observer, faucet, miner,
Prometheus and Grafana) plus `ops/docker/test-chain2-cluster.sh`.

## 6. Retired Networks

- **Chain 5** (`HYBRID_TESTNET`) — retired. The constant remains only as an alias for
  Chain 2 so that older configs and fixtures keep resolving; no node, genesis file or
  test suite targets Chain 5 as a distinct network.
- **Chain 4** (`TESTNET_V2`) — removed from the registry. Marketing and launch
  material for it is archived under `docs/archive/`.

Retired networks must not be reintroduced by copying their genesis files, keys or
data directories; a new network needs a new registered chain ID and a new genesis
bundle.

## 7. Operator Surface

- **Config:** `chain_id = 2` in `config.chain2.toml`
  (`ops/config.chain2.example.toml`); environment overrides use `KOVANICA_CHAIN_ID=2`
  and `KOVANICA_EMISSION_POLICY=no-stake-82m`.
- **Mining / DAG RPC:** `kovanica_getWork`, `kovanica_submitWork`,
  `kovanica_getMiningInfo`, `kovanica_getDagTips` are gated on `is_hybrid(chain_id)`
  and therefore available on Chain 2.
- **CLI:** `kovanica mining status`, `kovanica dag tips`, and the standalone solo
  miner validate the payout address with `Address::parse_for_chain(.., chain_id)`,
  i.e. canonical-only on Chain 2.
- **Network identity:** every protocol stream, identify string and gossipsub topic is
  tagged `<chain_id>-<first 16 hex chars of genesis_config_digest>`, so a Chain 2
  peer discards traffic from any other chain or genesis.

## 8. Where To Look

| Concern | Location |
|---|---|
| Chain registry, `is_hybrid`, `is_dev` | `crates/kovanica-types/src/signing.rs` |
| Consensus constants (4 s, $k=5$, DAA) | `crates/kovanica-params/src/lib.rs` |
| Emission policy and subsidy math | `crates/kovanica-types/src/emission.rs`, `docs/TOKENOMICS.md` |
| Address parsing (`parse_for_chain`, `parse_stored_label`) | `crates/kovanica-types/src/address.rs` |
| Genesis digest and startup guards | `crates/kovanica-node/src/main.rs` |
| Cluster deployment | `ops/docker/docker-compose.chain2.yml` |
| Cluster test | `crates/kovanica-node/tests/hybrid_node_cluster_integration.rs` (`hybrid_pow_testnet_cluster_e2e`) |

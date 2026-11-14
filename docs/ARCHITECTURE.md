# Kovanica Architecture Overview

## High-level flow

```
Network (P2P)
    │
    ▼
Mempool ──────────────────────────────────┐
    │                                     │
    ▼                                     │
DAG Block Producer (PoW miners)           │
    │                                     │
    ▼                                     │
Consensus (GHOSTDAG K=5 over PoW blocks)  │
    │  produces CommittedSubDag           │
    ▼                                     │
Execution Layer ◄─────────────────────────┘
    │  - apply native txs
    │  - execute WASM contracts (Wasmi)
    │  - pay block reward to the block author
    │  - advance treasury vesting
    ▼
State (accounts, staking, contracts)
    │
    ▼
Storage (redb / pruning / snapshots)
```

## Reward path (detail)

> Legacy Mysticeti reward path. Under Hybrid PoW + GHOSTDAG (chains 1 and 2)
> rewards are minted per committed block to the block author; there is no
> leader payout and no validator committee. See [`TOKENOMICS.md`](TOKENOMICS.md) §2.

1. `UniversalCommitter` decides a leader → `Linearizer` produces `CommittedSubDag`
2. Node calls `ExecutionContext::execute_committed_subdag(&subdag)`
3. Inside execution:
   - transactions are applied
   - `staking.on_leader_committed(subdag.leader_author)` is called
   - reward amount = `block_reward(committed_leader_height)`
   - balance of the leader's payout address is credited
   - treasury vesting is advanced

## Key crates

| Crate            | Responsibility                                      |
|------------------|-----------------------------------------------------|
| kovanica-types       | Core data structures                                |
| kovanica-crypto      | Signatures, hashing                                 |
| kovanica-dag         | Causal DAG store                                    |
| kovanica-consensus   | Commit rules, linearizer                            |
| kovanica-staking     | Emission, treasury (validator set is legacy / non-consensus) |
| kovanica-runtime     | Wasmi WASM engine                                   |
| kovanica-execution   | State transition + reward distribution              |
| kovanica-node        | Binary that wires everything together               |

## Resource targets

- Full node comfortable on **2–4 GB RAM** VPS
- Aggressive pruning of committed DAG history
- Wasmi chosen for deterministic, low-footprint contract execution

---

## Deployment & Operations

### Docker Compose (Current Milestone)
- `docker-compose.yml` defines `kovanica-quorum-4` — 4-node in-process cluster for integration tests.
- Single-node validation via `ops/docker/` scripts.
- Focus remains on P2P multi-node soak before K8s orchestration.

### K8s / Helm
**Status**: Implemented (`deploy/k8s/` and `deploy/helm/kovanica/` Kubernetes production manifests & Helm chart per PR #204).
- Production Helm chart with `StatefulSet` + `ConfigMap` for `kovanica-node`
- Persistent volume claim mapped to `KOVANICA_DATA`
- Liveness/readiness probes against `/health`

### Soak Testing
**Automated Multi-Node Soak Test Harness** (`ops/soak/run-soak.sh`):
- Script: `ops/soak/run-soak.sh` with automated load generator (`ops/soak/load_generator.py`)
- Resource limits: strict cgroups `cpus: 1.5`, `mem_limit: 2048m` (2 GB VPS limit)
- Verification: Monotonic block height advances, 0 container OOM kills (`oom_killed == false`), and Prometheus alert compliance.

---

## Development Platform (Phase 22)

| Component | Status | Location |
|-----------|--------|----------|
| TypeScript API client | Implemented | `packages/api-client/src/index.ts` |
| OpenAPI / JSON schema generator | Implemented | `crates/kovanica-rpc/src/openapi.rs` |
| Contract SDK (Rust + TS) | In progress | `kovanica-htlc`, `kovanica-vault`, `kovanica-multisig`, `kovanica-token`, `kovanica-sdk`, `packages/api-client/src/contracts.ts` |
| Dev faucet service | Implemented | `crates/kovanica-faucet` |
| Indexer (v0.1) | Implemented (skeleton) | crates/kovanica-indexer: REST /logs, /events, /status + WS /ws |

---

## Execution / Gas (Phase C — Partial)

### Current State
- Wasmi runtime: `kovanica-runtime` crate present
- `gas_limit` / `gas_used` fields present in `kovanica-execution/src/lib.rs` (lines 107, 387)

### Missing (Before Mainnet)
- ABI spec for contract calls
- Gas pricing per opcode/byte
- Deterministic metering audit trail
- Wasmi config must set `fuel` consistently; verify no `random`/`time` sources

---

## Observability (Phase 21)

### Done
- `kovanica-consensus/src/metrics.rs` — registry and recorders: `block_height`
  (committed leader height), `current_round`, `peer_count`, `mempool_size`,
  `commit_latency` (wall-clock commit work), `rss_proxy`, `mergeset_size`,
  `mysticghost_*` (mergeset size, colouring duration, pruned blocks/waves),
  `rpc_*` (per-method requests, errors, latency)
- `kovanica_*` operational set (PRO-10, additive to the names above):
  `kovanica_committed_leaders_total`, `kovanica_skipped_leaders_total{reason}`
  (`timeout` / `indirect` / `direct_blame`), `kovanica_finality_latency_seconds`
  (node-local: first sighting of the leader block → durable commit),
  `kovanica_last_committed_round`, `kovanica_connected_peers`,
  `kovanica_active_validators`, `kovanica_sync_errors_total{kind}`,
  `kovanica_last_committed_digest_prefix`, `kovanica_rss_bytes`
- `alerts_reference_registered_metrics` is a test, not a convention: it parses
  every `expr:` in `ops/prometheus/alerts.yml` and fails if a rule alerts on a
  series the registry never exports. Five series were dead before this test
  existed.
- `GET /metrics` on the RPC listener, Prometheus text exposition
- `ops/prometheus/alerts.yml` (alert rules) and
  `ops/grafana/kovanica-overview.json` (dashboard)
- `prometheus-client = 0.25` in workspace Cargo.toml

### Notes
- Metric series are process-global; `block_height` must never be fed the local
  round (`current_round` exists for that) — the soak runner treats a
  non-increasing `block_height` as a stall.
- Alert rules YAML: `peer_drop`, `sync_stall`, `commit_lag`, `high_memory`
- `kovanica_last_committed_digest_prefix` is a gauge, not an info metric: a
  `digest` label would create one series per commit (~7 200/day).
- `kovanica_committee_peers_connected`: optional gauge via `kovanica-p2p-identity` PeerId↔authority binding. Validators may advertise a signed `PeerIdentityClaim`; claimless peers work. Counts connected peers with verified authority in active committee. Inert/absent until claims advertised.
- Structured logging (`tracing`) + optional `trace_id` per round

### Exit Criteria
Scrape `block_height`, `peer_count`, `commit_latency`, `mergeset_size` and
`kovanica_committed_leaders_total`; the dashboard shows a live commit rate and
`kovanica_finality_latency_seconds` has samples.

---

## Security — Threat Model (Phase 20)

> Deterministic, auditable, no `kovanica-protocol` mix.

### Scope
- Consensus: Hybrid PoW + GHOSTDAG (K=5); legacy Mysticeti DAG k=3, wave=3 removed from the chains 1/2 path
- Execution: Wasmi, deterministic WASM
- Staking / validator set (legacy, non-consensus): 15–21, min 50k KVNC
- P2P: libp2p TCP + Noise + Yamux on :8000, DNS seeds only
- Keystore: client-side only — node never receives seed

### Hard Rules (Layer Classification)
- **Client-only**: keys stay in CLI / wallet; node never sees private seed
- **No raw hex in docs**: keystore paths documented, never literal hex
- **Encrypted keystore**: Argon2id + XChaCha20-Poly1305 (`kovanica-cli`)
- **Rate limit**: RPC write methods (`sendRawTransaction`) require bearer token (testnet open-read)
- **Memory budget**: hard cap `MemoryMax=3G` for 6h soak; RSS monitored

### Threats (Priority-Ranked)

| # | Threat | Mitigation |
|---|--------|------------|
| 1 | Key exposure (client-side leak) | Encrypted keystore + client-only path |
| 2 | Replay / nonce reuse | Strict sequential nonce in mempool + domain-separated hashes (`DOMAIN_TX`, `DOMAIN_BLOCK`) |
| 3 | Consensus fork / equivocation | Default: first-valid digest wins, a second digest is rejected. `use_min_digest_fork` (default off) may swap via `DagStore::replace_block` only while the round is undecided and the old digest has no children. Timeout skip on missing leader. |
| 4 | Double-sign / slash | `StakingState::slash` (fixed % evidence tx; legacy path — slashing is not active under hybrid) |
| 5 | P2P eclipse / Sybil | DNS-only kovanica-native seeds (TBD; no third-party hosts); no orange-cloud peers |

### Fuzz Targets (Future)
- `StatementBlock` deserialize (`kovanica-dag`)
- `process_block` ingest (`kovanica-consensus`)
- Mempool admission (`kovanica-mempool`)

### Keystore Verification Checklist
- [ ] No literal hex in `README.md`, `AGENTS.md`, docs
- [ ] `kovanica-cli` uses `keystore` subcommand (encrypted)
- [ ] Node env never contains `KOVANICA_SEED` or `PRIVATE_KEY`

### Tokenomics (Locked — Never Edited Without Approval)
- Ticker: KVNC (9 decimals, `ONE_KVNC = 10^9`, legacy alias `ONE_KUNA`)
- Total: 90.2M KVNC (`90_200_000 * 10^9` atoms)
- Founder premine: 200k KVNC; Treasury: 8M KVNC (linear 8yr)
- Mining subsidy budget: 82M KVNC (no genesis validator allocation on chains 1 and 2)
- Block reward: 2.5 KVNC / 8.2M rewarded blocks; decay ×3/4; minted to committed block author
- Consensus: Hybrid PoW + GHOSTDAG (NO stake in consensus; target block 4 s, GHOSTDAG K=5; PoW: KovanicaHash)

---

## OpenCode Agent Setup (Internal)

The `kovanica-opencode-setup/` package (now removed) provided:
- Specialized `kovanica` agent with strict isolation from `kovanica-protocol`
- `kovanica-plan` agent for planning-only workflows
- Permissions: `kovanica` = edit+bash allow, `external_directory: deny`; `kovanica-plan` = edit deny, bash ask
- Workflow: Ti → Grok (architect) → OpenCode (implement) → Grok (review)

Current agent config lives in root `AGENTS.md`, `.opencode/agents/kovanica.md`, `opencode.json`.

---

## Current Edit Notes (2026-10-13)

- `use_mysticghost` default = **true** in both `NodeConfig` and `ConsensusConfig` (an older note here said false; the code has defaulted to true for a while). `MysticGhostConfig::default()` is the separate colourer knob and does default `enabled: false`, which is what the committer overrides when it builds its own config. When enabled, `UniversalCommitter` applies scoped GHOSTDAG colouring. Since 2026-10-22 the `KOVANICA_MYSTICGHOST` override is only accepted on devnet/local chains.
- `init_genesis` extended with `founder_premine.hex` + `validators.json` support (Phase 14/16)
- Storage / staking compile fixes applied; `TASKLIST.md` updated
- Tokenomics locked (RFC-006 / `docs/TOKENOMICS.md`)
- Repo cleanup: removed 17 obsolete/duplicate files and 4 private keys
- All builds pass: `cargo check`, `clippy`, 394 unit tests
## Latest code snapshot

- `main`: `98d88cc` (tui + console merged; local edits committed)
- Workspace crates: 47 (`kovanica-*`)
- TS packages (5): `packages/api-client`, `packages/comms`, `packages/console`, `packages/signer`, `packages/web`
- Default P2P port: 8000; RPC: 8545; max peers: 50
- Workspace tests: 503+ passing (partial run stopped during sustained-liveness)
- Quarantined temp dirs cleaned; no `bot/*` remote branches remain

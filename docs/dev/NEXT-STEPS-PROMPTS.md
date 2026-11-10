# Next Implementation Prompts (Post-M8)

This document contains formatted, ready-to-paste prompts for the next phases of development following the completion of **Milestone M8** (Hybrid PoW Testnet Chain ID 2 Activation).

Choose any prompt below and paste it directly into an OpenCode / Antigravity pair-programming session.

---

## Prompt 1: Standalone Solo Miner CLI Tooling (`kovanica miner run`)

Use this prompt to build a standalone, multithreaded CPU mining tool directly in `kovanica-cli` that utilizes the `kovanica_getWork` and `kovanica_submitWork` RPC methods.

```markdown
GOAL: Standalone CPU Solo Miner CLI Command for Chain 2 (Hybrid PoW Testnet)

Base tip: main
Branch: feat/hybrid-solo-miner-cli
Open ONE PR into main. Do NOT force-push. Do NOT merge. Do NOT rebase over tip. Wait for Gate / Toni.

Context & Invariants (Toni 2026-11-05, locked per ADR 0003):
- Consensus: Hybrid PoW + GHOSTDAG; NO stake for consensus (odluka Toni 2026-11-05).
- Defaults: TARGET_BLOCK_MS = 4000 (4 s), GHOSTDAG_K = 5.
- PoW Hash: KovanicaHash (BLAKE3 + 2 MiB memory mix; audit required before mainnet).
- Resource target: VPS friendly, lagano na 2–4 GB RAM-a.
- Native ticker: KVNC (9 decimals, 1 KVNC = 10^9).
- Chain 2 (TESTNET / HYBRID_TESTNET): Kanonski hibridni javni testnet (PoW + GHOSTDAG).
- Milestone M8 is passed and merged into main.

Scope:
Implement a standalone, multithreaded solo miner CLI command `kovanica miner run` that connects to any Kovanica node RPC endpoint, continuously fetches work templates, mines KovanicaHash PoW nonces across worker threads, and submits valid block solutions.

DO:
1) Implement `kovanica miner run` subcommand in `crates/kovanica-cli`:
   - Arguments:
     - `--rpc-url <URL>` (default: `http://127.0.0.1:8545`)
     - `--address <PAYOUT_ADDRESS>` (required: canonical PRO-21 base32 `kvnc...dag`; hybrid networks reject legacy hex)
     - `--threads <N>` (default: available CPU logical cores)
     - `--poll-interval-ms <MS>` (default: 1000 ms to refresh candidate templates when tips advance)
2) Multithreaded Hashing Worker Loop:
   - Polls `kovanica_getWork(address)` via JSON-RPC to retrieve candidate `BlockHeader` template, `target_bits`, and target.
   - Partitions nonce space across worker threads using non-overlapping ranges or thread-strided search.
   - Continuously computes KovanicaHash PoW hash; detects stale work when node advertises newer parent tips / round.
   - When a winning nonce is discovered meeting `target_bits`, immediately submits `kovanica_submitWork(header)` via JSON-RPC.
3) Terminal UI & Hashrate Observability:
   - Live terminal display showing: hashrate (kH/s or MH/s), total hashes evaluated, blocks found, blocks accepted / rejected, current network difficulty / target bits.
   - Graceful shutdown on Ctrl-C with final mining statistics.
4) Test Coverage:
   - Unit tests for CLI parameter parsing, worker thread lifecycle, and work submission mock in `crates/kovanica-cli`.
   - Integration test verifying `kovanica miner run` fetches work, hashes, and submits solutions.

ACCEPTANCE:
- [ ] `kovanica miner run` parses flags and validates payout address according to Chain 2 address rules.
- [ ] Multithreaded worker hashes with KovanicaHash and updates real-time hashrate metrics.
- [ ] Submits solved blocks via `kovanica_submitWork` and handles accepted / rejected responses cleanly.
- [ ] All existing and new tests pass cleanly without regressions.
- [ ] GATES clean: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test`, `python scripts/lint_docs.py`.

GATES:
  cargo fmt --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace --locked --no-fail-fast
  python scripts/lint_docs.py

ANTI:
- Ne dirati legacy konsenzus na Chain 1/2/4.
- Bez ikakvog stake ponderiranja u PoW ruti za Chain 2.
- Bez dodavanja teških ili nepotrebnih vanjskih ovisnosti (koristiti postojeće kovanica-crypto / kovanica-types).
- Bez force-push, bez merge, čekati Gate / Toni review.
```

---

## Prompt 2: Live Multi-Node Docker Compose Cluster & Observability

Use this prompt to build containerized multi-node testnet orchestration with Prometheus and Grafana monitoring.

```markdown
GOAL: Live Multi-Node Docker Compose Cluster & Observability Suite for Chain 2

Base tip: main
Branch: feat/hybrid-docker-cluster-ops
Open ONE PR into main. Do NOT force-push. Do NOT merge. Do NOT rebase over tip. Wait for Gate / Toni.

Context & Invariants (Toni 2026-11-05, locked per ADR 0003):
- Consensus: Hybrid PoW + GHOSTDAG; NO stake for consensus (odluka Toni 2026-11-05).
- Defaults: TARGET_BLOCK_MS = 4000 (4 s), GHOSTDAG_K = 5.
- PoW Hash: KovanicaHash (BLAKE3 + 2 MiB memory mix; audit required before mainnet).
- Resource target: VPS friendly, lagano na 2–4 GB RAM-a.
- Native ticker: KVNC (9 decimals, 1 KVNC = 10^9).
- Chain 2 (TESTNET / HYBRID_TESTNET): Kanonski hibridni javni testnet (PoW + GHOSTDAG).
- Milestone M8 is passed and merged into main.

Scope:
Package a 1-command Docker Compose testnet cluster (`ops/docker/docker-compose.chain2.yml`) featuring 3 peered hybrid testnet nodes with mining enabled, a faucet service, and pre-configured Prometheus + Grafana dashboards tracking PoW and GHOSTDAG metrics.

DO:
1) Docker Compose Cluster Definition (`ops/docker/docker-compose.chain2.yml`):
   - `node1` (seed / bootnode): Chain 2, mining enabled (1 thread), RPC exposed on 8545, P2P on 8000, Prometheus metrics on 9100.
   - `node2` & `node3`: Peered to `node1` via P2P bootnodes, mining enabled (1 thread each), RPC on 8546/8547.
   - `faucet`: Connected to `node1` RPC, dispenses KVNC on Chain 2, exposed on 3000.
   - `prometheus`: Configured to scrape metrics from `node1`, `node2`, `node3`.
   - `grafana`: Pre-loaded with datasource and dashboard provisioning.
2) Grafana Dashboard for Hybrid PoW (`ops/docker/grafana/dashboards/hybrid-testnet.json`):
   - Panels for:
     - Total PoW Blocks Mined (`pow_blocks_mined_total`) & mining rate.
     - GHOSTDAG Blue vs Red blocks breakdown (`ghostdag_blue_blocks_total`, `ghostdag_red_blocks_total`).
     - Network hashrate and target difficulty over time.
     - Connected peer count per node (`peer_count`).
     - Mempool transaction count & block commit latency.
3) Automation Smoke Script (`ops/docker/test-chain2-cluster.sh`):
   - Starts cluster via `docker compose`, verifies containers become healthy, confirms blocks are being mined and blue sets committed, tests faucet payout, and reports cluster health.
4) Documentation:
   - Update `docs/dev/HYBRID-TESTNET-M8.md` and `ops/docker/README.md` with instructions for starting, monitoring, and stopping the Chain 2 Docker cluster.

ACCEPTANCE:
- [ ] `docker compose -f ops/docker/docker-compose.chain2.yml up -d` boots all services cleanly.
- [ ] Nodes peer over P2P, mine PoW blocks, and sequence via GHOSTDAG $k=5$.
- [ ] Prometheus scrapes metrics from all nodes without missing targets.
- [ ] Grafana dashboard renders valid panels for PoW and GHOSTDAG metrics.
- [ ] Smoke test script succeeds.
- [ ] GATES clean: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test`, `python scripts/lint_docs.py`.

GATES:
  cargo fmt --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace --locked --no-fail-fast
  python scripts/lint_docs.py

ANTI:
- Ne dirati niti mijenjati konfiguracije za postojeće mainnet / chain 4 servise.
- Bez ikakvih hardcoded tajni ili privatnih ključeva s produkcijskih čvorova.
- Bez force-push, bez merge, čekati Gate / Toni review.
```

---

## Prompt 3: TypeScript API Client & Explorer UI Alignment (`packages/`)

Use this prompt to align the TypeScript SDK and Explorer / Console frontend with Chain 2 and PoW GHOSTDAG structures.

```markdown
GOAL: TypeScript API Client & Web Explorer Alignment for Chain 2 Hybrid PoW Testnet

Base tip: main
Branch: feat/hybrid-packages-explorer-alignment
Open ONE PR into main. Do NOT force-push. Do NOT merge. Do NOT rebase over tip. Wait for Gate / Toni.

Context & Invariants (Toni 2026-11-05, locked per ADR 0003):
- Consensus: Hybrid PoW + GHOSTDAG; NO stake for consensus (odluka Toni 2026-11-05).
- Defaults: TARGET_BLOCK_MS = 4000 (4 s), GHOSTDAG_K = 5.
- PoW Hash: KovanicaHash (BLAKE3 + 2 MiB memory mix; audit required before mainnet).
- Resource target: VPS friendly, lagano na 2–4 GB RAM-a.
- Native ticker: KVNC (9 decimals, 1 KVNC = 10^9).
- Chain 2 (TESTNET / HYBRID_TESTNET): Kanonski hibridni javni testnet (PoW + GHOSTDAG).
- Milestone M8 is passed and merged into main.

Scope:
Update the frontend TypeScript packages (`packages/api-client`, `packages/console`, `packages/web`) to support Chain 2 (`TESTNET` / `HYBRID_TESTNET`), expose the new mining and DAG JSON-RPC methods, and render PoW / GHOSTDAG blocks and rewards in the explorer.

DO:
1) Update `packages/api-client`:
   - Add `ChainId.HYBRID_TESTNET = 5` to chain registry and default endpoints.
   - Implement typed client methods for:
     - `getMiningInfo(): Promise<MiningInfo>`
     - `getDagTips(): Promise<DagTips>`
     - `getWork(address: string): Promise<WorkTemplate>`
     - `submitWork(header: BlockHeaderSubmission): Promise<SubmitWorkResult>`
   - Export types: `MiningInfo`, `DagTips`, `WorkTemplate`, `SubmitWorkResult`.
2) Update `packages/console`:
   - Add Chain 2 (`TESTNET` / `HYBRID_TESTNET`) to the network dropdown selector.
   - Display mining info panel (hashrate estimate, difficulty, target bits, active DAG tips).
   - Display block reward transactions (2.5 KVNC minted to miners).
3) Update `packages/web` (Explorer):
   - Add DAG tips visualization and indicator for GHOSTDAG blue / red status on committed blocks.
   - Display PoW header fields (`nonce`, `pow_hash`, `target_bits`) when inspecting blocks on Chain 2.
4) Tests & Build:
   - Run TypeScript typechecks and unit tests in `packages/api-client`, `packages/console`, and `packages/web`.

ACCEPTANCE:
- [ ] `packages/api-client` exports Chain 2 and typed methods for `getMiningInfo`, `getDagTips`, `getWork`, `submitWork`.
- [ ] `packages/console` connects to Chain 2 nodes and displays mining and DAG metrics.
- [ ] `packages/web` renders PoW headers and DAG tips on Chain 2.
- [ ] `npm run build` and `npm run test` pass across packages without errors.
- [ ] Rust GATES clean: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test`, `python scripts/lint_docs.py`.

GATES:
  cargo fmt --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace --locked --no-fail-fast
  python scripts/lint_docs.py

ANTI:
- Ne dirati legacy chain definicije (Chain 1, 2, 3, 4).
- Bez mijenjanja tokenomics parametara (ticker ostaje KVNC, 9 decimala, 2.5 KVNC nagrada).
- Bez force-push, bez merge, čekati Gate / Toni review.
```

---

## Prompt 4: BIP32 / BIP44 Hierarchical Deterministic (HD) Wallet Derivation

Use this prompt to implement standard BIP32/BIP44 derivation paths for multi-account management from a single seed phrase.

```markdown
GOAL: BIP32 / BIP44 Hierarchical Deterministic (HD) Wallet Derivation for Kovanica

Base tip: main
Branch: feat/bip32-hd-wallet-derivation
Open ONE PR into main. Do NOT force-push. Do NOT merge. Do NOT rebase over tip. Wait for Gate / Toni.

Context & Invariants (Toni 2026-11-05):
- Coin: KVNC (9 decimals, 1 KVNC = 10^9).
- Canonical address: PRO-21 base32 (`kvnc` + 57 bech32 chars + `dag` = 64 chars).
- Cryptographic keys: Ed25519 signing keys.
- Coin type for derivation path: `9999'` (or registered coin type per Kovanica spec).
- Standard derivation path: `m/44'/9999'/account'/change/address_index`.
- Open item listed on root `TASKLIST.md`.

Scope:
Implement BIP32 / BIP44 hierarchical deterministic key derivation for Ed25519 (SLIP-0010 standard) in `crates/kovanica-wallet-core` and wire `--account` and `--index` options into `crates/kovanica-cli`.

DO:
1) Implement HD derivation in `crates/kovanica-wallet-core`:
   - Implement SLIP-0010 (Ed25519 master key derivation from 512-bit seed and hardened child key derivation).
   - Support standard derivation paths: `m/44'/9999'/<account>'/<change>'/<index>'`.
   - Implement derive function returning `(SigningKey, PublicKey, Address)`.
2) Integrate with `crates/kovanica-cli`:
   - Add `--account <INDEX>` (default: 0) and `--index <INDEX>` (default: 0) to wallet derivation and keystore generation commands.
   - Update wallet address display to show derived account path.
3) Security & Validation:
   - Constant-time operations for secret material; zeroize intermediate keys.
   - Test vectors verifying derivation against known SLIP-0010 test vectors.
   - Roundtrip sign-and-verify test confirming derived keys sign valid transactions across all chains.
4) Documentation & Tasklist:
   - Document HD derivation specification in `docs/dev/HD-WALLET-SPEC.md`.
   - Mark BIP32 HD derivation item as completed in `TASKLIST.md`.

ACCEPTANCE:
- [ ] SLIP-0010 Ed25519 HD derivation implemented and passes standard test vectors.
- [ ] `kovanica-cli` supports deriving multiple accounts/addresses from a single mnemonic seed phrase.
- [ ] All derived keys produce valid canonical PRO-21 addresses and verifiable transaction signatures.
- [ ] GATES clean: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test`, `python scripts/lint_docs.py`.

GATES:
  cargo fmt --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace --locked --no-fail-fast
  python scripts/lint_docs.py

ANTI:
- Ne podržavati ne-hardened derivation za Ed25519 (SLIP-0010 zahtijeva hardened derivation za sve komponente staze).
- Bez izlaganja privatnih ključeva u logovima ili nezaštićenoj memoriji (Zeroize).
- Bez force-push, bez merge, čekati Gate / Toni review.
```

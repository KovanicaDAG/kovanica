# Kovanica — CODE_INDEX

Maps documentation topics to authoritative source files under `/root/kovanica`.
Links are `file://` absolute paths (Obsidian-clickable). **Do not invent paths —
verify before adding.**

---

## Project & Conventions

| Topic | Source |
|-------|--------|
| Monorepo operating guide (entry point) | [file:///root/kovanica/AGENTS.md](file:///root/kovanica/AGENTS.md) |
| Meta-directory index of repos | [file:///root/kovanica/README.md](file:///root/kovanica/README.md) |
| Network & domain architecture | [file:///root/kovanica/NETWORK.md](file:///root/kovanica/NETWORK.md) |
| Protocol repo conventions (82 KB dev guide) | [file:///root/kovanica/kovanica-protocol/AGENTS.md](file:///root/kovanica/kovanica-protocol/AGENTS.md) |
| Restructure blueprint | [file:///root/kovanica/kovanica-protocol/Restructure.md](file:///root/kovanica/kovanica-protocol/Restructure.md) |
| Restructure plan (filter-repo) | [file:///root/kovanica/kovanica-protocol/Restructure-plan.md](file:///root/kovanica/kovanica-protocol/Restructure-plan.md) |
| Workspace manifest | [file:///root/kovanica/kovanica-protocol/Cargo.toml](file:///root/kovanica/kovanica-protocol/Cargo.toml) |

## Consensus & DAG — `kovanica-dag`

| Topic | Source |
|-------|--------|
| Block struct + BlockId (BLAKE3), work/timestamp/nonce | [file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/block.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/block.rs) |
| Dag store: insert/validate, reachability, mergeset, tips, preview, difficulty/PoW/VRF switches | [file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/dag.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/dag.rs) |
| GHOSTDAG: selected parent, mergeset, k-cluster colouring | [file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/ghostdag.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/ghostdag.rs) |
| Linearization (recursive GHOSTDAG order) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/ordering.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/ordering.rs) |
| Reachability oracle (interval-tree + future-covering sets) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/reachability.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/reachability.rs) |
| Proof-of-work (Nakamoto hash-target, mine) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/pow.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/pow.rs) |
| Difficulty retargeting | [file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/difficulty.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/difficulty.rs) |
| VRF (ECVRF Ristretto255) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/vrf.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/vrf.rs) |
| Snapshots (replay-log persistence) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/snapshot.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/src/snapshot.rs) |
| Consensus adversarial tests | [file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/tests/consensus.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-dag/tests/consensus.rs) |

## Ledger & State — `kovanica-state`

| Topic | Source |
|-------|--------|
| Ledger: apply_block/apply_dag, per-block state, finality/pruning, hybrid, activation gates | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/ledger.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/ledger.rs) |
| Keys/addresses — `kvnc…dag`, ed25519 | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/keys.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/keys.rs) |
| Transactions, sighash, per-asset outputs | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/tx.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/tx.rs) |
| UTXO set (+ per-UTXO creation height) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/utxo.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/utxo.rs) |
| Multisig (RFC-001) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/multisig.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/multisig.rs) |
| Native tokens / multi-asset (RFC-002) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/validation.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/validation.rs) |
| Script v2 stack machine (RFC-003) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/script_v2.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/script_v2.rs) |
| HTLC template (RFC-004) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/htlc.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/htlc.rs) |
| Vault / time-lock template (RFC-005) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/vault.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/vault.rs) |
| Stake registry (bond/unbond, sortition) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/stake.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/stake.rs) |
| SPV validation | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/spv.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/spv.rs) |
| LedgerStore (append-only replay log) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/store.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/src/store.rs) |
| Multisig adversarial suite (35 tests) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/tests/multisig_consensus.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/tests/multisig_consensus.rs) |
| HTLC suite (23 tests) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/tests/htlc.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/tests/htlc.rs) |
| Vault/CSV suite (26 tests) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/tests/vault.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/tests/vault.rs) |
| Native-token suite (29 tests) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-state/tests/native_token_consensus.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-state/tests/native_token_consensus.rs) |

## Node, Mempool, P2P — `kovanica-node`

| Topic | Source |
|-------|--------|
| Node orchestration (genesis, transfers, vault/htlc helpers) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/node.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/node.rs) |
| HTTP RPC | [file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/rpc.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/rpc.rs) |
| Mempool v2 (orphan pool, fees) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/mempool_v2.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/mempool_v2.rs) |
| P2P mesh | [file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/p2p.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/p2p.rs) |
| P2P hardening (rate limits, scoring/banning) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/p2p_hardening.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/p2p_hardening.rs) |
| TCP transport / block sync | [file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/net.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/net.rs) |
| DHT (Kademlia) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/dht.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/dht.rs) |
| DNS seed discovery | [file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/dns_seed.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/dns_seed.rs) |
| Explorer (JSON API + WS) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/explorer.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/explorer.rs) |
| SPV light client | [file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/spv.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/spv.rs) |
| Tier Nolan atomic swap (RFC-004) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/atomic_swap.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/atomic_swap.rs) |
| Metrics (Prometheus) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/metrics.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/metrics.rs) |
| Node binary (serve/demo) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/main.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-node/src/main.rs) |

## CLI & FFI

| Topic | Source |
|-------|--------|
| CLI wallet (`kovanica` binary, clap) | [file:///root/kovanica/kovanica-protocol/crates/kovanica-cli/src/main.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-cli/src/main.rs) |
| CLI wallet commands | [file:///root/kovanica/kovanica-protocol/crates/kovanica-cli/src/wallet.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-cli/src/wallet.rs) |
| FFI LightNode surface | [file:///root/kovanica/kovanica-protocol/crates/kovanica-ffi/src/light_node.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-ffi/src/light_node.rs) |
| FFI bindgen binary | [file:///root/kovanica/kovanica-protocol/crates/kovanica-ffi/src/bin/uniffi-bindgen.rs](file:///root/kovanica/kovanica-protocol/crates/kovanica-ffi/src/bin/uniffi-bindgen.rs) |

## Clients — Web / Wallet / Mobile

| Topic | Source |
|-------|--------|
| Web app package manifest | [file:///root/kovanica/kovanica-web/site/package.json](file:///root/kovanica/kovanica-web/site/package.json) |
| Web routes (index, explorer, wallet, faucet, htlc, multisig, vaults, stealth, roadmap…) | [file:///root/kovanica/kovanica-web/site/src/routes/](file:///root/kovanica/kovanica-web/site/src/routes/) |
| Wallet components | [file:///root/kovanica/kovanica-web/site/src/components/wallet/](file:///root/kovanica/kovanica-web/site/src/components/wallet/) |
| Web lib (network, wallet, ledger, protocol) | [file:///root/kovanica/kovanica-web/site/src/lib/](file:///root/kovanica/kovanica-web/site/src/lib/) |
| Wallet Android (Compose) | [file:///root/kovanica/kovanica-wallet/android/](file:///root/kovanica/kovanica-wallet/android/) |
| Wallet iOS (SwiftUI) | [file:///root/kovanica/kovanica-wallet/ios/](file:///root/kovanica/kovanica-wallet/ios/) |
| Wallet browser extension | [file:///root/kovanica/kovanica-wallet/extension/src/](file:///root/kovanica/kovanica-wallet/extension/src/) |
| Mobile light-node Android | [file:///root/kovanica/kovanica-mobile/android/app/src/main/java/com/kovanica/lightnode/](file:///root/kovanica/kovanica-mobile/android/app/src/main/java/com/kovanica/lightnode/) |

## Specs & Plans — `kovanica-protocol/docs/`

| Topic | Source |
|-------|--------|
| KVP index/status | [file:///root/kovanica/kovanica-protocol/docs/KVP.md](file:///root/kovanica/kovanica-protocol/docs/KVP.md) |
| KVP-102 native tokens | [file:///root/kovanica/kovanica-protocol/docs/KVP-102-NativeTokens.md](file:///root/kovanica/kovanica-protocol/docs/KVP-102-NativeTokens.md) |
| RFC-001 multisig | [file:///root/kovanica/kovanica-protocol/docs/RFC-001-Multisig.md](file:///root/kovanica/kovanica-protocol/docs/RFC-001-Multisig.md) |
| RFC-002 native tokens | [file:///root/kovanica/kovanica-protocol/docs/RFC-002-NativeTokens.md](file:///root/kovanica/kovanica-protocol/docs/RFC-002-NativeTokens.md) |
| RFC-003 stealth + script v2 | [file:///root/kovanica/kovanica-protocol/docs/RFC-003-ScriptV2-and-Stealth.md](file:///root/kovanica/kovanica-protocol/docs/RFC-003-ScriptV2-and-Stealth.md) |
| RFC-004 HTLC / atomic swap | [file:///root/kovanica/kovanica-protocol/docs/RFC-004-Htlc.md](file:///root/kovanica/kovanica-protocol/docs/RFC-004-Htlc.md) |
| RFC-005 vault / CSV | [file:///root/kovanica/kovanica-protocol/docs/RFC-005-Vault.md](file:///root/kovanica/kovanica-protocol/docs/RFC-005-Vault.md) |
| Tokenomics | [file:///root/kovanica/kovanica-protocol/docs/TOKENOMICS.md](file:///root/kovanica/kovanica-protocol/docs/TOKENOMICS.md) |
| P0/P1/P2 legit board | [file:///root/kovanica/kovanica-protocol/docs/LEGIT-BOARD.md](file:///root/kovanica/kovanica-protocol/docs/LEGIT-BOARD.md) |
| Mainnet exit criteria | [file:///root/kovanica/kovanica-protocol/docs/MAINNET-CRITERIA.md](file:///root/kovanica/kovanica-protocol/docs/MAINNET-CRITERIA.md) |
| Testnet soak plan | [file:///root/kovanica/kovanica-protocol/docs/TESTNET-SOAK.md](file:///root/kovanica/kovanica-protocol/docs/TESTNET-SOAK.md) |
| Audit plan (Q1 2027 target) | [file:///root/kovanica/kovanica-protocol/docs/AUDIT-PLAN.md](file:///root/kovanica/kovanica-protocol/docs/AUDIT-PLAN.md) |
| Bug bounty | [file:///root/kovanica/kovanica-protocol/docs/BUG-BOUNTY.md](file:///root/kovanica/kovanica-protocol/docs/BUG-BOUNTY.md) |
| Reproducible builds | [file:///root/kovanica/kovanica-protocol/docs/REPRODUCIBLE-BUILDS.md](file:///root/kovanica/kovanica-protocol/docs/REPRODUCIBLE-BUILDS.md) |
| Entity/legal | [file:///root/kovanica/kovanica-protocol/docs/ENTITY-LEGAL.md](file:///root/kovanica/kovanica-protocol/docs/ENTITY-LEGAL.md) |
| Ops hardening | [file:///root/kovanica/kovanica-protocol/docs/OPS-HARDENING.md](file:///root/kovanica/kovanica-protocol/docs/OPS-HARDENING.md) |
| Product polish | [file:///root/kovanica/kovanica-protocol/docs/PRODUCT-POLISH.md](file:///root/kovanica/kovanica-protocol/docs/PRODUCT-POLISH.md) |
| Explorer API docs | [file:///root/kovanica/kovanica-protocol/docs/api/explorer.md](file:///root/kovanica/kovanica-protocol/docs/api/explorer.md) |
| Plans (mobile-light-node, htlc, vault-time-lock, stealth-script-v2, android app) | [file:///root/kovanica/kovanica-protocol/docs/plans/](file:///root/kovanica/kovanica-protocol/docs/plans/) |

## Ops & Deployment

| Topic | Source |
|-------|--------|
| Seed ops runbook | [file:///root/kovanica/kovanica-protocol/OPERATIONS.md](file:///root/kovanica/kovanica-protocol/OPERATIONS.md) |
| Testnet params | [file:///root/kovanica/kovanica-protocol/TESTNET.md](file:///root/kovanica/kovanica-protocol/TESTNET.md) |
| Testnet RFC-006 notes | [file:///root/kovanica/kovanica-protocol/TESTNET-RFC006.md](file:///root/kovanica/kovanica-protocol/TESTNET-RFC006.md) |
| Deploy seeds | [file:///root/kovanica/kovanica-protocol/scripts/deploy-seed.sh](file:///root/kovanica/kovanica-protocol/scripts/deploy-seed.sh) · [file:///root/kovanica/kovanica-protocol/scripts/deploy-seed2.sh](file:///root/kovanica/kovanica-protocol/scripts/deploy-seed2.sh) |
| Installer | [file:///root/kovanica/kovanica-installer/install-kovanica.sh](file:///root/kovanica/kovanica-installer/install-kovanica.sh) |
| Node operator onboarding | [file:///root/kovanica/kovanica-node/README.md](file:///root/kovanica/kovanica-node/README.md) · [file:///root/kovanica/kovanica-node/JOIN.md](file:///root/kovanica/kovanica-node/JOIN.md) |
| How to mine / light node / get KVNC | [file:///root/kovanica/kovanica-protocol/HOWTO_MINE.md](file:///root/kovanica/kovanica-protocol/HOWTO_MINE.md) · [HOWTO_LIGHT_NODE.md](file:///root/kovanica/kovanica-protocol/HOWTO_LIGHT_NODE.md) · [HOWTO_GET_KVNC.md](file:///root/kovanica/kovanica-protocol/HOWTO_GET_KVNC.md) |

## Hardware Wallet & Ecosystem (new)

| Topic | Source |
|-------|--------|
| Ledger app (no_std Rust, APDU) | [file:///root/kovanica/kovanica-ledger-app/src/rust/src/lib.rs](file:///root/kovanica/kovanica-ledger-app/src/rust/src/lib.rs) (dir: [file:///root/kovanica/kovanica-ledger-app/](file:///root/kovanica/kovanica-ledger-app/)) |
| Trezor coin definition | [file:///root/kovanica/trezor-coin-def/kovanica.py](file:///root/kovanica/trezor-coin-def/kovanica.py) · [PR_DESCRIPTION.md](file:///root/kovanica/trezor-coin-def/PR_DESCRIPTION.md) |
| RAG agent service | [file:///root/kovanica/kovanica-agent/agent/main.py](file:///root/kovanica/kovanica-agent/agent/main.py) (dir: [file:///root/kovanica/kovanica-agent/](file:///root/kovanica/kovanica-agent/)) |
| Knowledge base vault | [file:///root/kovanica/kovanica-brain-vault/README.md](file:///root/kovanica/kovanica-brain-vault/README.md) (dir: [file:///root/kovanica/kovanica-brain-vault/](file:///root/kovanica/kovanica-brain-vault/)) |

---

*Verify before claiming: paths are from the 2026-09-16 restructure map; re-check if the source moves.*
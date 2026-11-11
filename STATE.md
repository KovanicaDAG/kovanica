# STATE — living source of truth

**Tip:** `9029514b448fd825cd9f19f6d72885974bc3340a` (merge of `chore/canonical-cleanup-chain1-hybrid` #224 on 2026-11-10: chain 1 declared canonical hybrid, chain 4 removed from the registry, chain-independent stored-address parsing; prior #223 `feat(hybrid)` merge `7b5b5a7`, prior living tips `d0b129639150e114d88d6d97336cb82546ae4f76`, `89394a2da7d9fb119072aacd1ff7997f0e3225c4`, `559cc2014592e1c621da2df78d9671e7476972d1`).
Update this file when frozen params or open product gaps change. Point-in-time dumps live under `docs/archive/`; they are not SoT.

## Active Priority: Hybrid PoW + GHOSTDAG (P0–P8)

- **Consensus Direction (Toni 2026-11-05):** Hybrid PoW + GHOSTDAG. **NO stake for consensus**.
- **Priority Rank:** Hybrid PoW + GHOSTDAG phases P0–P8 rank strictly above PRO-6, PRO-14, and Gate leftover 20/36/7.
- **PRO-7:** **FROZEN** for hybrid (Mysticeti VRF leader path dead under hybrid PoW + GHOSTDAG).
- **Public Launch:** Public launch is delayed from unconditional 2026-11-25 until **Hybrid Testnet M8** passes.
- **PoW Hash:** Planned PoW hash name is **KovanicaHash** (strictly NOT KunaHash; blake3 + 2 MiB memory mix; audit required before mainnet).
- **Consensus Defaults:** `TARGET_BLOCK_MS = 4000`, `GHOSTDAG_K = 5`.
- **Product Ticker:** **KVNC** (renamed back from KUNA per Toni 2026-11-05).

## Canonical chains (Hybrid PoW + GHOSTDAG)

- `chain_id = 1` (Mainnet) and `chain_id = 2` (Testnet) are both Hybrid PoW + GHOSTDAG (`NoStake82M`). `chain_id = 3` (Devnet) and `chain_id = 1337` (Local) are dev chains. The former `chain_id = 4` (Testnet v2) was removed from the registry.
- `chain_id = 2`. Canonical Hybrid PoW + GHOSTDAG public testnet (`NETWORK_TAG_HYBRID_TESTNET = "kovanica-testnet"`, network tag `hybrid-testnet`). Canonical PRO-21 base32 addresses only; legacy hex rejected.
- Network tag form: `<chain_id>-<16 hex genesis digest>` (see `docs/TESTNET-SEED-NODES.md` for topology).
- Launch gating: public launch delayed from unconditional 2026-11-25 until hybrid testnet M8 passes.
- Genesis bootstrap seeds: **1..=20** on mainnet and testnet; they hold no stake (`NoStake82M`).
- Ordering default: GHOSTDAG (lifted k=3 hard-lock in P0 to allow `GHOSTDAG_K = 5`).
- Topology, seeds, and ops runbooks: `docs/TESTNET-SEED-NODES.md`, `docs/TESTNET-RESET.md`, `docs/VALIDATOR-ONBOARDING.md`. Do not treat archived `sync-*.md` tables as live inventory.

## Frozen protocol params (code-true decisions)

| Topic | Value |
| --- | --- |
| Coin | KVNC, **9** decimals (`ONE_KVNC = 10^9`, legacy alias `ONE_KUNA`) |
| Consensus | Hybrid PoW + GHOSTDAG (NO stake in consensus path; `TARGET_BLOCK_MS = 4000`, `GHOSTDAG_K = 5`; PoW: KovanicaHash) |
| Cap / subsidy sketch | 90.2M total; mining budget 82M; treasury 8M; genesis validators 0 (1M folded into mining); founder premine 200k |
| Address | `kvnc` + 57 bech32-alphabet base32 + `dag` (**64** chars); 32 B Ed25519 ‖ 29-bit blake3 checksum. Legacy hex only on chains **3, 1337** |
| Epoch (legacy) | **7200** DAG rounds ≈ 8 h ≈ **2400** leader heights (legacy Mysticeti constant still in tree until hybrid flag) |
| Jail (legacy) | `JAIL_ROUNDS = 7200` committed leaders (legacy / non-consensus under hybrid) |
| Min validator stake (legacy) | `50_000` KVNC (`MIN_VALIDATOR_STAKE = 50_000 * ONE_KUNA`; legacy / non-consensus) |
| Min delegation (legacy) | `50_000` KVNC (`MIN_DELEGATION = 50_000 * ONE_KUNA`; legacy / non-consensus) |
| `funded_accounts.toml` | Allowed on every chain **except** mainnet (chain 1) |
| Hash / sig domains | `KUNA-*` / `KVNC-*` crypto context tags remain for protocol compatibility |

Gated hardening still inert on live chain until a genesis reset flips `*_ACTIVE_FROM_ROUND` to `0` (gas, contract, size limits, slashing).

## Open product gaps (not archived noise)

1. Ops drift (live fleet cutover): VPS systemd units & migration script prepared in `ops/systemd/` and `scripts/cutover-vps-systemd.sh` (live cutover execution pending VPS Seed Ops deploy); DNS seed audit probe automated in `scripts/check-dns-seeds.sh` (`docs/SEED-DNS-AUDIT.md`).
2. External security review (formal audit dispatch of `v1.0.0-mainnet` covering KovanicaHash and Wasmi sandbox; template: `docs/AUDIT_REPORT.template.md`, `docs/AUDIT-SCOPE.md`, `docs/AUDITOR-INVITATION.md`).
3. Production Key Ceremony & Launch ($T_0 - 48\text{h}$): Live key collection and final distribution for 15–20 mainnet bootstrap operators via `ops/genesis/ceremony.sh` per `docs/KEY-CEREMONY.md` and `docs/MAINNET-RUNBOOK.md`.
4. Chain-2 genesis bundle publication (ops): no `kovanica-chain2-genesis.tar.gz` exists on any `v<version>-node` release, so `node/scripts/install.sh` (chain 2 by default, fail-closed on a missing tag) cannot complete an install until ops cuts the bundle from the live chain-2 genesis set with an `expected-tag` file. The archive itself no longer has to be assembled by hand: `kovanica-node genesis-bundle --chain-id 2 --source <genesis-dir>` (#226) writes the deterministic `kovanica-chain<chain-id>-genesis.tar.gz` with `SHA256SUMS` and `expected-tag`, refusing non-genesis files (keys/keystores/configs). `.github/workflows/release-node.yml` still packages the retired `testnet/chain4/` bundle and must be re-pointed (or switched to `genesis-bundle`) in the same step.

**Done / frozen, do not re-open as missing:**
- Mainnet Freeze & Gate Sign-Off (Phases C1–C3): Audit scoping updated (`docs/AUDIT-SCOPE.md`), genesis key ceremony streamlined (`ops/genesis/ceremony.sh`), validator bounds relaxed to `1..=20`, byte-for-byte deterministic genesis validated (`2f46c80940dc5226f8f25aad6eff428c3cd58c7fb505cf9861ad63db4cdbabc9`), soak pre-flight verification passed (monotonic height growth, 0 OOM kills under 2 GB limit), Gates 1–6 signed off in `docs/MAINNET-FREEZE-CHECKLIST.md`, and release tag `v1.0.0-mainnet` tagged on `main`.
- Automated CI pipeline: fmt, clippy, doc lint, workspace tests, cargo-deny (licenses/advisories/bans/sources), gitleaks 8.24.3 (`.github/workflows/ci.yml`, `scripts/ci-local.sh`).
- Governance (proposals and stake-weighted voting via `kovanica-cli` and `kovanica-staking`), BIP32 HD derivation (`kovanica-wallet-core`), soak harness (`ops/soak/`), DNS audit probe (`scripts/check-dns-seeds.sh`), and mainnet runbook (`docs/MAINNET-RUNBOOK.md`).
- PRO-7 (Mysticeti VRF leader set + pin): **FROZEN** for hybrid.
- Hybrid PoW + GHOSTDAG P1–P8 & Milestone M8 (PoW header primitives, preimages, target conversion, Chain ID 2 wiring, KovanicaHash BLAKE3+2MiB algorithm, header verification, CPU mining worker, DAA, P2P network anti-DoS ingress, GHOSTDAG k=5 committer, 2.5 KVNC reward minting, node miner loop, JSON-RPC getWork/submitWork/miningInfo/dagTips, CLI mining/dag commands, Prometheus metrics, live 3-node cluster integration test suite, faucet alignment, ops templates). Milestone M8 PASSED.
- Contract SDK (`crates/kovanica-contract-sdk`) and Indexer (`crates/kovanica-indexer`).
- Node installer (`node/scripts/install.sh`) is chain-agnostic (#225): `--chain-id` (default 2), bundle asset derived from the chain id, expected tag from `--expected-tag`/`KOVANICA_EXPECTED_TAG` or an `expected-tag` file inside the checksum-verified bundle, fail-closed when unpinned, `--skip-tag-check` as the explicit opt-out, chain-1 `funded_accounts.toml` refusal, and no legacy Mysticeti knobs in the generated config.
- Genesis bundle writer (`kovanica-node genesis-bundle`, #226): deterministic hand-written ustar tar + stored-deflate gzip (no compression dependency), allowlist of genesis files, `SHA256SUMS` and `expected-tag` inside the archive, refusals for unregistered chains, non-genesis files, a `chain_id` mismatch and mainnet `funded_accounts.toml`; tag computed from the files plus `--chain-id` only (no `KOVANICA_*` overrides).

# DECISIONS — Architecture & Operations Decision Log

> **Links:** [[FACTS]] · [[OPERATIONS]] · [[myObsidianVaultDAG]] · [[SESSIONS]] · [[MASTER-STATUS]]
> **Master status:** See [[MASTER-STATUS]] for consolidated project status including this decision log

Why things are the way they are. Append-only; never delete rows — mark
superseded decisions instead. Sources: git history, session logs, runbooks.

| Date | Decision | Rationale / Consequence | Source |
|---|---|---|---|
| 2026-08-23 | Single merged repo `kovanica-protocol` (renamed from `kovanica-ledger`); CLI → `crates/kovanica-cli`, web → `web/` | one workspace, easier dev; all workspace deps reference the single GitHub URL | `myObsidianVaultDAG.md` §Migration Notes |
| 2026-08-23 | Vault strategy = **doc snapshots synced by script**, never submodules | submodule attempts failed and were deliberately removed; committing nested repos creates phantom submodules | root `AGENTS.md` §5 |
| 2026-08-24 | P2P is **TCP only** on `KOVANICA_LISTEN`; libp2p/30333 removed | it bound a port and never gossiped blocks — no second network path exists | `TESTNET.md` |
| 2026-08-24 | Bootstrap is DNS-only via grey-cloud `seed.kovanica.online:9000`; seed keeps `KOVANICA_PEERS=off` | Cloudflare orange-cloud proxying breaks raw TCP :9000 — clones must dial the grey-cloud name/origin IP | `TESTNET.md`, `OPERATIONS.md` §3 |
| 2026-08-24 | Runtime chain data lives outside any git checkout (`/root/kovanica-data`) | lost-chain incident: pre-reset data dir was deleted while the old process held it (genesis `27d5f750…`, 127 blocks gone) | `OPERATIONS.md` §1, §4.5 |
| 2026-08-24 | Deploys SSH to **:2222**, not :22 | Hostinger-level filtering times out GitHub-runner :22 after repeated logins; sshd listens on both | `OPERATIONS.md` §2, §4.1 |
| 2026-08-24 | `metrics-exporter-prometheus` with `default-features = false` | we render `/metrics` ourselves; http-listener feature drags openssl and breaks ARM cross-builds | `OPERATIONS.md` §4.6 |
| 2026-08-24 | Public mirror excludes `kovanica-cli` (workspace membership filtered in mirror manifest) | publication decision still open; mirror pipeline keeps it private by design | [[2026-08-24-public-mirror-and-seed3]], `TODO.md` |
| 2026-08-24 | Release = rolling tag `v<workspace-version>` replaced in place; publish skips if any build fails | never a partial release; assets carry sha256 | [[2026-08-24-public-mirror-and-seed3]] |
| 2026-08-24 | seed3 = AWS EC2 (eu-north-1), systemd unit, mining on, explorer/metrics loopback-only | first true off-box node for soak testing; proves deploy-seed.sh beyond same-host seed2 | `OPERATIONS.md` §6, ROADMAP |
| standing | `#![forbid(unsafe_code)]` crate-wide | consensus determinism + auditability over micro-optimizations | protocol `AGENTS.md` §4 |
| standing | Consensus changes need rationale + adversarial tests; tie-breaks fall back to `BlockId` byte order | GHOSTDAG output must be a pure function of the DAG | protocol `AGENTS.md` §5, `myObsidianVaultDAG.md` §Design Principles |

## Superseded

| Date | Superseded decision | By |
|---|---|---|
| ≤2026-08-23 | multi-repo layout (`kovanica-ledger` / `kovanica-cli` / `kovanica-web`) | merged repo, 2026-08-23 |
| ≤2026-08-24 | vault `ecosystem.config.js` snapshot pointing at old `/home/BetterCallDzuks/kovanica-ledger` layout | `deploy/ecosystem.config.cjs` snapshot (`kovanica-explorer`, cwd `/root/kovanica-protocol`) |

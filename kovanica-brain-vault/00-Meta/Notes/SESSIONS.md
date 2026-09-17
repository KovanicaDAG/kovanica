# SESSIONS — Work Log Index

> **Links:** [[ROADMAP]] · [[TODO]] · [[DECISIONS]] · [[FACTS]]

Continuity across sessions: each working session appends one log file and adds
a row here. Read the latest row first — it is the current state of the world.

## Convention

1. File name: `YYYY-MM-DD-<slug>.md`, placed next to this index.
2. Header: wiki-link banner (`> **Links:** …`) linking what you touched.
3. Sections in order: **Shipped** · **Lessons burned in** · **Open follow-ups**
   (checkboxes so TODO/ROADMAP can reference them).
4. Update [[ROADMAP]] / [[TODO]] in the same commit; add decisions to
   [[DECISIONS]]; push with `docs:` prefix (root `AGENTS.md` §4).
5. Never edit old logs to change facts — append a correction note or a new log.

## Index

| Date | Log | Theme | Outcome snapshot |
|---|---|---|---|
| 2026-09-08 | [[2026-09-08-rfc-004-htlc-shipped]] | RFC-004 HTLC shipped + route to 5.2 Vault | Rebased + merged **RFC-004 HTLC atomic swap** (PR #88 `fb13741`, +4612/−20, `cargo test` 740 pass); restored `main` web typecheck that was blocking all PRs (PR #95 `a960742`, regenerated `routeTree.gen.ts` + `AddressQr value=`); docs status → Shipped (PR #96 `9fb83b5`, KVP-104 Shipped, LEGIT-BOARD P1.1 done). Next per evolving plan critical path: **5.2 Time-lock Vault (CSV/CLTV)** — needs per-UTXO creation-height (RFC-004 deferred CSV for exactly this). |
| 2026-09-03 | [[2026-09-03-kovanica-agent-scaffold]] | kovanica-agent scaffold review + starting work | Reviewed `kovanica-agent/` (RAG+tool LangGraph dev assistant), then **shipped starting work** (draft PR #79, branch `agent/kovanica-agent-starting-work`, gated green): RAG indexer/search (Rust-aware chunking), read-only testnet RPC allowlist, real JWT auth, SQLite persistent checkpointer, and a `sandbox/runner` sidecar as the sole docker.sock owner (agent-api calls it over HTTP). Then shipped the **human-gated apply→draft-PR path** (`patchstore.py` SQLite proposal store + `apply.py` throwaway-worktree apply/commit/push/gh-draft-PR, fail-closed behind `AGENT_GIT_APPLY_ENABLED`, never mutates the main checkout). **PR #79 MERGED** `2026-09-03` (merge commit `85c4149`), all CI checks green. Not yet: cargo dep pre-vendoring, gVisor, sidecar token, arming apply in a real container. |
| 2026-09-03 | [[2026-09-03-kvnc-branding-and-wallets]] | KVNC brand + remove __grok + standalone wallet + iOS sideload | `kvnc-logo.png` is the token icon on web; `__grok` PWA installer fully removed (PR #76, merged); standalone API-backed Kovanica Wallet — Android Compose + iOS SwiftUI, no light node/FFI — with wallet.yml CI (PR #77, merged; all checks green). PR #78 adds an unsigned sideloadable iOS `.ipa` CI artifact so iPhone install works with no Mac (AltStore/Sideloadly), verified green. |
| 2026-08-24 | [[2026-08-24-public-mirror-and-seed3]] | Public mirror pipeline + seed3 | `sync-public-node` workflow (mirror→build→publish), rolling `v0.1.0`, `install.sh` prebuilt-first, dnf support, seed3 live on AWS eu-north-1; open items live in ROADMAP/TODO |

## Before starting any session

1. Read the newest row above + its log.
2. Skim `TODO.md` "Next session" block and ROADMAP active item.
3. `git -C /root/Obsidian-Vault pull --rebase origin main` and
   `/root/Obsidian-Vault/scripts/sync-vault.sh --check` before touching docs.
4. In code matters, `/root/kovanica-protocol` is truth — read its `AGENTS.md`.

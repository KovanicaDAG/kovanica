# 2026-09-03 — kovanica-agent scaffold review + possibilities

> **Links:** [[myObsidianVaultDAG]] · [[CODE_INDEX]] · [[SESSIONS]] · [[ROADMAP]] · [[TODO]]

Reviewed the **`kovanica-agent/`** directory in `kovanica-protocol` (a scaffold for a
"Kovanica DevTeam Agent" — an AI coding assistant for the Rust codebase). This note
records what it is, its current (unfinished) state, and where it could go. Authoritative
source: `/home/antonio/KovanicaDAG/kovanica-protocol/kovanica-agent`.

> ⚠ **Status: this is a **scaffold**. It is NOT wired to the real repo or the real
> testnet yet. Nothing under `kovanica-agent/` is deployed or referenced by
> `kovanica-protocol/AGENTS.md` stages.** Do not mistake the docker-compose stack for a
> live service.

## What it is (architecture)

A self-hosted, RAG + tool-using dev assistant built around the protocol, in Python
(FastAPI + LangGraph) with a vLLM-hosted local LLM, plus an ephemeral Docker sandbox
for running cargo on the real repo.

```
docker-compose.yml                 full stack wiring
  vllm        Qwen2.5-Coder-32B-Instruct-AWQ, NVIDIA GPU, OpenAI-compatible :8000
  qdrant      vector DB for RAG retrieval :6333
  agent-api   FastAPI /chat + /confirm (LangGraph), :8080
  open-webui  chat frontend, :3000
  sandbox-image  one-shot per-cargo-call container (profiled "build-only", not persistent)

agent/
  main.py      FastAPI: /chat, /confirm, /healthz  (role from auth, audit trail)
  graph.py     LangGraph: router → agent → tools/human-gate
  requirements.txt
sandbox/
  Dockerfile       rust:1.82-slim + build deps, no network tools
  entrypoint.sh    whitelist re-check (defense in depth) + `cargo … --offline`
```

## Design rules baked in (good — do not weaken)

1. **Role is from auth, never the message.** `role` ("dev" vs "user") is set by
   `resolve_role()` from the `Authorization` header (`agent/graph.py:1-12`, `main.py:34-39`).
   A user cannot talk their way into `dev` tools.
2. **cargo only ever runs in the ephemeral sandbox**, whitelisted to `check | test |
   clippy | build`, `network_disabled=True`, `mem_limit=2g`, 2 CPUs, non-root
   (`agent/graph.py:75-105`). `entrypoint.sh` re-checks the whitelist as a second line
   of defense (defense-in-depth, `sandbox/entrypoint.sh`).
3. **Repo mutations are proposed, never applied.** `git_diff_suggest` only stages a
   proposal; graph stops at a `human_gate` interrupt until a human `POST /confirm`
   approves (`agent/graph.py:108-116, 174-184`).
4. **Safety whitelist for cargo** and a suspicious-arg rejection (`; | & $() ` rm sudo
   curl wget`) mirror the project's "run real output, don't invent" ethos.

## Current state — what is actually implemented vs stubbed

| Piece | State |
|---|---|
| FastAPI `/chat` `/confirm` `/healthz` + audit log | ✅ implemented |
| LangGraph router / agent / tools / human-gate graph | ✅ implemented |
| Role split (dev vs user tool sets) | ✅ implemented |
| `search_codebase` | ⚠ **stub** — returns `"TODO: wire to Qdrant"` |
| `explain_concept` | ⚠ stub (delegates to the above stub) |
| `query_node_api` | ⚠ **stub** — returns `"TODO: wire to node RPC"` |
| `git_diff_suggest` | ⚠ proposal only; **real `apply_patch` deliberately not implemented** (`main.py:89-92`) |
| Qdrant **indexing pipeline** (chunking the Rust codebase) | ❌ not present — README says "indexing pipeline not included here" |
| Pre-vendored crate deps for `--offline` | ❌ not done (`sandbox/Dockerfile:25-27` commented out) |
| gVisor (`runsc`) confinement | ❌ not enabled (`graph.py:99` commented) |
| Real auth (Keycloak/JWT) | ❌ stub token placeholder (`main.py:34-39`) |
| docker.sock → scoped runner sidecar | ❌ still mounts `/var/run/docker.sock` (⚠ security note in compose) |
| Persistent LangGraph checkpoint (vs `MemorySaver`) | ❌ in-memory only (`graph.py:217`) |
| Repo indexing job (GitHub webhook → embed → Qdrant) | ❌ not present |

So today it is an **auth-split + sandboxed-cargo + human-gated filter scaffold** with two
live RAG/RPC tools and the mutation path all stubbed.

## Security notes (from reading the code)

- **`docker.sock` is mounted into `agent-api`** — that is effective root on the host; it
  can launch *any* container, not just the sandbox. The compose file flags this
  (`docker-compose.yml:41-47`). Fine only for a trusted DevTeam on a private box until a
  scoped sandbox-runner sidecar replaces it.
- `resolve_role()` accepts `Bearer DEV_TOKEN_PLACEHOLDER` → `dev`. That is a dev scaffold;
  replacing with real JWT/Keycloak is a stated prerequisite before it leaves localhost.
- No paths in this note should be treated as shipped API — they are scaffold names.

## Possibilities / where this could go

Ideas grounded in what the scaffold already has and what the repo needs — none are
committed decisions; the scaffold's own README already lists most of these as
before-release todos.

1. **Make the RAG real (highest value).** Build the indexing pipeline the README
   advertises: Rust-aware chunking by `fn`/`impl` block + markdown chunking over
   `docs/`, re-indexed on a GitHub webhook (or on merge-to-main). Populate the
   `kovanica_codebase` Qdrant collection and wire `search_codebase` to it. Until then,
   the agent cannot answer `explain_concept` / `search_codebase` with real citations
   (and the project's own convention says "if you can't find a real citation, say so").

2. **Grounded citations are the killer feature here.** A dev assistant that answers
   with file:line citations into `kovanica-protocol` (matching the `doc-writer`/
   `SYSTEM_PROMPT.md` citation rule) would slot cleanly into the existing docs culture.
   The SYSTEM_PROMPT already mandates "cite file:line or say you can't."

3. **Reuse the existing testnet contract.** `query_node_api` should talk to the live
   explorer/node RPC (`/api/head`, `/api/state`, `/metrics`, WS `/ws`) — read-only for
   `user`, plus node status for `dev`. This turns the agent into a "ask the chain" bot
   rather than pure code search. Requires the doc-writer's authority rule preserved:
   read-only for RPC, never write.

4. **Live-soak operator companion.** Point it at the testnet soak data + Prometheus
   (`alerting_rules.yml`, `/metrics`) so it can summarise block rate, peer count, mempool,
   seed3 OOM state, etc. That operationalizes the "production hardening" phase's talk.

5. **DevTools that mirror AGENTS.md workflows.** The scaffold's `git_diff_suggest` +
   `/confirm` gate matches the repo rule "propose, human approves, never auto-apply to
   the default branch." A natural next step is finishing the `apply_patch` tool so it
   creates a feature branch + draft PR on approval (still human-gated), instead of
   stopping at a proposal.

6. **Reindexing + drift.** Since snapshots under `KovanicaDAG/kovanica-*` describe the
   *old* multi-repo layout while `myObsidianVaultDAG.md` describes the merged
   `kovanica-protocol`, any indexing pipeline must target the **current** merged repo
   paths/links or it will answer with stale locations.

7. **If/when this matures**, it becomes a natural candidate for a vault **synced-file**
   entry (its README/system prompt carry real conventions), kept in lockstep with
   `AGENTS.md` like the ledger/node/cli snapshots. Not now — it is still a scaffold.

8. **Self-hosting the LLM is itself a product decision.** vLLM + Qwen2.5-Coder-32B on
   a GPU is a real infra cost and needs a GPU host; per-request sandbox containers need
   a Docker host. Worth deciding *before* investing: dev-only internal tool, vs shipped
   user-facing assistant (which needs the auth + sidecar + non-root work first).

## Update — starting work shipped (same day, draft PR #79)

> **Branch:** `agent/kovanica-agent-starting-work` → **draft PR #79**
> (`feat(agent): wire kovanica-agent scaffold to reality`). Python-only under
> `kovanica-agent/`; gated clean (`fmt`, `clippy`, `cargo test` — 0 failures).

Shipped (supersedes the "stub" rows in the table above):
- **RAG**: `agent/{indexer,rag,embed}.py` — Rust-aware chunker (fn/impl/struct/
  enum/trait/mod, accurate line numbers) + markdown chunking, fastembed
  `bge-small-en-v1.5` → Qdrant `kovanica_codebase`. `search_codebase`/
  `explain_concept` now live.
- **Read-only RPC**: `query_node_api` allowlisted against `KOVANICA_NODE_URL`
  (default live explorer); rejects `mine`/`faucet`/`submit`/`operator`.
- **JWT auth**: `agent/auth.py` — JWKS mode via `AUTH_JWKS_URL` or dev-token
  mode; role derived server-side, fail-closed to `user`. Stub `resolve_role()`
  gone.
- **Persistent sessions**: `agent/checkpoint.py` SQLite checkpointer replaces
  `MemorySaver`; audit log under `agent-data` volume.
- **Security hardening**: `sandbox/runner/` sidecar is now the ONLY service
  mounting `docker.sock`; `agent-api` calls it via `agent/sandbox_client.py`.
  `run_cargo_command` delegates to the sidecar.
- **Integration fixes**: imports reconciled to the Docker top-level-module
  layout; docker-compose `agent-data` volume added; README updated.

Not yet done (documented in README checklist): pre-vendoring cargo deps,
gVisor `runtime="runsc"`, sidecar API token, and arming the apply→PR path in a
real repo (git/gh identity + `AGENT_GIT_APPLY_ENABLED=1` in the container).

### Update 2 — apply→draft-PR path shipped (same branch/PR #79)

Commit `2fd02a4` adds the human-gated mutation path that was the "next step":
- `agent/patchstore.py` — SQLite proposal store keyed by session ($AGENT_DB,
  same file as the checkpointer); add/get/mark_applied/clear.
- `agent/apply.py` — `apply_and_open_pr(...)`: validates every path (reject
  absolute / `..` / git metadata) and patch (`git apply --check`), applies to
  a **throwaway git worktree** branched off the remote base, commits, pushes,
  opens a **draft PR** via `gh`; teardown in `finally`. **Fail-closed**: without
  `AGENT_GIT_APPLY_ENABLED=1` it only validates + reports (`dry_run`);
  `AGENT_GIT_DRY_RUN=1` forces validate-only. Never mutates the main checkout.
- `graph.py` `git_diff_suggest` stages into the patchstore (session id from
  LangGraph config `thread_id`, env fallback); `main.py` `/confirm` approves →
  `apply_and_open_pr`, audits outcome, marks applied on success, resumes the
  graph past the gate in all paths. docker-compose documents/passes `AGENT_GIT_*`.

Verified: py_compile clean, no relative imports, patchstore smoke green, apply
dry-run/unsafe-path behavior green; enabled run exercised fetch+worktree then
tore down on the expected no-network error (fail-closed + cleanup intact).
Cargo gate green (fmt/clippy/test).

## Open follow-ups

- [x] **Review/merge draft PR #79** — **MERGED** `2026-09-03` as commit `85c4149`
      (merge of `agent/kovanica-agent-starting-work` → `main`; both agent commits
      `ead9c9a` + `2fd02a4` landed). All CI checks passed (Build/lint/typecheck,
      Rust build/test/lint, Web build); VPS deploy skipped as expected. The
      kovanica-agent now ships: RAG search, read-only RPC, JWT auth, persistent
      sessions, docker.sock-owning sandbox-runner sidecar, and a human-gated
      apply → throwaway-branch → draft-PR path.
- [ ] After merge, decide whether to track `kovanica-agent` as a real workstream
      (add to ROADMAP). Next milestones: automate re-index (webhook), sidecar
      hardening, and arming the apply→PR path in a deployed container.
- [ ] If worked on, keep its README/SYSTEM_PROMPT in sync with
      `kovanica-protocol/AGENTS.md` conventions in the same commits that touch either.

---
name: api-design
description: Design and evolve node RPC, explorer, and wallet APIs — use when adding endpoints, changing schemas, or generating SDKs
---

# API Design Skill

Use when adding/changing REST, JSON-RPC, GraphQL, or WebSocket interfaces in kovanica-node/explorer/web.

## Trigger Keywords
API, endpoint, RPC, REST, GraphQL, WebSocket, OpenAPI, schema, SDK, breaking change

## Decision Guide

| Need | Choose |
|------|--------|
| Simple resource CRUD | REST `/v1/<resource>` |
| Complex nested queries (explorer) | GraphQL |
| Push updates / live data | WebSocket subscription |
| Wallet/node control plane | JSON-RPC 2.0 |

## Adding an Endpoint (checklist)

1. Does it leak private data? (mempool contents OK; keys/peer IPs behind auth)
2. Resource-oriented name, plural noun, kebab-case: `GET /v1/block-rewards`
3. Pagination required if unbounded: `page[size]` ≤ 100 + `Link` header
4. Errors follow RFC 7807 shape with stable `code` field
5. Add to OpenAPI spec (`utoipa` annotations) — spec is source of truth
6. Regenerate SDKs; add integration test hitting real handler
7. If modifying existing field semantics → new version, never mutate in place

## Compatibility Rules

- Adding optional field/param: safe. Removing/retyping: breaking.
- Breaking changes require `/v2` prefix + 6-month `/v1` sunset window with `Sunset` header
- GraphQL: only add fields or deprecate with `@deprecated(reason:)`; never remove within major version

## Wire Examples

```jsonc
// GET /v1/blocks?filter[height]=100..105&page[size]=2
{
  "data": [{ "id": "blk…", "height": 100, "blue_score": 42 }],
  "links": { "next": "/v1/blocks?filter[height]=102..105&page[size]=2" }
}

// WS subscribe
→ {"jsonrpc":"2.0","id":1,"method":"subscribe","params":["blockAdded"]}
← {"jsonrpc":"2.0","method":"blockAdded","params":{"block":{...}}}
```

## Rate Limiting

Public endpoints: token bucket per IP (default 100 req/s), headers `X-RateLimit-Limit/Remaining/Reset`. Authenticated tiers override via `X-API-Key`.

## Finish Line

A task is not done when the files are edited — it is done when the work is shipped.

- **Vault edits** (`/root/Obsidian-Vault`): `git add -A && git commit -m "docs: <what changed>" && git push origin main`. If the push is rejected: `git pull --rebase origin main`, resolve, push again.
- **Code edits** (`/root/kovanica-protocol`): that repo's `AGENTS.md` governs — feature branch (never the default branch), gate with `cargo fmt --check && cargo clippy --all-targets && cargo test`, push branch, open draft PR.
- **No files changed?** Nothing to commit — say so explicitly and stop.

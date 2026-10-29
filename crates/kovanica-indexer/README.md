# kovanica-indexer

MVP crate for Kovanica. Indexes committed-leader receipts into a local redb
store and serves them over REST (`/logs`, `/events`, `/status`) and a
WebSocket push endpoint (`/ws`).

## Local dev wiring

Run a node on `:8545`, then run the indexer pointing at it:

```sh
KOVANICA_INDEXER_NODE_RPC=http://127.0.0.1:8545/rpc \\
KOVANICA_INDEXER_LISTEN=0.0.0.0:8787 \\
cargo run -p kovanica-indexer
```

Point the web apps at it via env (see `packages/web/.env.example` and
`packages/console/.env.example`):

```
VITE_INDEXER_HTTP_URL=http://127.0.0.1:8787
VITE_INDEXER_WS_URL=ws://127.0.0.1:8787/ws
```

The indexer is optional: if it is unset or unreachable, the explorer and
console fall back to node RPC (`kovanica_getLogs`) where possible.

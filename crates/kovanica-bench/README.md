# kovanica-bench

Optional load-generation and performance benchmark helpers for Kovanica post-reset
soak. Ships unused and has no dependency on the node or consensus engines — it
talks only over the public JSON-RPC interface.

## Build

```bash
cargo build -p kovanica-bench
```

## Usage

```bash
# Health poll at 50 requests/s for 30s
cargo run -p kovanica-bench -- health --rate 50 --duration 30s

# Transfer load: 8 workers, ~100 tx/s total, 60s, 1 atom fee
cargo run -p kovanica-bench -- transfer-load \
    --rpc-url http://127.0.0.1:8545 \
    --chain-id 1337 \
    --workers 8 \
    --rate 100 \
    --duration 60s \
    --amount 1 \
    --fee 1
```

## Modes

- `health`: hammer `kovanica_syncStatus` at a rate; reports ok/error counts and
  latency percentiles.
- `transfer-load`: N workers sign and submit native transfers via
  `kovanica_sendRawTransaction` using `kovanica-sdk` builders; reports latency
  histogram, success/error counts, tps and a greppable CSV summary line.

## Flags

| Flag | Default | Meaning |
|------|---------|---------|
| `--rpc-url` | `http://127.0.0.1:8545` | JSON-RPC endpoint |
| `--chain-id` | `1337` | chain id pinned into signatures |
| `--workers` / `--concurrency` | `8` | concurrent submitters |
| `--rate` | `100` | requests per second (rate limit) |
| `--duration` | `60s` | run length (`30s`, `60000ms`) |
| `--amount` | `1` | transfer amount in atoms |
| `--fee` | `1` | fee per tx in atoms |

## Design

- Rate limiter is per-worker (single-threaded pacing), no cross-worker
  coordination, so the crate stays dependency-light and deterministic.
- Each worker signs transactions locally with `kovanica_sdk::transfer` and
  submits raw bytes; nonce handling is left to the operator (workers start at a
  hint nonce derived from worker index).
- Output is one summary line plus an optional `CSV: ...` line for plotting.
- Safe to ship unused: compiled only when the `kovanica-bench` bin is built.

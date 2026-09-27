# kovanica-cli

> **Command-line client for the Kovanica testnet** — A Rust GHOSTDAG BlockDAG whose token is **KVNC** (8 decimals, 1 KVNC = 10⁸ atoms). Talks to the public explorer JSON API and manages a local Ed25519 wallet.

---

## Install

```bash
cargo build --release
# Binary at target/release/kovanica
```

---

## Global Options

| Option | Env Var | Description |
|--------|---------|-------------|
| `--api <url>` | `KOVANICA_API` | Explorer base URL (default: `https://explorer.kovanica.online`) |

---

## Commands

### Read-Only Queries

| Command | Description |
|---------|-------------|
| `kovanica head` | Chain head: network, genesis, selected tip, block count, min fee |
| `kovanica p2p` | P2P listen address, peers, and bootstrap node |
| `kovanica bootstrap` | Network bootstrap parameters |
| `kovanica state` | Full node state snapshot |
| `kovanica blocks` | Blocks in the DAG (from `/api/state` — `/api/blocks` returns binary) |
| `kovanica balance <address>` | Balance and unspent outputs for an address |

> **Address formats accepted**: `kvnc…dag` (base58) or 64-hex.

### Wallet

| Command | Description |
|---------|-------------|
| `kovanica keygen [--key <path>] [--force]` | Generate new Ed25519 key, save it (0600), print address |
| `kovanica address [--key <path>]` | Print address for a saved key |

**Key file** (default: `kovanica.key`, override with `--key` or `KOVANICA_KEY`):
- Stores 32-byte seed as hex
- Owner-only permissions (0600)
- **Secret** — `.gitignore` excludes `*.key`

### Send

```bash
kovanica send --key kovanica.key --to <address> --amount <atoms>
```

**Flow**: `send` fetches the transaction's signature hash from `/api/prepare`, signs those exact bytes locally with the saved key, and broadcasts the 64-byte signature via `/api/submit`. Amounts in **atoms** (1 KVNC = 100,000,000 atoms). The node recomputes and re-verifies the spend — the signature hash is never trusted from the client.

---

## Example Session

```bash
# Generate key
kovanica keygen --key alice.key

# Check balance
kovanica balance kvnc…dag

# Query local node
kovanica --api http://127.0.0.1:8080 head

# Send 1 KVNC
kovanica send --key alice.key --to kvnc…dag --amount 100000000
```

---

## Architecture

- **Address encoding** (`kvnc…dag`) and **spend signing** delegated to `kovanica-state` (the node's own crate) — CLI can never drift from the ledger
- **Offline signing** — private keys never leave the CLI process
- **Prepare → Sign → Submit** — matches browser wallet and node's mempool test exactly

---

## Related Repositories

| Repo | Purpose |
|------|---------|
| [kovanica-protocol](https://github.com/KovanicaDAG/kovanica-protocol) | Core consensus + ledger (source of `kovanica-state`) |
| [kovanica-node](https://github.com/KovanicaDAG/kovanica-node) | Node binary (serves API) |
| [kovanica-sdk](https://github.com/KovanicaDAG/kovanica-sdk) | Rust/WASM SDK (programmatic equivalent) |
| [kovanica-web](https://github.com/KovanicaDAG/kovanica-web) | Web wallet/explorer |

---

## License

**MIT OR Apache-2.0**
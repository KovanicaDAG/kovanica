# kovanica-node

> **Public release snapshot for node operators** — Thin packaging surface over the `kovanica-protocol` crates. Builds the single `kovanica-node` binary. The VPS builds from here.

---

## Quick Start

### One-Command Install (Linux/macOS)

```bash
curl -sSfL https://raw.githubusercontent.com/KovanicaDAG/kovanica-node/main/scripts/install.sh | bash
```

### With systemd (auto-start on login)

```bash
curl -sSfL https://raw.githubusercontent.com/KovanicaDAG/kovanica-node/main/scripts/install.sh | bash -s -- --systemd
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/KovanicaDAG/kovanica-node/main/scripts/install.ps1 | iex
```

The installer prefers a **prebuilt binary** from the latest GitHub Release; falls back to building from source (installs Rust automatically if needed).

---

## After Install

```bash
# Linux/macOS
~/kovanica-node/run.sh

# Windows
%USERPROFILE%\kovanica-node\run.cmd
```

Then open **http://127.0.0.1:8080** and verify:

```bash
curl -s http://127.0.0.1:8080/api/head
curl -s https://explorer.kovanica.online/api/head
```

`network` and `genesis` must match. Blocks/tip will catch up after first pull.

---

## Live Testnet

| Property | Value |
|----------|-------|
| **Network** | `kovanica-testnet` |
| **Explorer** | https://explorer.kovanica.online |
| **Wallet** | https://wallet.kovanica.online |
| **P2P Port** | TCP **9000** only |
| **Bootstrap** | `seed.kovanica.online:9000` |
| **Token** | KVNC (8 decimals) |

> **Your node never sees your wallet seed.** You sign in the browser; the node only verifies.

---

## Join the Public Testnet

```bash
export KOVANICA_LISTEN=0.0.0.0:9000
export KOVANICA_PEERS=seed.kovanica.online:9000
export KOVANICA_FAUCET=0
export KOVANICA_ALLOW_RESET=0
export KOVANICA_OPERATOR=0
export KOVANICA_DATA="$PWD/data"

./target/release/kovanica-node explorer 127.0.0.1:8080
```

- Outbound TCP 9000 to the seed is enough to catch up
- Open **inbound 9000/tcp** only if you want to serve other peers
- If DNS/IPv6 stalls, use the origin IP: `export KOVANICA_PEERS=145.223.116.178:9000`

---

## Run Locally (Solo/Offline)

```bash
export KOVANICA_FAUCET=0
export KOVANICA_ALLOW_RESET=0
export KOVANICA_OPERATOR=0
export KOVANICA_DATA="$PWD/data"

./target/release/kovanica-node explorer 127.0.0.1:8080
```

First start writes genesis into `KOVANICA_DATA`. **Keep that directory.**

---

## Environment Variables

| Variable | Default | Notes |
|----------|---------|-------|
| `KOVANICA_LISTEN` | `0.0.0.0:9000` | P2P bind (also tries `[::]:9000`) |
| `KOVANICA_PEERS` | `seed.kovanica.online:9000` | Comma-separated bootstrap list |
| `KOVANICA_CONSENSUS` | `poa` (when unset) | Admission mode (`poa` or `pow`; `pow` is `[TARGET]`-removed) |
| `KOVANICA_AUTHORITIES` | *(unset)* | PoA genesis authority set (64-hex Ed25519 pubkeys, comma-separated) |
| `KOVANICA_AUTHORITY_THRESHOLD` | Strict majority | Signatures needed for `AuthorityUpdateTx` |
| `KOVANICA_SLOT_DURATION` | `3000` | Slot length in ms |
| `KOVANICA_FAUCET` | `0` | Enable open faucet (operator funds) |
| `KOVANICA_DATA` | `./data` | Persistence directory |
| `KOVANICA_ALLOW_RESET` | `0` | Allow chain reset (dev only) |
| `KOVANICA_OPERATOR` | `0` | **Never enable on public clones** |

> **No `KOVANICA_DIFFICULTY` variable exists** — PoW difficulty was always node-local policy; under PoA there is nothing to retarget.

---

## Build from Source (Developers)

```bash
git clone https://github.com/KovanicaDAG/kovanica-node.git
cd kovanica-node
cargo build --release
```

Binary: `./target/release/kovanica-node` (or `.exe` on Windows).

---

## Repository Structure

```
kovanica-node/
├── kovanica-node-bin/      # Thin binary wrapper → builds `kovanica-node`
├── scripts/                # One-click installers (Linux/macOS/Windows) + USB
└── deploy/                 # Optional nginx/Caddy/systemd examples
```

The five protocol crates (`kovanica-dag`, `kovanica-state`, `kovanica-node`, `kovanica-cli`, `kovanica-ffi`) live in `kovanica-protocol/crates/` (single source of truth); this workspace builds over them via path dependencies.

---

## Publishing Prebuilt Binaries (Maintainers)

After tagging a release, attach tarballs named:

- `kovanica-node-x86_64-linux.tar.gz`
- `kovanica-node-aarch64-linux.tar.gz`
- `kovanica-node-x86_64-macos.tar.gz`
- `kovanica-node-aarch64-macos.tar.gz`

Each tarball should contain a single `kovanica-node` (or `kovanica-node.exe`) binary at root. See [RELEASES.md](RELEASES.md) for exact steps.

---

## USB Stick (Offline-Friendly)

Copy the folder `scripts/usb/` onto a FAT32 stick. On the target machine run `install.sh` (or `install.ps1` on Windows) from that folder. Network still required for first build/binary download.

---

## Documentation

| Document | Path |
|----------|------|
| **Join the Testnet** | [JOIN.md](JOIN.md) |
| **Testnet Parameters** | [TESTNET.md](TESTNET.md) |
| **RFC-006 Testnet** | [TESTNET-RFC006.md](../protocol/TESTNET-RFC006.md) |
| **Operations Runbook** | [OPERATIONS.md](../protocol/OPERATIONS.md) |
| **Network/Domains** | [NETWORK.md](../NETWORK.md) |

---

## License

**MIT OR Apache-2.0** — See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
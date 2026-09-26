# kovanica-node

Run a **KovanicaDAG** node on the public testnet.

GHOSTDAG BlockDAG + UTXO ledger (Ed25519). Native token **KVNC** (8 decimals).

| | |
| --- | --- |
| Explorer | https://explorer.kovanica.online |
| Wallet | https://wallet.kovanica.online |
| Network | `kovanica-testnet` |
| P2P | TCP **9000** only (no libp2p) |
| Bootstrap | `seed.kovanica.online:9000` |

The node never sees your wallet seed. You sign in the browser; the node only verifies.

> > **Consensus decision (ratified 2026-09-25): Kovanica is PoA-only.**
> > Proof-of-Work is being removed from the protocol. See
> > `protocol/docs/RFC-POA-Migration.md` §0 (canonical). Items marked `[TARGET]`
> > are ratified but not yet implemented; `[CURRENT]` items describe shipped code.
> >
> > `[TARGET]` `KOVANICA_POW`, `KOVANICA_MINE` and `KOVANICA_MINE_SECS` are
> > **removed**, and "Consensus PoW" becomes "Proof-of-Authority: fixed authority
> > set, slot round-robin, Ed25519 authority signature per block". PoA needs
> > `KOVANICA_CONSENSUS`, `KOVANICA_AUTHORITIES`,
> > `KOVANICA_AUTHORITY_THRESHOLD` and `KOVANICA_SLOT_DURATION`; for a normal
> > node run you set none of them, because `KOVANICA_CONSENSUS` already defaults
> > to `poa` when unset.
> >
> > `[CURRENT]` The env blocks below still work as written — they describe the
> > pre-reset PoW testnet, and are annotated rather than deleted so the current
> > chain stays reproducible.
> >
> > **Unchanged by the migration:** GHOSTDAG **k=3**, the UTXO ledger, Ed25519
> > signing, and every RFC-006 constant — MAX_SUPPLY **90.2M KVNC**, s₀
> > **10 KVNC/block**, era **2,000,000 blocks**, α **3/4**, maturity **100
> > blocks**, fee split **75% burned / 25% producer**,
> > **1 KVNC = 100_000_000 atoms**. The curve is height-indexed and
> > `cumulative_minted` is capped in `apply_block`, so supply math is
> > independent of admission. Only the wall-clock *pace* changes: fixed 3000 ms
> > slots, no difficulty retarget, no gap-fill.

---

## Quick start (recommended)

**Linux / macOS** — one command, no `git clone` required:

```sh
curl -sSfL https://raw.githubusercontent.com/KovanicaDAG/kovanica-node/main/scripts/install.sh | bash
```

Start on login (systemd user unit):

```sh
curl -sSfL https://raw.githubusercontent.com/KovanicaDAG/kovanica-node/main/scripts/install.sh | bash -s -- --systemd
```

**Windows** (PowerShell):

```powershell
irm https://raw.githubusercontent.com/KovanicaDAG/kovanica-node/main/scripts/install.ps1 | iex
```

The installer prefers a **prebuilt binary** from the latest GitHub Release when available, and falls back to building from source (Rust is installed automatically if needed).

After install:

```sh
# Linux / macOS
~/kovanica-node/run.sh

# Windows
%USERPROFILE%\kovanica-node\run.cmd
```

Then open http://127.0.0.1:8080 and verify:

```sh
curl -s http://127.0.0.1:8080/api/head
curl -s https://explorer.kovanica.online/api/head
```

`network` and `genesis` must match. `blocks` / tip will catch up after the first pull.

More detail: **[JOIN.md](./JOIN.md)** · Testnet parameters: **[TESTNET.md](./TESTNET.md)**

---

## USB stick (offline-friendly scripts)

Copy the folder [`scripts/usb/`](./scripts/usb/) onto a FAT32 stick. On the target machine run `install.sh` (or `install.ps1` on Windows) from that folder. Network is still required for the first build / binary download.

---

## Build from source (developers)

Requirements: Rust 1.75+ ([rustup](https://rustup.rs)), Linux / macOS / Windows.

```sh
git clone https://github.com/KovanicaDAG/kovanica-node.git
cd kovanica-node
cargo build --release
```

Binary: `./target/release/kovanica-node` (or `.exe` on Windows).

### Run a local node (solo / offline)

```sh
export KOVANICA_POW=1        # [CURRENT] pre-reset PoW; [TARGET] removed
export KOVANICA_MINE=0
export KOVANICA_MINE_SECS=120
export KOVANICA_FAUCET=0
export KOVANICA_ALLOW_RESET=0
export KOVANICA_OPERATOR=0
export KOVANICA_DATA="$PWD/data"

./target/release/kovanica-node explorer 127.0.0.1:8080
```

`[TARGET]` Under PoA-only the first three exports are gone; PoA is the default
when `KOVANICA_CONSENSUS` is unset, so a solo node just needs:

```sh
export KOVANICA_FAUCET=0
export KOVANICA_ALLOW_RESET=0
export KOVANICA_OPERATOR=0
export KOVANICA_DATA="$PWD/data"

./target/release/kovanica-node explorer 127.0.0.1:8080
```

Open http://127.0.0.1:8080  
First start writes genesis into `KOVANICA_DATA`. Keep that directory.

### Join the public testnet

```sh
export KOVANICA_LISTEN=0.0.0.0:9000
export KOVANICA_PEERS=seed.kovanica.online:9000
export KOVANICA_POW=1        # [CURRENT] pre-reset PoW; [TARGET] removed
export KOVANICA_MINE=0
export KOVANICA_MINE_SECS=120
export KOVANICA_FAUCET=0
export KOVANICA_ALLOW_RESET=0
export KOVANICA_DATA="$PWD/data"

./target/release/kovanica-node explorer 127.0.0.1:8080
```

- Outbound TCP 9000 to the seed is enough to catch up.
- Open **inbound** 9000/tcp only if you want to serve other peers.
- If DNS/IPv6 stalls the pull on some Ubuntu setups, use the origin IP:

  ```sh
  export KOVANICA_PEERS=145.223.116.178:9000
  ```

---

## Environment variables (clone defaults)

| Variable | Default | Notes |
| --- | --- | --- |
| `KOVANICA_LISTEN` | `0.0.0.0:9000` | P2P bind (also tries `[::]:9000`) |
| `KOVANICA_PEERS` | `seed.kovanica.online:9000` | Comma-separated bootstrap list |
| `KOVANICA_CONSENSUS` | `poa` when unset | Admission mode (`poa` or `pow`; anything else panics). `[TARGET]` `pow` goes away |
| `KOVANICA_AUTHORITIES` | *(unset)* | PoA genesis authority set, comma-separated 64-hex Ed25519 pubkeys. Unset → testnet placeholder from the public constant `AUTHORITY_PLACEHOLDER_BASE = 9001`; mainnet refuses to boot |
| `KOVANICA_AUTHORITY_THRESHOLD` | strict majority | Signatures needed for an `AuthorityUpdateTx` |
| `KOVANICA_SLOT_DURATION` | `3000` | Slot length in ms |
| `KOVANICA_POW` | `1` | Consensus PoW — `[CURRENT]` pre-reset / **`[TARGET]`-removed** |
| `KOVANICA_MINE` | `0` | Leave off unless you intend to mint — `[TARGET]`-removed |
| `KOVANICA_MINE_SECS` | `120` | Interval when mining is on — `[TARGET]`-removed |
| `KOVANICA_HYBRID` | `0` | Hybrid PoW+staked admission — `[TARGET]`-removed entirely (RFC-POA §0.7.1, decided 2026-09-25) |
| `KOVANICA_FAUCET` | `0` | |
| `KOVANICA_DATA` | `./data` | Persistence directory |
| `KOVANICA_ALLOW_RESET` | `0` | |
| `KOVANICA_OPERATOR` | `0` | Never enable on public clones |

**There is no `KOVANICA_DIFFICULTY` variable, and none is planned** — PoW
difficulty was always node-local policy, never operator-tunable, and under PoA
there is nothing to retarget.

Addresses on screen look like `kvnc…dag` (base58). The ledger stores 64-hex; both forms work in the API / send UI.

---

## About this repository

This repo is the **thin packaging surface** for the runnable node. The five
protocol crates (`kovanica-dag`, `kovanica-state`, `kovanica-node`,
`kovanica-cli`, `kovanica-ffi`) live in the monorepo's `protocol/crates/`
(single source of truth); this workspace builds a single `kovanica-node`
binary over them via path dependencies.

```
kovanica-node/                  ← this repository
├── kovanica-node-bin/          # Thin binary wrapper → builds `kovanica-node`
├── scripts/                    # One-click installers + USB
└── deploy/                     # Optional nginx / Caddy / systemd examples
```

The broader protocol (web UI source, extra tooling) lives in the unified monorepo when you need it for development. For simply **running a node**, this repository is enough.

---

## Publishing prebuilt binaries (maintainers)

The install script prefers assets from GitHub Releases. After tagging a release, attach tarballs named:

- `kovanica-node-x86_64-linux.tar.gz`
- `kovanica-node-aarch64-linux.tar.gz`
- `kovanica-node-x86_64-macos.tar.gz`
- `kovanica-node-aarch64-macos.tar.gz`

Each tarball should contain a single `kovanica-node` (or `kovanica-node.exe`) binary at the root. See [RELEASES.md](./RELEASES.md) for the exact steps.

---

## License

MIT OR Apache-2.0. See [LICENSE-MIT](./LICENSE-MIT) and [LICENSE-APACHE](./LICENSE-APACHE).

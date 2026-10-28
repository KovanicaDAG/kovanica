# Running a Kovanica node

Prebuilt `kovanica-node` binaries for Linux (x86_64, aarch64) and the chain-4
genesis bundle are published as GitHub releases by
`.github/workflows/release-node.yml`. Tag naming: `v<version>-node`.

## Install (public testnet, chain 4)

```sh
curl -fsSL https://kovanica.online/download/install.sh | sh
```

The installer:

1. detects the architecture and downloads
   `kovanica-node-<arch>-linux.tar.gz` plus `kovanica-chain4-genesis.tar.gz`
   from the release, verifying both against the published `SHA256SUMS`;
2. installs the binary (default `/usr/local/bin/kovanica-node`);
3. copies the genesis bundle into the data directory (default
   `/var/lib/kovanica`);
4. writes a non-validating config (`/etc/kovanica/kovanica.toml`) with the
   three seed nodes as bootnodes and RPC bound to loopback;
5. runs `kovanica-node --check-config` and `kovanica-node network-tag`, and
   **refuses to continue** unless the tag equals the published
   `4-0eacd5699210b145`;
6. on systemd hosts, installs and starts `kovanica-node.service` as the
   `kovanica` system user.

Useful options: `--no-service`, `--data-dir <dir>`, `--prefix <dir>`,
`--version <v>`, `--skip-tag-check`, `--uninstall`, `--help`.
Without root it installs under `$HOME/.local` and `$HOME/.kovanica` and does
not touch systemd.

## Why the tag check matters

Every chain-4 node must share the same chain id, binary and genesis file set;
the network tag (`<chain_id>-<16 hex of the genesis config digest>`) is mixed
into all libp2p protocol ids and gossip topics. A node with a different tag
silently has no peers. See `docs/TESTNET-RESET.md`.

## Validators

The installer never generates or handles validator keys — it produces a
read-only full node. Validator setup (keystore, stake, `run_validator = true`,
`validator_key`) is documented in `docs/VALIDATOR-ONBOARDING.md`.

## Cutting a release

```sh
git tag v0.1.0-node
git push origin v0.1.0-node
```

The workflow builds both architectures, runs the tag check against
`testnet/chain4/`, and publishes the tarballs, the bundle and `SHA256SUMS`.
Bump the default `VERSION` in `node/scripts/install.sh` when the download
should move to the new release.

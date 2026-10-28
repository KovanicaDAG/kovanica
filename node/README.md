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

## Known limitation on chain 4: a fresh node cannot import state yet

A node installed by this script starts at height 0 with the chain-4 genesis
file set. It peers correctly (correct tag, seed nodes reachable) but it cannot
advance, because the only bootstrap path for a pruned chain is the chunked
state snapshot, and every snapshot is refused with:

```
rejected state snapshot ... sealed header state_root ... != full-state
commitment ... recomputed from the snapshot accounts, contracts and staking
```

The cause is not the download or the bundle. Sealed header roots on chain 4 are
account-only because `STATE_COMMITMENT_ACTIVE_FROM_ROUND` is still `u64::MAX`
(`crates/kovanica-types/src/lib.rs`), while `snapshot_committed_root()` always
recomputes the full-state root that includes the reserved
`AUX_STATE_LEAF_ADDRESS` leaf. The two can never be equal, so
`import_verified_snapshot()` fails closed for every peer — by design, until the
sealed root covers the full state. Weakening that check would let a peer forge
staking and contract rows.

The fix is the documented one: activate full-state commitment at the next
genesis / testnet reset (`STATE_COMMITMENT_ACTIVE_FROM_ROUND = 0`) with every
validator on a build that contains it. That is a consensus change and a new
network tag, so it needs the owner's approval and the full reset procedure in
`docs/TESTNET-RESET.md`. Until then, a chain-4 node can peer, serve the API and
hold the genesis file set, but cannot sync balances or produce blocks.

Validators that already hold the state are unaffected; this only blocks *new*
nodes joining the mesh.

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

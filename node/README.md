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

## State sync on chain 4: full-state commitment is being activated

A node installed by this script starts at height 0 with the chain-4 genesis
file set and bootstraps from the chunked state snapshot served by its peers.
Sealed header roots on chain 4 are still **account-only**, because
`STATE_COMMITMENT_ACTIVE_FROM_ROUND` was `u64::MAX`
(`crates/kovanica-types/src/lib.rs`) when those blocks were produced, while
`snapshot_committed_root()` always recomputes the full-state root that
includes the reserved `AUX_STATE_LEAF_ADDRESS` leaf. The two can never be
equal, so `import_verified_snapshot()` fails closed with:

```
rejected state snapshot ... sealed header state_root ... != full-state
commitment ... recomputed from the snapshot accounts, contracts and staking
```

That check must not be weakened: without it a peer could forge staking and
contract rows while every signature still verifies. Instead it is replaced by
an explicit activation and a clear refusal reason:

- `STATE_COMMITMENT_ACTIVE_FROM_ROUND` is now **500 000** (about five days of
  committed leaders at 12 s each). Every sealed header produced at or above
  that leader round commits to the full state, so snapshots from those heights
  import normally.
- Below it, the importer refuses the snapshot up front with
  `... predates the full-state commitment (active from round 500000, chain 4)`
  instead of a root mismatch. Nothing is applied.

**Before round 500 000 every validator must run a build that contains the
activation constant** (v0.2.0-node or later). Validators that sign a header at
or above the activation round with an older build would seal an account-only
root that newer nodes reject; a mixed fleet would therefore stall state sync
again. Roll the binary out with the usual canary-first procedure in
`docs/VALIDATOR-ONBOARDING.md`; each node logs the activation round at startup
(`state_commitment_active_from_round=...`), so check that line on every node
before the round arrives.

Snapshots taken below the activation round remain valid for peers that already
hold the state: the refusal only affects *imports*, so a node that was running
before the activation keeps producing blocks. A brand-new node can join once a
sealed header at or above the activation round exists (and the serving peer's
snapshot is at least that new).

## Validators

The installer never generates or handles validator keys — it produces a
read-only full node. Validator setup (keystore, stake, `run_validator = true`,
`validator_key`) is documented in `docs/VALIDATOR-ONBOARDING.md`.

## Cutting a release

```sh
git tag v0.2.0-node
git push origin v0.2.0-node
```

The workflow builds both architectures, runs the tag check against
`testnet/chain4/`, and publishes the tarballs, the bundle and `SHA256SUMS`.
Bump the default `VERSION` in `node/scripts/install.sh` when the download
should move to the new release.

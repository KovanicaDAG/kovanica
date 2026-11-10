# Running a Kovanica node

Prebuilt `kovanica-node` binaries for Linux (x86_64, aarch64) and a genesis
bundle per network are published as GitHub releases by
`.github/workflows/release-node.yml`. Tag naming: `v<version>-node`.

One binary serves every network; a network is chosen by its `chain_id` and its
genesis bundle, not by a separate build.

| `--chain-id` | Network                     | Registry name       |
|--------------|-----------------------------|---------------------|
| `1`          | mainnet                     | `MAINNET`           |
| `2`          | hybrid public testnet       | `TESTNET` (default) |
| `3`          | devnet                      | `DEVNET`            |
| `1337`       | local devnet (single node)  | `LOCAL`             |

Chain 4 (`TESTNET_V2`) is **retired** and no longer registered: a binary built
from current `main` refuses to start on it. See
`docs/dev/CANONICAL-TESTNET-2.md`.

## Install (public testnet, chain 2)

```sh
curl -fsSL https://kovanica.online/download/install.sh | sh
```

The installer:

1. detects the architecture and downloads
   `kovanica-node-<arch>-linux.tar.gz` plus the network's genesis bundle
   (`kovanica-chain<chain-id>-genesis.tar.gz` by default), verifying both
   against the published `SHA256SUMS`;
2. installs the binary (default `/usr/local/bin/kovanica-node`);
3. copies the genesis bundle into the data directory (default
   `/var/lib/kovanica`), refusing a data directory that already holds chain
   data;
4. writes a non-validating config (`/etc/kovanica/kovanica.toml`) with the
   current seed nodes as bootnodes and RPC bound to loopback;
5. runs `kovanica-node --check-config` and `kovanica-node network-tag` against
   that candidate config and **refuses to continue** on a mismatch, before the
   config is moved into place;
6. on systemd hosts, installs and starts `kovanica-node.service` as the
   `kovanica` system user.

Useful options: `--chain-id <id>`, `--bundle-asset <name>`,
`--expected-tag <tag>`, `--no-service`, `--data-dir <dir>`, `--prefix <dir>`,
`--version <v>`, `--skip-tag-check`, `--uninstall`, `--help`.
Without root it installs under `$HOME/.local` and `$HOME/.kovanica` and does
not touch systemd.

## Why the tag check matters

Every node on a network must share the same chain id and genesis file set; the
network tag (`<chain_id>-<16 hex of the genesis config digest>`) is mixed into
all libp2p protocol ids and gossip topics. A node with a different tag silently
has no peers. See `docs/TESTNET-RESET.md`.

The installer takes the expected tag from, in order:

1. `--expected-tag` (or `KOVANICA_EXPECTED_TAG`);
2. an `expected-tag` file inside the checksum-verified bundle.

If neither exists it **fails closed** — an unpinned install cannot tell a
forked or wrong-network bundle from the right one. `--skip-tag-check` is the
explicit opt-out; the node still prints its tag.

## Cutting a genesis bundle

Do not build the bundle by hand. The node binary writes the archive, the
checksums and the expected tag in one step:

```sh
kovanica-node genesis-bundle --chain-id 2 --source <genesis-dir>
```

`--source` may only contain genesis files (`genesis_validators.toml`,
`founder_premine.hex`, `funded_accounts.toml`, `validators.json`); anything
else — a signing key, a keystore, a config — is refused, so it cannot be
published by accident. The output
`kovanica-chain<chain-id>-genesis.tar.gz` (`--out` to change it) carries a
`SHA256SUMS` over the genesis files and an `expected-tag`, and is
byte-identical across rebuilds. Attach it to the `v<version>-node` release
next to the binaries. Details and the refusal rules: `docs/TESTNET-RESET.md`.

## State sync: full-state commitment is activated at round 500 000

A node installed by this script starts at height 0 with the network's genesis
file set and bootstraps from the chunked state snapshot served by its peers.
Sealed headers produced **below** `STATE_COMMITMENT_ACTIVE_FROM_ROUND` are
account-only, because the constant was `u64::MAX` when those blocks were
produced, while `snapshot_committed_root()` always recomputes the full-state
root that includes the reserved `AUX_STATE_LEAF_ADDRESS` leaf. The two can
never be equal, so `import_verified_snapshot()` fails closed with:

```
rejected state snapshot ... sealed header state_root ... != full-state
commitment ... recomputed from the snapshot accounts, contracts and staking
```

That check must not be weakened: without it a peer could forge staking and
contract rows while every signature still verifies. Instead it is replaced by
an explicit activation and a clear refusal reason:

- `STATE_COMMITMENT_ACTIVE_FROM_ROUND` is **500 000** (about five days of
  committed leaders at 12 s each). Every sealed header produced at or above
  that leader round commits to the full state, so snapshots from those heights
  import normally.
- Below it, the importer refuses the snapshot up front with
  `... predates the full-state commitment (active from round 500000, chain <id>)`
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

The workflow builds both architectures, runs the tag check against the bundle
source, and publishes the tarballs, the bundle and `SHA256SUMS`. The bundle is
expected to contain an `expected-tag` file (the installer reads it when no
`--expected-tag` is given). Bump the default `VERSION` in
`node/scripts/install.sh` when the download should move to the new release.

> **Known follow-up:** `.github/workflows/release-node.yml` still packages the
> retired chain-4 bundle (`testnet/chain4/`, tag `4-0eacd5699210b145`). It must
> be re-pointed at a chain-2 bundle cut from the live chain-2 genesis set
> before a release can serve chain 2; that needs the deployed validator keys,
> so it is an ops step, not an in-repo edit.

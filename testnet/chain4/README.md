# Chain-4 genesis bundle

These three files are the **exact** bytes the live public testnet (chain 4)
nodes hold in their data directory. They are hashed into the genesis config
digest, which becomes the network tag (`4-0eacd5699210b145`) carried by every
libp2p protocol id and gossip topic.

| File | sha256 |
|---|---|
| `genesis_validators.toml` | `7b9c9d3d63ac1c97456a8f4de9ede5739c91696a095a88c0c7f50fc466400c60` |
| `founder_premine.hex` | `485c1352b4cb10140030322b3750b96df91a0904cee5afd7f6e982de23b06238` |
| `funded_accounts.toml` | `ee120047d2bf35d2e45813929250c5439816c578d865061d699a6a0e5dd6aa84` |

Rules:

- **Do not edit, reformat or regenerate these files.** A single byte change
  gives every node using them a different tag and no peers.
- Do **not** add `validators.json` to the bundle: the live nodes do not have
  one, and its presence changes the digest too.
- The treasury is unset on chain 4, so the digest covers the all-zero
  treasury address. Do not set `treasury_address` / `KOVANICA_TREASURY_ADDRESS`
  on a node joining this mesh.

Verification (on any node, after copying the files into the data directory):

```sh
sha256sum -c <<'EOF'
7b9c9d3d63ac1c97456a8f4de9ede5739c91696a095a88c0c7f50fc466400c60  genesis_validators.toml
485c1352b4cb10140030322b3750b96df91a0904cee5afd7f6e982de23b06238  founder_premine.hex
ee120047d2bf35d2e45813929250c5439816c578d865061d699a6a0e5dd6aa84  funded_accounts.toml
EOF
kovanica-node network-tag --config kovanica.toml   # must print 4-0eacd5699210b145
```

`founder_premine.hex` and `funded_accounts.toml` are testnet-only conveniences
(founder premine address, 1 000 000 KUNA pre-funded for the faucet). Mainnet
(chain 1) forbids `funded_accounts.toml` entirely.

---
title: "Configuration, Feature Flags & Genesis"
category: 60-Planning
source: kovanica-poa-migration/06-CONFIG-AND-FLAGS.md
synced: 2026-09-26
---
# Configuration, Feature Flags & Genesis

## 1. Environment / Config flags

| Variable / Key              | Values              | Meaning |
|-----------------------------|---------------------|---------|
| `KOVANICA_CONSENSUS`        | `pow` / `poa`       | Active consensus mode |
| `KOVANICA_SLOT_DURATION`    | seconds (default 3) | Slot length under PoA |
| `KOVANICA_POW`              | 0 / 1               | Legacy – force old PoW path (for testing) |
| `KOVANICA_AUTHORITIES`      | comma-separated hex | Initial authority public keys (genesis) |
| `KOVANICA_AUTHORITY_THRESHOLD` | integer          | e.g. 3 for 3-of-4 |

## 2. Genesis changes

In the genesis configuration add:

```toml
[consensus]
mode = "poa"
slot_duration = 3

[authority]
# ordered list of Ed25519 public keys (hex)
keys = [
  "ed25519:abcd...",
  "ed25519:ef01...",
  "ed25519:2345...",
  "ed25519:6789..."
]
threshold = 3
```

The genesis block / state must create the first live Authority UTXO (or state object) containing the above set.

## 3. Feature flags in Cargo (optional but clean)

```toml
[features]
default = ["poa"]
poa = []
pow-vrf = []          # keep old code compilable
```

Then guard old mining paths with `#[cfg(feature = "pow-vrf")]`.

## 4. Runtime behaviour

```rust
match config.consensus_mode {
    ConsensusMode::Poa => {
        // use AuthoritySet + slot round-robin
        // skip PoW & VRF leader checks
    }
    ConsensusMode::PowVrf => {
        // original path
    }
}
```

This allows running both modes during development and easy A/B testing on private networks.
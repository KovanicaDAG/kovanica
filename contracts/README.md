# Contract artifacts

Committed WebAssembly built from this workspace, so a binary that ships the CLI
(`kovanica token create`) or an Android app can deploy a contract without a
checkout, a Rust toolchain, or network access.

| File | Source crate | sha256 |
|------|--------------|--------|
| `kovanica-token.wasm` | `crates/kovanica-token` | `a6988f5e1742b1ac0a069a3ef274d7590a54266dee57435438664e23036dc49b` |

## Rebuilding

```bash
rustup target add wasm32-unknown-unknown
./scripts/build-contracts.sh           # writes target/contracts/*.wasm
cp target/contracts/kovanica-token.wasm contracts/kovanica-token.wasm
sha256sum contracts/kovanica-token.wasm
```

`scripts/build-contracts.sh` builds twice and compares hashes, so the artifact
is reproducible.

## Copies that must stay identical

- `contracts/kovanica-token.wasm` — embedded by `kovanica-cli` (`include_bytes!`).
- `crates/kovanica-cli/fixtures/kovanica_token.wasm` — L1 pin of the bytes the apps ship.
- [KovanicaDAG/kovanica-apps](https://github.com/KovanicaDAG/kovanica-apps):
  `android-wallet/app/src/main/assets/kovanica_token.wasm` and
  `android/app/src/main/assets/kovanica_token.wasm`

The Android builds read their own asset, so they cannot point at this directory
directly. `crates/kovanica-cli/src/contracts.rs` asserts that the embedded
contract and the fixture are byte-identical
(`shipped_token_wasm_matches_the_repo_copies`), so a rebuild fails the CLI test
suite until the fixture — and the kovanica-apps assets — are updated with it.

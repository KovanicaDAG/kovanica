# Contract artifacts

Committed WebAssembly built from this workspace, so a binary that ships the CLI
(`kovanica token create`) or an Android app can deploy a contract without a
checkout, a Rust toolchain, or network access.

| File | Source crate | sha256 |
|------|--------------|--------|
| `kovanica-token.wasm` | `crates/kovanica-token` | `c3aa7ba49f1070e4cc00860c81b7a579df16a1d462acac5b7e24e6b4d7817adf` |

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
- `apps/android-wallet/app/src/main/assets/kovanica_token.wasm`
- `apps/android/app/src/main/assets/kovanica_token.wasm`

The Android builds read their own asset, so they cannot point at this directory
directly. `crates/kovanica-cli/src/contracts.rs` asserts that all three are
byte-identical (`shipped_token_wasm_matches_the_repo_copies`), which fails the
CLI test suite if a rebuild only updates one of them.

# Kovanica SDK — Release & Publish Runbook (S-12)

Goal: take the six `kovanica-sdk` crates (plus the WASM binding) from "works in
the monorepo" to "published and consumable by `cargo add` / `npm install`".

Status: **prepared, not executed.** Publishing is semi-irreversible and needs
crates.io / npm tokens — a human decision. This document is the go/no-go
checklist; everything below was verified against the working tree at commit
`7ddbcd7`.

---

## 1. Workspace facts (verified)

| Crate | Depends on | Publishes from |
|---|---|---|
| `kovanica-types` | — (registry only) | `sdk/crates/kovanica-types` |
| `kovanica-keys` | types | `sdk/crates/kovanica-keys` |
| `kovanica-tx` | types, keys | `sdk/crates/kovanica-tx` |
| `kovanica-rpc` | types, tx | `sdk/crates/kovanica-rpc` |
| `kovanica-fee` | types | `sdk/crates/kovanica-fee` |
| `kovanica-sdk` | all five | `sdk/crates/kovanica-sdk` |
| `kovanica-wasm` | sdk | `sdk/bindings/kovanica-wasm` |

Shared metadata (from `[workspace.package]`):

- version `0.1.0-alpha.1`, edition 2021, rust-version 1.75
- license `MIT OR Apache-2.0` (SPDX expression — no license file required)
- repository `https://github.com/KovanicaDAG/kovanica`
- readme: only `kovanica-sdk` sets `readme = "../../README.md"` (included in
  its package); the leaf crates publish without a README (allowed).

## 2. Why `cargo package` fails today (expected)

All internal deps are `path = …` + `version = "0.1.0-alpha.1"` workspace deps.
`cargo package`/`publish` verifies the package, which resolves internal deps
from **crates.io** — they don't exist there yet:

```
error: no matching package named `kovanica-types` found
```
…or `kovanica-keys`, `kovanica-tx` depending on which crate is being packaged.

This is **not** a defect: it enforces the publish order. Verified good on the
current tree:

```bash
cargo package -p kovanica-types --allow-dirty --list
# -> .cargo_vcs_info.json, Cargo.lock, Cargo.toml, Cargo.toml.orig,
#    src/lib.rs, tests/sighash_vector.rs   (packages cleanly)
```

After `kovanica-types` is live on crates.io, `kovanica-keys` packages; and so
on. The order is exactly the dependency table above.

## 3. Publish order + commands

Never publish without running the full gate first (§4). Then, strictly in
order — each step requires the previous one to be visible on crates.io:

```bash
cd sdk

cargo publish -p kovanica-types   # seed: breaks the path-dep chain
cargo publish -p kovanica-keys
cargo publish -p kovanica-tx
cargo publish -p kovanica-rpc
cargo publish -p kovanica-fee
cargo publish -p kovanica-sdk     # facade — do LAST
```

Per-crate dry run before the real one (no network upload):

```bash
cargo publish -p kovanica-sdk --dry-run --allow-dirty
```

> `--allow-dirty` is needed only while `plans/` or other untracked files sit in
> the workspace; prefer committing everything first (linear history, PR review)
> and publishing from a clean tree.

## 4. Go / no-go gate (run before every publish wave)

| # | Check | Command / evidence |
|---|---|---|
| 1 | Workspace tests | `cargo test --workspace` (48 passed on plain `main`; 55 with PR #17 KVP-102 tests) |
| 2 | Strict clippy | `cargo clippy --workspace --all-targets -- -D warnings` |
| 3 | Live suite | `cargo test -p kovanica-rpc --features live-testnet -- --nocapture` (6/6 vs api.kovanica.online) |
| 4 | Parity locks | `sdk/crates/kovanica-types/tests/sighash_vector.rs` + `kovanica-keys/tests/script_vectors.rs` ↔ node mirrors |
| 5 | `cargo package` dry | leaf crate at least once: `cargo package -p kovanica-types --allow-dirty` |
| 6 | Version bump agreed | `0.1.0-alpha.1` → next semver AFTER first publish (crates.io forbids re-publishing the same version) |
| 7 | README examples warning | publish prints `ignoring example ... not included` for `sdk/examples/*` — **intentional**: examples stay monorepo-only, docs link to them |

Manual gate (human, before wave):

- crates.io API tokens scoped to the `KovanicaDAG` owner live in the
  maintainer's secret store, not in the repo.
- Confirm git `main` has the merged, reviewed SDK (do not publish from a feature
  branch).
- Announce on the Kovanica channel before touching the facade crate.

## 5. `kovanica-wasm` (npm side)

The crate is publishable to crates.io too, but npm is the primary channel for
browser/Node users. Current state: **no `package.json`** in
`sdk/bindings/kovanica-wasm/` — this step is still work.

```bash
cd sdk/bindings/kovanica-wasm
wasm-pack build --target web --out-dir pkg        # generates pkg/*.js/.wasm/d.ts
cat > pkg/package.json <<'EOF'
{
  "name": "@kovanica/sdk-wasm",
  "version": "0.1.0-alpha.1",
  "description": "WASM bindings for kovanica-sdk (browser & Node)",
  "license": "MIT OR Apache-2.0",
  "repository": { "type": "git", "url": "https://github.com/KovanicaDAG/kovanica" },
  "files": ["*.js", "*.wasm", "*.d.ts"]
}
EOF
npm pack --dry-run                                  # inspect tarball contents
# real publish (human, token required):
# npm publish --access public
```

`wasm-pack` uses the crate name/version unless `--out-name`/package.json
override it; keep the versions in lockstep with the workspace.

## 6. Post-publish smoke

```bash
# from an empty temp dir (no workspace): each layer resolves from crates.io
cargo new smoke && cd smoke
cargo add kovanica-sdk
# Examples are NOT shipped with the published crate (monorepo-only by design),
# so drive the API directly:
cat > src/main.rs <<'EOF'
use kovanica_sdk::prelude::*;
fn main() {
    let kp = Keypair::from_secret_bytes([1; 32]);
    println!("{}", kp.address().to_hex());
}
EOF
cargo run
```

[NPM] consume the wasm tarball from `npm pack` output in a vite app and check
`window`-less Node import resolves.

## 7. Rollback policy (there is no true rollback)

- crates.io: `cargo yank --version <ver>` hides the version; users on the yanked
  version keep their lockfiles. Yank only for build-breakage, never for
  "re-upload a fixed 0.1.0-alpha.1" — publish a new patch instead.
- npm: `npm unpublish` only inside first 72h and only with `--force`; prefer
  `npm deprecate` + a fixed release.
- Fix-forward rule: semver patch/minor for fixes; `0.x` allows breaking minor
  bumps, but the SDK address/derivation formats are locked before mainnet
  (see README "Versioning").

---

*This runbook is a guide for maintainers; tokens and the final decision to
publish stay with humans.*
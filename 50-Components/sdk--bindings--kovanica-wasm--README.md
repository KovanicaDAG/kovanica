---
title: "kovanica-wasm → npm"
category: 50-Components
source: sdk/bindings/kovanica-wasm/README.md
synced: 2026-09-26
---
# kovanica-wasm → npm

WASM bindings for `kovanica-sdk` (browser & Node, ESM only). The Rust surface
is `src/lib.rs`: `generate_mnemonic(words)`, `address_from_mnemonic(phrase)`,
`version()`.

## Build

```bash
cd sdk
wasm-pack build bindings/kovanica-wasm --target web --out-dir pkg   # wasm-pack 0.15: --out-dir je relativan na crate dir
```

`pkg/` is gitignored (build output). wasm-pack generates the JS glue as
`kovanica_wasm.js` (crate name `kovanica-wasm`, `-` → `_`), the Wasm binary as
`kovanica_wasm_bg.wasm`, and `kovanica_wasm.d.ts`.

## Publish manifest

The **source of truth** for the npm package is the repo-level `package.json`
(name `@kovanica/sdk-wasm`, version locked to the workspace
`0.1.0-alpha.1`, `"type": "module"`, entry `kovanica_wasm.js`).

wasm-pack also writes its own `pkg/package.json` (crate-name based); before
packing, copy the repo manifest over it so the published name/entry points are
the SDK's, not the crate's:

```bash
cd sdk/bindings/kovanica-wasm
cp package.json pkg/package.json
```

## Gate & publish (human, token required)

```bash
cd sdk/bindings/kovanica-wasm
npm pack --dry-run   # inspect tarball: *.js, *.wasm, *.d.ts, package.json
# real publish:
# npm publish --access public --tag alpha   # scoped @kovanica/* needs --access public; pre-release → dist-tag alpha
```

Keep the version in `package.json` in lockstep with the workspace
(`sdk/Cargo.toml` `[workspace.package] version`); both must be the same
pre-release tag (e.g. `0.1.0-alpha.1`):

Edge cases:

- `--target web` output is ESM; `"type": "module"` in the manifest makes Node
  `import "@kovanica/sdk-wasm"` resolve it. CommonJS `require()` is **not** a
  supported entry (use the browser/wallet packages for CJS builds).
- If the glue file names ever differ (e.g. `--out-name` is used), update
  `module`/`types`/`exports` in `package.json` — the `files` globs
  (`*.js`, `*.wasm`, `*.d.ts`) already pick up any names.
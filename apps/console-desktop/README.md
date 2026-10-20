# Kovanica Console

Desktop app for Linux, Windows, and macOS. Version 0.1.0 shows whether a node
is reachable and what chain it sealed.

`apps/desktop` is the Comms client. This app lives beside it and does not
replace it.

```
apps/console-desktop/
├── src/                  connection screen (plain TypeScript)
└── src-tauri/            Tauri v2 shell
```

## Connection

The endpoint comes from a setting saved in the app data directory, then from
`KOVANICA_RPC_URL`, then from `https://rpc-testnet.kovanica.online/rpc`.

The screen shows chain id, committed height, leader round, epoch, and peers.
Chain id is the registered id whose commit seals verify through
`kovanica-light-ffi`. A node that cannot be reached shows that error and
clears the previous status. The shell has no shell plugin and no filesystem
plugin. RPC calls stay in the Rust process.

## Run

```sh
cd apps/console-desktop
npm install
npm test
npm run tauri dev
```

A Linux bundle:

```sh
npm run tauri build
```

## Checks

```sh
npm run lint
npm test
cd src-tauri && cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test
```

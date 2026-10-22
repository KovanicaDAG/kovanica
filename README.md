# kovanica-web

Source of the public site **kovanica.online** (TanStack Start + Vite + Nitro 3).

## Develop

```sh
npm ci
npm run dev
```

## Build

```sh
npm run build:vps   # Nitro node-server preset, output in .output/
```

`@kovanica/sdk-wasm` is vendored as a prebuilt wasm-pack package in
`vendor/kovanica-sdk-wasm` (built from the Kovanica SDK `kovanica-wasm` bindings).

Runtime configuration comes from environment variables (`KOVANICA_*`, `GROK_AUTH_*`,
`GROK_PREVIEW_CLIENT_SECRET`); never commit hosts, IPs, ports, keys or secrets.

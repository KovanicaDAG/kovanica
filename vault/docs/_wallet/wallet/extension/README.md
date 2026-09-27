# Kovanica Wallet — Browser Extension

> **Vite + React + TypeScript** browser extension for the Kovanica testnet. Pure API-backed — no light node, no FFI. Connects to a remote Kovanica node over HTTP.

---

## Features (v1)

- ✅ Create / import wallet (BIP-39 mnemonic, encrypted storage)
- ✅ Display `kvnc…` address with QR code + copy
- ✅ Balance display (KVNC + atoms)
- ✅ Transaction history (paginated)
- ✅ Send KVNC (offline Ed25519 signing via `/api/prepare` → sign → `/api/submit`)
- ✅ Faucet request (1 testnet KVNC)
- ✅ Multi-asset support (KVP-102 native tokens)
- ✅ Configurable RPC endpoint (defaults to `https://explorer.kovanica.online`)

---

## Quick Start

```bash
# Install dependencies
npm ci

# Development (HMR)
npm run dev

# Production build
npm run build

# Load in browser
# Chrome/Edge: chrome://extensions → Developer mode → Load unpacked → dist/
# Firefox: about:debugging → This Firefox → Load Temporary Add-on → dist/manifest.json
```

---

## Build Output

```
extension/
├── dist/                    # Production build (manifest.json, JS, CSS, assets)
│   ├── manifest.json        # Manifest V3
│   ├── popup.html           # Extension popup
│   ├── background.js        # Service worker
│   └── assets/              # Chunked JS/CSS
├── public/                  # Static assets (copied to dist/)
│   ├── manifest.json        # Source manifest
│   ├── kvnc-logo.png        # Token icon
│   ├── favicon.svg
│   └── icons.svg
└── src/                     # Source code
    ├── main.tsx             # Entry point
    ├── App.tsx              # Root component
    ├── components/          # React components
    ├── utils/               # RPC, seed phrase handling
    └── assets/              # Images, BIP-39 wordlists
```

---

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| **RPC URL** | `https://explorer.kovanica.online` | Kovanica node HTTP API endpoint |
| **Network** | `kovanica-testnet` | Network identifier |
| **Fee Estimate** | Auto (mempool p90) | Atoms/byte |

Settings persisted in `chrome.storage.local` (sync across devices if signed in).

---

## Security Model

- **Mnemonic never leaves the extension** — encrypted with user password via Web Crypto API (PBKDF2 + AES-GCM)
- **Signing is offline** — extension fetches sighash from `/api/prepare`, signs locally with Ed25519, submits signature via `/api/submit`
- **Node never sees private keys** — only verifies signatures
- **No analytics, no tracking** — pure client-side

---

## API Endpoints Used

| Endpoint | Purpose |
|----------|---------|
| `GET /api/head` | Chain tip, network, genesis, min fee |
| `GET /api/bootstrap` | Network params (k, subsidy, token, max_supply) |
| `GET /api/fee_estimate` | Current fee rate (atoms/byte) |
| `GET /api/utxos?address=` | Spendable UTXOs for address |
| `POST /api/prepare` | Build unsigned tx, return sighash |
| `POST /api/submit` | Submit signed tx (64-byte Ed25519 signature) |
| `GET /api/history?address=` | Transaction history |
| `POST /api/faucet` | Request testnet KVNC |

---

## Development

```bash
# Type-check
npm run typecheck

# Lint
npm run lint

# Format
npm run format

# Test
npm run test
```

---

## Manifest V3 Notes

- **Service worker** (`background.js`) handles RPC caching, badge updates
- **Popup** (`popup.html`) — main UI, mounts React app
- **Permissions**: `storage`, `activeTab`, `host_permissions` for configured RPC host
- **CSP**: `script-src 'self' 'wasm-unsafe-eval'` (for WASM crypto if used)

---

## Related

| Repo | Purpose |
|------|---------|
| [kovanica-wallet](https://github.com/KovanicaDAG/kovanica-wallet) | Parent repo (Android/iOS/Extension) |
| [kovanica-sdk](https://github.com/KovanicaDAG/kovanica-sdk) | Rust/WASM SDK (types, keys, tx builders) |
| [kovanica-node](https://github.com/KovanicaDAG/kovanica-node) | Node binary (serves API) |
| [kovanica-web](https://github.com/KovanicaDAG/kovanica-web) | Web wallet (same features, no install) |

---

## License

**MIT OR Apache-2.0**
# kovanica-wallet

> **Mobile wallet apps + browser extension** — Pure API-backed wallets for the Kovanica testnet. No light node, no SPV sync, no FFI/native bindings. They trust a remote Kovanica node's REST API.

---

## Repository Structure

```
kovanica-wallet/
├── android/       # Android wallet (Kotlin + Jetpack Compose, Material3)
├── ios/           # iOS wallet (Swift + SwiftUI)
├── extension/     # Browser extension (Vite + React + TypeScript)
└── shared/        # Shared assets (kvnc-logo.png)
```

---

## Platform Status

| Platform | Stack | Path | Build | CI |
|----------|-------|------|-------|----|
| **Android** | Kotlin + Jetpack Compose (Material3) | `android/` | `./gradlew :app:assembleDebug` | ✅ `.github/workflows/wallet.yml` |
| **iOS** | Swift + SwiftUI | `ios/` | `xcodebuild` (needs macOS) | ✅ Exports unsigned `.ipa` for sideloading |
| **Browser Extension** | Vite + React + TypeScript | `extension/` | `npm run build` | ✅ |

---

## Features (v1)

- ✅ Show KVNC logo, wallet address (`kvnc…`), formatted balance, network badge
- ✅ **Receive**: Display full `kvnc…` address with Copy button
- ✅ **History**: Paginated transactions from `/api/history`
- ✅ **Faucet**: Request 1 testnet KVNC via `POST /api/faucet`
- ✅ Poll `/api/head` for current chain tip / last block
- ❌ **Send / Sign** — *intentionally not implemented in v1* (requires Ed25519 + sighash crypto binding; planned follow-up)

> The user enters a `kvnc…` address to watch (receive/monitoring wallet in v1). No private keys on device.

---

## Node API Contract (Verified Live)

Default node: `https://explorer.kovanica.online` (user-configurable)

| Endpoint | Method | Response |
|----------|--------|----------|
| `/api/head` | GET | `{network, genesis, tip, blocks, min_fee, atom}` |
| `/api/bootstrap` | GET | Token params, k, subsidy, founder, depths, consensus |
| `/api/fee_estimate` | GET | `{fee_rate, unit, mempool, bytes}` |
| `/api/state` | GET | Node/network state |
| `/api/address/<addr>` | GET | `{address, balance (atoms), tx_count}` |
| `/api/utxos` | GET | UTXO list (query: `address`, `limit`) |
| `/api/history` | GET | Paginated history (query: `address`, `limit`, `offset`) |
| `/api/blocks` | GET | Block list (query: `from`) |
| `/api/faucet` | POST | Body: `{"address":"kvnc…"}` → tx receipt |

**Token Constants**: Symbol **KVNC**, Name **Kovanica (KVNC)**, **8 decimals**, `1 KVNC = 100,000,000 atoms`

---

## Quick Start

### Android

```bash
cd android
chmod +x gradlew
./gradlew :app:assembleDebug
# Install: ./gradlew :app:installDebug (device/emulator required)
```

### iOS

```bash
# Requires macOS + Xcode 15+
open KovanicaWallet.xcodeproj
# Select iOS 16+ simulator/device → Cmd+R
```

**No Mac?** CI builds an unsigned `.ipa` for sideloading via AltStore/Sideloadly on any PC. See [ios/README.md](ios/README.md#install-on-a-real-iphone--no-mac-needed).

### Browser Extension

```bash
cd extension
npm ci
npm run dev      # Development (load unpacked in browser)
npm run build    # Production build
```

---

## Architecture

- **Pure HTTP clients** — URLSession (iOS), OkHttp/Retrofit (Android), Fetch API (Extension)
- **No crypto on device** — v1 is read-only; signing requires Ed25519 binding (planned)
- **Shared branding** — Only `shared/kvnc-logo.png` used as token/launcher icon
- **Configurable node URL** — Defaults to `https://explorer.kovanica.online`

---

## Development Notes

- **Android**: Min SDK 24, Material3, Kotlin Coroutines + Flow
- **iOS**: Swift Concurrency (`async/await`), `@MainActor` ViewModels, zero external deps
- **Extension**: React 18, Vite, TypeScript, Oxlint

---

## Related Repositories

| Repo | Purpose |
|------|---------|
| [kovanica-protocol](https://github.com/KovanicaDAG/kovanica-protocol) | Core consensus + ledger |
| [kovanica-node](https://github.com/KovanicaDAG/kovanica-node) | Node binary (serves API) |
| [kovanica-web](https://github.com/KovanicaDAG/kovanica-web) | Web wallet/explorer/map |
| [kovanica-mobile](https://github.com/KovanicaDAG/kovanica-mobile) | Light-node mobile clients (FFI-based) |
| [android-light-node](https://github.com/KovanicaDAG/android-light-node) | Actively developed Android light node |
| [kovanica-sdk](https://github.com/KovanicaDAG/kovanica-sdk) | Rust/WASM SDK (for future signing support) |

---

## License

**MIT OR Apache-2.0**
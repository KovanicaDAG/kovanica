# Kovanica Wallet (Unified)

> **Standalone Kovanica Wallet app for Android and iOS** — Pure API-backed wallet reading a remote Kovanica node over HTTP. **No light node, no SPV sync, no FFI/native bindings.**
>
> Only the canonical `shared/kvnc-logo.png` is used as the token/launcher icon. No other image assets.

---

## Platforms

| Platform | Stack | Path | Build |
|----------|-------|------|-------|
| **Android** | Kotlin + Jetpack Compose (Material3) | `android/` | `./gradlew -p android :app:assembleDebug` (wired into `.github/workflows/wallet.yml`) |
| **iOS** | Swift + SwiftUI | `ios/` | `xcodebuild -project KovanicaWallet.xcodeproj -scheme KovanicaWallet` (needs macOS/Xcode). CI exports **unsigned, sideloadable `.ipa`** — see [ios/README.md](ios/README.md#install-on-a-real-iphone--no-mac-needed) |

---

## What It Does (v1)

- ✅ Show KVNC logo, wallet address (`kvnc…`), formatted balance, KVNC · network badge
- ✅ **Receive**: Display full `kvnc…` watch address with Copy button + QR code
- ✅ **History**: Paginated transactions from `/api/history`
- ✅ **Faucet**: Request 1 testnet KVNC via `POST /api/faucet`
- ✅ Poll `/api/head` for current chain tip / last block
- ❌ **Send / Sign** — *not implemented in v1* (requires Ed25519 + sighash crypto binding; planned follow-up)
- ❌ **Light node / SPV sync** — this app never downloads or verifies the chain locally; it trusts the configured node's REST API

The user enters a `kvnc…` address to watch (receive/monitoring wallet in v1).

---

## What It Deliberately Does NOT Do (v1)

| Feature | Reason |
|---------|--------|
| Send / Sign | Requires crypto binding (Ed25519 + Kovanica sighash) — follow-up |
| Light node / SPV | Trusts remote node API; no local verification |
| Multi-asset balances | API returns native-only in v1; KVP-102 support planned |

---

## Node API Contract (Verified Live)

Default node: `https://explorer.kovanica.online` (user-settable)

| Endpoint | Method | Response |
|----------|--------|----------|
| `/api/head` | GET | `{network, genesis, tip, blocks, min_fee, atom}` |
| `/api/bootstrap` | GET | `{network, ..., token:"KVNC", k, subsidy, founder_amount, ...}` |
| `/api/fee_estimate` | GET | `{fee_rate, unit, mempool, bytes}` |
| `/api/state` | GET | Node/network state |
| `/api/address/<addr>` | GET | `{address, balance (atoms), tx_count}` |
| `/api/utxos` | GET | UTXO list (query: `address`, `limit`) |
| `/api/history` | GET | Paginated history (query: `address`, `limit`, `offset`) |
| `/api/blocks` | GET | Block list (query: `from`) |
| `/api/faucet` | POST | Body: `{"address":"kvnc…"}` → tx receipt |

**Token Facts**: Symbol **KVNC**, Name **Kovanica (KVNC)**, **8 decimals**, `1 KVNC = 100,000,000 atoms`

---

## Architecture

- **Android**: Kotlin + Jetpack Compose (Material3), OkHttp/Retrofit/Moshi, Coroutines/Flow
- **iOS**: Swift + SwiftUI, URLSession, Combine/Swift Concurrency, **zero external dependencies**
- **Shared**: Only `shared/kvnc-logo.png` for branding

---

## Quick Start

### Android

```bash
cd android
chmod +x gradlew
./gradlew :app:assembleDebug
./gradlew :app:installDebug  # Requires device/emulator
```

### iOS

```bash
# Requires macOS + Xcode 15+
open KovanicaWallet.xcodeproj
# Select iOS 16+ simulator/device → Cmd+R
```

**No Mac?** Download unsigned `.ipa` from GitHub Actions → sideload via AltStore/Sideloadly on any PC. See [ios/README.md](ios/README.md#install-on-a-real-iphone--no-mac-needed).

---

## Related Repositories

| Repo | Purpose |
|------|---------|
| [kovanica-protocol](https://github.com/KovanicaDAG/kovanica-protocol) | Core consensus + ledger |
| [kovanica-node](https://github.com/KovanicaDAG/kovanica-node) | Node binary (serves API) |
| [kovanica-web](https://github.com/KovanicaDAG/kovanica-web) | Web wallet/explorer/map |
| [kovanica-mobile](https://github.com/KovanicaDAG/kovanica-mobile) | Light-node mobile clients (FFI-based) |
| [android-light-node](https://github.com/KovanicaDAG/android-light-node) | Actively developed Android light node |
| [kovanica-sdk](https://github.com/KovanicaDAG/kovanica-sdk) | Rust/WASM SDK (for future signing) |

---

## License

**MIT OR Apache-2.0**
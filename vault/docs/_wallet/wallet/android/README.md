# Kovanica Wallet (Android)

> **Standalone Android KVNC wallet** — Read + receive + faucet wallet that talks to a remote Kovanica node over HTTP. Built with **Kotlin + Jetpack Compose (Material3)**.
>
> **Pure API-backed** — Does **NOT** run a light node, does **NOT** do SPV sync, uses **no FFI** (`LightNode`, `uniffi.kovanica`, or any native `.so` libs). Only reads node state through the public REST API.

---

## Features (v1)

- ✅ Shows KVNC logo, wallet address (`kvnc…`), formatted balance, network/token badge
- ✅ **Receive screen**: Full `kvnc…` address with **Copy** button + QR code; explains this is a receive-only watch wallet (no private key on device in v1)
- ✅ **History screen**: Paginated list from `/api/history` — each row with kind label, signed formatted amount, tx hex prefix
- ✅ **Faucet button**: Requests testnet KVNC via `POST /api/faucet`
- ✅ **Polls `/api/head`** — shows current chain tip / last block on home screen
- ❌ **Sending / signing is intentionally NOT implemented** — building and signing a transaction requires a crypto binding (Ed25519 keys + Kovanica sighash). This is a documented follow-up. The Send button is disabled/omitted by design.

---

## API Contract (Verified Live)

Base URL configurable. Default: `KOVANICA_API_DEFAULT = "https://explorer.kovanica.online"`

| Endpoint | Method | Notes |
|----------|--------|-------|
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

## Build

```bash
cd android
chmod +x gradlew
./gradlew :app:assembleDebug

# Install on connected device/emulator
./gradlew :app:installDebug
```

> **Note**: Building requires Android SDK on the machine. CI verifies via self-review; the project is wired into `.github/workflows/wallet.yml`.

---

## Project Structure

```
android/
├── app/
│   ├── src/main/
│   │   ├── java/com/kovanica/wallet/
│   │   │   ├── MainActivity.kt
│   │   │   ├── data/
│   │   │   │   ├── SeedStore.kt          # SharedPreferences (v1 plain; encrypted v1.1)
│   │   │   │   └── WalletRepository.kt   # API data layer
│   │   │   ├── ui/
│   │   │   │   ├── HomeScreen.kt
│   │   │   │   ├── ReceiveScreen.kt
│   │   │   │   ├── HistoryScreen.kt
│   │   │   │   └── theme/                # Material3 + KVNCBrand colors
│   │   │   └── viewmodel/
│   │   │       └── WalletViewModel.kt
│   │   └── res/
│   │       └── values/strings.xml
│   ├── build.gradle.kts
│   └── proguard-rules.pro
├── gradle/
│   ├── libs.versions.toml               # Dependency versions
│   └── wrapper/gradle-wrapper.properties
├── settings.gradle.kts
├── build.gradle.kts
└── gradle.properties
```

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| Language | Kotlin |
| UI | Jetpack Compose (Material3) |
| Networking | OkHttp + Retrofit + Moshi |
| Async | Kotlin Coroutines + Flow |
| DI | Manual (no Hilt/Koin in v1) |
| Min SDK | 24 (Android 7.0) |

---

## Signing Follow-up (v1.1)

A future v1.1 will add send/sign backed by:
- **BIP-39 mnemonic** → seed → Ed25519 keypair → Kovanica address + sighash
- **Encrypted seed storage** (Android Keystore + AES-GCM)
- **Offline signing**: `/api/prepare` → local Ed25519 sign → `/api/submit`

Watch address display already supported via `SeedStore`.

---

## Related Repositories

| Repo | Purpose |
|------|---------|
| [kovanica-wallet](https://github.com/KovanicaDAG/kovanica-wallet) | Parent repo (iOS/Extension) |
| [kovanica-protocol](https://github.com/KovanicaDAG/kovanica-protocol) | Core consensus + ledger |
| [kovanica-node](https://github.com/KovanicaDAG/kovanica-node) | Node binary (serves API) |
| [kovanica-sdk](https://github.com/KovanicaDAG/kovanica-sdk) | Rust/WASM SDK (for future signing) |
| [android-light-node](https://github.com/KovanicaDAG/android-light-node) | Light-node app (FFI-based, actively developed) |

---

## License

**MIT OR Apache-2.0**
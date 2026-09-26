# Kovanica Wallet (iOS)

> **API-backed wallet for Kovanica testnet** — Pure SwiftUI + URLSession. Connects to a remote Kovanica node over HTTP. No light node, no SPV sync, no FFI/UniFFI bindings.

---

## Features (v1)

- ✅ View wallet balance (atoms + KVNC formatted to 8 decimals)
- ✅ Receive address display + copy to clipboard + QR code
- ✅ Faucet: request 1 testnet KVNC
- ✅ Transaction history with pagination
- ✅ Configurable node URL
- ❌ **Send / Sign** — *coming soon* (requires Ed25519 + sighash crypto binding)

---

## Architecture

| Layer | Technology |
|-------|------------|
| Networking | Pure `URLSession` — REST API calls |
| UI | SwiftUI + Combine (dark-themed native) |
| Concurrency | Swift Concurrency (`async/await`, `@MainActor` ViewModel) |
| Dependencies | **Zero external deps** — no CocoaPods, SPM, or Carthage |

---

## Build & Run

```bash
# Requires macOS + Xcode 15+
open KovanicaWallet.xcodeproj
# Select iOS 16+ simulator or device → Cmd+R
```

---

## Install on Real iPhone — No Mac Needed

iOS apps can only be **built** on macOS, but you **don't need a Mac to install** on your phone:

1. **Get the `.ipa`** — In GitHub Actions, run the **"kovanica wallet"** workflow (on `main`), then open the **"Export iOS wallet .ipa (unsigned, sideloadable)"** job and download the `kovanica-wallet-ios-ipa` artifact (`KovanicaWallet-ios-unsigned.ipa`, ~3.6 MB).

2. **Install a sideload tool on your PC** — **AltStore** (altstore.io) or **Sideloadly** (sideloadly.io).

3. **Sign in** with your Apple ID, connect iPhone via USB, **trust** the device when prompted.

4. **Drag the `.ipa`** onto the tool. It installs and re-signs with your Apple ID.

5. **On iPhone**: Settings → General → VPN & Device Management → tap your Apple ID profile → **Trust**, then open Kovanica Wallet.

> **Free Apple ID limits**: Sideloads expire after **7 days** (re-trust/reinstall weekly); covers ~3 devices. Paid Apple Developer account removes expiry and device limits.

---

## Project Structure

```
KovanicaWallet.xcodeproj/
KovanicaWallet/
├── KovanicaWalletApp.swift        # @main entry point
├── ContentView.swift               # Navigation root, address entry
├── Models/
│   └── WalletModels.swift         # Codable models, formatting
├── Networking/
│   ├── KovAPIClient.swift         # URLSession HTTP client
│   └── WalletRepository.swift     # Data layer
├── ViewModels/
│   └── WalletViewModel.swift      # @MainActor state
├── Views/
│   ├── HomeView.swift             # Balance, actions, faucet
│   ├── ReceiveView.swift          # Address display + copy + QR
│   ├── HistoryView.swift          # Paginated tx list
│   └── Theme.swift                # KVNCBrand color palette
├── Info.plist
└── Assets.xcassets/               # App icon (kvnc-logo.png), accent color
```

---

## API Endpoints Used

Default node: `https://explorer.kovanica.online` (settable in-app)

| Endpoint | Purpose |
|----------|---------|
| `GET /api/head` | Chain tip, network, genesis, min fee |
| `GET /api/bootstrap` | Network params (k, subsidy, token, max_supply) |
| `GET /api/state` | Node/network state |
| `GET /api/address/<addr>` | Balance + tx count |
| `GET /api/utxos` | UTXO list (for future send) |
| `GET /api/history` | Paginated transaction history |
| `GET /api/fee_estimate` | Current fee rate |
| `POST /api/faucet` | Request 1 testnet KVNC |

---

## Branding

| Role | Color |
|------|-------|
| Background | `#09090B` |
| Foreground | `#D8D4CC` |
| Accent Gold | `#C9A227` |
| App Icon | `kvnc-logo.png` (1024×1024, universal iOS) |

---

## Related Repositories

| Repo | Purpose |
|------|---------|
| [kovanica-wallet](https://github.com/KovanicaDAG/kovanica-wallet) | Parent repo (Android/Extension) |
| [kovanica-protocol](https://github.com/KovanicaDAG/kovanica-protocol) | Core consensus + ledger |
| [kovanica-node](https://github.com/KovanicaDAG/kovanica-node) | Node binary (serves API) |
| [kovanica-sdk](https://github.com/KovanicaDAG/kovanica-sdk) | Rust/WASM SDK (for future signing) |

---

## License

**MIT OR Apache-2.0**
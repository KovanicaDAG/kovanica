# Kovanica Mobile Apps v0.1.0 — Vault Documentation Update

**Date**: 2026-09-30
**Tag**: `v0.1.0-mobile`
**Commit**: `571e24e`

---

## Build Artifacts

### Android Wallet
| Artifact | Path | Size |
|----------|------|------|
| Debug APK | `mobile/android/app/build/outputs/apk/debug/app-debug.apk` | 24 MB |
| Release APK (signed) | `mobile/android/app/build/outputs/apk/release/app-release.apk` | 18 MB |

**Signing Certificate**:
- CN=Kovanica, OU=Wallet, O=Kovanica Protocol
- Keystore: `mobile/android/keystore/release.keystore` (alias: `kovanica`)

**Key Changes**:
- Replaced missing JitPack dependency `io.github.tony19:bip39-android:1.0.1` with internal pure-Kotlin BIP-39 implementation (`com.kovanica.lightnode.ui.util.Bip39`) using BouncyCastle
- Fixed WalletSetupScreen Compose imports and val reassignment bug
- Added release signing config to `build.gradle.kts`

---

### Linux Tauri (Kovanica Console)
| Artifact | Path | Size |
|----------|------|------|
| AppImage | `mobile/console/kovanica/src-tauri/target/release/bundle/appimage/kovanica-console_0.1.0_amd64.AppImage` | 83 MB |
| .deb | `mobile/console/kovanica/src-tauri/target/release/bundle/deb/kovanica-console_0.1.0_amd64.deb` | 2.5 MB |
| .rpm | `mobile/console/kovanica/src-tauri/target/release/bundle/rpm/kovanica-console-0.1.0-1.x86_64.rpm` | 2.5 MB |

**Routes (9)**: Explorer, Blocks, Transactions, Peers, Authorities, Wallet, Multisig, HTLC, Developer
**Production Hardening**: CSP, FS scope, HTTP scope, NSIS/Deb/AppImage config, macOS entitlements, updater endpoints

---

### Windows Tauri (Kovanica Console)
| Artifact | Path | Size |
|----------|------|------|
| NSIS Installer | `mobile/console/kovanica/src-tauri/target/x86_64-pc-windows-gnu/release/bundle/nsis/Kovanica Console_0.1.0_x64-setup.exe` | 3.4 MB |

*Cross-compiled from Linux via mingw-w64*
**Production Hardening**: NSIS config with license, icons, CSP, FS scope, HTTP scope, updater endpoints

---

### Web Consoles (pm2 Deploy Ready)

#### Kovanica Console
- **Build Output**: `mobile/console/kovanica/dist/`
- **pm2 Config**: `mobile/console/kovanica/ecosystem.config.js`
- **Port**: 3000
- **Routes (9)**: Explorer, Blocks, Transactions, Peers, Authorities, Wallet, Multisig, HTLC, Developer

#### Enterprise Console
- **Build Output**: `mobile/console/enterprise/dist/`
- **pm2 Config**: `mobile/console/enterprise/ecosystem.config.js`
- **Port**: 3001
- **Routes (8)**: Dashboard, Wallets, Assets, Transactions, API Keys, Webhooks, Reports, Settings

---

### iOS Wallet (SwiftUI Skeleton)
- **Source**: `mobile/ios/KovanicaWallet/` (10 Swift files)
- **Screens (8)**: ContentView, HomeView, SendView, ReceiveView, WalletSetupView, SettingsView, Theme, Models, WalletViewModel, KovanicaWalletApp
- **Build Config**: `mobile/ios/Package.swift` (SPM), `mobile/ios/xcodegen.yml`, `mobile/ios/KovanicaWallet/Info.plist`
- **Build Guide**: `mobile/ios/BUILD.md`
- **Status**: Requires macOS + Xcode 15.4+ to build
- **Dependencies**: KeychainSwift, BIP39Swift, SwiftCrypto (via SPM)
- **FFI Integration**: Needs UniFFI-generated Swift bindings from Rust `kovanica` crate

---

## CI/CD Pipeline

**Workflow**: `.github/workflows/ci-cd.yml`

### Jobs
1. **android** — Ubuntu, builds debug + release APKs
2. **linux-tauri** — Ubuntu, builds AppImage/.deb/.rpm via `tauri-action`
3. **windows-tauri** — Ubuntu, cross-compiles NSIS installer via mingw-w64
4. **ios** — macOS, builds `.app` via xcodebuild
5. **web-consoles** — Ubuntu, builds both React apps, deploys to VPS via pm2 on main branch
6. **rust-tests** — Ubuntu, cargo check/clippy/test workspace
7. **release** — Aggregates all artifacts, creates GitHub Release on version tags

### Required GitHub Secrets
| Secret | Purpose |
|--------|---------|
| `KEYSTORE_PASSWORD` | Android release keystore password |
| `KEY_PASSWORD` | Android release key password |
| `VPS_HOST` | VPS hostname/IP for pm2 deploy |
| `VPS_USER` | SSH username for VPS |
| `VPS_SSH_KEY` | SSH private key for VPS deploy |

---

## P2P Fixes Included (from PRs #88, #89)
- Dial-order rotation to prevent eclipse attacks
- No-op dump prevention (silence peers sending full-chain dumps unrequested)

---

## Deployment Checklist

- [x] Push tag to remote: `git push kovanica main v0.1.0-mobile` ✅
- [ ] Configure GitHub Secrets for CI/CD
- [ ] Test Android release APK on physical device
- [ ] Build iOS on macOS: `cd mobile/ios && xcodegen generate && xcodebuild -scheme KovanicaWallet -configuration Release`
- [ ] Deploy web consoles to VPS:
  ```bash
  cd /opt/kovanica/mobile/console/kovanica && pm2 start ecosystem.config.js
  cd /opt/kovanica/mobile/console/enterprise && pm2 start ecosystem.config.js
  pm2 save
  ```
- [ ] Verify pm2 processes: `pm2 list && pm2 logs`
- [ ] Test NSIS installer on Windows VM
- [ ] Compute SHA256 for all artifacts and update this doc

---

## Version Map
| Component | Version |
|-----------|---------|
| Android Wallet | 1.0.0 (versionCode 1) |
| Kovanica Console (Tauri) | 0.1.0 |
| Enterprise Console (Tauri) | 0.1.0 |
| Web Consoles | 0.1.0 |
| iOS Wallet | 0.1.0 (skeleton) |
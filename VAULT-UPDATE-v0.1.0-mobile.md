# Kovanica Mobile Apps v0.1.0 — Vault Documentation Update

**Date**: 2026-09-30
**Tag**: `v0.1.0-mobile`
**Commit**: `478abbc`

> The tag was first cut at `caeab4e`, which shipped Android builds carrying the
> key-derivation defect fixed in PR #91. The APKs were rebuilt from `478abbc`,
> the merge commit for that PR, and the tag re-cut onto that tree. The four Tauri binaries are unchanged — the fix touches no
> desktop console code — and their digests still match the original cut.

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
- **Build Config**: `mobile/ios/xcodegen.yml` (**single** build definition), `mobile/ios/KovanicaWallet/Info.plist`
- **Build Guide**: `mobile/ios/BUILD.md`
- **Status**: Requires macOS + Xcode 15.4+ to build. No `.xcodeproj` is committed — run `xcodegen generate`.
- **Dependencies**: `kovanica.xcframework` (built by `protocol/crates/kovanica-ffi/build-apple.sh`), KeychainSwift (via SPM)
- **Build order**: `build-apple.sh` → `xcodegen generate` → `xcodebuild`. The first step is mandatory; without the framework the build fails at link time.
- **Note**: `mobile/ios/Package.swift` was **removed** in PR #90 — it would have needed the generated bindings both compiled into the app target (as xcodegen does) and imported as a module, so two build definitions would each carry their own FFI wiring. The mnemonic-handling Swift packages were dropped for the same reason; see the derivation note below.

---

## Key Derivation — Client Correctness (PR #90)

**Classification: client-only.** No GHOSTDAG, UTXO, emission, fee, or validation rule is affected. A node never derives an address from a phrase; it receives a 32-byte signing key and does no derivation.

Both mobile wallets derived addresses by **truncating the stretched key material to its first 32 bytes** and skipping SLIP-0010, against a rule that has exactly one implementation (Rust, `protocol/crates/kovanica-wallet`, path `m/44'/3007'/0'/0'/i'`). The FFI exposed no mnemonic→key function at all, which is why each client re-derived — and both got it wrong.

The light-node FFI's `keypair_from_secret` requires exactly 32 bytes and does `KeyPair::from_seed`, i.e. it expects the **SLIP-0010 child**. Android passed the truncated stretch, so every send would have targeted a keypair owning no UTXOs, and funds sent to the address the app displayed were unreachable. iOS produced a base64 of the phrase behind a `kvnc1` prefix — not an address — and ignored the passphrase, so a protected phrase resolved silently to an empty account instead of erroring.

Resolution:
- `protocol/crates/kovanica-ffi/src/deriv.rs` (new) delegates to `kovanica-wallet`; nothing is reimplemented. Exports `deriveAccountFromMnemonic`, `deriveAddressFromMnemonic`, `deriveSigningSecretFromMnemonic`, `accountFromSigningSecret`, `addressFromSigningSecret`, `mnemonicIsValid`, `slip10DerivationPath`, `slip44CoinType`.
- Android `KovanicaKeys` (Kotlin SLIP-0010) wired into all call sites; the truncating seed helper is deprecated with a pointer to the correct call.
- iOS `KovanicaKeys` wraps the FFI. A failed derivation leaves the wallet **closed** rather than showing a placeholder. `send()` reports the gap instead of claiming a transaction was prepared.
- The same zero-entropy vectors are pinned in four places — `kovanica-wallet/tests/slip10_vectors.rs`, `kovanica-ffi/tests/ffi_deriv.rs`, `KovanicaKeysTest.kt`, `KovanicaKeysTests.swift`. A client regression now fails a client test instead of producing an empty wallet.

**Carried forward from PR #90:**
- The bindings drift guard was **already failing**: `LightNode.fetch_stake_proof` existed in Rust and was never regenerated. Both languages regenerated; CI now diffs them.
- The iOS CI job could never have passed — it ran `xcodebuild` against a project that was never generated, with no FFI framework. It now runs `build-apple.sh` → drift check → `xcodegen generate` → `xcodebuild`.
- The signing key is held in memory for the duration of derivation only; never written to `UserDefaults`, a plist, or a log. Key custody moves to the Keychain with the change that wires signing.

**Verification**: `cargo test -p kovanica-ffi -p kovanica-wallet` all green, `cargo clippy --workspace --all-targets -D warnings` clean, `./gradlew :app:testDebugUnitTest` 9/9, `./gradlew assembleDebug assembleRelease` successful. The Swift is **not** compiler-verified — no Swift toolchain on the build host; the new iOS CI job is what will run it.

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
- [x] Merge PR #91 (key-derivation fix) into main — merge commit `478abbc` ✅
- [x] Re-cut `v0.1.0-mobile` onto main after `478abbc`, rebuild both APKs, regenerate
      `checksums-v0.1.0-mobile.txt` ✅
- [ ] Configure GitHub Secrets for CI/CD
- [ ] Test Android release APK on physical device
- [ ] Build iOS on macOS: `./protocol/crates/kovanica-ffi/build-apple.sh` then `cd mobile/ios && xcodegen generate && xcodebuild -scheme KovanicaWallet -configuration Release -destination generic/platform=iOS` (PR #91: the framework step is mandatory; see BUILD.md)
- [ ] Compile-test the Swift derivation (`KovanicaKeysTests`) on macOS — it is not compiler-verified on the build host (PR #91)
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
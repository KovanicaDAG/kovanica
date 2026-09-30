# Kovanica Mobile Apps v0.1.0 — Release Notes

**Release Date**: 2026-09-30
**Git Tag**: `v0.1.0-mobile`
**Commit**: `478abbc`

> **Re-tagged.** This tag was first cut at `caeab4e`, which shipped wallet builds
> that derived the wrong key. The binaries below were rebuilt from `478abbc`,
> the merge commit for PR #91; this tag now points at that tree on `main`. See
> [Key Derivation Fix](#-key-derivation-fix) below. The Linux and Windows
> binaries are unchanged — the fix touches no desktop console code — so only the
> two Android APKs differ from the original cut.
> `checksums-v0.1.0-mobile.txt` was regenerated to match.

---

## 🎉 What's New

This is the first mobile apps release for the Kovanica Protocol, delivering native wallet applications for Android and desktop, web-based consoles, and an iOS skeleton.

### 📱 Android Wallet (Kovanica Wallet)
- **Pure-Kotlin BIP-39**: Replaced missing JitPack dependency with internal implementation using BouncyCastle
- **Compose UI**: Material3 design with wallet setup, send/receive, settings screens
- **Release Signing**: Properly signed APK with RSA-2048 keystore
- **FFI Integration**: UniFFI bindings to Rust core (kovanica-dag, kovanica-state)

### 🖥️ Desktop Apps (Tauri)
- **Linux**: AppImage, .deb, .rpm packages
- **Windows**: NSIS installer (cross-compiled from Linux)
- **Shared React Codebase**: Kovanica Console (9 routes) + Enterprise Console (8 routes)

### 🌐 Web Consoles
- **Kovanica Console** (port 3000): Block explorer, network monitoring, wallet, multisig, HTLC, developer tools
- **Enterprise Console** (port 3001): Multi-user wallets, API keys, asset management, reporting, webhooks
- **pm2 Deployment**: Production-ready process management configs

### 🍎 iOS Wallet (Skeleton)
- SwiftUI implementation with 8 screens ready for macOS build
- Same feature parity planned as Android

---

## 🔑 Key Derivation Fix

**If you installed the original `v0.1.0-mobile` Android build, delete it and
install this one.** The original build derived the wrong key and could not
spend.

### What was wrong

Wallet key derivation in Kovanica is a frozen rule: a key stretch followed by
the fully-hardened SLIP-0010 ed25519 path `m/44'/3007'/0'/0'/i'`. The light node
receives the 32-byte child of that path — never a phrase, never the raw stretch.

The Android build was instead passing the **first 32 bytes of the stretched
material**, skipping the SLIP-0010 step entirely. The consequence was not a
subtle display bug: the app showed an address that the ledger does not credit,
reported a zero balance, and any send would have targeted a keypair holding no
outputs. The iOS skeleton had the same class of problem, fabricating an address
by encoding the recovery phrase itself.

Two root causes, both fixed:

1. **The FFI exposed no derivation at all**, so each client was left to
   re-implement a cryptographic rule. Two clients re-implemented it; two
   disagreed with each other and with Rust.
2. **The one place that was correct had a sibling that truncated**, and the
   truncation was easy to miss because it type-checked.

### What changed

- `protocol/crates/kovanica-ffi/src/deriv.rs` now exposes derivation, delegating
  to `kovanica-wallet` — the existing Rust authority. There is still exactly one
  implementation of the rule.
- All three Android call sites derive through it. The truncated path is deleted
  and its entry point is marked deprecated, so it cannot be reintroduced by
  accident.
- The iOS wallet derives through the same FFI. Its fabricated address is gone,
  and if derivation ever fails the app now refuses to open rather than showing
  an address that holds nothing.
- The passphrase is honoured rather than ignored on both clients. The same
  phrase under a different passphrase is a different account, and dropping it
  showed an empty address instead of an error.

### Verification

The same vectors are now pinned in four places, so a divergence shows up as a
failing test rather than a wallet holding the wrong coins: the Rust
known-answer suite, the FFI boundary test, the Android unit test, and the iOS
unit test. The vectors use a zero-entropy phrase; they are public test data, not
keys.

This change is **client-side only**. No consensus, UTXO, emission, or validation
rule was touched, and existing chain state is unaffected.

## 🔧 Technical Improvements

### P2P Network Fixes (from PRs #88, #89)
- **Dial-order rotation**: Prevents eclipse attacks by randomizing peer connection order
- **No-op dump prevention**: Silences peers sending unsolicited full-chain dumps

### CI/CD Pipeline
- GitHub Actions workflow covering all platforms
- Automated builds on push/PR/tag
- VPS deployment via pm2 on main branch
- Release asset aggregation on version tags

### Build System
- Android: Gradle 8.4, Kotlin 1.9, Compose BOM 2024.04
- Tauri: Rust 1.78+, Node 20, Vite 5
- iOS: Xcode 15.4, Swift 5.9

---

## 📦 Downloads

| Platform | Artifact | Size |
|----------|----------|------|
| Android | `app-release.apk` (signed) | 18 MB |
| Android | `app-debug.apk` | 24 MB |
| Linux | `kovanica-console_0.1.0_amd64.AppImage` | 83 MB |
| Linux | `kovanica-console_0.1.0_amd64.deb` | 2.5 MB |
| Linux | `kovanica-console-0.1.0-1.x86_64.rpm` | 2.5 MB |
| Windows | `Kovanica Console_0.1.0_x64-setup.exe` | 3.4 MB |

*All artifacts attached to this release.*

---

## 🚀 Installation

### Android
```bash
# Install release APK
adb install -r app-release.apk

# Or download and install manually on device
```

### Linux
```bash
# AppImage (universal)
chmod +x kovanica-console_0.1.0_amd64.AppImage
./kovanica-console_0.1.0_amd64.AppImage

# Debian/Ubuntu
sudo dpkg -i kovanica-console_0.1.0_amd64.deb

# Fedora/RHEL
sudo rpm -i kovanica-console-0.1.0-1.x86_64.rpm
```

### Windows
```
Run Kovanica Console_0.1.0_x64-setup.exe and follow installer prompts.
```

### Web Consoles (VPS)
```bash
# Deploy via pm2 (see deploy-consoles.sh)
pm2 start ecosystem.config.js
pm2 save
```

---

## ⚠️ Known Limitations

1. **iOS**: Requires macOS + Xcode to build (not included in this release)
2. **Android Emulator**: Needs KVM hardware acceleration for x86_64 images
3. **Windows Installer**: Cross-compiled — test on native Windows before distribution
4. **Web Consoles**: Require VPS with Node.js 20+ and pm2 for production deployment
5. **RPC/Network**: Defaults to testnet; configure seed node for mainnet

---

## 🔐 Security Notes

- Android release APK signed with dedicated keystore (RSA-2048)
- Private keys never leave device (client-side signing only)
- BIP-39 mnemonics encrypted at rest via Android Keystore
- P2P: Plaintext TCP on port 9000, DNS-only seed (`seed.kovanica.online`)

---

## 📋 Full Changelog

See `git log v0.1.0-mobile --oneline` or GitHub compare view.

Key commits:
- `571e24e` — mobile: fix Android BIP-39 dependency, add release signing, CI/CD pipeline, pm2 deploy configs
- `7279ddd` — Merge PR #88 (P2P: silence no-op dump, rotate dial order)
- `c2868e4` — Mobile apps: Android wallet, iOS wallet, Enterprise Console, Kovanica Console

---

## 🙏 Credits

Built with:
- **Rust**: kovanica-dag (GHOSTDAG k=3), kovanica-state (UTXO), kovanica-node
- **Kotlin**: Android SDK 34, Jetpack Compose, BouncyCastle
- **Swift**: SwiftUI, Combine
- **TypeScript**: React 18, Vite, TailwindCSS, React Router
- **Tauri**: v1.5 for desktop app wrapper
- **UniFFI**: Rust ↔ Kotlin/Swift FFI bindings

---

## 📄 License

MIT License — see LICENSE file for details.

Kovanica Protocol © 2026
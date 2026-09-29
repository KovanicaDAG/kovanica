# iOS Build Guide — Kovanica Wallet

## Prerequisites

- macOS 14+ (Sonoma or later)
- Xcode 15.4+
- Xcode Command Line Tools: `xcode-select --install`
- Homebrew (recommended): `/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"`

## Setup

```bash
# Install xcodegen (for project generation)
brew install xcodegen

# Navigate to iOS directory
cd mobile/ios

# Generate Xcode project
xcodegen generate --spec xcodegen.yml

# Open in Xcode
open KovanicaWallet.xcodeproj
```

## Xcode Configuration

1. **Select Team**: In Project → Signing & Capabilities → Team
2. **Bundle Identifier**: `com.kovanica.wallet` (already set)
3. **Capabilities** (add as needed):
   - Keychain Sharing
   - Face ID / Touch ID
   - Camera (for QR scanning)

## Build Commands

### Debug Build (Simulator)
```bash
xcodebuild -scheme KovanicaWallet -configuration Debug \
  -destination 'platform=iOS Simulator,name=iPhone 15' \
  -derivedDataPath build
```

### Release Build (Device)
```bash
xcodebuild -scheme KovanicaWallet -configuration Release \
  -destination generic/platform=iOS \
  -archivePath build/KovanicaWallet.xcarchive \
  archive
```

### Export IPA
```bash
xcodebuild -exportArchive \
  -archivePath build/KovanicaWallet.xcarchive \
  -exportPath build/Release \
  -exportOptionsPlist ExportOptions.plist
```

## ExportOptions.plist

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>method</key>
    <string>app-store</string>
    <key>teamID</key>
    <string>YOUR_TEAM_ID</string>
    <key>stripSwiftSymbols</key>
    <true/>
    <key>uploadBitcode</key>
    <false/>
    <key>uploadSymbols</key>
    <true/>
</dict>
</plist>
```

## FFI Integration (Production)

The SwiftUI skeleton uses REST API calls. For production, integrate UniFFI-generated Swift bindings:

```bash
# From Rust workspace root
cd protocol
cargo install uniffi_bindgen

# Generate Swift bindings for kovanica crate
uniffi-bindgen generate ../sdk/crates/kovanica-ffi/ffi/kovanica.udl \
  --language swift \
  --out-dir ../mobile/ios/KovanicaWallet/Generated
```

Add to Xcode:
1. Drag `Generated/kovanicaFFI.swift` into project
2. Add `libkovanica.a` (built from Rust) to Frameworks & Libraries
3. Set Header Search Paths to include generated headers

## Dependencies (via Package.swift)

| Package | Purpose |
|---------|---------|
| KeychainSwift | Secure mnemonic/key storage |
| BIP39Swift | Mnemonic generation/validation |
| SwiftCrypto | Ed25519, SHA-256, PBKDF2 |

## CI/CD (GitHub Actions)

The `.github/workflows/ci-cd.yml` includes an `ios` job that runs on `macos-latest`:

```yaml
ios:
  runs-on: macos-latest
  steps:
    - uses: maxim-lobanov/setup-xcode@v1
      with: { xcode-version: '15.4' }
    - run: xcodebuild -scheme KovanicaWallet -configuration Release ...
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "No signing certificate" | Set Team in Xcode, enable Automatic signing |
| "Package resolution failed" | `xcodebuild -resolvePackageDependencies` |
| "UniFFI not found" | `cargo install uniffi_bindgen` |
| "Architecture mismatch" | Build for `arm64` only (iOS devices) |

## App Store Connect

1. Create app in App Store Connect with Bundle ID `com.kovanica.wallet`
2. Configure provisioning profiles (Xcode manages automatically)
3. Upload via Transporter or `xcrun altool`
4. TestFlight for beta, then App Store review

## Current Status

- ✅ SwiftUI Views (10 files)
- ✅ ViewModel with REST API
- ✅ Package.swift (SPM)
- ✅ xcodegen.yml
- ✅ Info.plist
- ⏳ Xcode project (generate with xcodegen)
- ⏳ FFI bindings (needs Rust build)
- ⏳ Keychain integration
- ⏳ BIP-39 mnemonic flow
- ⏳ QR code scanning
- ⏳ Push notifications (optional)
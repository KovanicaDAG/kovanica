# 2026-09-03 — KVNC brand (kvnc-logo) + remove __grok + standalone wallets

> **Links:** [[myObsidianVaultDAG]] · [[CODE_INDEX]] · [[SESSIONS]] · [[ROADMAP]]

Branding and mobile wallet work on **kovanica-protocol**. Authoritative source:
`/home/antonio/KovanicaDAG/kovanica-protocol` (GitHub `KovanicaDAG/kovanica-protocol`).

## Shipped

### 1. `kvnc-logo.png` is now the token icon across web surfaces (PR #76)
- Added the canonical **`kvnc-logo.png`** (user-provided; only this logo is used — no
  kuna/rarity/favicon/coin assets) to `web/public/` and `wallet-extension/public/`.
- `web/src/routes/__root.tsx`: serves `/kvnc-logo.png` as the PNG favicon +
  apple-touch-icon; `wallet-extension/index.html` likewise.
- New `web/public/site.webmanifest` (KVNC name/theme/`kvnc-logo.png` icon).

### 2. Removed the `__grok` PWA/install branding system (PR #76, full removal)
- Deleted Grok PWA installer + branded assets: `web/public/__grok/` (Grok install
  assets), `scripts/grok-pwa-plugin.{mjs,test.mjs}`, `scripts/grok-pwa-shared.{mjs,d.mts}`,
  `scripts/install-page.html`, `server/middleware/grok-pwa.ts`,
  `server/virtual-grok-og-identity.d.ts`, and the Grok `scripts/brand-check.{mjs,test.mjs}`.
- `vite.config.ts`: dropped `grokPwaPlugin()`; `__root.tsx` manifest → `/site.webmanifest`;
  `browser-smoke.mjs` no longer calls brand-check.
- **Verified**: `npm run typecheck` 0 errors, `npm run lint` 0 errors,
  `npm test` 84/88 (4 pre-existing `with-app-env` failures — confirmed identical
  without these changes), `npm run build` (Vite + Nitro + db:migrate) succeeds.
- Branch `brand/kvnc-logo-token-icon` → **draft PR #76**.

### 3. Standalone Kovanica Wallet — Android + iOS (PR #77)
- New **`kovanica-wallet/`** at repo root: a **pure API-backed** wallet (read +
  receive + faucet) with **no light node, no SPV, no FFI**.
- **Android** (`android/`): Kotlin + Jetpack Compose (Material3), OkHttp + org.json
  client, Home/Receive/History, KVNC branding via `kvnc_logo.png`, adaptive (v26+)
  + legacy density launcher icons (minSdk 24). Pure Kotlin — no Rust/NDK.
- **iOS** (`ios/`): Swift + SwiftUI, hand-authored canonical Xcode project (flat
  `objects` map, asset catalog wired into the Resources phase), KVNC brand palette,
  `kvnc-logo.png` AppIcon.
- **CI**: `.github/workflows/wallet.yml` builds the Android APK (JDK17 + SDK only).
- **Wallet CI expanded (after first Android run failed)**: `wallet.yml` gained a
  `build-ios-app` job (`macos-latest`) that runs
  `xcodebuild -project …/KovanicaWallet.xcodeproj -scheme KovanicaWallet -sdk
  iphonesimulator … CODE_SIGNING_ALLOWED=NO build` — the only real iOS compile
  verification (no Mac locally). The first Android run exposed a
  `:app:checkDebugAarMetadata FAILED` stemming from
  `androidx.lifecycle:lifecycle-*:2.11.0`, which demands AGP 9.1+ AND
  compileSdk 37. Fix: downgrade lifecycle to **2.9.4** and keep
  `compileSdk`/`targetSdk` at **36** (`platforms;android-36` + `build-tools;36.0.0`
  in CI) — the correct resolution, not a bump to 37 (`platforms;android-37` is not
  even an installable sdkmanager package on the GitHub runner image).
- **Final green commit (`aa07127`)**: all six PR checks pass — Android APK build,
  iOS simulator `xcodebuild`, Build/lint/typecheck, Rust build/test/lint, Web build.
- **v1 scope**: watch address, balance, history, faucet. **Send/sign is deliberately
  NOT implemented** — building+siging needs an ed25519 + sighash crypto binding
  (documented follow-up); Send control is disabled/omitted.
- Branch `wallet/standalone-api-wallet` → **draft PR #77**.

## Lessons burned in
- The repo-wide `.gitignore` had a bare `data/` rule (intended to ignore root-level
  node/ledger data dirs like `crates/kovanica-node/data/`). As a glob it ALSO
  matched the wallet's Kotlin source package `…/com/kovanica/wallet/data/`, which is
  why `KovanicaWalletApp.kt`/`ui/*` failed to resolve `data.*` references in CI — the
  fresh checkout never had those `.kt` files. Fix: add a negation
  `!kovanica-wallet/android/app/src/main/java/com/kovanica/wallet/data/` (mirroring
  the existing `!android-light-node/…/lightnode/data/`), then `git add` the now
  un-ignored files (they were never committed). `android-light-node` compiled fine
  precisely because it already had such a negation.
- `androidx.lifecycle:lifecycle-*:2.11.0` requires **AGP 9.1.0+ AND compileSdk 37**;
  on AGP 8.10.1 / compileSdk 36 it fails `:app:checkDebugAarMetadata`. `android-37`
  is NOT an installable sdkmanager package on the GitHub runner, so the fix is to
  **downgrade lifecycle to 2.9.4** (keep compileSdk 36), not to "compile against 37".
- Compose `by viewModel.uiState.collectAsStateWithLifecycle()` needs
  `import androidx.compose.runtime.getValue`; omitting it surfaces as
  `State<T> … cannot serve as a delegate`, which cascades into downstream
  `Unresolved reference` / overload-resolution noise in the same file
  (`HistoryScreen.kt`). `HomeScreen.kt`/`ReceiveScreen.kt` already had the import.
- iOS deployment target was **15.0**, but the SwiftUI views use iOS-16 APIs
  (`NavigationStack`, `toolbarColorScheme`, `.tracking`). Bump
  `IPHONEOS_DEPLOYMENT_TARGET` to **16.0** in all pbxproj build configs — not add
  `#available` guards around every call site.
- `struct WalletData: Equatable { var nodeURL: String }` gives bare `WalletData()`
  calls a "missing argument for parameter 'nodeURL'" error unless `nodeURL` has a
  default value; and a `[HistoryEntry]` member means `Equatable` can't be synthesized
  until `HistoryEntry` itself is `Equatable`/`Hashable`.
- The `__grok` PWA installer was wired into the Vite/Nitro build *and* `npm test`
  (`grok-pwa-plugin.test.mjs`), so full removal also meant dropping the dead test and
  brand-check tooling — not just the folder. Verified via `npm run build` + `npm test`.
- The first iOS `project.pbxproj` drafts used JSON-style syntax. **Xcode's pbxproj is
  an OpenStep property list** (unquoted keys, semicolon-terminated lines), not JSON —
  rewrote in canonical flat-`objects` format and validated structure (all refs
  resolve, asset catalog in the Resources phase).
- Neither mobile app can be **compiled** on the dev Linux host (no Android SDK /
  Xcode) — Android config is verified by the CI job; iOS compiles only on the
  macOS CI runner (iOS CI job was added to run it without a local Mac).

## Open follow-ups
- [x] **PRs merged to `main`**: #76 (brand + `__grok` removal, `62365d2`) and #77
      (wallet, `02dd3ea`). All `wallet.yml` jobs + repo checks green.
- [x] **iOS install without a Mac (PR #78)**: added `build-ios-ipa` job to
      `wallet.yml` — a cloud-macOS job builds the app for a physical device
      (`-sdk iphoneos`, `CODE_SIGNING_ALLOWED=NO`) and packages an **unsigned
      `KovanicaWallet-ios-unsigned.ipa`** (~3.6 MB) uploaded as the
      `kovanica-wallet-ios-ipa` artifact. Sideload from a PC via AltStore /
      Sideloadly (re-signs with a free Apple ID). Job verified green; 7-day
      expiry on the free-Apple-ID route. Docs added to
      `kovanica-wallet/ios/README.md` ("Install on a real iPhone — no Mac needed")
      and `kovanica-wallet/README.md`. Draft PR #78 (2 commits).
- [ ] Wallet sending/signing — needs an ed25519 + sighash crypto binding (Rust FFI
      crypto-only, or a Kotlin/Swift port) before Send can be enabled.
- [ ] Consider wiring the wallet's `/api` contract into `CODE_INDEX` when it grows.

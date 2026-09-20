# Kovanica Light Node — Android App (Slice 9+)

Android light-node client built on the landed FFI foundation (slices 4–8).
Target: testnet v0.1 (debug-signed APKs). Mainnet gating: Play-signed release + `/api/bootstrap` exposes subsidy/premine/seed.

---

## Structure

```
android-light-node/
├── settings.gradle.kts          # + include AAR via project dir link
├── gradle/libs.versions.toml    # AGP, Kotlin, Compose BOM, JNA pin
├── gradle.properties
├── app/
│   ├── build.gradle.kts         # depends on kovanica-ffi AAR (project-dir link)
│   └── src/main/...             # MainActivity, Compose App, strings, themes
└── README.md
```

---

## Prerequisites

- **Android SDK** (API 36, NDK r27c) — only available in GitHub Actions
- **Rust 1.82+** with `aarch64-linux-android` / `x86_64-linux-android` targets
- **cargo-ndk** (`cargo install cargo-ndk`)
- **Android NDK** (exported as `ANDROID_NDK_HOME`)

---

## Build Locally (Development)

```bash
cd android-light-node

# 1. Build the FFI AAR first (from protocol/ root)
cd ../protocol/crates/kovanica-ffi
./build-android.sh

# 2. Build the Android app (project-dir AAR link)
cd ../../android-light-node
./gradlew assembleDebug
```

**Output:** `app/build/outputs/apk/debug/app-debug.apk`

---

## CI Build (GitHub Actions)

The workflow `.github/workflows/build-android.yml` handles AAR + APK build.
It mirrors the `build-web` pattern:

1. Builds AAR via `cargo-ndk` (both ABIs: arm64-v8a, x86_64)
2. Uploads AAR artifact
3. (Future) Builds APK in `android-light-node/` using downloaded AAR

---

## Architecture (Slice 9a–9f)

| Slice | Status | Description |
|-------|--------|-------------|
| **9a** | ✅ Scaffold | App scaffold + FFI wiring; genesis gate (live `/api/bootstrap` + `/api/state` params) |
| **9b** | 🔜 Wallet UX | Onboarding (create/import mnemonic), home, send, receive, history, settings |
| **9c** | 🔜 Light Sync | `GET /api/light_sync` + `receiveLightSync` + KVLS v1 persistence |
| **9d** | 🔜 Staking | Bond/unbond, `setValidatorSeed` + `enableHybrid`, `produceBlock` → `POST /api/mine/submit` |
| **9e** | 🔜 Background | WorkManager periodic sync + local notifications |
| **9f** | 🔜 Release | Branding, CI APK artifact, Play signing decision |

---

## Genesis Gate (Slice 9a — Critical)

The app **must** reproduce the live network genesis before any UI work.

**Live parameters (hard requirement):**
```kotlin
LightConfig(
    k = 3,
    subsidy = 20_000_000_000L,      // 200 KVNC in atoms
    founderAmount = 20_000_000_000L,
    founderSeed = 1,
    finalityDepth = Long.MAX_VALUE,
    payloadPruningDepth = Long.MAX_VALUE
)
```

Derived from `crates/kovanica-node/src/explorer.rs` `genesis_node()` +
`GENESIS_SUBSIDY` / `GENESIS_PREMINE` constants.

**Gate test** (already landed in Rust layer):
- `LightConfig::default()` (subsidy 1000) → **diverges** from live genesis
- Live params above → **exact match** to live genesis `596874ea…`
- `receiveBlocks(live blob)` → 10 blocks applied, tip matches live `4927b982…`

**v0.1 testnet pins these params as app constants**; a node slice should add them to `/api/bootstrap` before mainnet.

---

## FFI Integration

- AAR produced from `protocol/crates/kovanica-ffi` (`build-android.sh`)
- Kotlin bindings committed under `bindings/kotlin/uniffi/kovanica/`
- JNA-based (UniFFI 0.32) — loads `libkovanica_ffi.so` on first use
- App consumes AAR via project-dir link during dev, published AAR in CI

---

## Key Classes

| Class | Purpose |
|-------|---------|
| `LightNodeRepository` | Owns `LightNode`, persists KVLS v1 blob (`light_sync.bin`), handles sync (`/api/light_sync` → `receiveLightSync` → filters → `/api/blocks`) |
| `WalletRepository` | `sendFrom`, `bondStake`, `unbond`, `enableValidator` (`setValidatorSeed` + `enableHybrid`), `produceAndSubmitBlock` |
| `WalletViewModel` | State management, lifecycle-aware coroutines |
| `MainActivity` | Genesis gate screen → Wallet screen |

---

## Network Identity

| Network | Genesis | `/api/bootstrap` | Seeds |
|---------|---------|------------------|-------|
| `kovanica-testnet` | `9565fc20cb465eec...` | `https://explorer.kovanica.online/api/bootstrap` | `seed.kovanica.online:9000`, `seed2.kovanica.online:9000` |

Live HTTP surface on seed (`explorer.rs`):
- `GET /api/bootstrap` → JSON (genesis, tip, subsidy, premine, seed, k, **light_config**)
- `GET /api/blocks` → `application/octet-stream` = `encode_records` (feed to `receiveBlocks`)
- `GET /api/light_sync` → KVLS v1 blob
- `GET /api/head`, `/api/history`, `/api/utxos`, `/api/faucet`
- `POST /api/mine/submit` → block uplink (octet-stream for staked blocks)

---

## Build Commands

```bash
# Debug APK (v0.1)
./gradlew assembleDebug

# Release APK (requires signing config)
./gradlew assembleRelease

# Lint + tests
./gradlew check

# Clean
./gradlew clean
```

---

## Signing (Slice 9f)

- **v0.1 testnet**: debug-signed APKs (Play requires signed release for production)
- **Decision needed**: obtain/repo-managed keystore vs local debug builds

---

## Related

- Protocol FFI: `../protocol/crates/kovanica-ffi/`
- Android build script: `../protocol/crates/kovanica-ffi/build-android.sh`
- Live sync spike test: `../protocol/crates/kovanica-ffi/tests/live_sync_spike.rs`
- Plan: `../protocol/docs/plans/android-light-node-app.md`
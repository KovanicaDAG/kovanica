# 2026-08-28 — Phase 4 Slice 9b: Wallet UX + Bootstrap LightConfig

> **Links:** [[ROADMAP]] · [[TODO]] · [[UPGRADE-PHASES]] · [[TESTNET]] · [[HYBRID_POW_AND_PHONE_LIGHT_NODE]]

---

## Shipped

| Item | Status | Notes |
|------|--------|-------|
| Swift FFI bindings regenerated | ✅ Done | `crates/kovanica-ffi/bindings/swift/` now matches Kotlin surface (38 `LightNode` methods) |
| Android `NodeRepository` owns `LightNode` | ✅ Done | Single-thread dispatcher; all FFI calls serialized; `close()` on ViewModel clear |
| Android `WalletViewModel` chain operations wired | ✅ Done | Balance, sync, send, bond, unbond, produce block, history — all `TODO()` replaced |
| Secure BIP39 mnemonic storage | ✅ Done | Android Keystore + AES/GCM; only base64 IV/ciphertext in `SharedPreferences` |
| Gradle wrapper added | ✅ Done | `android-light-node/gradlew` + wrapper JAR/properties; AGP 8.10.1 ↔ Gradle 8.14 |
| `/api/bootstrap` extended with `light_config` | ✅ Done | `k`, `subsidy`, `premine`, `founder_seed`, `finality_depth`, `payload_pruning_depth` |
| Android consumes `/api/bootstrap` | ✅ Done | `NodeRepository.fetchLightConfig()` caches server config; offline fallback to testnet defaults |
| `/api/light_sync` confirmed present | ✅ Already shipped | `GET /api/light_sync[?from=<id>]` emits KVLS v1 headers+filters |
| `POST /api/mine/submit` staked wire confirmed | ✅ Already shipped | wire `Content-Type: application/octet-stream` accepts `BlockRecord` with `Option<StakedVrf>` |
| Dead code / plaintext storage removed | ✅ Done | Deleted `LightNodeRepository.kt`, `WalletRepository.kt`; removed plaintext `seedPhrase` from prefs |
| Rust workspace green | ✅ Done | `cargo test --workspace` + `cargo clippy --workspace --all-targets` pass |

---

## Key files changed

```
android-light-node/
  gradlew (new)
  gradlew.bat (new)
  gradle/wrapper/gradle-wrapper.jar (new)
  gradle/wrapper/gradle-wrapper.properties (new)
  app/src/main/java/com/kovanica/lightnode/data/NodeRepository.kt
  app/src/main/java/com/kovanica/lightnode/ui/WalletViewModel.kt
  app/src/main/java/com/kovanica/lightnode/ui/prefs/WalletPrefs.kt
  app/src/main/java/com/kovanica/lightnode/MainActivity.kt
crates/kovanica-ffi/bindings/swift/kovanica.swift
crates/kovanica-ffi/bindings/swift/kovanicaFFI.h
crates/kovanica-node/src/explorer.rs
```

---

## Decisions & trade-offs

- **Bootstrap `light_config` object:** chose a nested object rather than flat top-level fields only, so the mobile client can pass one JSON blob straight into `LightConfig`. Existing top-level fields preserved for backward compatibility.
- **Finality / pruning depths:** no named constants existed in the node code; introduced `TESTNET_FINALITY_DEPTH = 100` and `TESTNET_PAYLOAD_PRUNING_DEPTH = 1000` in `NetworkProfile` and wired `genesis_node()` to `Node::genesis_with_finality()` using those values. This is a testnet policy choice, not consensus.
- **Offline fallback:** if `/api/bootstrap` fails (airplane mode, bad URL), the app falls back to hard-coded testnet defaults so the wallet UI stays open. It will re-fetch on the next cold boot.
- **Staking actor derivation:** bonding still derives the spending actor from the first 8 bytes (LE) of the validator seed, which is **not** the BIP39 wallet address. Wallet coins must be moved to that actor address before bonding (known v0.1 limitation).

---

## Open follow-ups

- [ ] Slice 9c — incremental sync + persistence: switch `syncNode` from full `/api/blocks` pull to `/api/light_sync[?from=tip]` and persist imported blocks locally
- [ ] Slice 9d — staking uplink: enable hybrid validator block production from the phone and submit staked wire blocks via `POST /api/mine/submit`
- [ ] Slice 9e — background sync + notifications
- [ ] Slice 9f — release APK signing keystore + GitHub Actions build
- [ ] Android build verification in CI (host lacks Android SDK)

---

## Verification

```bash
cd /root/kovanica-protocol
cargo test --workspace
cargo clippy --workspace --all-targets
```

Result: all green.

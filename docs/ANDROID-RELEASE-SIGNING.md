# Android release signing

Both apps (`apps/android` = Lightnode, `apps/android-wallet` = Wallet) build
`app-release-unsigned.apk`; signing happens on the build box after Gradle.

## Current signing identity

| Field | Value |
|---|---|
| Keystore | `/root/keys/kvnc-release.p12` (PKCS12, mode 600) |
| Alias | `kvnc` |
| Passphrase | `/root/keys/kvnc-release.pass` (mode 600, **not** in git) |
| Subject | `CN=Kovanica, OU=Release, O=KovanicaDAG, C=US` |
| Key | RSA 4096, 10 000 days, SHA384withRSA |
| Certificate SHA-256 | `fbb7abeb78346989418bbd24146f0d3b4ec3da3672c1d4de75e6f56ebc56617f` |

> **Rotation.** The original release key (`/tmp/opencode/kvnc-release.p12`,
> cert SHA-256 `630a6308…4562`) was lost with the box's `/tmp`. Releases
> `v0.3.6-wallet` and `v0.8.7-android` were the first signed with the key above.
> Android ties in-place updates to the signing certificate, so installs of
> 0.3.5 / 0.8.6 or older must be **uninstalled before installing 0.3.6 / 0.8.7**.
> Back the keystore and passphrase up off-box; losing them again means another
> forced reinstall.
>
> **0.3.7 / 0.8.8 — stale native library fix.** Releases `v0.3.7-wallet` and
> `v0.8.8-android` are the first releases built after the AMM rebuild. That
> rebuilds shipped a **stale** UniFFI library: `apps/android*/jniLibs/**/*.so` is
> **gitignored**, so Gradle packaged the pre-PRO-21
> `libkovanica_light_ffi.so` (address strings in legacy hex, canonical base32
> `kvnc…dag` parsing missing) even though the source and the wasm artifacts were
> current. 0.3.6 – 0.8.7 therefore rendered legacy-hex addresses and refused
> canonical base32 addresses on the testnet. 0.3.7 / 0.8.8 rebuild the native lib
> from source; the account itself is unchanged (the 24-word seed still derives
> the same Ed25519 key and the same balance).

## Rebuild the UniFFI native library before packaging

`apps/android/app/src/main/jniLibs/arm64-v8a/libkovanica_light_ffi.so` (same for the
wallet and the comms app) is **not** tracked in git. Refresh it from the current
source before every `./gradlew assembleRelease` or you ship a stale library:

```bash
export ANDROID_NDK_HOME=/opt/android-sdk/ndk/26.1.10909125
rustup target add aarch64-linux-android
rm -rf /tmp/opencode/jnilibs && cargo ndk -t arm64-v8a -o /tmp/opencode/jnilibs \
  build --release -p kovanica-light-ffi -p kovanica-name-registry
for a in apps/android apps/android-wallet apps/android-comms; do
  cp /tmp/opencode/jnilibs/arm64-v8a/*.so "$a/app/src/main/jniLibs/arm64-v8a/"
done
```

The fresh `.so` carries the bech32 base32 address body and the post-PRO-21
address validation; assert before packaging:

```bash
strings -a apps/android/app/src/main/jniLibs/arm64-v8a/libkovanica_light_ffi.so \
  | grep -q qpzry9x8gf2tvdw0s3jn54khce6mua7l && echo BASE32_OK
```

## Build + sign

```bash
export ANDROID_HOME=/opt/android-sdk ANDROID_SDK_ROOT=/opt/android-sdk
cd apps/android-wallet && ./gradlew --no-daemon assembleRelease   # or apps/android
cd -   # back to the repo root

BT=/opt/android-sdk/build-tools/34.0.0
OUT=dist/android && mkdir -p "$OUT"

sign() { # <unsigned.apk> <output.apk>
  "$BT/zipalign" -p -f 4 "$1" "$OUT/aligned.tmp.apk"
  "$BT/apksigner" sign \
    --ks /root/keys/kvnc-release.p12 --ks-key-alias kvnc \
    --ks-pass file:/root/keys/kvnc-release.pass \
    --key-pass file:/root/keys/kvnc-release.pass \
    --v1-signing-enabled true --v2-signing-enabled true --v3-signing-enabled true \
    --out "$2" "$OUT/aligned.tmp.apk"
  rm -f "$OUT/aligned.tmp.apk"
}

sign apps/android-wallet/app/build/outputs/apk/release/app-release-unsigned.apk \
     "$OUT/Kovanica-Wallet-0.3.7-arm64-v8a.apk"
sign apps/android/app/build/outputs/apk/release/app-release-unsigned.apk \
     "$OUT/Kovanica-Lightnode-0.8.8-arm64-v8a.apk"
```

The passphrase file holds the password **twice**, one line each: `apksigner`
reads the store password and the key password sequentially from the same
reader, so a single line fails with `end of file reached`.

## Verify before publishing

```bash
"$BT/apksigner" verify --print-certs "$OUT/Kovanica-Wallet-0.3.7-arm64-v8a.apk"
"$BT/zipalign" -c -p 4 "$OUT/Kovanica-Wallet-0.3.7-arm64-v8a.apk"
"$BT/aapt" dump badging "$OUT/Kovanica-Wallet-0.3.7-arm64-v8a.apk" | head -1
# the AMM artifact must be the current one
unzip -p "$OUT/Kovanica-Wallet-0.3.7-arm64-v8a.apk" assets/kovanica_token.wasm | sha256sum
# the native lib must be the post-PRO-21 build (base32 addresses)
unzip -p "$OUT/Kovanica-Wallet-0.3.7-arm64-v8a.apk" lib/arm64-v8a/libkovanica_light_ffi.so \
  | strings -a | grep -q qpzry9x8gf2tvdw0s3jn54khce6mua7l \
  && echo NATIVE_BASE32_OK
```

Expected wasm sha256: `a6988f5e1742b1ac0a069a3ef274d7590a54266dee57435438664e23036dc49b`
(91 797 bytes) — see `docs/AMM-REBUILD.md`.

## Publish

Releases live on `KovanicaDAG/kovanica`, tags `v<version>-wallet` and
`v<version>-android`, assets `Kovanica-Wallet-<version>-arm64-v8a.apk` and
`Kovanica-Lightnode-<version>-arm64-v8a.apk`. After publishing, bump the pins in
`kovanica-web` (`src/routes/download/$.ts` and the wallet download component) in
the same change.

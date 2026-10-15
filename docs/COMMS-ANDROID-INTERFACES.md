# COMMS Android — interface spec (v0.3.0 "Messages")

Status: **frozen** for the Android comms wiring. Owner: orchestrator. Lanes: `android-ffi` (Rust),
`android-logic` (Kotlin, non-UI), `android-ui` (@designer).

**Layer: client-only end-to-end.** No consensus / ledger / tokenomics change. Sealing reuses
`kvnc-comms-crypto` + `kvnc-comms-relay` through `kvnc-light-ffi`; the relay is off-chain.

---

## §1 FFI additions — `crates/kvnc-light-ffi/src/lib.rs`

Mechanism is unchanged: `uniffi::setup_scaffolding!("kvnc_light")` + `#[uniffi::export]` (proc-macro,
no `.udl`). Add exactly these three functions; reuse the existing `LightFfiError` and the existing
keystore-decrypt path used by `sign_detached`.

| Rust (new) | Kotlin (generated) |
|---|---|
| `comms_seal_envelope(keystore_json: String, passphrase: String, recipient_pubkey_hex: String, payload_json: String, sent_at_ms: u64, expires_at_ms: u64) -> Result<String, LightFfiError>` | `commsSealEnvelope(keystoreJson, passphrase, recipientPubkeyHex, payloadJson, sentAtMs: ULong, expiresAtMs: ULong): String` |
| `comms_open_envelope(keystore_json: String, passphrase: String, envelope_json: String) -> Result<String, LightFfiError>` | `commsOpenEnvelope(keystoreJson, passphrase, envelopeJson): String` |
| `comms_verify_envelope(envelope_json: String, sender_pubkey_hex: String) -> Result<Boolean, LightFfiError>` | `commsVerifyEnvelope(envelopeJson, senderPubkeyHex): Boolean` |

Semantics (normative):

- **seal**: decrypt keystore → Ed25519 seed; derive recipient Ed25519 pubkey from
  `recipient_pubkey_hex` (64 lowercase hex, `0x` optional); call
  `kvnc_comms_relay::seal_and_sign(...)` / `Envelope::to_json()`; return **compact envelope JSON**
  (the exact wire shape in §3.3 of the M1 envelope: keys `v,id,to,from,sentAt,expiresAt,alg,epk,nonce,ct,sig`).
  Reject payloads larger than `MAX_PLAINTEXT` (16384) and invalid recipient keys via `LightFfiError`.
- **open**: parse `envelope_json`; require `envelope.to == comms_id_from_hex(my comms_id)` (reject
  otherwise); decrypt via the recipient's static X25519 key; `trim_payload`; return the **payload JSON
  string**. Does **not** verify the sender signature (that is `comms_verify_envelope`).
- **verify**: `Envelope::verify(sender_pubkey)` semantics — Ed25519 sig over `signing_bytes` **and**
  `from == comms_id(sender_pubkey)`; returns `true`/`false` (only malformed JSON/hex is an error).

New deps on `kvnc-light-ffi`: `kvnc-comms-crypto`, `kvnc-comms-relay` (both path deps; `kvnc-comms-core`
already present). Regenerate Kotlin bindings with:

```
cargo run -p kvnc-light-ffi --features bindgen --bin kvnc-light-bindgen -- \
  generate --library target/debug/libkvnc_light_ffi.so --language kotlin --out-dir apps/android/app/src/main/java
```

(Also regenerate the Swift bindings for iOS to keep parity, using `--language swift`.)

Rust tests: `cargo test -p kvnc-light-ffi`, `cargo clippy -p kvnc-light-ffi -- -D warnings`, and a
`comms_seal_envelope` → `comms_open_envelope` round-trip KAT.

---

## §2 Kotlin modules — package `io.kovanica.light`

Three new files; no new gradle dependencies (use `java.net.HttpURLConnection` + `org.json`, matching
`NodeClient.kt`; background work on the existing `Executors.newSingleThreadExecutor()` pattern,
marshalling with `runOnUiThread`).

### `RelayClient.kt`
```kotlin
class RelayException(val status: Int, val code: String, override val message: String) : Exception(message)

class RelayClient(private val baseUrl: String) {          // baseUrl e.g. "http://10.0.2.2:8787"
    fun health(): Boolean
    fun postEnvelope(envelopeJson: String): String        // 201 -> id (64 hex)
    fun getInbox(commsId: String, limit: Int = 50, sinceMs: Long? = null): List<String>  // raw envelope JSONs, oldest-first
    fun ack(commsId: String, ids: List<String>): Int      // -> deleted count
}
```
Errors map the relay error body `{"error":{"code","message"}}` to `RelayException`.

### `CommsClient.kt`
```kotlin
data class ReceivedMessage(
    val envelopeId: String,
    val senderCommsId: String,
    val payloadJson: String,
    val sentAtMs: Long,
)

class CommsClient(
    private val keystoreJson: String,
    private val passphrase: String,
    private val relay: RelayClient,
) {
    fun myPubkeyHex(): String                     // walletPubkeyHex(keystoreJson)
    fun myCommsId(): String                       // deriveCommsId(myPubkeyHex())
    fun sendEnvelope(recipientPubkeyHex: String, body: String, kind: String = "text", ttlMs: Long = 7L*24*3600*1000): String
    fun fetchInbox(limit: Int = 50, sinceMs: Long? = null): List<ReceivedMessage>
    fun ackMessages(ids: List<String>): Int
}
```
- `sendEnvelope` builds the payload JSON `{"kind":<kind>,"body":<body>,"ts":<nowMs>}` (compact, key
  order `kind,body,ts` — mirrors `payloadJson` in `packages/comms/src/messaging.ts`), calls
  `commsSealEnvelope(...)` with `sentAt=now`, `expiresAt=now+ttlMs`, then `relay.postEnvelope(...)`.
- `fetchInbox` calls `relay.getInbox(myCommsId(), limit, sinceMs)`, then `commsOpenEnvelope(...)` per
  envelope; unparseable/undecryptable envelopes are skipped, not fatal.

### `CommsStore.kt` (SharedPreferences `"kvnc"`, matching the existing pattern)
Persist: `relay_url`, `display_name`, and a contact map `pubkeyHex -> label`.

---

## §3 Relay HTTP (v1) — what the Kotlin client speaks

Base `baseUrl + "/v1"`. `Content-Type: application/json`.

| Method | Path | Body | Success |
|---|---|---|---|
| POST | `/v1/envelope` | bare envelope JSON | `201 {"ok":true,"id":"<hex64>"}` |
| GET | `/v1/inbox/{comms_id}?limit=<n>&since=<ms>` | — | `200 {"envelopes":[<envelope>,…]}` |
| POST | `/v1/ack` | `{"to":"<hex64>","ids":["<hex64>",…]}` | `200 {"ok":true,"deleted":<n>}` |
| GET | `/v1/health` | — | `200 {"ok":true,"version":"…","envelopes":<n>}` |

MVP sends **bare envelopes** (no PoW/payment wrapper). Note for later: if a wrapper is ever sent, the
payment keys are **snake_case** (`htlc_id`,`amount_atoms`,`sender_addr`,`claimer_addr`), not the
camelCase in the TS `relay.ts`.

---

## §4 UI deliverable — `MessagesActivity.kt`

New activity (registered in `AndroidManifest.xml`, launched from a button added to `MainActivity`).
Programmatic `android.widget.*` (the app has no `res/` and no Compose). Screen contains:

1. Identity block: comms id + wallet pubkey (selectable/copyable).
2. Relay URL field (default `http://10.0.2.2:8787`, persisted).
3. Passphrase field + "Unlock" (keystore from the existing `"kvnc"` prefs).
4. Compose: recipient pubkey hex field + message field + "Send".
5. Inbox list: refresh action; each row shows sender comms id + body + timestamp; ack on refresh.
6. Clear inline status/error text.

Design intent (buttons, spacing, hierarchy) is @designer's call; keep the wiring contract above fixed.

---

## §5 Build & release

Bump `apps/android/app/build.gradle.kts` to `versionCode 3` / `versionName "0.3.0"`; rebuild native
lib + `assembleDebug assembleRelease bundleRelease`; re-sign the release APK **v2+v3**; publish as a
GitHub release `v0.3.0-android` (replace `v0.2.0-android`). Orchestrator owns build/publish.

## §6 Non-goals (this cut)

Group/MLS, video, TURN, registry name lookup from the app, payment/PoW submission from the app,
push notifications, contact-graph privacy work. Forward secrecy remains out of scope (MLS at M4).

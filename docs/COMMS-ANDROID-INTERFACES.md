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

---

## §7 v0.4.0 additions — contacts + on-chain name lookup (read-only)

Supersedes the "registry name lookup from the app" line in §6 for this cut. Everything here is
**client-only and read-only**: no transaction is ever signed, nothing is registered, and the app never
deploys a contract.

### §7.1 How a name is resolved over RPC (normative)

There is **no view / `name_resolve` RPC** — every contract-call RPC handler is `NotImplemented`. The only
working read path is the raw contract-storage slot:

1. `key = b"kvnc/v1/name/" ‖ ascii_lowercase(name)` (the contract's `entry_key`).
2. `slot = BLAKE3(key)` → 32 bytes → `"0x" + hex(slot)` (the host stores under `blake3(raw_key)`).
3. `POST /rpc` → `kvnc_getStorageAt`, `params = [contract_address, "0x"+hex(slot)]` (unauthenticated read;
   `contract_address` is a `kvnc…dag` string or raw 32-byte hex).
4. Result is `"0x" + hex(value)` where `value = bincode(NameEntry)`. **An absent key and a not-found name
   both return `"0x"`** — they are indistinguishable; callers treat `"0x"` as "not found".

`NameEntry` bincode 1.3 (fixed-int, LE): `owner[32] ‖ pubkey[32] ‖ u64le(count) ‖ count×(u64le(len) ‖ utf8)`.

### §7.2 FFI additions — `crates/kvnc-light-ffi/src/lib.rs` (lane android-ffi)

| Rust (new) | Kotlin (generated) |
|---|---|
| `registry_slot_hex(name: String) -> Result<String, LightFfiError>` | `registrySlotHex(name): String` |
| `registry_decode_entry(value_hex: String) -> Result<String, LightFfiError>` | `registryDecodeEntry(valueHex): String` |

- `registry_slot_hex`: lowercase the name, validate (3..=32 UTF-8 bytes, only `[a-z0-9-]`, no leading or
  trailing `-`, no `--`), return **64 lowercase hex** (no `0x`). **Single source of truth:** depend on
  `kvnc-name-registry` (`entry_key`) + `kvnc_common::hash` **iff** that does not drag heavy/incompatible
  deps into the mobile cdylib; otherwise define the prefix as a local constant and add a Rust test
  asserting the computed slot equals what the existing wasmi e2e path reads
  (`crates/kvnc-name-registry/tests/wasmi_e2e.rs`).
- `registry_decode_entry`: parse `"0x"+hex` (accept with/without `0x`, case-insensitive); return compact
  JSON `{"owner":"<64hex>","pubkey":"<64hex>","relay_endpoints":["…"]}`. Malformed/truncated bytes → an
  error. Callers check for `""`/`"0x"` **before** calling this.

Tests: a slot KAT against the contract's own `entry_key`/`hash` value, plus a decode round-trip from raw
`bincode::serialize(NameEntry)` bytes. Regenerate the Kotlin **and** Swift bindings (same bindgen command
as §1).

### §7.3 Kotlin modules (lane android-logic)

New `RegistryClient.kt`:
```kotlin
data class RegistryEntry(val ownerHex: String, val pubkeyHex: String, val relayEndpoints: List<String>)

class RegistryClient(private val node: NodeClient, private val contractAddress: String) {
    fun resolve(name: String): RegistryEntry?   // null when unconfigured / not found / empty slot
}
```
- `resolve` validates the name (mirror §7.2), computes `registrySlotHex(name)`, and calls
  `node.call("kvnc_getStorageAt", JSONArray().put(contractAddress).put("0x" + slot))`.
- Blank address, null/blank result, or `"0x"` → return `null`. Otherwise `registryDecodeEntry(result)` →
  parse JSON → `RegistryEntry` (pubkey lowercased, no `0x`).

`CommsStore.kt` — schema v2 (backward compatible):
- New key `registry_contract` (String, default `""`).
- `contacts` becomes `JSONObject`: `pubkeyHex -> {"label":String,"name":String?,"relay_endpoints":[String]?}`;
  a legacy plain-String value is still read as `label` with no name/endpoints.
- API: `registryContract: String` / `setRegistryContract(String)`; `contacts(): List<Contact>` with
  `data class Contact(val pubkeyHex: String, val label: String, val name: String?, val relayEndpoints: List<String>?)`;
  `upsertContact(contact: Contact)`; `removeContact(pubkeyHex: String)`; `label(pubkeyHex: String): String?`.

### §7.4 UI (lane android-ui, @designer)

Extend `MessagesActivity.kt` with a **Contacts** section (identity / relay / compose / inbox stay):
- contact list (label, truncated pubkey, optional `@name`); tap a row → prefill the recipient field;
- add-contact flow: name field + "Look up" → shows the resolved pubkey + relay endpoints → "Add"; also
  allow adding by raw pubkey + label when the registry is not configured;
- registry contract address field (persisted via `CommsStore`) with a clear "not configured" state so
  lookup degrades gracefully;
- errors surface in the existing status banner (invalid name, RPC failure, not found).

### §7.5 Non-goals (unchanged + explicit)

No name **registration**/transfer from the app (read-only lookup only), no reverse lookup / enumeration,
no contract deployment from the app, no transactions. Deploying `kvnc-name-registry` to any network is an
**operational action requiring explicit approval** and is out of scope; until a contract address is
configured, lookup reports "not configured".

### §7.6 Build & release (this cut)

Bump `apps/android/app/build.gradle.kts` to `versionCode 4` / `versionName "0.4.0"`; rebuild native lib +
`assembleDebug assembleRelease bundleRelease`; re-sign the release APK **v2+v3**; publish a GitHub release
`v0.4.0-android`. Orchestrator owns build/publish/version bump.

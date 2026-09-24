# Kovanica Derivation — Frozen SLIP-0010 Spec

**Status:** FROZEN (2026-09-23, coin type migrated 917 → 3007) · Client-side
only · **Breaking change to pre-RFC-006 testnet addresses** — intentional,
done pre-mainnet; RFC-006 activation reset already wiped balances. The
2026-09-23 freeze used coin type `917`; the clean-number migration to `3007`
(2026-09-23) changed every derived address and is the current canonical value.

**Owners:** `kovanica-keys` (Rust) · `web/site/src/lib/wallet/keys.ts`
(TypeScript/WebCrypto) · `kovanica-cli` (Rust) · `kovanica-wasm` (WASM glue).

Parallel implementations MUST match byte-for-byte. The **authoritative**
known-answer vectors live in code, not here:

- Rust (SDK): `sdk/crates/kovanica-keys/tests/slip10_vectors.rs`
- Node-side command vectors: `protocol/crates/kovanica-state/tests/sighash_vector.rs`
- TypeScript: `web/site/tests/` (mirrors the Rust suite)

Any side that drifts fails its suite loudly. This document is the algorithm
reference only — it deliberately holds no hex constants.

---

## 1. The path

```
m/44'/3007'/0'/0'/i'
```

- Scheme: **SLIP-0010**, curve **ed25519** (hardened-only children).
- Coin type: `3007` (Kovanica; unregistered in SLIP-44 — frozen constant).
- Account: `0`, change: `0`, address index: `i` (0, 1, 2, …).
- All five segments are HARDENED (`| 0x80000000`).

Constants (Rust, `kovanica-keys`):
- `DERIVATION_PATH = "m/44'/3007'/0'/0'/i'"`
- `SLIP44_COIN_TYPE = 3007`
- `DERIVATION_ACCOUNT = 0`

## 2. Algorithm

1. **BIP-39 phrase** (English, 12 or 24 words) + optional passphrase → 64-byte
   seed via PBKDF2-HMAC-SHA512, 2048 iterations, salt = `"mnemonic" +
   passphrase` (NFKD-normalized phrase). Zeroized on drop.
2. **Master node**:
   `I = HMAC-SHA512(key = "ed25519 seed", data = seed64)` → `sk = I[..32]`,
   `chain = I[32..]`.
3. **Child (hardened only)**:
   `data = 0x00 ‖ sk(32) ‖ ser32(index | 0x80000000)`;
   `I = HMAC-SHA512(key = chain, data)` → new `sk = I[..32]`,
   new `chain = I[32..]`.
4. Repeat for `44, 3007, 0, 0, i` (in that order).
5. Final 32-byte `sk` → Ed25519 keypair (seed-style) → public key →
   `kvnc` + base58(`[0x00] ‖ pubkey`) + `dag` P2PK address.

## 3. Cross-client test vectors (pointer)

Input: the standard **zero-entropy 128-bit** BIP-39 phrase (12 words, empty
passphrase) — the public test phrase built from entropy `[0u8; 16]` in the
Rust suite (no mnemonic-like string appears in source). Expected derived
keys at indices 0, 1, 2 are asserted in `slip10_vectors.rs` — that file is
the single source of truth; web/CLI tests compare against the same values.

Canonical SLIP-0010 official vectors (ed25519, seeds `000102…0f` and
`000102…1f` padded to 64B) are also asserted there.

## 4. Won't fix / notes

- No non-hardened children (SLIP-0010 ed25519 restriction).
- Passphrase ("25th word") is honored and changes every derived key.
- Mnemonic/keys stay client-side; the node only ever sees `tx_hex` +
  `sighash` + signatures.
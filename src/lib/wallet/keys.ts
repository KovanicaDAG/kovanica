import { bytesToHex } from "@noble/hashes/utils.js";

import { normalizePhrase, phraseToSeed } from "./bip39";
import { ensureWasmReady, wasm } from "./wasm";

export { loadWordlist } from "./bip39";

/**
 * Wallet key facade.
 *
 * The recovery rule is fixed by ADR-0002: a 24-word English BIP-39 phrase
 * encodes 32 bytes of entropy and those bytes *are* the Ed25519 seed. There is
 * no PBKDF2 passphrase and no HD derivation, so **one phrase is exactly one
 * wallet** — the same address Android (`kovanica-light-ffi`) and the CLI show.
 *
 * Public keys, addresses and signatures still go through the Rust
 * `@kovanica/sdk-wasm` package, so the browser has no independent Ed25519 code.
 *
 * The `index` parameters are kept only so existing call sites keep compiling;
 * they are ignored, because the chain has no accounts per phrase.
 *
 * All functions are async: the wasm module is initialised on first use.
 */

/** Lowercase / NFKD-normalise / collapse whitespace, without touching crypto. */
export function normalizeMnemonic(phrase: string): string {
  return normalizePhrase(phrase);
}

/**
 * English BIP-39 mnemonic from caller-supplied entropy (16 or 32 bytes). The
 * checksum is computed by the Rust `bip39` crate, so the wordlist is always the
 * canonical one.
 */
export async function entropyToMnemonic(entropy: Uint8Array): Promise<string> {
  await ensureWasmReady();
  return wasm.entropy_to_mnemonic(entropy);
}

/**
 * Generate a fresh recovery phrase. Always 24 words: a shorter phrase encodes
 * 128 bits and cannot seed Ed25519 under the chain rule.
 */
export async function createMnemonic(): Promise<string> {
  await ensureWasmReady();
  return wasm.generate_mnemonic(24);
}

/**
 * 64-byte BIP-39 seed (PBKDF2-HMAC-SHA512). Kept as a utility; the wallet does
 * **not** use it — {@link seedFromMnemonic} is the chain rule.
 */
export async function mnemonicToSeed(mnemonic: string, passphrase = ""): Promise<Uint8Array> {
  await ensureWasmReady();
  return wasm.mnemonic_to_seed(mnemonic, passphrase);
}

/**
 * The one and only 32-byte Ed25519 signing seed for a recovery phrase: its
 * BIP-39 entropy. Secret material — never log or transmit it.
 */
export async function seedFromMnemonic(mnemonic: string, _index = 0): Promise<Uint8Array> {
  return phraseToSeed(mnemonic);
}

/**
 * Public key (64 hex chars) for a recovery phrase. The wallet shows the
 * `kvnc…dag` form of this value; see `hexToKvnc`.
 */
export async function addressFromMnemonic(mnemonic: string, _index = 0): Promise<string> {
  const seed = await phraseToSeed(mnemonic);
  await ensureWasmReady();
  return wasm.public_key_from_secret_bytes(bytesToHex(seed));
}

/** Sign a prepare sighash (hex) with the phrase's single key; 128 hex chars. */
export async function signSighash(
  mnemonic: string,
  _index: number,
  sighashHex: string,
): Promise<string> {
  const seed = await phraseToSeed(mnemonic);
  await ensureWasmReady();
  return wasm.sign_sighash_with_seed_hex(bytesToHex(seed), sighashHex);
}

/** Sign a prepare sighash with a raw 32-byte Ed25519 seed (64 hex chars). */
export async function signSighashWithSeedHex(
  seedHex: string,
  sighashHex: string,
): Promise<string> {
  await ensureWasmReady();
  return wasm.sign_sighash_with_seed_hex(seedHex, sighashHex);
}

/**
 * Validate and normalise an imported phrase: exactly 24 words, wordlist
 * membership (so a typo gets its own message) and the checksum. Rejects
 * anything the node would reject.
 */
export async function importMnemonic(phrase: string): Promise<string> {
  await ensureWasmReady();
  const normalized = wasm.normalize_mnemonic(phrase);
  const words = normalized.length > 0 ? normalized.split(" ") : [];
  if (words.length !== 24) {
    throw new Error("Your recovery phrase needs exactly 24 words.");
  }
  const unknown = wasm.mnemonic_unknown_words(normalized);
  if (unknown.length > 0) {
    throw new Error(`"${unknown[0]}" isn't a recovery word — check for a typo.`);
  }
  if (!wasm.mnemonic_is_valid(normalized)) {
    throw new Error(
      "That recovery phrase doesn't look valid — please check for a typo or a wrong word.",
    );
  }
  // Decode once so the wallet seed is known to exist before we store the phrase.
  await phraseToSeed(normalized);
  return normalized;
}

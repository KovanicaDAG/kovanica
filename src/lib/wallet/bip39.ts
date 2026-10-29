import { sha256 } from "@noble/hashes/sha2.js";

/**
 * BIP-39 English wordlist + entropy decode.
 *
 * The chain does not run PBKDF2 and has no HD path (ADR-0002): the 256 bits a
 * 24-word phrase encodes *are* the Ed25519 seed. This module recovers those
 * bytes so the browser signs with exactly the same key as Android and the CLI.
 *
 * Under `node --test` there is no `fetch` for `/bip39.txt`; tests install
 * `globalThis.__kovanicaWordlist` with the file contents first (same pattern as
 * `__kovanicaWasmInit` in `./wasm`).
 */

let wordsCache: string[] | null = null;
let indexCache: Map<string, number> | null = null;

/** Load the canonical 2048-word list (browser: `/bip39.txt`). */
export async function loadWordlist(): Promise<string[]> {
  if (wordsCache) return wordsCache;
  const injected = (globalThis as { __kovanicaWordlist?: string }).__kovanicaWordlist;
  let text: string;
  if (injected) {
    text = injected;
  } else {
    const res = await fetch("/bip39.txt");
    text = await res.text();
  }
  const words = text.trim().split(/\s+/);
  if (words.length !== 2048) throw new Error("bip39 wordlist");
  wordsCache = words;
  return words;
}

/** Lowercase / NFKD-normalise / collapse whitespace, without touching crypto. */
export function normalizePhrase(phrase: string): string {
  return phrase.normalize("NFKD").trim().toLowerCase().split(/\s+/).join(" ");
}

/** 32-byte Ed25519 seed encoded by a 24-word English BIP-39 phrase. Secret. */
export async function phraseToSeed(phrase: string): Promise<Uint8Array> {
  const normalized = normalizePhrase(phrase);
  const words = await loadWordlist();
  return entropyOf(normalized, words);
}

function wordIndex(words: string[]): Map<string, number> {
  if (!indexCache) {
    indexCache = new Map(words.map((word, i) => [word, i] as const));
  }
  return indexCache;
}

function entropyOf(phrase: string, words: string[]): Uint8Array {
  const parts = phrase.length > 0 ? phrase.split(" ") : [];
  if (parts.length !== 24) {
    throw new Error("Your recovery phrase must be exactly 24 words.");
  }
  const lookup = wordIndex(words);
  // 24 words × 11 bits = 264 bits = 256 entropy + 8 checksum.
  const bits = new Uint8Array(33);
  let pos = 0;
  for (const word of parts) {
    const i = lookup.get(word);
    if (i === undefined) {
      throw new Error(`"${word}" isn't a recovery word — check for a typo.`);
    }
    for (let b = 10; b >= 0; b -= 1) {
      if ((i >> b) & 1) bits[pos >> 3] |= 0x80 >> (pos & 7);
      pos += 1;
    }
  }
  const entropy = bits.slice(0, 32);
  if (bits[32] !== sha256(entropy)[0]) {
    throw new Error(
      "That recovery phrase doesn't look valid — please check for a typo or a wrong word.",
    );
  }
  return entropy;
}

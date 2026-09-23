import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { webcrypto } from "node:crypto";

// The browser build relies on the global WebCrypto object; Node <19 keeps it
// behind node:crypto. Polyfill before any wallet/keys code runs (it only uses
// crypto inside functions, so import order is safe).
if (!globalThis.crypto) (globalThis as Record<string, unknown>).crypto = webcrypto;

const wordlistText = await readFile(new URL("../public/bip39.txt", import.meta.url), "utf8");
// loadWordlist fetches "/bip39.txt", the browser public path. Serve it from disk.
globalThis.fetch = (async () => new Response(wordlistText)) as typeof fetch;

import {
  addressFromMnemonic,
  createMnemonic,
  entropyToMnemonic,
  importMnemonic,
  mnemonicToSeed,
} from "../src/lib/wallet/keys";

const WORDS = wordlistText.trim().split(/\s+/);
const W = "abandon";
const M12 = [W, W, W, W, W, W, W, W, W, W, W, "about"].join(" ");
const M24 = [W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, "art"].join(" ");

test("BIP-39: zero 128-bit entropy maps to the 12-word 'abandon … about' vector", async () => {
  const mnemonic = await entropyToMnemonic(new Uint8Array(16), WORDS);
  assert.equal(mnemonic, M12);
  assert.equal(mnemonic.split(" ").length, 12);
});

test("BIP-39: zero 256-bit entropy maps to the 24-word 'abandon … art' vector", async () => {
  const mnemonic = await entropyToMnemonic(new Uint8Array(32), WORDS);
  assert.equal(mnemonic, M24);
  assert.equal(mnemonic.split(" ").length, 24);
});

test("BIP-39: 64-byte PBKDF2 output prefixes match the canonical vector", async () => {
  const derived = await mnemonicToSeed(M12);
  assert.equal(derived.length, 64);
  const first16 = Buffer.from(derived.subarray(0, 16)).toString("hex");
  assert.equal(first16, "5eb00bbddcf069084889a8ab91555681");
});

test("importMnemonic: accepts valid 12- and 24-word phrases", async () => {
  assert.equal(await importMnemonic(M12), M12);
  assert.equal(await importMnemonic(M24), M24);
  // Case / whitespace tolerance.
  assert.equal(await importMnemonic(`  ${M12.toUpperCase()} `), M12);
});

test("importMnemonic: rejects a phrase with a bad checksum (12x abandon)", async () => {
  await assert.rejects(
    importMnemonic([W, W, W, W, W, W, W, W, W, W, W, W].join(" ")),
    /doesn't look valid/,
  );
});

test("importMnemonic: rejects a single swapped word (checksum breaks)", async () => {
  // The last word of the valid vector carries 4 checksum bits - "about" to "zoo"
  // keeps the wordlist membership but breaks the checksum.
  await assert.rejects(
    importMnemonic([W, W, W, W, W, W, W, W, W, W, W, "zoo"].join(" ")),
    /doesn't look valid/,
  );
});

test("importMnemonic: rejects an unknown word with a typo hint", async () => {
  await assert.rejects(
    importMnemonic([W, W, W, W, W, W, W, W, W, W, W, "aboutt"].join(" ")),
    /typo/,
  );
});

test("createMnemonic: default is 24 words, accepts 12; both round-trip via importMnemonic", async () => {
  const m24 = await createMnemonic();
  assert.equal(m24.split(" ").length, 24);
  assert.equal(await importMnemonic(m24), m24);
  const m12 = await createMnemonic(12);
  assert.equal(m12.split(" ").length, 12);
  assert.equal(await importMnemonic(m12), m12);
});

// ---------------------------------------------------------------------------
// SLIP-0010 derivation addresses: m/44'/917'/0'/0'/i'. These are PUBLIC keys
// (wallet addresses), not secret material. The constants were produced by an
// independent node:crypto implementation and must match the Rust-side frozen
// vectors in docs/backlog/DERIVATION.md once that branch lands.
// ---------------------------------------------------------------------------
const EXPECTED_ADDRESSES_12W = {
  0: "a3de48fc8da9bbaaea5b34d08d4027019687241530ddbdddb8f8ec1541e19888",
  1: "3ec915cec87958c216b4f0ef69e73a18b60f675ce25852ac42215725d299bb8d",
  2: "86714ad9a71f44e369ae065956cd19da0d8b5777240b2912cdc7f61c5dce0f02",
} as const;

const EXPECTED_ADDRESSES_24W = {
  0: "90f71f92feb19675c9368194bb0aabfc5dc8e85b1be8827ab1fe0f4f99f6d0e8",
  1: "2301a7a3892b93aa9660035794f4caac1c353c1e4cced3bc005f130e4faead40",
  2: "4261b3b637bbd1264110c8f41c0f2a02cd7454205420a8ebcadf47993bb8c24e",
} as const;

for (const index of [0, 1, 2] as const) {
  test(`SLIP-0010 m/44'/917'/0'/0'/${index}': 12-word phrase derives the frozen address`, async () => {
    assert.equal(await addressFromMnemonic(M12, index), EXPECTED_ADDRESSES_12W[index]);
  });

  test(`SLIP-0010 m/44'/917'/0'/0'/${index}': 24-word phrase derives the frozen address`, async () => {
    assert.equal(await addressFromMnemonic(M24, index), EXPECTED_ADDRESSES_24W[index]);
  });
}

test("SLIP-0010: account index bounds are enforced", async () => {
  await assert.rejects(Promise.resolve(addressFromMnemonic(M12, -1)), /out of range/);
  await assert.rejects(Promise.resolve(addressFromMnemonic(M12, 0x80000000)), /out of range/);
});

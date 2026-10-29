import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { webcrypto } from "node:crypto";
import * as wasmModule from "@kovanica/sdk-wasm";

// The browser build relies on the global WebCrypto object; Node <19 keeps it
// behind node:crypto. Polyfill before any wallet/keys code runs (it only uses
// crypto inside functions, so import order is safe).
if (!globalThis.crypto) (globalThis as Record<string, unknown>).crypto = webcrypto;

// The wallet layer is Rust wasm now. Under `node --test` there is no `fetch`
// for a `file:` URL, so hand the module its bytes synchronously before the
// first wallet call. The browser path uses the generated async `init()`
// instead — see src/lib/wallet/wasm.ts.
const wasmBytes = await readFile(
  new URL("../vendor/kovanica-sdk-wasm/pkg/kovanica_wasm_bg.wasm", import.meta.url),
);
(globalThis as { __kovanicaWasmInit?: () => void }).__kovanicaWasmInit = () => {
  wasmModule.initSync({ module: wasmBytes });
};

// `loadWordlist()` fetches `/bip39.txt` in the browser; under `node --test`
// there is no dev server, so inject the file bytes (same idea as the wasm hook).
(globalThis as { __kovanicaWordlist?: string }).__kovanicaWordlist = await readFile(
  new URL("../public/bip39.txt", import.meta.url),
  "utf8",
);

import {
  addressFromMnemonic,
  createMnemonic,
  entropyToMnemonic,
  importMnemonic,
  mnemonicToSeed,
  signSighash,
} from "../src/lib/wallet/keys";
import { hexToKvnc } from "../src/lib/wallet/address";

const W = "abandon";
const M12 = [W, W, W, W, W, W, W, W, W, W, W, "about"].join(" ");
const M24 = [W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, W, "art"].join(" ");

test("BIP-39: zero 256-bit entropy maps to the 24-word 'abandon … art' vector", async () => {
  const mnemonic = await entropyToMnemonic(new Uint8Array(32));
  assert.equal(mnemonic, M24);
  assert.equal(mnemonic.split(" ").length, 24);
});

test("BIP-39: 64-byte PBKDF2 output prefixes match the canonical vector", async () => {
  const derived = await mnemonicToSeed(M12);
  assert.equal(derived.length, 64);
  const first16 = Buffer.from(derived.subarray(0, 16)).toString("hex");
  assert.equal(first16, "5eb00bbddcf069084889a8ab91555681");
});

test("importMnemonic: accepts a valid 24-word phrase", async () => {
  assert.equal(await importMnemonic(M24), M24);
  // Case / whitespace tolerance.
  assert.equal(await importMnemonic(`  ${M24.toUpperCase()} `), M24);
});

test("importMnemonic: rejects 12-word phrases (128 bits cannot seed Ed25519)", async () => {
  await assert.rejects(importMnemonic(M12), /exactly 24 words/);
});

test("importMnemonic: rejects a phrase with a bad checksum (23x abandon + art)", async () => {
  const bad = [W, W, W, W, W, W, W, W, W, W, W, W].join(" ");
  await assert.rejects(importMnemonic(bad), /exactly 24 words|doesn't look valid/);
});

test("importMnemonic: rejects a single swapped word (checksum breaks)", async () => {
  // The last word of the valid vector carries the checksum bits — "art" to
  // "zoo" keeps the wordlist membership but breaks the checksum.
  const swapped = [...M24.split(" ").slice(0, 23), "zoo"].join(" ");
  await assert.rejects(importMnemonic(swapped), /doesn't look valid/);
});

test("importMnemonic: rejects an unknown word with a typo hint", async () => {
  const typo = [...M24.split(" ").slice(0, 23), "artt"].join(" ");
  await assert.rejects(importMnemonic(typo), /typo/);
});

test("createMnemonic: always 24 words and round-trips via importMnemonic", async () => {
  const m = await createMnemonic();
  assert.equal(m.split(" ").length, 24);
  assert.equal(await importMnemonic(m), m);
});

// ---------------------------------------------------------------------------
// Chain recovery rule (ADR-0002): the 24-word BIP-39 entropy *is* the Ed25519
// seed. No PBKDF2 passphrase, no HD path. These public keys/addresses were
// produced by the Rust chain code (`kovanica-keystore`) and must be identical
// in Android, the CLI and the browser. One phrase = one wallet.
// ---------------------------------------------------------------------------
const CHAIN_VECTORS = [
  {
    name: "zero entropy",
    mnemonic: M24,
    pubkey: "3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29",
    address: "kvnc8d4z00xwk6jz6c4r4rgz5mcdwdjny9thrh3y8f36cpy2rz6emg54rufktdag",
  },
  {
    name: "0x7f… entropy",
    mnemonic:
      "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth title",
    pubkey: "b2a942ff4c98718bed76e255987f6d59b1a72d3b2cd2510003e6170ac63a9ffb",
    address: "kvnck2559l6vnpcchmtkuf2eslmdtxc6wtfm9nf9zqqructs4336nla58y78mdag",
  },
  {
    name: "0x11… entropy",
    mnemonic:
      "baby mass dust captain baby mass dust captain baby mass dust captain baby mass dust captain baby mass dust captain baby mass dust cake",
    pubkey: "d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737",
    address: "kvnc6p9tyvn59w62kwsndz75v90yumgzyj4hrgqkhtu9yz3n9jthsum557y80dag",
  },
] as const;

for (const v of CHAIN_VECTORS) {
  test(`chain vectors: ${v.name} derives the frozen pubkey/address`, async () => {
    const pub = await addressFromMnemonic(v.mnemonic);
    assert.equal(pub, v.pubkey);
    assert.equal(hexToKvnc(pub), v.address);
  });
}

test("one phrase = one wallet: the account index is ignored", async () => {
  const zero = await addressFromMnemonic(M24, 0);
  for (const index of [1, 2, 7, 0x80000000]) {
    assert.equal(await addressFromMnemonic(M24, index), zero);
  }
  const sighash = "ab".repeat(32);
  const sig0 = await signSighash(M24, 0, sighash);
  const sig2 = await signSighash(M24, 2, sighash);
  assert.equal(sig0, sig2);
  assert.equal(sig0.length, 128);
});

// ---------------------------------------------------------------------------
// Shared golden vectors: protocol/testvectors/vectors.json, generated from the
// Rust implementation. Only index-0 entries apply to the chain rule — the chain
// has no per-index accounts.
// ---------------------------------------------------------------------------
interface SharedVector {
  kind: string;
  name: string;
  mnemonic: string;
  passphrase: string;
  index: number;
  pubkey_hex: string;
  address: string;
}

// Standalone repo: the vector file is vendored as a fixture. Inside the
// monorepo the canonical protocol/testvectors copy is used instead.
const VECTOR_CANDIDATES = [
  new URL("./fixtures/vectors.json", import.meta.url),
  new URL("../../../protocol/testvectors/vectors.json", import.meta.url),
];

async function loadSharedVectors(): Promise<{ vectors: SharedVector[] } | null> {
  for (const candidate of VECTOR_CANDIDATES) {
    try {
      return JSON.parse(await readFile(candidate, "utf8")) as { vectors: SharedVector[] };
    } catch (e) {
      if ((e as NodeJS.ErrnoException).code !== "ENOENT") throw e;
    }
  }
  return null;
}

const sharedVectors = (await loadSharedVectors()) ?? { vectors: [] };
const vectorsMissing = sharedVectors.vectors.length === 0;

// The web `addressFromMnemonic` has no passphrase argument, so only the
// empty-passphrase cases apply here (the passphrase case is covered by Rust).
const webDerivationVectors = sharedVectors.vectors.filter(
  (v) => v.kind === "derivation" && v.passphrase === "" && v.index === 0,
);

test(
  "shared vectors: the file has derivation cases to check",
  {
    skip:
      vectorsMissing &&
      "tests/fixtures/vectors.json not vendored yet (copy protocol/testvectors/vectors.json)",
  },
  () => {
    assert.ok(webDerivationVectors.length > 0);
  },
);

for (const v of webDerivationVectors) {
  test(`shared vectors: ${v.name} derives the pinned pubkey/address`, async () => {
    const pub = await addressFromMnemonic(v.mnemonic);
    assert.equal(pub, v.pubkey_hex);
    assert.equal(hexToKvnc(pub), v.address);
  });
}

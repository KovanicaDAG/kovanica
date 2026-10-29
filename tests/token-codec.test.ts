import { test } from "node:test";
import assert from "node:assert/strict";
import {
  balanceOf,
  decodeToken,
  formatTokenAmount,
  isRenderableIcon,
  STATE_SLOT_HEX,
} from "../src/lib/tokens/codec";

/**
 * Byte-for-byte builder for the `kovanica-token` singleton state.
 *
 * Mirrors `crates/kovanica-token/src/lib.rs` (`Token`) under bincode 1.3's
 * default fixint + little-endian config: `u64` lengths/counts, bare 32-byte
 * arrays, `u128` LE amounts, `bool` as one byte.
 */
function u64(value: number | bigint): Uint8Array {
  const out = new Uint8Array(8);
  new DataView(out.buffer).setBigUint64(0, BigInt(value), true);
  return out;
}

function u128(value: bigint): Uint8Array {
  const out = new Uint8Array(16);
  let rest = value;
  for (let i = 0; i < 16; i += 1) {
    out[i] = Number(rest & 0xffn);
    rest >>= 8n;
  }
  return out;
}

function text(value: string): Uint8Array {
  const bytes = new TextEncoder().encode(value);
  return concat([u64(bytes.length), bytes]);
}

function concat(parts: Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((sum, part) => sum + part.length, 0));
  let offset = 0;
  for (const part of parts) {
    out.set(part, offset);
    offset += part.length;
  }
  return out;
}

const OWNER = new Uint8Array(32).fill(0x11);
const OTHER = new Uint8Array(32).fill(0x22);
const SUPPLY = 1_000_000_000_000n; // 1000.000000000 tokens at 9 decimals

type Layout = {
  name: string;
  symbol: string;
  decimals: number;
  supply: bigint;
  balances: [Uint8Array, bigint][];
  allowances?: [Uint8Array, Uint8Array, bigint][];
  paused?: boolean;
  /** `undefined` writes the v1 layout (no tail at all). */
  icon?: string;
};

function encodeToken(layout: Layout): Uint8Array {
  const parts: Uint8Array[] = [
    text(layout.name),
    text(layout.symbol),
    Uint8Array.of(layout.decimals),
    u128(layout.supply),
    OWNER,
    OWNER,
    u64(layout.balances.length),
  ];
  for (const [address, amount] of layout.balances) parts.push(address, u128(amount));

  const allowances = layout.allowances ?? [];
  parts.push(u64(allowances.length));
  for (const [from, spender, amount] of allowances) parts.push(from, spender, u128(amount));

  parts.push(Uint8Array.of(layout.paused ? 1 : 0));
  if (layout.icon !== undefined) parts.push(text(layout.icon));
  return concat(parts);
}

test("state slot matches the hash the Android app pins", () => {
  // blake3("kvnc/v1/state") — the host namespaces contract keys with this.
  assert.equal(STATE_SLOT_HEX, "d490059b20a51cd31ad8724021231c581d67a62d2946f7f2a4405dd8ae738df1");
});

test("decodes a v2 token, including the icon tail", () => {
  const icon = "data:image/png;base64,iVBORw0KGgo=";
  const raw = encodeToken({
    name: "Kovanica Gold",
    symbol: "KGOLD",
    decimals: 9,
    supply: SUPPLY,
    balances: [[OWNER, SUPPLY]],
    allowances: [[OWNER, OTHER, 5n]],
    icon,
  });

  const token = decodeToken(raw);
  assert.ok(token, "decodes");
  assert.equal(token.name, "Kovanica Gold");
  assert.equal(token.symbol, "KGOLD");
  assert.equal(token.decimals, 9);
  assert.equal(token.totalSupply, SUPPLY);
  assert.equal(token.owner, "11".repeat(32));
  assert.equal(token.minter, "11".repeat(32));
  assert.equal(token.paused, false);
  assert.equal(token.icon, icon);
  assert.equal(balanceOf(token, "11".repeat(32)), SUPPLY);
  assert.equal(balanceOf(token, "22".repeat(32)), 0n);
});

test("decodes the v1 layout (no icon) with an empty icon", () => {
  const raw = encodeToken({
    name: "Old Token",
    symbol: "OLD",
    decimals: 2,
    supply: 500n,
    balances: [[OWNER, 500n]],
  });

  const token = decodeToken(raw);
  assert.ok(token);
  assert.equal(token.symbol, "OLD");
  assert.equal(token.icon, "");
  assert.equal(token.totalSupply, 500n);
});

test("rejects foreign bytes instead of inventing a token", () => {
  assert.equal(decodeToken(new Uint8Array()), null);
  assert.equal(decodeToken(new Uint8Array(64)), null);

  // A blank symbol cannot be a token.
  const blank = encodeToken({
    name: "X",
    symbol: "  ",
    decimals: 0,
    supply: 1n,
    balances: [],
  });
  assert.equal(decodeToken(blank), null);

  // A balance map that claims more entries than the bytes hold.
  const truncated = encodeToken({
    name: "X",
    symbol: "X",
    decimals: 0,
    supply: 1n,
    balances: [[OWNER, 1n]],
  }).subarray(0, 40);
  assert.equal(decodeToken(truncated), null);
});

test("only https and image data URIs are renderable", () => {
  assert.equal(isRenderableIcon("https://cdn.example/i.png"), true);
  assert.equal(isRenderableIcon("data:image/png;base64,AAAA"), true);
  assert.equal(isRenderableIcon("http://cdn.example/i.png"), false);
  assert.equal(isRenderableIcon("javascript:alert(1)"), false);
  assert.equal(isRenderableIcon(""), false);
});

test("formats token amounts in token units", () => {
  assert.equal(formatTokenAmount(0n, 9), "0");
  assert.equal(formatTokenAmount(1_000_000_000n, 9), "1");
  assert.equal(formatTokenAmount(1_500_000_000n, 9), "1.5");
  assert.equal(formatTokenAmount(1n, 9), "0.000000001");
  assert.equal(formatTokenAmount(42n, 0), "42");
  assert.equal(formatTokenAmount(-1_500_000_000n, 9), "-1.5");
});

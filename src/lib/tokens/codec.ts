/**
 * KVP-102 token state decoder (web).
 *
 * Token balances are **not** native KUNA and the node has no `token_balance`
 * view RPC, so the only read path is the contract's own singleton storage slot,
 * fetched with `kovanica_getStorageAt` and decoded here.
 *
 * The byte layout is owned by `crates/kovanica-token/src/lib.rs` (`Token`) and
 * `crates/kovanica-common/src/lib.rs` (`Address = [u8; 32]`,
 * `Amount = u128`), written by bincode 1.3 with fixint + little-endian encoding
 * and `u64` lengths:
 *
 * ```
 * name          String                     u64le len + utf8 bytes
 * symbol        String                     u64le len + utf8 bytes
 * decimals      u8                         1 byte
 * total_supply  u128                       16 bytes LE
 * owner         [u8; 32]                   32 bytes
 * minter        [u8; 32]                   32 bytes
 * balances      BTreeMap<[u8;32], u128>    u64le count + count × (32 + 16)
 * allowances    BTreeMap<([u8;32],[u8;32]), u128>
 *                                          u64le count + count × (32 + 32 + 16)
 * paused        bool                       1 byte
 * icon          String                     u64le len + utf8 bytes (v2 only)
 * ```
 *
 * `icon` was appended after the first release, so state written by the older
 * contract simply ends after `paused`; {@link decodeToken} accepts both and
 * reports an empty icon for the short layout.
 */
import { blake3 } from "@noble/hashes/blake3.js";

/** Raw key the contract writes its state under. */
const STATE_KEY = "kvnc/v1/state";

/**
 * Storage slot the host actually stores that key at: the host namespaces every
 * contract key as `blake3(raw_key)`.
 */
export const STATE_SLOT_HEX = bytesToHex(blake3(new TextEncoder().encode(STATE_KEY)));

export type TokenState = {
  name: string;
  symbol: string;
  decimals: number;
  /** Opening + minted − burned, in the token's smallest unit. */
  totalSupply: bigint;
  /** Lowercase 64-hex, no `0x`. */
  owner: string;
  /** Lowercase 64-hex, no `0x`. */
  minter: string;
  /** Holder (lowercase 64-hex) → balance, exactly as stored on chain. */
  balances: Map<string, bigint>;
  paused: boolean;
  /** `https://…` or `data:image/…`; empty when the token has no artwork. */
  icon: string;
};

/** A token as the wallet API returns it: decoded state plus the caller's balance. */
export type ApiToken = {
  /** `0x` + 64 hex characters: the token contract address. */
  address: string;
  name: string;
  symbol: string;
  decimals: number;
  /** Total supply in the token's smallest unit, as a decimal string. */
  total_supply: string;
  /** Number of addresses with a non-zero balance. */
  holders: number;
  paused: boolean;
  /** `https://…` or `data:image/…`; empty when the token has no artwork. */
  icon: string;
  /** Balance of the requested holder, in atoms; `null` when none was requested. */
  balance: string | null;
};

/** A token always has a symbol; names, artwork and supply are free to vary. */
const MAX_TEXT_BYTES = 512;
const MAX_ICON_BYTES = 8 * 1024;
const BALANCE_ENTRY_BYTES = 32 + 16;
const ALLOWANCE_ENTRY_BYTES = 32 + 32 + 16;

class Reader {
  private pos = 0;

  constructor(private readonly bytes: Uint8Array) {}

  /** Bytes left after the last read; 0 means the value had no icon tail. */
  remaining(): number {
    return this.bytes.length - this.pos;
  }

  private want(n: number): boolean {
    return n >= 0 && this.pos + n <= this.bytes.length;
  }

  u8(): number | null {
    if (!this.want(1)) return null;
    return this.bytes[this.pos++]!;
  }

  bool(): boolean | null {
    const value = this.u8();
    return value === null ? null : value !== 0;
  }

  u64(): bigint | null {
    if (!this.want(8)) return null;
    let value = 0n;
    for (let i = 7; i >= 0; i -= 1) value = (value << 8n) | BigInt(this.bytes[this.pos + i]!);
    this.pos += 8;
    return value;
  }

  /** A collection length, clamped to what could physically fit in the rest. */
  count(entryBytes: number): number | null {
    const raw = this.u64();
    if (raw === null) return null;
    if (raw > BigInt(Math.floor((this.bytes.length - this.pos) / entryBytes))) return null;
    return Number(raw);
  }

  u128(): bigint | null {
    if (!this.want(16)) return null;
    let value = 0n;
    for (let i = 15; i >= 0; i -= 1) value = (value << 8n) | BigInt(this.bytes[this.pos + i]!);
    this.pos += 16;
    return value;
  }

  bytesOf(n: number): Uint8Array | null {
    if (!this.want(n)) return null;
    const out = this.bytes.subarray(this.pos, this.pos + n);
    this.pos += n;
    return out;
  }

  text(maxBytes: number): string | null {
    const len = this.u64();
    if (len === null || len > BigInt(maxBytes)) return null;
    const raw = this.bytesOf(Number(len));
    if (raw === null) return null;
    return decodeUtf8(raw);
  }
}

/** Strict UTF-8 decode; `null` for invalid sequences (a real token never has any). */
function decodeUtf8(raw: Uint8Array): string | null {
  try {
    return new TextDecoder("utf-8", { fatal: true }).decode(raw);
  } catch {
    return null;
  }
}

/** Decode a raw `kovanica_getStorageAt` value, or `null` when it is not a token. */
export function decodeToken(raw: Uint8Array): TokenState | null {
  const r = new Reader(raw);

  const name = r.text(MAX_TEXT_BYTES);
  if (name === null) return null;
  const symbol = r.text(MAX_TEXT_BYTES);
  if (symbol === null) return null;
  const decimals = r.u8();
  if (decimals === null) return null;
  const totalSupply = r.u128();
  if (totalSupply === null) return null;
  const owner = r.bytesOf(32);
  if (owner === null) return null;
  const minter = r.bytesOf(32);
  if (minter === null) return null;

  const balanceCount = r.count(BALANCE_ENTRY_BYTES);
  if (balanceCount === null) return null;
  const balances = new Map<string, bigint>();
  for (let i = 0; i < balanceCount; i += 1) {
    const holder = r.bytesOf(32);
    if (holder === null) return null;
    const amount = r.u128();
    if (amount === null) return null;
    const key = bytesToHex(holder);
    // The on-chain map is deduplicated; a repeat means these are not its bytes.
    if (balances.has(key)) return null;
    balances.set(key, amount);
  }

  // Allowances are decoded only to prove the tail is well-formed.
  const allowanceCount = r.count(ALLOWANCE_ENTRY_BYTES);
  if (allowanceCount === null) return null;
  for (let i = 0; i < allowanceCount; i += 1) {
    if (r.bytesOf(32) === null || r.bytesOf(32) === null || r.u128() === null) return null;
  }

  const paused = r.bool();
  if (paused === null) return null;

  const icon = r.remaining() === 0 ? "" : r.text(MAX_ICON_BYTES);
  if (icon === null) return null;

  if (symbol.trim() === "") return null;

  return {
    name,
    symbol,
    decimals,
    totalSupply,
    owner: bytesToHex(owner),
    minter: bytesToHex(minter),
    balances,
    paused,
    icon: isRenderableIcon(icon) ? icon : "",
  };
}

/** Only these can be handed to an `<img src>` without letting a token inject markup. */
export function isRenderableIcon(icon: string): boolean {
  return icon.startsWith("https://") || icon.startsWith("data:image/");
}

/** Balance of `holderHex` (lowercase 64-hex, no `0x`); `0n` when absent. */
export function balanceOf(token: TokenState, holderHex: string): bigint {
  return token.balances.get(holderHex.toLowerCase()) ?? 0n;
}

/**
 * Atoms → display string with exactly `decimals` fraction digits (trailing
 * zeros trimmed, never in scientific notation).
 */
export function formatTokenAmount(atoms: bigint, decimals: number): string {
  const d = Math.min(Math.max(decimals, 0), 30);
  const negative = atoms < 0n;
  const abs = negative ? -atoms : atoms;
  const base = 10n ** BigInt(d);
  const whole = abs / base;
  const frac = (abs % base).toString().padStart(d, "0").replace(/0+$/, "");
  return `${negative ? "-" : ""}${whole.toString()}${frac ? `.${frac}` : ""}`;
}

/**
 * Display string → base units, or `null` when the input is not a positive
 * decimal with at most `decimals` fraction digits.
 *
 * Unlike `parseKvnc` this stays in `bigint` end to end: token supplies are
 * `u128` and a `number` round-trip silently loses the low digits.
 */
export function parseTokenAmount(input: string, decimals: number): bigint | null {
  const raw = input.trim();
  if (!/^\d+(\.\d+)?$/.test(raw)) return null;
  const d = Math.min(Math.max(decimals, 0), 30);
  const [whole, frac = ""] = raw.split(".");
  if (frac.length > d) return null;
  const fracAtoms = d === 0 ? 0n : BigInt(frac.padEnd(d, "0"));
  const atoms = BigInt(whole) * 10n ** BigInt(d) + fracAtoms;
  return atoms > 0n ? atoms : null;
}

export function bytesToHex(bytes: Uint8Array): string {
  let out = "";
  for (const byte of bytes) out += byte.toString(16).padStart(2, "0");
  return out;
}

/** Parse `0x…` hex into bytes; `null` when malformed. */
export function hexToBytes(hex: string): Uint8Array | null {
  const s = hex.trim().replace(/^0x/i, "");
  if (s === "" || s.length % 2 !== 0 || !/^[0-9a-fA-F]+$/.test(s)) return null;
  const out = new Uint8Array(s.length / 2);
  for (let i = 0; i < out.length; i += 1) out[i] = parseInt(s.slice(i * 2, i * 2 + 2), 16);
  return out;
}

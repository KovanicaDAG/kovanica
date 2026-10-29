/**
 * KVP-102 AMM pool decoder (web).
 *
 * A token contract may expose a constant-product pool between its own token
 * balance and the native KUNA balance it holds. The pool is **contract state**,
 * not an RPC view: it lives in the same storage namespace as the token, under
 * the raw key `kvnc/v1/pool`, which the host stores at `blake3(raw_key)`.
 *
 * Layout is owned by `crates/kovanica-token/src/lib.rs` (`Pool`) and written by
 * bincode 1.3 with fixint + little-endian integers and no length prefixes:
 *
 * ```
 * kuna_reserve   u128   16 bytes LE
 * token_reserve  u128   16 bytes LE
 * fee_bps        u16     2 bytes LE
 * lp_total       u128   16 bytes LE
 * paused         bool    1 byte
 * ```
 *
 * Exactly 51 bytes; anything else is not a pool (a token with no pool has no
 * bytes at this slot at all).
 */
import { blake3 } from "@noble/hashes/blake3.js";
import { bytesToHex } from "./codec";

/** Raw key the contract writes its singleton pool under. */
const POOL_KEY = "kvnc/v1/pool";

/** Storage slot the host actually stores that key at (`blake3(raw_key)`). */
export const POOL_SLOT_HEX = bytesToHex(blake3(new TextEncoder().encode(POOL_KEY)));

/** Largest fee the contract accepts, in basis points (10%). */
export const MAX_FEE_BPS = 1_000;

/** Bincode size of a `Pool`: 16 + 16 + 2 + 16 + 1. */
export const POOL_BYTES = 51;

export type PoolState = {
  /** Native KUNA atoms held by the contract. */
  kunaReserve: bigint;
  /** Token base units held by the contract (mirrors its token balance). */
  tokenReserve: bigint;
  /** Swap fee in basis points, retained in the reserve for LP yield. */
  feeBps: number;
  /** Sum of all LP shares. */
  lpTotal: bigint;
  /** Owner switch that stops both swap directions. */
  paused: boolean;
};

/**
 * A pool row as `/api/pools` returns it: token metadata plus decoded reserves,
 * all integers as decimal strings so nothing is lost to a JS number.
 */
export type ApiPool = {
  /** `0x` + 64 hex: the token contract address that owns the pool. */
  address: string;
  name: string;
  symbol: string;
  decimals: number;
  icon: string;
  /** Pool or token owner switch — either one stops swaps. */
  paused: boolean;
  fee_bps: number;
  /** Native KUNA atoms backing the pool, decimal string. */
  kuna_reserve: string;
  /** Token base units held by the contract, decimal string. */
  token_reserve: string;
  /** Sum of all LP shares, decimal string. */
  lp_total: string;
  /** KUNA atoms per whole token at the current reserve ratio, or `null`. */
  price_kuna: number | null;
};

function readU128(bytes: Uint8Array, offset: number): bigint {
  let value = 0n;
  for (let i = 15; i >= 0; i -= 1) value = (value << 8n) | BigInt(bytes[offset + i]!);
  return value;
}

/** Decode a raw `kovanica_getStorageAt` value, or `null` when it is not a pool. */
export function decodePool(raw: Uint8Array): PoolState | null {
  if (raw.length !== POOL_BYTES) return null;
  const feeBps = raw[32]! | (raw[33]! << 8);
  if (feeBps > MAX_FEE_BPS) return null;
  const paused = raw[50]!;
  if (paused > 1) return null;
  return {
    kunaReserve: readU128(raw, 0),
    tokenReserve: readU128(raw, 16),
    feeBps,
    lpTotal: readU128(raw, 34),
    paused: paused === 1,
  };
}

/**
 * Constant-product quote with the fee staying in the reserve.
 *
 * Mirrors `kovanica_token::quote`: `in_after = amount_in * (10_000 - fee_bps) /
 * 10_000`, then `out = in_after * r_out / (r_in + in_after)`, floored.
 * Returns `null` for a zero input/reserve or an invalid fee.
 */
export function quote(
  amountIn: bigint,
  reserveIn: bigint,
  reserveOut: bigint,
  feeBps: number,
): bigint | null {
  if (feeBps < 0 || feeBps > MAX_FEE_BPS) return null;
  if (amountIn <= 0n || reserveIn <= 0n || reserveOut <= 0n) return null;
  const keep = 10_000n - BigInt(feeBps);
  const inAfter = (amountIn * keep) / 10_000n;
  if (inAfter === 0n) return 0n;
  return (inAfter * reserveOut) / (reserveIn + inAfter);
}

/** Spot price of one whole token, in KUNA atoms (before fee and slippage). */
export function spotPriceKuna(pool: PoolState, tokenDecimals: number): number | null {
  if (pool.tokenReserve === 0n) return null;
  const oneToken = 10n ** BigInt(Math.min(Math.max(tokenDecimals, 0), 30));
  // Keep the intermediate in bigint space; the display value is a JS number.
  const numerator = pool.kunaReserve * oneToken;
  const whole = numerator / pool.tokenReserve;
  const frac = numerator % pool.tokenReserve;
  return Number(whole) + Number(frac) / Number(pool.tokenReserve);
}

/**
 * Price impact of a quote, in basis points, measured against the spot rate.
 * `null` when the spot output is zero.
 */
export function priceImpactBps(
  amountIn: bigint,
  reserveIn: bigint,
  reserveOut: bigint,
  actualOut: bigint,
): number | null {
  if (reserveIn <= 0n) return null;
  const spotOut = (amountIn * reserveOut) / reserveIn;
  if (spotOut === 0n) return null;
  const impact = 10_000n - (actualOut * 10_000n) / spotOut;
  return Number(impact < 0n ? 0n : impact);
}

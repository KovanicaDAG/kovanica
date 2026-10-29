/**
 * On-chain token discovery for the web wallet.
 *
 * There is no token registry: a KVP-102 token exists because a `Deploy`
 * published its bytecode and a `token_create` call emitted `token_created`.
 * So discovery is a log scan:
 *
 * 1. `kovanica_getLogs` for topic `token_created`, walked in chunks because the
 *    node caps a single query at 500 heights and 200 ms of scanning;
 * 2. `kovanica_getStorageAt` of each contract's singleton slot, decoded with the
 *    shared codec.
 *
 * Results are cached in memory and the scan is incremental: the cache remembers
 * the highest height already scanned, so a warm server only looks at new blocks.
 */
import {
  balanceOf,
  decodeToken,
  hexToBytes,
  STATE_SLOT_HEX,
  type ApiToken,
  type TokenState,
} from "../tokens/codec";
import { parseAddr } from "../wallet/address";

export type { ApiToken };

const RPC_URL = "http://127.0.0.1:8545/rpc";

/** Heights per `getLogs` query — below the node's 500 cap, leaving retry room. */
const CHUNK_HEIGHTS = 400;

/** Hard stop for pathological scans (and for a node that keeps saying "retry"). */
const MAX_CHUNKS = 5_000;

const CACHE_TTL_MS = 30_000;

type Entry = { height: number; address: string; state: TokenState };

type Cache = {
  at: number;
  /** Highest height already scanned with `token_created` in the filter. */
  scannedTo: number;
  entries: Entry[];
};

let cache: Cache | null = null;

/** The scan currently running, if any (see `listTokens`). */
let inflight: Promise<{ entries: Entry[] }> | null = null;

async function rpc<T>(method: string, params: unknown[] = []): Promise<T> {
  const res = await fetch(RPC_URL, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", method, params, id: 1 }),
  });
  if (!res.ok) throw new Error(`RPC HTTP ${res.status}`);
  const data = (await res.json()) as { result?: T; error?: { message?: string } };
  if (data.error) throw new Error(data.error.message ?? "RPC error");
  return data.result as T;
}

function hexHeight(value: string | number | bigint): string {
  return `0x${BigInt(value).toString(16)}`;
}

async function latestHeight(): Promise<number> {
  const hex = await rpc<string>("kovanica_blockNumber");
  return Number(BigInt(hex));
}

/**
 * Contract addresses that emitted `token_created` in `[from, to]`.
 *
 * The node prints addresses in their canonical `kvnc…dag` form, which is also
 * what the read RPCs accept on serving chains (some chains reject legacy hex),
 * so the value is kept verbatim and only rendered as hex for the API response.
 */
async function createdIn(from: number, to: number): Promise<{ address: string; height: number }[]> {
  if (from > to) return [];
  const logs = await rpc<
    { address: string; blockNumber: string; topics: string[]; data: string }[]
  >("kovanica_getLogs", [{ fromBlock: hexHeight(from), toBlock: hexHeight(to), topics: ["token_created"] }]);

  const seen = new Set<string>();
  const out: { address: string; height: number }[] = [];
  for (const log of logs) {
    const address = log.address.trim();
    if (seen.has(address)) continue;
    seen.add(address);
    out.push({ address, height: Number(BigInt(log.blockNumber ?? "0x0")) });
  }
  return out;
}

/**
 * Scan `[from, to]`, halving on the node's range/scan-timeout rejections. A
 * chunk that still fails at one height is skipped rather than failing the page.
 */
async function scanRange(from: number, to: number): Promise<{ address: string; height: number }[]> {
  const found: { address: string; height: number }[] = [];
  let start = from;
  let chunks = 0;

  while (start <= to) {
    if (chunks >= MAX_CHUNKS) break;
    chunks += 1;
    const end = Math.min(start + CHUNK_HEIGHTS - 1, to);
    try {
      found.push(...(await createdIn(start, end)));
      start = end + 1;
    } catch {
      // Too wide or too slow: halve and retry the same start, down to one height.
      const span = end - start + 1;
      if (span <= 1) {
        start += 1;
        continue;
      }
      const half = Math.max(1, Math.floor(span / 2));
      found.push(...(await scanRange(start, start + half - 1)));
      start += half;
    }
  }

  return found;
}

async function readToken(address: string): Promise<TokenState | null> {
  try {
    const raw = await rpc<string>("kovanica_getStorageAt", [address, STATE_SLOT_HEX]);
    const bytes = hexToBytes(raw);
    if (!bytes || bytes.length === 0) return null;
    return decodeToken(bytes);
  } catch {
    return null;
  }
}

async function collect(): Promise<{ entries: Entry[] }> {
  const height = await latestHeight();

  // A warm cache that already covers the tip is served as-is for a short while.
  if (cache && cache.scannedTo >= height && Date.now() - cache.at < CACHE_TTL_MS) {
    return { entries: cache.entries };
  }
  // Chain reset / reorg back below what we scanned: start over.
  const warm = cache && cache.scannedTo <= height ? cache : null;
  const from = warm ? warm.scannedTo + 1 : 0;

  const found = from > height ? [] : await scanRange(from, height);
  const decoded = await Promise.all(
    found.map(async (hit) => {
      const state = await readToken(hit.address);
      return state ? { height: hit.height, address: hit.address, state } : null;
    }),
  );

  // One entry per contract: a newer `token_created` for the same address wins.
  const byAddress = new Map<string, Entry>();
  for (const entry of warm?.entries ?? []) byAddress.set(entry.address, entry);
  for (const entry of decoded) {
    if (!entry) continue;
    const previous = byAddress.get(entry.address);
    if (!previous || entry.height >= previous.height) byAddress.set(entry.address, entry);
  }

  const entries = [...byAddress.values()].sort((a, b) => b.height - a.height);

  // Balances, supply, `paused` and the icon all change without a new
  // `token_created`, so a warm entry must not keep the state it was discovered
  // with: refreshing only the newly found contracts froze every balance and
  // holder count at discovery time. One storage read per token per rebuild.
  const fresh = await Promise.all(
    entries.map(async (entry) => {
      const state = await readToken(entry.address);
      return state ? { ...entry, state } : entry;
    }),
  );

  cache = { at: Date.now(), scannedTo: height, entries: fresh };
  return { entries: fresh };
}

/**
 * Every readable token on the local chain, newest first.
 *
 * `holderHex` (lowercase 64-hex, no `0x`) adds the holder's balance to each row.
 */
export async function listTokens(holderHex?: string): Promise<ApiToken[]> {
  // A cold server scans the whole chain; concurrent page requests share it
  // instead of each starting the same scan.
  const { entries } = await (inflight ??= collect().finally(() => {
    inflight = null;
  }));
  const holder = holderHex?.toLowerCase() ?? null;
  return entries.flatMap((entry) => {
    // The node prints `kvnc…dag`; the API and the UI keys use 64-hex.
    const hex = parseAddr(entry.address);
    if (!hex) return [];
    return [
      {
        address: `0x${hex}`,
        name: entry.state.name,
        symbol: entry.state.symbol,
        decimals: entry.state.decimals,
        total_supply: entry.state.totalSupply.toString(),
        holders: [...entry.state.balances.values()].filter((v) => v > 0n).length,
        paused: entry.state.paused,
        icon: entry.state.icon,
        balance: holder ? balanceOf(entry.state, holder).toString() : null,
      },
    ];
  });
}

/**
 * KVP-102 AMM pools — read-only explorer plus an exact quote calculator.
 *
 * A pool is contract state, not an RPC view: `/api/pools` scans the token
 * contracts and decodes each `kvnc/v1/pool` slot. Swapping is a signed
 * `token_buy` / `token_sell` call, so this page deliberately stops at quoting —
 * the same formula the contract runs, so the numbers match on execution.
 */
import { useEffect, useMemo, useState } from "react";
import { ArrowDownUp, Copy, Info, RefreshCw, Waves } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { api } from "@/lib/api/client";
import { formatTokenAmount, parseTokenAmount } from "@/lib/tokens/codec";
import { priceImpactBps, quote, type ApiPool } from "@/lib/tokens/pool";

/** Native KVNC always has 9 decimals. */
const KVNC_DECIMALS = 9;

export function PoolView() {
  const [pools, setPools] = useState<ApiPool[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [dir, setDir] = useState<"buy" | "sell">("buy");
  const [amount, setAmount] = useState("1");
  const [loading, setLoading] = useState(false);

  async function load() {
    setLoading(true);
    setError(null);
    try {
      const rows = await api<ApiPool[]>("/api/pools");
      setPools(rows);
      setSelected((current) => current ?? rows[0]?.address ?? null);
    } catch (e) {
      setPools([]);
      setError(e instanceof Error ? e.message : "Failed to load pools");
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void load();
    // Load once on mount; the button re-runs it.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const pool = useMemo(
    () => pools?.find((p) => p.address === selected) ?? pools?.[0] ?? null,
    [pools, selected],
  );

  const kunaReserve = pool ? BigInt(pool.kuna_reserve) : 0n;
  const tokenReserve = pool ? BigInt(pool.token_reserve) : 0n;
  const inDecimals = dir === "buy" ? KVNC_DECIMALS : (pool?.decimals ?? 0);
  const outDecimals = dir === "buy" ? (pool?.decimals ?? 0) : KVNC_DECIMALS;
  const atoms = pool ? parseTokenAmount(amount, inDecimals) : null;
  const out =
    pool && atoms !== null
      ? dir === "buy"
        ? quote(atoms, kunaReserve, tokenReserve, pool.fee_bps)
        : quote(atoms, tokenReserve, kunaReserve, pool.fee_bps)
      : null;
  const impact =
    pool && atoms !== null && out !== null && out > 0n
      ? priceImpactBps(
          atoms,
          dir === "buy" ? kunaReserve : tokenReserve,
          dir === "buy" ? tokenReserve : kunaReserve,
          out,
        )
      : null;

  async function copyAddress(address: string) {
    try {
      await navigator.clipboard.writeText(address);
      toast.success("Contract address copied");
    } catch {
      toast.error("Could not copy");
    }
  }

  return (
    <div className="mx-auto flex w-full max-w-5xl flex-col gap-6 px-4 py-6 md:px-6 md:py-8">
      <header className="flex items-start justify-between gap-3">
        <div>
          <p className="font-mono text-[11px] tracking-brand text-blue uppercase">
            KVP-102 · RFC-002
          </p>
          <h1 className="mt-1 font-display text-3xl tracking-tight text-fg italic md:text-4xl">
            Liquidity pools
          </h1>
          <p className="mt-2 max-w-2xl text-sm leading-relaxed text-muted">
            Every token contract can hold a constant-product pool between its own token and
            KVNC. Reserves, fee and LP supply are read straight from contract storage.
          </p>
        </div>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => void load()}
          disabled={loading}
        >
          <RefreshCw className={loading ? "size-3.5 animate-spin" : "size-3.5"} /> Refresh
        </Button>
      </header>

      {error && (
        <p className="rounded-xl border border-red/30 bg-red/5 p-3 text-sm text-red">{error}</p>
      )}

      {pools !== null && pools.length === 0 && !error && (
        <div className="rounded-xl border border-border bg-surface p-6">
          <div className="flex items-center gap-2 text-gold">
            <Waves className="size-4" />
            <p className="text-sm font-medium">No pools initialized yet</p>
          </div>
          <p className="mt-2 text-sm leading-relaxed text-muted">
            A token becomes a pool only after its owner or minter calls{" "}
            <code className="rounded bg-surface-2 px-1.5 py-0.5 font-mono text-xs text-fg">
              token_init_pool
            </code>{" "}
            on the token contract with an initial KVNC reserve, token reserve and fee (max 10%).
            The token keeps working as a normal KVP-102 token until then.
          </p>
        </div>
      )}

      {pools !== null && pools.length > 0 && (
        <div className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_360px]">
          <div className="flex min-w-0 flex-col gap-3">
            {pools.map((row) => {
              const on = pool?.address === row.address;
              return (
                <button
                  key={row.address}
                  type="button"
                  onClick={() => setSelected(row.address)}
                  className={
                    on
                      ? "rounded-xl border border-gold/50 bg-gold/5 p-4 text-left"
                      : "rounded-xl border border-border bg-surface p-4 text-left transition-colors hover:border-border hover:bg-surface-2"
                  }
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="flex min-w-0 items-center gap-2">
                      {row.icon ? (
                        <img src={row.icon} alt="" className="size-6 rounded-full object-cover" />
                      ) : (
                        <Waves className="size-5 text-blue" />
                      )}
                      <div className="min-w-0">
                        <p className="truncate font-display text-lg tracking-tight text-fg">
                          {row.symbol} / KVNC
                        </p>
                        <p className="truncate font-mono text-[11px] text-subtle">{row.name}</p>
                      </div>
                    </div>
                    {row.paused && (
                      <span className="rounded-full border border-red/40 bg-red/10 px-2 py-0.5 font-mono text-[10px] text-red uppercase">
                        Paused
                      </span>
                    )}
                  </div>

                  <dl className="mt-3 grid grid-cols-2 gap-x-4 gap-y-1 font-mono text-[11px]">
                    <dt className="text-subtle">KVNC reserve</dt>
                    <dd className="text-right text-fg">
                      {formatTokenAmount(BigInt(row.kuna_reserve), KVNC_DECIMALS)}
                    </dd>
                    <dt className="text-subtle">{row.symbol} reserve</dt>
                    <dd className="text-right text-fg">
                      {formatTokenAmount(BigInt(row.token_reserve), row.decimals)}
                    </dd>
                    <dt className="text-subtle">Fee</dt>
                    <dd className="text-right text-fg">{(row.fee_bps / 100).toFixed(2)}%</dd>
                    <dt className="text-subtle">Spot price</dt>
                    <dd className="text-right text-gold">
                      {row.price_kuna === null
                        ? "—"
                        : `${row.price_kuna.toLocaleString(undefined, {
                            maximumFractionDigits: 9,
                          })} KVNC`}
                    </dd>
                  </dl>
                </button>
              );
            })}
          </div>

          {pool && (
            <div className="flex min-w-0 flex-col gap-4 self-start rounded-xl border border-border bg-surface p-4">
              <div className="flex items-center justify-between gap-2">
                <p className="text-xs font-medium uppercase tracking-wide text-muted">Quote</p>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  aria-label="Copy contract address"
                  onClick={() => void copyAddress(pool.address)}
                >
                  <Copy className="size-3.5" />
                </Button>
              </div>

              <div className="flex gap-2">
                <Button
                  type="button"
                  variant={dir === "buy" ? "default" : "outline"}
                  size="sm"
                  className="flex-1"
                  onClick={() => setDir("buy")}
                >
                  KVNC → {pool.symbol}
                </Button>
                <Button
                  type="button"
                  variant={dir === "sell" ? "default" : "outline"}
                  size="sm"
                  className="flex-1"
                  onClick={() => setDir("sell")}
                >
                  {pool.symbol} → KVNC
                </Button>
              </div>

              <label className="text-xs text-muted">
                You pay ({dir === "buy" ? "KVNC" : pool.symbol})
                <input
                  value={amount}
                  onChange={(e) => setAmount(e.target.value)}
                  inputMode="decimal"
                  className="mt-1 h-11 w-full rounded-md border border-border bg-bg px-3 font-mono text-sm text-fg outline-none"
                  placeholder="0"
                />
              </label>

              <div className="rounded-lg border border-border bg-bg p-3">
                <p className="text-[10px] uppercase tracking-wide text-subtle">You receive</p>
                <p className="mt-1 break-all font-mono text-sm text-fg">
                  {out === null
                    ? "—"
                    : `${formatTokenAmount(out, outDecimals)} ${
                        dir === "buy" ? pool.symbol : "KVNC"
                      }`}
                </p>
                <p className="mt-2 font-mono text-[10px] text-subtle">
                  Price impact {impact === null ? "—" : `${(impact / 100).toFixed(2)}%`} · Fee{" "}
                  {(pool.fee_bps / 100).toFixed(2)}%
                </p>
              </div>

              <p className="flex items-start gap-1.5 text-[11px] leading-relaxed text-muted">
                <Info className="mt-0.5 size-3.5 shrink-0 text-blue" />
                Quote uses the contract&apos;s own constant-product formula (fee retained in the
                reserve). Executing a swap is a signed{" "}
                <code className="font-mono text-[10px] text-fg">token_buy</code> /{" "}
                <code className="font-mono text-[10px] text-fg">token_sell</code> call — use the
                CLI or the Android wallet.
              </p>

              <p className="flex items-center gap-1.5 font-mono text-[10px] text-subtle">
                <ArrowDownUp className="size-3" /> LP supply{" "}
                {formatTokenAmount(BigInt(pool.lp_total), 0)}
              </p>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

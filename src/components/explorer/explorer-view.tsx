import { useEffect, useMemo, useRef, useState } from "react";
import { Link } from "@tanstack/react-router";
import { DagMark } from "@/components/brand/dag-mark";
import { MAINNET_ID } from "@/lib/networkConstants";
import { SURFACE } from "@/lib/surfaces";
import { Compass, RefreshCw, AlertCircle, ShieldAlert } from "lucide-react";
import { DagCanvas, DagLegend } from "@/components/explorer/dag-canvas";
import { createDagBlocksCache } from "@/lib/api/map-blocks";
import { useNode } from "@/lib/api/use-node";
import { shortId } from "@/lib/ledger/hash";
import { cn } from "@/lib/utils";
import { getNetworkId } from "@/lib/network";
import { Button } from "@/components/ui/button";
import type { Block } from "@/lib/ledger/types";
import { type ApiNode, type ApiDagBlock } from "@/lib/api/contract";

const TABS = ["Graph", "Blocks", "Mempool", "Order", "Console", "Analytics", "Docs"] as const;
type Tab = (typeof TABS)[number];

/** Explorer BlockDAG surface. Truthful hybrid consensus metrics and DAG inspection. */
export function ExplorerView() {
  const { state, error, refresh, source } = useNode(2000);
  const [tab, setTab] = useState<Tab>("Graph");
  const [selectedBlock, setSelectedBlock] = useState<string | null>(null);

  const dagCacheRef = useRef(createDagBlocksCache());
  const blocks = useMemo(
    () => (state ? dagCacheRef.current(state.node.dag, state.node.order) : []),
    [state],
  );

  useEffect(() => {
    if (!state) return;
    if (!selectedBlock || !blocks.some((b) => b.id === selectedBlock)) {
      setSelectedBlock(state.node.selected_tip);
    }
  }, [state, blocks, selectedBlock]);

  const selected = blocks.find((b) => b.id === selectedBlock) ?? null;
  const selectedApi = state?.node.dag.find((b) => b.id === selectedBlock) ?? null;
  const n = state?.node;

  // Endpoint verification check for hybrid capability
  const isHybridVerified =
    state?.network?.includes("2") ||
    state?.network?.includes("5") ||
    state?.network?.toLowerCase().includes("testnet") ||
    state?.network?.toLowerCase().includes("hybrid") ||
    source === "local";

  if (source === "mainnet") {
    return <MainnetExplorerPlaceholder />;
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {/* Explorer Header */}
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-border px-4 py-3 md:px-6">
        <div>
          <div className="flex items-center gap-3">
            <h1 className="font-display text-2xl tracking-tight text-fg">BlockDAG Explorer</h1>
            <span className="inline-flex items-center gap-1.5 rounded-full border border-gold/30 bg-gold/10 px-2.5 py-0.5 text-[11px] font-medium text-gold">
              <span className="size-1.5 rounded-full bg-gold animate-pulse" />
              Hybrid PoW + GHOSTDAG · 4s Target
            </span>
          </div>
          <p className="mt-0.5 text-xs text-muted md:text-sm">
            Native coin <strong className="text-fg">KVNC</strong> on {state?.network ?? getNetworkId()}.
            Target block interval: 4 s (not a finality guarantee) · GHOSTDAG K=5.
            {error ? <span className="text-danger"> · {error}</span> : null}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => void refresh()}
            className="h-8 gap-1.5 text-xs"
          >
            <RefreshCw className="size-3.5" />
            <span>Refresh</span>
          </Button>
          <span className="font-mono text-[11px] tracking-wide text-subtle uppercase">
            source · {source}
          </span>
        </div>
      </div>

      {/* Endpoint caveat notice when hybrid capabilities not established */}
      {state && !isHybridVerified && (
        <div className="flex items-center justify-between gap-3 border-b border-amber-500/30 bg-amber-500/10 px-4 py-2 text-xs text-amber-300 md:px-6">
          <div className="flex items-center gap-2">
            <AlertCircle className="size-4 shrink-0" />
            <span>
              Connected endpoint reports &ldquo;{state.network}&rdquo;. Hybrid Chain 2 data unavailable / endpoint not yet verified.
            </span>
          </div>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => void refresh()}
            className="h-7 border-amber-500/40 text-xs text-amber-300 hover:bg-amber-500/20"
          >
            Retry
          </Button>
        </div>
      )}

      {/* Real-data grid: at most 6 compact measured telemetry cells */}
      <dl className="grid grid-cols-2 border-b border-border bg-border gap-px sm:grid-cols-3 lg:grid-cols-6">
        <Stat
          label="DAG Blocks"
          value={blocks.length > 0 ? `${blocks.length} Blocks` : String(n?.blocks ?? "—")}
        />
        <Stat
          label="Selected Head"
          value={n?.selected_tip ? shortId(n.selected_tip, 6) : "—"}
        />
        <Stat
          label="Connected Peers"
          value={state?.peers !== undefined ? `${state.peers.length} Peers` : "—"}
        />
        <Stat
          label="Mempool"
          value={n?.mempool !== undefined ? `${n.mempool} Pending` : "—"}
        />
        <Stat
          label="Genesis Anchor"
          value={n?.genesis ? shortId(n.genesis, 6) : "—"}
        />
        <Stat
          label="Blue Score"
          value={n?.blue_score !== undefined ? String(n.blue_score) : "—"}
        />
      </dl>

      {/* Static protocol specification banner */}
      <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-1 border-b border-border bg-surface-2/60 px-4 py-2 text-[11px] text-muted md:px-6">
        <span className="font-mono text-subtle uppercase tracking-wider">Protocol Target / Config:</span>
        <div className="flex flex-wrap items-center gap-x-4 gap-y-1 font-mono text-fg">
          <span>Target Interval: <strong className="text-gold">4 s</strong></span>
          <span>GHOSTDAG: <strong className="text-gold">K=5</strong></span>
          <span>PoW: <strong>KovanicaHash</strong> (audit required)</span>
          <span>Initial Reward: <strong>2.5 KVNC</strong></span>
          <span>Max Supply: <strong>90.2M KVNC</strong></span>
        </div>
      </div>

      {/* Main DAG Graph canvas section with distinct state handling */}
      <section className="border-b border-border bg-surface px-4 py-4 md:px-6">
        <div className="mb-3 flex flex-wrap items-end justify-between gap-3">
          <DagLegend />
        </div>
        <div className="h-[min(54vh,480px)]">
          {error && !state ? (
            <div className="flex h-full flex-col items-center justify-center gap-3 rounded-lg border border-danger/30 bg-danger/5 p-6 text-center">
              <ShieldAlert className="size-8 text-danger" />
              <p className="text-sm font-medium text-danger">Unable to reach node endpoint</p>
              <p className="max-w-md text-xs text-muted">{error}</p>
              <Button type="button" variant="outline" size="sm" onClick={() => void refresh()}>
                <RefreshCw className="mr-1.5 size-3.5" /> Retry Connection
              </Button>
            </div>
          ) : !state ? (
            <div className="flex h-full flex-col items-center justify-center gap-3 rounded-lg border border-border bg-bg/50 p-6 text-center">
              <div className="size-6 animate-spin rounded-full border-2 border-gold border-t-transparent" />
              <p className="font-mono text-xs text-muted">Loading BlockDAG state from node…</p>
            </div>
          ) : blocks.length === 0 ? (
            <div className="flex h-full flex-col items-center justify-center gap-3 rounded-lg border border-border bg-bg/50 p-6 text-center">
              <Compass className="size-8 text-muted" />
              <p className="text-sm font-medium text-fg">No blocks returned in DAG</p>
              <p className="text-xs text-muted">The node state currently reports 0 DAG blocks.</p>
              <Button type="button" variant="outline" size="sm" onClick={() => void refresh()}>
                <RefreshCw className="mr-1.5 size-3.5" /> Refresh
              </Button>
            </div>
          ) : (
            <DagCanvas blocks={blocks} selectedId={selectedBlock} onSelect={setSelectedBlock} />
          )}
        </div>
      </section>

      {/* Tabs navigation */}
      <div className="flex flex-wrap items-center justify-between border-b border-border bg-surface/30 px-4 md:px-6">
        <div className="flex gap-1 overflow-x-auto" role="tablist">
          {TABS.map((t) => {
            const count =
              t === "Blocks" ? blocks.length :
              t === "Mempool" ? (n?.mempool ?? 0) :
              undefined;
            return (
              <button
                key={t}
                type="button"
                role="tab"
                aria-selected={tab === t}
                onClick={() => setTab(t)}
                className={cn(
                  "h-11 shrink-0 border-b-2 px-3.5 text-sm font-medium transition-colors duration-150 inline-flex items-center gap-2",
                  tab === t
                    ? "border-gold text-fg font-semibold"
                    : "border-transparent text-muted hover:text-fg hover:border-border",
                )}
              >
                <span>{t}</span>
                {count !== undefined ? (
                  <span className="rounded-full bg-border/80 px-1.5 py-0.2 text-[10px] font-mono text-muted">
                    {count}
                  </span>
                ) : null}
              </button>
            );
          })}
        </div>
      </div>

      {/* Tab Panels */}
      <section className="px-4 py-5 md:px-6">
        {tab === "Graph" ? <GraphPanel selected={selected} apiBlock={selectedApi} /> : null}
        {tab === "Blocks" ? (
          <BlocksTable
            blocks={[...blocks].reverse()}
            selectedId={selectedBlock}
            onSelect={(id) => {
              setSelectedBlock(id);
              setTab("Graph");
            }}
          />
        ) : null}
        {tab === "Mempool" ? <MempoolPanel node={n} /> : null}
        {tab === "Order" ? <OrderPanel node={n} /> : null}
        {tab === "Console" ? <ConsolePanel node={n} /> : null}
        {tab === "Analytics" ? <AnalyticsPanel node={n} /> : null}
        {tab === "Docs" ? <DocsPanel /> : null}
      </section>
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-bg px-3.5 py-3 hover:bg-surface/50 transition-colors">
      <dt className="text-[10px] font-medium tracking-wider text-subtle uppercase truncate">{label}</dt>
      <dd className="mt-1 font-mono text-xs tabular-nums text-fg md:text-sm font-semibold truncate">
        {value}
      </dd>
    </div>
  );
}

function GraphPanel({ selected, apiBlock }: { selected: Block | null; apiBlock: ApiDagBlock | null }) {
  if (!selected) return <p className="text-sm text-muted">Select a block on the graph or table to view details.</p>;
  const ageSec = Math.max(0, Math.floor((Date.now() - selected.timestamp) / 1000));
  const ageStr = ageSec < 60 ? `${ageSec}s ago` : `${Math.floor(ageSec / 60)}m ago`;
  return (
    <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_360px]">
      <article className="rounded-lg border border-border bg-surface p-4">
        <div className="flex items-center justify-between">
          <p className="text-[10px] tracking-wide text-subtle uppercase">
            {selected.colour === "genesis" ? "Genesis Block (#0)" :
             selected.colour === "chain" ? "Selected GHOSTDAG Tip" :
             selected.colour === "red" ? "Fork / Alternative Block" :
             "Ordered DAG Block"}
          </p>
          <span className={cn(
            "inline-flex items-center gap-1 rounded px-2 py-0.5 text-[11px] font-medium",
            selected.colour === "red" ? "bg-rose-500/10 text-rose-400" :
            selected.colour === "genesis" ? "bg-white/10 text-white" :
            selected.colour === "chain" ? "bg-amber-500/10 text-amber-400" :
            "bg-blue/10 text-blue"
          )}>
            <span className={cn(
              "size-1.5 rounded-full",
              selected.colour === "red" ? "bg-rose-400" :
              selected.colour === "genesis" ? "bg-white" :
              selected.colour === "chain" ? "bg-amber-400 animate-pulse" :
              "bg-blue animate-pulse"
            )} />
            {selected.colour === "red" ? "Fork / Alternative" :
             selected.colour === "genesis" ? "Genesis Anchor" :
             selected.colour === "chain" ? "Active Tip" :
             "GHOSTDAG Ordered"}
          </span>
        </div>
        <p className="mt-1 break-all font-mono text-sm text-fg">{selected.id}</p>
        <dl className="mt-4 grid grid-cols-2 gap-3 text-sm">
          <div className="flex justify-between"><dt className="text-xs text-subtle">Height</dt><dd className="font-mono font-semibold text-fg">#{selected.height}</dd></div>
          <div className="flex justify-between"><dt className="text-xs text-subtle">GHOSTDAG Score</dt><dd className="font-mono tabular-nums text-fg">{selected.blueScore}</dd></div>
          <div className="flex justify-between"><dt className="text-xs text-subtle">Observed Age</dt><dd className="font-mono text-fg">{ageStr}</dd></div>
          <div className="flex justify-between"><dt className="text-xs text-subtle">Parent Count</dt><dd className="font-mono text-fg">{selected.parents.length}</dd></div>
        </dl>
        {selected.selectedParent && (
          <div className="mt-3 border-t border-border/50 pt-2 text-xs">
            <span className="text-subtle">Selected Parent: </span>
            <span className="font-mono text-muted break-all">{selected.selectedParent}</span>
          </div>
        )}
      </article>
      <aside className="rounded-lg border border-border bg-surface p-4">
        <p className="text-[10px] tracking-wide text-subtle uppercase">Executed Transactions ({apiBlock?.txs?.length ?? 0})</p>
        {(apiBlock?.txs?.length ?? 0) === 0 ? (
          <p className="mt-3 text-xs text-muted">No user transactions in this block (coinbase/heartbeat).</p>
        ) : (
          <ul className="mt-3 space-y-3">
            {(apiBlock?.txs ?? []).map((tx) => (
              <li key={tx.id} className="rounded-lg border border-border/70 bg-bg/30 p-2.5">
                <div className="flex items-center justify-between gap-2 font-mono text-[11px]">
                  <span className="text-fg">{shortId(tx.id)}</span>
                  <span className="text-ok">Success</span>
                </div>
              </li>
            ))}
          </ul>
        )}
      </aside>
    </div>
  );
}

function BlocksTable({ blocks, selectedId, onSelect }: { blocks: Block[]; selectedId: string | null; onSelect: (id: string) => void }) {
  return (
    <div className="overflow-x-auto">
      <table className="w-full min-w-[560px] text-left text-sm">
        <thead>
          <tr>
            <th className="py-2 text-[10px] font-medium tracking-wide text-subtle uppercase">Height</th>
            <th className="py-2 text-[10px] font-medium tracking-wide text-subtle uppercase">Block Digest</th>
            <th className="py-2 text-[10px] font-medium tracking-wide text-subtle uppercase">Score</th>
            <th className="py-2 text-[10px] font-medium tracking-wide text-subtle uppercase">Age</th>
            <th className="py-2 text-[10px] font-medium tracking-wide text-subtle uppercase">Txs</th>
          </tr>
        </thead>
        <tbody>
          {blocks.map((b) => {
            const ageSec = Math.max(0, Math.floor((Date.now() - b.timestamp) / 1000));
            const ageStr = ageSec < 60 ? `${ageSec}s ago` : `${Math.floor(ageSec / 60)}m ago`;
            return (
              <tr key={b.id} className={cn("cursor-pointer border-t border-border transition-colors", b.id === selectedId ? "bg-surface-2" : "hover:bg-surface")} onClick={() => onSelect(b.id)}>
                <td className="py-2.5 font-mono tabular-nums font-semibold text-fg">#{b.height}</td>
                <td className="py-2.5 font-mono text-muted hover:text-fg">{shortId(b.id)}</td>
                <td className="py-2.5 font-mono tabular-nums text-muted">{b.blueScore}</td>
                <td className="py-2.5 font-mono text-subtle">{ageStr}</td>
                <td className="py-2.5 font-mono tabular-nums text-muted">{b.txs.length}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function MempoolPanel({ node }: { node: ApiNode | undefined }) {
  if (!node || node.pending.length === 0) {
    return <p className="text-sm text-muted">Mempool is clear. Transactions are mined in 4-second blocks.</p>;
  }
  return (
    <div>
      <p className="mb-3 text-sm text-muted">Min fee 0.00001 KVNC. {node.pending.length} in pool.</p>
      <ul className="font-mono text-xs text-muted space-y-1">
        {node.pending.map((id) => (
          <li key={id}>mempool · {shortId(id)}</li>
        ))}
      </ul>
    </div>
  );
}

function OrderPanel({ node }: { node: ApiNode | undefined }) {
  if (!node || node.order.length === 0) return <p className="text-sm text-muted">No blocks in order.</p>;
  return (
    <ol className="list-decimal pl-5 font-mono text-xs text-muted">
      {node.order.map((id, i) => (
        <li key={id} className="py-1">
          <span className="text-subtle">{i}</span> · {id}
          {id === node.selected_tip && <span className="ml-2 text-accent">← tip</span>}
        </li>
      ))}
    </ol>
  );
}

function AnalyticsPanel({ node }: { node: ApiNode | undefined }) {
  if (!node) return <p className="text-sm text-muted">No data available.</p>;
  return (
    <div className="grid gap-4 lg:grid-cols-2">
      <div className="rounded-lg border border-border bg-surface p-4">
        <h3 className="mb-3 font-display text-lg text-fg">Hybrid Protocol Metrics</h3>
        <dl className="space-y-2 text-sm">
          <div className="flex justify-between"><dt className="text-subtle">Consensus Model</dt><dd className="font-mono font-medium text-fg">Hybrid PoW + GHOSTDAG</dd></div>
          <div className="flex justify-between"><dt className="text-subtle">Target Block Interval</dt><dd className="font-mono">4.0 s</dd></div>
          <div className="flex justify-between"><dt className="text-subtle">GHOSTDAG Parameter K</dt><dd className="font-mono">5</dd></div>
          <div className="flex justify-between"><dt className="text-subtle">PoW Algorithm</dt><dd className="font-mono">KovanicaHash</dd></div>
          <div className="flex justify-between"><dt className="text-subtle">Total Coin Supply</dt><dd className="font-mono">90,200,000 KVNC</dd></div>
          <div className="flex justify-between"><dt className="text-subtle">Mining Subsidy Budget</dt><dd className="font-mono">81,000,000 KVNC</dd></div>
          <div className="flex justify-between"><dt className="text-subtle">Initial Block Reward</dt><dd className="font-mono">2.5 KVNC</dd></div>
          <div className="flex justify-between"><dt className="text-subtle">Subsidy Era</dt><dd className="font-mono">8,100,000 blocks</dd></div>
        </dl>
      </div>
    </div>
  );
}

function ConsolePanel({ node }: { node: ApiNode | undefined }) {
  return (
    <div className="overflow-auto rounded-lg border border-border bg-surface/50 p-4 max-h-[400px]">
      <p className="text-[10px] tracking-wide text-subtle uppercase">Consensus Status</p>
      <div className="mt-2 font-mono text-xs text-muted space-y-1">
        <p>• Consensus Model: Hybrid PoW + GHOSTDAG (NO stake in consensus)</p>
        <p>• GHOSTDAG Parameter: K=5</p>
        <p>• Target Block Time: 4,000 ms (4 s)</p>
        <p>• PoW Hash Algorithm: KovanicaHash (BLAKE3 + 2 MiB mix)</p>
        <p>• Security Audit: Required before mainnet launch</p>
        <p>• Reported Head Height: #{node?.blocks ?? 0}</p>
        <p>• Native Token: KVNC (9 decimals)</p>
      </div>
    </div>
  );
}

function MainnetExplorerPlaceholder() {
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-border px-4 py-3 md:px-6">
        <div>
          <div className="flex items-center gap-3">
            <h1 className="font-display text-2xl tracking-tight text-fg">BlockDAG Explorer</h1>
            <span className="inline-flex items-center gap-1.5 rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2.5 py-0.5 text-[11px] font-medium text-emerald-400">
              Mainnet · Launch Scheduled 1.1.2027
            </span>
          </div>
          <p className="mt-0.5 text-xs text-muted md:text-sm">
            Native coin <strong className="text-fg">KVNC</strong> on {MAINNET_ID}. Target block interval: 4 s · GHOSTDAG K=5.
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <span className="font-mono text-[11px] tracking-wide text-subtle uppercase">
            source · mainnet
          </span>
        </div>
      </div>

      <div className="flex flex-1 items-center justify-center p-6 md:p-12">
        <div className="mx-auto w-full max-w-lg text-center">
          <DagMark variant="gold" className="mx-auto size-14" />
          <p className="eyebrow mt-5 font-mono text-xs text-muted uppercase tracking-wider">{MAINNET_ID} · KVNC</p>
          <h2 className="mt-2 font-display text-3xl tracking-tight text-fg italic md:text-4xl">
            Mainnet Explorer Unavailable
          </h2>
          <p className="mt-1 font-mono text-xs text-gold">Public launch: 1.1.2027</p>
          <p className="mt-3 text-sm leading-relaxed text-muted">
            The mainnet BlockDAG has not opened its public RPC endpoint yet. The live DAG graph and block telemetry will begin streaming upon genesis launch on 1.1.2027 following the hybrid testnet and KovanicaHash audit.
          </p>

          <div className="mt-8 flex flex-col items-center justify-center gap-3 sm:flex-row">
            <Button asChild variant="gold" className="h-11 px-6">
              <a href={`${SURFACE.testnet}/explorer`}>Open Testnet Explorer</a>
            </Button>
            <Button asChild variant="outline" className="h-11 px-6">
              <Link to="/roadmap">Protocol Roadmap</Link>
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

function DocsPanel() {
  return (
    <div className="max-w-2xl space-y-4 text-sm text-muted">
      <h3 className="font-display text-lg text-fg">Hybrid PoW + GHOSTDAG Consensus</h3>
      <p>
        Kovanica uses Proof-of-Work admission coupled with GHOSTDAG ordering.
        Mining admission is governed by KovanicaHash (BLAKE3 + 2 MiB memory mix),
        and all valid blocks form a Directed Acyclic Graph ordered by GHOSTDAG with K=5.
      </p>
      <p>
        The target block interval is 4 seconds. A target interval is not a finality guarantee.
        There is zero stake in consensus — no validator bonds, delegations, or slashing.
      </p>
    </div>
  );
}

import { useEffect, useState } from "react";
import { SURFACE } from "@/lib/surfaces";
import { Activity, Globe, Server, Shield, Radio, RefreshCw } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { api, useApiSource, isPublic } from "@/lib/api/client";
import type { ApiBootstrap, ApiHead } from "@/lib/api/contract";
import { ATOM, TOKEN } from "@/lib/api/contract";
import { shortId } from "@/lib/ledger/hash";
import { cn } from "@/lib/utils";

type P2pInfo = {
  path: string;
  listen: string;
  peers: string[];
  bootstrap: string;
};

type Snapshot = {
  head: ApiHead;
  bootstrap: ApiBootstrap;
  p2p: P2pInfo | null;
  fetchedAt: number;
};

function fmtKvnc(atoms: number): string {
  return `${(atoms / ATOM).toLocaleString(undefined, { maximumFractionDigits: 2 })} ${TOKEN}`;
}

export function NetworkView() {
  const source = useApiSource();
  const live = isPublic(source);

  if (source === "mainnet") {
    return <MainnetNetworkView />;
  }
  const [snap, setSnap] = useState<Snapshot | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function load() {
    setBusy(true);
    setError(null);
    try {
      const [head, bootstrap, p2p] = await Promise.all([
        api<ApiHead>("/api/head"),
        api<ApiBootstrap>("/api/bootstrap"),
        api<P2pInfo>("/api/p2p").catch(() => null),
      ]);
      setSnap({ head, bootstrap, p2p, fetchedAt: Date.now() });
    } catch (e) {
      const msg = e instanceof Error ? e.message : "Failed to load network status";
      setError(msg);
      toast.error(msg);
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    void load();
    const id = setInterval(() => void load(), 15_000);
    return () => clearInterval(id);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [source]);

  const b = snap?.bootstrap;
  const h = snap?.head;
  const p2p = snap?.p2p;

  return (
    <div className="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 md:px-6 md:py-8">
      <header className="flex items-start justify-between gap-3">
        <div>
          <p className="font-mono text-[10px] tracking-brand text-gold uppercase">Status</p>
          <h1 className="font-display text-3xl tracking-tight text-fg">Network</h1>
          <p className="mt-2 text-sm leading-relaxed text-muted">
            Live view of the selected source ({live ? "public testnet" : source}). Auto-refreshes
            every 15s.
          </p>
        </div>
        <Button
          type="button"
          variant="outline"
          size="icon"
          disabled={busy}
          onClick={() => void load()}
          aria-label="Refresh"
        >
          <RefreshCw className={cn("size-4", busy && "animate-spin")} />
        </Button>
      </header>

      {error && (
        <div className="rounded-xl border border-danger/40 bg-danger/5 px-4 py-3 text-sm text-danger">
          {error}
        </div>
      )}

      {!snap && !error && (
        <p className="text-center text-sm text-muted">Loading network status…</p>
      )}

      {snap && h && b && (
        <>
          <section className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            <StatCard
              icon={Activity}
              label="Blocks"
              value={h.blocks.toLocaleString()}
              sub={`tip ${shortId(h.tip)}`}
            />
            <StatCard
              icon={Globe}
              label="Network"
              value={h.network}
              sub={`k = ${b.k} · min fee ${(h.min_fee / ATOM).toFixed(4)} ${TOKEN}`}
            />
            <StatCard
              icon={Server}
              label="Peers"
              value={String((p2p?.peers?.length ?? b.peers?.length ?? 0))}
              sub={p2p?.path ?? "tcp"}
            />
          </section>

          <section className="rounded-xl border border-border bg-surface p-4">
            <p className="text-[10px] tracking-wide text-subtle uppercase">Chain parameters</p>
            <dl className="mt-3 grid gap-3 sm:grid-cols-2">
              <Row label="Genesis" value={shortId(h.genesis)} mono full={h.genesis} />
              <Row label="Tip" value={shortId(h.tip)} mono full={h.tip} />
              <Row label="Token" value={b.token ?? TOKEN} />
              <Row label="Subsidy" value={fmtKvnc(b.subsidy)} />
              <Row label="Founder amount" value={fmtKvnc(b.founder_amount)} />
              <Row label="Founder seed" value={String(b.founder_seed)} />
              <Row label="Atom" value={`1 ${TOKEN} = ${h.atom.toLocaleString()} atoms`} />
              <Row label="MysticGhost k" value={String(b.k)} />
              {"finality_depth" in b && (
                <Row label="Finality depth" value={String((b as Record<string, unknown>).finality_depth)} />
              )}
              {"payload_pruning_depth" in b && (
                <Row
                  label="Payload prune"
                  value={String((b as Record<string, unknown>).payload_pruning_depth)}
                />
              )}
            </dl>
          </section>

          <section className="rounded-xl border border-border bg-surface p-4">
            <div className="flex items-center gap-2">
              <Radio className="size-4 text-teal" />
              <p className="text-[10px] tracking-wide text-subtle uppercase">P2P</p>
            </div>
            <dl className="mt-3 grid gap-3 sm:grid-cols-2">
              <Row label="Listen" value={p2p?.listen ?? b.listen ?? "—"} mono />
              <Row label="Bootstrap" value={p2p?.bootstrap ?? "seed1.kovanica.online:8000,seed2.kovanica.online:8000,seed3.kovanica.online:8000"} mono />
            </dl>
            <p className="mt-4 text-[10px] tracking-wide text-subtle uppercase">Bootstrap seeds</p>
            <ul className="mt-2 divide-y divide-border rounded-lg border border-border">
              <li className="break-all px-3 py-2 font-mono text-xs text-fg">
                seed1.kovanica.online:8000 <span className="text-subtle">(grey-cloud DNS)</span>
              </li>
              <li className="break-all px-3 py-2 font-mono text-xs text-fg">
                seed2.kovanica.online:8000 <span className="text-subtle">(grey-cloud DNS)</span>
              </li>
              <li className="break-all px-3 py-2 font-mono text-xs text-fg">
                seed3.kovanica.online:8000 <span className="text-subtle">(grey-cloud DNS)</span>
              </li>
            </ul>
            <p className="mt-3 text-[11px] leading-relaxed text-subtle">
              Dial the DNS names on TCP 8000 — never the Cloudflare-proxied explorer
              hostnames.
            </p>
            <p className="mt-4 text-[10px] tracking-wide text-subtle uppercase">Connected peers</p>
            <ul className="mt-2 divide-y divide-border rounded-lg border border-border">
              {(p2p?.peers ?? b.peers ?? []).length === 0 && (
                <li className="px-3 py-2 font-mono text-xs text-muted">No peers reported</li>
              )}
              {(p2p?.peers ?? b.peers ?? []).map((peer) => (
                <li key={peer} className="break-all px-3 py-2 font-mono text-xs text-fg">
                  {peer}
                </li>
              ))}
            </ul>
          </section>

          {b.upstream && (
            <section className="rounded-xl border border-border bg-surface p-4">
              <p className="text-[10px] tracking-wide text-subtle uppercase">Upstream probe</p>
              {b.upstream.ok ? (
                <p className="mt-2 font-mono text-sm text-ok">
                  OK · tip {shortId(b.upstream.head.tip)} · {b.upstream.head.blocks} blocks
                </p>
              ) : (
                <p className="mt-2 text-sm text-danger">{b.upstream.error}</p>
              )}
            </section>
          )}

          <p className="text-center font-mono text-[11px] text-subtle">
            Last fetch {new Date(snap.fetchedAt).toLocaleTimeString()}
          </p>
        </>
      )}
    </div>
  );
}

function StatCard({
  icon: Icon,
  label,
  value,
  sub,
  accent = "fg",
}: {
  icon: typeof Activity;
  label: string;
  value: string;
  sub?: string;
  accent?: "fg" | "ok" | "muted";
}) {
  return (
    <div className="rounded-xl border border-border bg-surface p-4">
      <div className="flex items-center gap-2 text-muted">
        <Icon className="size-3.5" />
        <p className="text-[10px] tracking-wide uppercase">{label}</p>
      </div>
      <p
        className={cn(
          "mt-2 font-display text-2xl tracking-tight",
          accent === "ok" && "text-ok",
          accent === "muted" && "text-muted",
          accent === "fg" && "text-fg",
        )}
      >
        {value}
      </p>
      {sub && <p className="mt-1 truncate font-mono text-[11px] text-subtle">{sub}</p>}
    </div>
  );
}

function MainnetNetworkView() {
  return (
    <div className="mx-auto flex w-full max-w-3xl flex-col gap-6 px-4 py-6 md:px-6 md:py-8">
      <header className="flex items-start justify-between gap-3">
        <div>
          <p className="font-mono text-[10px] tracking-brand text-emerald-400 uppercase">Status</p>
          <h1 className="font-display text-3xl tracking-tight text-fg">Mainnet Network</h1>
          <p className="mt-2 text-sm leading-relaxed text-muted">
            The production mainnet is scheduled for public launch on 1.1.2027 following the hybrid testnet and KovanicaHash audit.
          </p>
        </div>
      </header>

      <section className="rounded-xl border border-emerald-500/30 bg-surface p-6">
        <div className="flex items-center justify-between">
          <div>
            <span className="font-mono text-[10px] uppercase tracking-wider text-muted">Environment</span>
            <h2 className="mt-1 font-display text-2xl text-fg">kovanica-mainnet</h2>
          </div>
          <span className="inline-flex items-center gap-1.5 rounded-full border border-emerald-500/40 bg-emerald-500/10 px-3 py-1 font-mono text-xs text-emerald-400">
            <span className="size-1.5 rounded-full bg-emerald-400" />
            Launch: 1.1.2027
          </span>
        </div>

        <dl className="mt-6 grid grid-cols-1 gap-4 sm:grid-cols-2">
          <div className="rounded-lg bg-bg/50 p-3">
            <dt className="text-xs text-muted">Consensus</dt>
            <dd className="mt-1 font-mono text-sm text-fg">Hybrid PoW + GHOSTDAG (k=5)</dd>
          </div>
          <div className="rounded-lg bg-bg/50 p-3">
            <dt className="text-xs text-muted">Target Block Interval</dt>
            <dd className="mt-1 font-mono text-sm text-gold">4 s</dd>
          </div>
          <div className="rounded-lg bg-bg/50 p-3">
            <dt className="text-xs text-muted">Proof-of-Work</dt>
            <dd className="mt-1 font-mono text-sm text-fg">KovanicaHash (audit required)</dd>
          </div>
          <div className="rounded-lg bg-bg/50 p-3">
            <dt className="text-xs text-muted">Native Token</dt>
            <dd className="mt-1 font-mono text-sm text-fg">KVNC (9 decimals)</dd>
          </div>
          <div className="rounded-lg bg-bg/50 p-3">
            <dt className="text-xs text-muted">Public RPC Endpoint</dt>
            <dd className="mt-1 font-mono text-sm text-muted">Not open yet (opens at genesis)</dd>
          </div>
          <div className="rounded-lg bg-bg/50 p-3">
            <dt className="text-xs text-muted">Staking / Validators</dt>
            <dd className="mt-1 font-mono text-sm text-fg">None (Zero-stake PoW consensus)</dd>
          </div>
        </dl>

        <div className="mt-6 flex flex-wrap gap-3">
          <Button asChild variant="gold" className="h-10 px-5">
            <a href={`${SURFACE.testnet}/network`}>View Testnet Network</a>
          </Button>
          <Button asChild variant="outline" className="h-10 px-5">
            <a href={`${SURFACE.landing}/roadmap`}>Protocol Roadmap</a>
          </Button>
        </div>
      </section>
    </div>
  );
}

function Row({
  label,
  value,
  mono,
  full,
}: {
  label: string;
  value: string;
  mono?: boolean;
  full?: string;
}) {
  return (
    <div>
      <dt className="text-[10px] tracking-wide text-subtle uppercase">{label}</dt>
      <dd
        className={cn("mt-0.5 break-all text-sm text-fg", mono && "font-mono text-xs")}
        title={full}
      >
        {value}
      </dd>
    </div>
  );
}

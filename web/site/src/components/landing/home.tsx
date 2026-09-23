import { useEffect, useState } from "react";
import { Link } from "@tanstack/react-router";
import {
  Activity,
  Bot,
  Braces,
  Coins,
  Compass,
  Droplets,
  FileText,
  Fingerprint,
  Gem,
  Landmark,
  Lock,
  Map,
  Network,
  Route,
  ShieldCheck,
  Terminal,
  Wallet,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { NetworkBadge } from "@/components/layout/network-badge";
import { DagMark } from "@/components/brand/dag-mark";
import { WalletDownloads } from "@/components/wallet/wallet-downloads";
import { SURFACE } from "@/lib/surfaces";
import { fmtKvnc } from "@/lib/ledger/format";
import type { ApiState } from "@/lib/api/contract";

/**
 * Landing (kovanica.online) — protocol-first framing.
 * Kovanica Protocol is the product; every surface is a window into it.
 * The ticker fetches client-side only (SSR-safe).
 */

const PILLARS = [
  {
    icon: Network,
    eyebrow: "Consensus",
    title: "BlockDAG + GHOSTDAG",
    body: "k=3 GHOSTDAG ordering — blocks reference multiple parents, every block earns its place, and the selected chain is the anchor of truth.",
  },
  {
    icon: Fingerprint,
    eyebrow: "Block production",
    title: "Hybrid PoW + VRF staking",
    body: "Proof-of-work secures the base while VRF-staked producers earn the right to mint — a hybrid that resists nothing-at-stake.",
  },
  {
    icon: Coins,
    eyebrow: "Assets",
    title: "Native multi-asset UTXOs",
    body: "KVP-102: second-layer assets live in the same UTXO model as KVNC — one ledger, many assets, atomic by construction.",
  },
  {
    icon: ShieldCheck,
    eyebrow: "Privacy",
    title: "Stealth, HTLC, vaults",
    body: "One-time ECDH stealth addresses, hashed time-locked swaps and CLTV/CSV vaults — privacy and custody as protocol, not app.",
  },
];

const SURFACES = [
  {
    href: `${SURFACE.testnet}/explorer`,
    icon: Compass,
    title: "Explorer",
    body: "The BlockDAG, live — graph, mempool, blocks, console and analytics for the selected network.",
  },
  {
    href: `${SURFACE.testnet}/wallet`,
    icon: Wallet,
    title: "Wallet",
    body: "Seed, hardware and watch wallets; multi-asset, stealth, HTLC, vaults and multisig in one place.",
  },
  {
    href: `${SURFACE.testnet}/map`,
    icon: Map,
    title: "Origins map",
    body: "A choropleth of real origin pulses from recorded visits — record your origin to leave yours.",
  },
  {
    href: `${SURFACE.testnet}/network`,
    icon: Activity,
    title: "Network status",
    body: "Live head, peers, PoW, subsidy, finality and bootstrap seeds for the selected source.",
  },
  {
    href: SURFACE.faucet,
    icon: Droplets,
    title: "Faucet",
    body: "Get testnet KVNC — 5 KVNC lifetime cap per address, straight into your wallet.",
  },
  {
    href: SURFACE.docs,
    icon: FileText,
    title: "Docs",
    body: "Getting started, wallet, protocol spec, node operations and the API contract.",
  },
  {
    href: SURFACE.api,
    icon: Terminal,
    title: "API reference",
    body: "Every endpoint, live upstream status, and a read-only playground against the public node.",
  },
  {
    href: `${SURFACE.testnet}/wallet/rwa-issue`,
    icon: Landmark,
    title: "RWA issuance",
    body: "Issue real-world asset tokens — real estate, bonds, invoices — with IPFS-backed metadata.",
  },
  {
    href: `${SURFACE.docs}#nft`,
    icon: Gem,
    title: "NFT (KVP-106)",
    body: "Non-fungible tokens with collections and on-ledger metadata hashes — draft standard.",
  },
  {
    href: `${SURFACE.docs}#staking`,
    icon: Lock,
    title: "Staking",
    body: "Bond KVNC (or any asset) to a VRF key for hybrid block production — bonds, unbonds, maturity.",
  },
  {
    href: "https://github.com/KovanicaDAG/kovanica-protocol/tree/main/sdk",
    icon: Braces,
    title: "SDK",
    body: "kovanica-sdk — typed Rust + WASM builders and RPC client, with a cookbook of live recipes.",
  },
  {
    href: SURFACE.kovi,
    icon: Bot,
    title: "Kovi",
    body: "Ask the Kovanica engineering agent about GHOSTDAG, the testnet and the protocol crates.",
  },
  {
    href: `${SURFACE.landing}/roadmap`,
    icon: Route,
    title: "Roadmap",
    body: "RFC status, client surfaces, and what is shipping next on the protocol.",
  },
];

export function HomeLanding() {
  return (
    <main className="relative mx-auto flex w-full max-w-5xl flex-1 flex-col px-4 pb-10 pt-6 md:px-8 md:pt-10">
      {/* Hero — protocol first */}
      <section className="relative text-center">
        <div className="pointer-events-none absolute inset-x-0 -top-24 -z-10 h-[480px] bg-[radial-gradient(ellipse_at_top,rgba(47,186,164,0.07),transparent_60%)]" />
        <div className="flex items-center justify-center gap-2">
          <p className="eyebrow">kovanica protocol · KVNC</p>
          <NetworkBadge />
        </div>
        <h1 className="mt-3 font-display text-5xl tracking-tight text-fg italic md:text-7xl">
          Kovanica
        </h1>
        <p className="mt-3 font-display text-xl tracking-tight text-muted italic md:text-2xl">
          The Directed Acyclic Chain.
        </p>
        <p className="mx-auto mt-3 max-w-xl text-sm leading-relaxed text-muted md:text-base">
          A BlockDAG Layer-1 for programmable value — GHOSTDAG consensus, hybrid PoW + VRF-staked
          block production, native multi-asset UTXOs, and privacy primitives in the protocol itself.
        </p>
        <div className="mt-8 flex flex-col items-stretch justify-center gap-3 sm:flex-row sm:items-center">
          <Button asChild className="h-12 px-6">
            <a href={`${SURFACE.testnet}/explorer`}>Open Testnet Explorer</a>
          </Button>
          <Button asChild variant="outline" className="h-12 px-6">
            <a href={SURFACE.docs}>Read the docs</a>
          </Button>
          <Button asChild variant="ghost" className="h-12 px-6">
            <a href={SURFACE.faucet}>Get testnet KVNC</a>
          </Button>
        </div>
        <LiveTicker className="mt-10" />
      </section>

      {/* Protocol pillars */}
      <section className="mt-16">
        <div className="flex items-end justify-between gap-4">
          <div>
            <p className="eyebrow">The protocol</p>
            <h2 className="mt-1 font-display text-2xl tracking-tight text-fg md:text-3xl">
              One ledger, four ideas
            </h2>
          </div>
          <Link
            to="/roadmap"
            className="hidden font-mono text-[11px] tracking-wide text-blue uppercase transition-colors hover:text-fg sm:block"
          >
            Roadmap & RFCs →
          </Link>
        </div>
        <ul className="mt-6 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          {PILLARS.map((p) => (
            <li
              key={p.title}
              className="flex h-full flex-col rounded-xl border border-border bg-surface p-4"
            >
              <p.icon className="size-4 text-gold" />
              <p className="eyebrow mt-4">{p.eyebrow}</p>
              <h3 className="mt-1 font-display text-lg tracking-tight text-fg">{p.title}</h3>
              <p className="mt-1 text-sm leading-relaxed text-muted">{p.body}</p>
            </li>
          ))}
        </ul>
      </section>

      {/* Surfaces — windows into the protocol */}
      <section className="mt-16">
        <div>
          <p className="eyebrow">Surfaces</p>
          <h2 className="mt-1 font-display text-2xl tracking-tight text-fg md:text-3xl">
            Windows into the protocol
          </h2>
          <p className="mt-2 max-w-xl text-sm text-muted">
            Every tool is a view of the same ledger — open any of them on the live testnet.
          </p>
        </div>
        <ul className="mt-6 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
          {SURFACES.map((s) => (
            <SurfaceCard key={s.title} {...s} />
          ))}
        </ul>
        <WalletDownloads className="mt-8" variant="card" />
      </section>

      {/* Network chooser */}
      <section className="mt-16">
        <div>
          <p className="eyebrow">Get started</p>
          <h2 className="mt-1 font-display text-2xl tracking-tight text-fg md:text-3xl">
            Which network?
          </h2>
        </div>
        <div className="mt-6 grid gap-3 md:grid-cols-2">
          <NetworkCard
            tone="testnet"
            status="Live now"
            title="Testnet"
            body="The full protocol, running today. Get testnet KVNC from the faucet, open the explorer and watch the DAG grow."
            primaryLabel="Open testnet"
            primaryHref={SURFACE.testnet}
            secondaryLabel="Get KVNC"
            secondaryHref={SURFACE.faucet}
          />
          <NetworkCard
            tone="mainnet"
            status="Launching soon"
            title="Mainnet"
            body="The production network. Genesis, node distribution and the VRF-staked producer set are being finalized."
            primaryLabel="Notify me"
            primaryHref={SURFACE.mainnet}
            disabled
          />
        </div>
      </section>

      {/* Roadmap / RFC strip */}
      <section className="mt-16">
        <div className="flex flex-col items-center justify-between gap-4 rounded-xl border border-border bg-surface/60 px-5 py-4 sm:flex-row">
          <div>
            <p className="eyebrow">Roadmap</p>
            <p className="mt-1 text-sm text-muted">
              RFC status, the KVP index and what ships next on the protocol.
            </p>
          </div>
          <Button asChild variant="outline" size="sm">
            <Link to="/roadmap">Open roadmap</Link>
          </Button>
        </div>
      </section>

      <LandingFooter />
    </main>
  );
}

/* ------------------------------------------------------------------ */

function LiveTicker({ className }: { className?: string }) {
  const [stats, setStats] = useState<{
    blocks: number;
    blueScore: number;
    tips: number;
    subsidy: number;
    mempool: number;
  } | null>(null);
  const [offline, setOffline] = useState(false);

  useEffect(() => {
    let mounted = true;
    async function load() {
      try {
        const r = await fetch("/api/state?source=testnet");
        if (!r.ok) throw new Error(String(r.status));
        const d = (await r.json()) as ApiState;
        if (!mounted) return;
        setStats({
          blocks: d.node.blocks,
          blueScore: d.node.blue_score,
          tips: d.node.tips.length,
          subsidy: d.node.subsidy,
          mempool: d.node.mempool,
        });
        setOffline(false);
      } catch {
        if (mounted) setOffline(true);
      }
    }
    void load();
    const id = window.setInterval(() => void load(), 15_000);
    return () => {
      mounted = false;
      window.clearInterval(id);
    };
  }, []);

  const cells = [
    { label: "Blocks", value: stats ? stats.blocks.toLocaleString() : "—" },
    { label: "Blue score", value: stats ? stats.blueScore.toLocaleString() : "—" },
    { label: "Tips", value: stats ? String(stats.tips) : "—" },
    { label: "Subsidy", value: stats ? fmtKvnc(stats.subsidy) : "—" },
    { label: "Mempool", value: stats ? String(stats.mempool) : "—" },
  ];

  return (
    <div className={className}>
      <div className="flex items-center justify-center gap-2">
        <span className="size-1.5 rounded-full bg-ok" />
        <span className="font-mono text-[10px] tracking-wide text-subtle uppercase">
          {offline ? "chain head unreachable — retrying" : "live testnet chain"}
        </span>
      </div>
      <dl className="mx-auto mt-3 flex max-w-2xl flex-wrap items-stretch overflow-hidden rounded-xl border border-border bg-surface/60">
        {cells.map((c) => (
          <div key={c.label} className="flex min-w-[110px] flex-1 flex-col items-center px-3 py-3">
            <dt className="eyebrow">{c.label}</dt>
            <dd className="mt-1 font-mono text-sm text-fg tabular-nums">{c.value}</dd>
          </div>
        ))}
      </dl>
    </div>
  );
}

function SurfaceCard({
  href,
  icon: Icon,
  title,
  body,
}: {
  href: string;
  icon: typeof Compass;
  title: string;
  body: string;
}) {
  return (
    <li>
      <a
        href={href}
        className="flex h-full flex-col rounded-xl border border-border bg-surface p-4 transition-colors duration-150 hover:bg-surface-2"
      >
        <Icon className="size-4 text-blue" />
        <h3 className="mt-3 font-display text-xl tracking-tight text-fg">{title}</h3>
        <p className="mt-1 text-sm leading-relaxed text-muted">{body}</p>
      </a>
    </li>
  );
}

function NetworkCard({
  tone,
  status,
  title,
  body,
  primaryLabel,
  primaryHref,
  secondaryLabel,
  secondaryHref,
  disabled = false,
}: {
  tone: "testnet" | "mainnet";
  status: string;
  title: string;
  body: string;
  primaryLabel: string;
  primaryHref: string;
  secondaryLabel?: string;
  secondaryHref?: string;
  disabled?: boolean;
}) {
  const color = tone === "mainnet" ? "#16a765" : "#f59e0b";
  return (
    <div className="flex h-full flex-col rounded-xl border border-border bg-surface p-5">
      <div className="flex items-center justify-between gap-2">
        <span className="font-mono text-[10px] tracking-wide uppercase" style={{ color }}>
          {title}
        </span>
        <span
          className="inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 font-mono text-[10px] tracking-wide uppercase"
          style={{ backgroundColor: `${color}22`, color }}
        >
          <span className="size-1.5 rounded-full" style={{ backgroundColor: color }} />
          {status}
        </span>
      </div>
      <p className="mt-2 text-sm leading-relaxed text-muted">{body}</p>
      <div className="mt-4 flex flex-wrap gap-2">
        <Button asChild size="sm" disabled={disabled}>
          <a href={primaryHref}>{primaryLabel}</a>
        </Button>
        {!disabled && secondaryHref && (
          <Button asChild variant="outline" size="sm">
            <a href={secondaryHref}>{secondaryLabel}</a>
          </Button>
        )}
      </div>
    </div>
  );
}

function LandingFooter() {
  const domains = [
    { host: SURFACE.landing, label: "landing" },
    { host: SURFACE.testnet, label: "explorer, wallet, map" },
    { host: SURFACE.faucet, label: "testnet KVNC" },
    { host: SURFACE.docs, label: "protocol & node docs" },
    { host: SURFACE.api, label: "API reference" },
  ];
  return (
    <footer className="mt-16 border-t border-border pt-8">
      <div className="grid gap-8 md:grid-cols-3">
        <div>
          <div className="flex items-center gap-2">
            <DagMark variant="gold" className="size-7" />
            <span className="font-display text-xl tracking-tight text-fg italic">Kovanica</span>
            <span className="font-mono text-[10px] tracking-brand text-blue uppercase">
              Protocol
            </span>
          </div>
          <p className="mt-3 max-w-xs text-xs leading-relaxed text-subtle">
            A BlockDAG Layer-1 for programmable value. Native token KVNC, 8 decimals, 90.2M hard
            cap.
          </p>
        </div>
        <div>
          <p className="eyebrow">Domains</p>
          <ul className="mt-3 space-y-1.5 font-mono text-xs">
            {domains.map((d) => (
              <li key={d.host}>
                <a
                  href={d.host}
                  className="text-muted transition-colors hover:text-fg"
                  target="_blank"
                  rel="noreferrer"
                >
                  {d.host.replace("https://", "")}
                </a>{" "}
                <span className="text-subtle">— {d.label}</span>
              </li>
            ))}
          </ul>
        </div>
        <div>
          <p className="eyebrow">Token</p>
          <ul className="mt-3 space-y-1.5 font-mono text-xs text-muted">
            <li>
              Subsidy <span className="text-fg">10 KVNC / block</span>
            </li>
            <li>
              Decay <span className="text-fg">×¾ every 2,000,000 blocks</span>
            </li>
            <li>
              Hard cap <span className="text-fg">90.2M KVNC</span>
            </li>
            <li>
              Fee floor <span className="text-fg">max(1, subsidy/500k) atoms/byte</span>
            </li>
            <li>
              Fees <span className="text-fg">75% burned · 25% producer</span>
            </li>
            <li>
              Consensus <span className="text-fg">GHOSTDAG k=3</span>
            </li>
          </ul>
        </div>
      </div>
      <div className="mt-8 flex flex-col items-center justify-between gap-2 border-t border-border py-4 text-[11px] text-subtle md:flex-row">
        <span className="font-mono">© {new Date().getFullYear()} Kovanica Protocol</span>
        <span className="font-mono">KVNC · kovanica-testnet live · mainnet launching soon</span>
      </div>
    </footer>
  );
}

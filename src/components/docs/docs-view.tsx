import { Link } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import { SourceSwitch } from "@/components/layout/source-switch";
import { Button } from "@/components/ui/button";
import { api } from "@/lib/api/client";
import { type ApiBootstrap } from "@/lib/api/contract";
import { getCurrentSpecText, getNetworkId } from "@/lib/network";
import { SURFACE } from "@/lib/surfaces";

export const SECTIONS = [
  { id: "getting-started", label: "Getting started" },
  { id: "wallet", label: "Wallet" },
  { id: "sdk", label: "SDK" },
  { id: "contracts", label: "Smart Contracts" },
  { id: "consensus", label: "Consensus" },
  { id: "protocol", label: "Protocol" },
  { id: "tokenomics", label: "Tokenomics" },
  { id: "node-ops", label: "Node operations" },
  { id: "api-reference", label: "API reference" },
  { id: "roadmap", label: "Roadmap" },
];

export function DocsView() {
  const [boot, setBoot] = useState<ApiBootstrap | null>(null);
  const [_err, setErr] = useState<string | null>(null);

  useEffect(() => {
    void api<ApiBootstrap>("/api/bootstrap?source=testnet")
      .then(setBoot)
      .catch((e: unknown) => setErr(e instanceof Error ? e.message : "offline"));
  }, []);

  const up = boot?.upstream;
  const _runNode = getCurrentSpecText().split("## Run a public node")[1]?.trim() ?? "";

  return (
    <div className="mx-auto flex w-full max-w-6xl flex-1 gap-8 px-4 py-8 md:px-6">
      {/* Sticky sidebar */}
      <aside className="hidden w-52 shrink-0 lg:block">
        <nav className="sticky top-20 flex flex-col gap-0.5" aria-label="Docs sections">
          <p className="eyebrow mb-2">Sections</p>
          {SECTIONS.map((s) => (
            <a
              key={s.id}
              href={`#${s.id}`}
              className="rounded-md px-2.5 py-1.5 font-mono text-[11px] tracking-wide text-muted uppercase transition-colors hover:bg-surface-2 hover:text-fg"
            >
              {s.label}
            </a>
          ))}
          <div className="mt-4 border-t border-border pt-3">
            <a
              href={SURFACE.api}
              className="rounded-md px-2.5 py-1.5 font-mono text-[11px] tracking-wide text-amber-400 uppercase transition-colors hover:bg-surface-2 hover:text-fg"
            >
              api.kovanica.online →
            </a>
          </div>
        </nav>
      </aside>

      <div className="min-w-0 flex-1">
        {/* Mobile section chips */}
        <div className="mb-6 flex gap-2 overflow-x-auto pb-1 lg:hidden">
          {SECTIONS.map((s) => (
            <a
              key={s.id}
              href={`#${s.id}`}
              className="shrink-0 rounded-full border border-border bg-surface px-3 py-1.5 font-mono text-[10px] tracking-wide text-muted uppercase transition-colors hover:text-fg"
            >
              {s.label}
            </a>
          ))}
        </div>

        <header className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <p className="eyebrow">{getNetworkId()}</p>
            <h1 className="font-display text-3xl tracking-tight text-fg md:text-4xl">
              Documentation
            </h1>
            <p className="mt-2 max-w-xl text-sm leading-relaxed text-muted">
              Getting started, wallet, consensus architecture, deterministic Wasmi runtime and node operations for the Kovanica BlockDAG.
            </p>
          </div>
          <SourceSwitch />
        </header>

        <section className="mt-6 overflow-hidden rounded-xl border border-border">
          <StatusCard
            title="Testnet"
            ok={up?.ok === true || boot !== null}
            lines={[
              `${boot?.blocks ?? 4650} blocks · ${boot?.network ?? "Chain 4"}`,
              `genesis ${boot?.genesis?.slice(0, 10) ?? "0x17e79432"}… (Val 2)`,
            ]}
          />
        </section>

        {/* Getting started */}
        <section id="getting-started" className="mt-10 scroll-mt-20">
          <h2 className="font-display text-2xl tracking-tight text-fg">Getting started</h2>
          <p className="mt-2 text-sm leading-relaxed text-muted">
            Kovanica is a clean, from-scratch Rust Layer-1 blockchain powered by hybrid PoW + GHOSTDAG consensus (4s target block interval, GHOSTDAG k=5) and deterministic Wasmi execution. The native currency is{" "}
            <strong className="text-fg">KVNC</strong> (9 decimals). Testnet is live; mainnet is planned.
          </p>
          <ol className="mt-4 list-decimal space-y-2 pl-5 text-sm leading-relaxed text-muted">
            <li>
              Get testnet KVNC from the{" "}
              <a className="text-fg underline-offset-2 hover:underline" href={`${SURFACE.testnet}/faucet`}>
                faucet
              </a>{" "}
              (10 KVNC per request).
            </li>
            <li>
              Create or import a wallet on{" "}
              <a
                className="text-fg underline-offset-2 hover:underline"
                href={`${SURFACE.testnet}/wallet`}
              >
                testnet.kovanica.online/wallet
              </a>
              .
            </li>
            <li>
              Send your first transaction — the browser signs canonical BLAKE3 sighashes (<code>KUNA-TX-v1</code> compatibility domain) with Ed25519 keys; the node never sees your seed phrase.
            </li>
            <li>
              Watch the DAG grow in real-time on the{" "}
              <a
                className="text-fg underline-offset-2 hover:underline"
                href={`${SURFACE.testnet}/explorer`}
              >
                explorer
              </a>{" "}
              across the block graph.
            </li>
          </ol>
        </section>

        {/* Wallet */}
        <section id="wallet" className="mt-10 scroll-mt-20">
          <h2 className="font-display text-2xl tracking-tight text-fg">Wallet</h2>
          <dl className="mt-3 grid grid-cols-1 gap-3 text-sm sm:grid-cols-2">
            <Fact k="Addresses" v="Canonical PRO-21 address (64 chars: kvnc + 57 base32 + dag)" />
            <Fact k="Derivation" v="ADR-0002: 24-word BIP-39 entropy IS Ed25519 seed; no PBKDF2; no HD; one phrase = one address" />
            <Fact k="Account Model" v="Account-based (sequence nonces)" />
            <Fact k="Signing" v="Client-side Ed25519 + BLAKE3 — keys never leave the browser" />
          </dl>
          <ul className="mt-4 space-y-2 text-sm text-muted">
            <li>
              <strong className="text-fg">Native Transfers</strong> — instant peer-to-peer KVNC transfers with account sequence nonces.
            </li>
            <li>
              <strong className="text-fg">HTLC</strong> — hashed time-locked contracts for atomic cross-chain swaps (<code>kovanica-htlc</code>).
            </li>
            <li>
              <strong className="text-fg">Vaults</strong> — time-lock vaults, linear vesting schedules, and treasury release (<code>kovanica-vault</code>).
            </li>
            <li>
              <strong className="text-fg">Multisig</strong> — M-of-N threshold multi-signature accounts (<code>kovanica-multisig</code>).
            </li>
            <li>
              <strong className="text-fg">Tokens</strong> — deterministic WASM fungible tokens (<code>kovanica-token</code>).
            </li>
          </ul>
        </section>

        {/* SDK */}
        <section id="sdk" className="mt-10 scroll-mt-20">
          <h2 className="font-display text-2xl tracking-tight text-fg">SDK</h2>
          <p className="mt-2 text-sm leading-relaxed text-muted">
            <strong className="text-fg">kovanica-sdk</strong> is the official Rust and WASM SDK for Kovanica. It provides typed domain models, offline transaction builders, and JSON-RPC 2.0 client bindings.
          </p>
          <dl className="mt-3 grid grid-cols-1 gap-3 text-sm sm:grid-cols-2">
            <Fact
              k="Derivation"
              v="ADR-0002: 24-word BIP-39 entropy IS Ed25519 seed (no PBKDF2; no HD; one phrase = one address)"
            />
            <Fact
              k="Sighash"
              v="BLAKE3 KUNA-TX-v1 sighash byte-identical to kovanica-consensus"
            />
            <Fact k="Builders" v="TransferBuilder, HtlcBuilder, VaultBuilder, MultisigSigner" />
            <Fact k="RPC" v="kovanica_syncStatus · kovanica_getLeaderHeader · kovanica_getBalance · kovanica_sendRawTransaction" />
          </dl>
        </section>

        {/* Smart Contracts */}
        <section id="contracts" className="mt-10 scroll-mt-20">
          <h2 className="font-display text-2xl tracking-tight text-fg">Smart Contracts (Wasmi Runtime)</h2>
          <p className="mt-2 text-sm leading-relaxed text-muted">
            Kovanica executes smart contracts in a sandboxed, deterministic WebAssembly engine (Wasmi). All gas metering and opcode execution are strictly deterministic with zero external side effects. Gas fees are paid exclusively in native KVNC.
          </p>
          <dl className="mt-3 grid grid-cols-1 gap-3 text-sm sm:grid-cols-2">
            <Fact
              k="Execution Engine"
              v="Wasmi (Deterministic WASM runtime)"
            />
            <Fact
              k="Gas Metering"
              v="Deterministic opcode & memory instruction counting"
            />
            <Fact
              k="Standard Contracts"
              v="HTLC (Atomic Swaps), Vaults (Vesting), Multisig (M-of-N), Token (CW20-style)"
            />
            <Fact
              k="Fee Currency"
              v="KVNC (min fee 10,000 atoms = 0.00001 KVNC)"
            />
          </dl>
        </section>

        {/* Consensus */}
        <section id="consensus" className="mt-10 scroll-mt-20">
          <h2 className="font-display text-2xl tracking-tight text-fg">Consensus</h2>
          <p className="mt-2 text-sm leading-relaxed text-muted">
            Kovanica pairs Proof-of-Work (PoW) block admission with GHOSTDAG graph ordering. Blocks target a 4-second interval and are topologically ordered by GHOSTDAG with parameter k=5 (target block interval is a network pacing parameter, not a finality guarantee). Proof-of-Work uses the KovanicaHash algorithm (BLAKE3 + 2 MiB memory mix; a formal security audit is required prior to mainnet launch). There is no validator staking, bonding, committee rotation, or slashing in consensus.
          </p>
          <dl className="mt-3 grid grid-cols-2 gap-3 text-sm md:grid-cols-3">
            <Fact k="Consensus" v="Hybrid PoW + GHOSTDAG (k = 5)" />
            <Fact k="Target Interval" v="4.0s target block time" />
            <Fact k="GHOSTDAG k" v="k = 5" />
            <Fact k="PoW Algorithm" v="KovanicaHash (BLAKE3 + 2 MiB mix, audit required)" />
            <Fact k="Consensus Stake" v="None (pure PoW + GHOSTDAG)" />
            <Fact k="Execution Engine" v="Wasmi (Deterministic WASM)" />
          </dl>
        </section>

        {/* Protocol */}
        <section id="protocol" className="mt-10 scroll-mt-20">
          <h2 className="font-display text-2xl tracking-tight text-fg">Protocol</h2>
          <dl className="mt-3 grid grid-cols-2 gap-3 text-sm md:grid-cols-3">
            <Fact k="Token" v="KVNC · 9 decimals" />
            <Fact k="Atom" v="1,000,000,000 atoms / KVNC" />
            <Fact
              k="Initial Reward"
              v="2.5 KVNC / committed block"
            />
            <Fact k="Min fee" v="10,000 atoms (0.00001 KVNC)" />
            <Fact k="Consensus" v="Hybrid PoW + GHOSTDAG (k = 5)" />
            <Fact k="P2P" v="TCP :8000 · seed1, seed2, seed3" />
          </dl>
        </section>

        {/* Tokenomics */}
        <section id="tokenomics" className="mt-10 scroll-mt-20">
          <h2 className="font-display text-2xl tracking-tight text-fg">Tokenomics</h2>
          <p className="mt-2 text-sm leading-relaxed text-muted">
            Kovanica has a locked total supply of 90,200,000 KVNC with geometric subsidy decay every 8,100,000 blocks. Block rewards (initially 2.5 KVNC) are minted to committed block authors.
          </p>
          <dl className="mt-3 grid grid-cols-2 gap-3 text-sm md:grid-cols-3">
            <Fact k="Hard cap" v="90,200,000 KVNC" />
            <Fact k="Subsidy budget" v="81,000,000 KVNC" />
            <Fact k="Initial Reward s₀" v="2.5 KVNC / block" />
            <Fact k="Founder premine" v="200,000 KVNC" />
            <Fact k="Treasury" v="8,000,000 KVNC (1M/yr × 8 yrs linear)" />
            <Fact k="Genesis Allocation" v="1,000,000 KVNC" />
            <Fact k="Subsidy Era" v="8,100,000 blocks" />
            <Fact k="Decay Factor" v="× 3/4 each era" />
            <Fact k="Target Block" v="4.0s (target interval)" />
          </dl>
        </section>

        {/* Node operations */}
        <section id="node-ops" className="mt-10 scroll-mt-20">
          <h2 className="font-display text-2xl tracking-tight text-fg">Node operations</h2>
          <ol className="mt-3 list-decimal space-y-2 pl-5 text-sm leading-relaxed text-muted">
            <li>Testnet RPC is available on port 8545 and proxied via <code>https://rpc-testnet.kovanica.online</code>.</li>
            <li>Transactions require an Ed25519 signature over the BLAKE3 <code>sighash</code>. All nodes verify 64-byte signatures.</li>
            <li>
              P2P nodes listen on TCP <code>0.0.0.0:8000</code>. Bootstrap seed peers:
              <ul className="mt-1 list-disc pl-5 font-mono text-xs">
                <li>seed1.kovanica.online:8000</li>
                <li>seed2.kovanica.online:8000</li>
                <li>seed3.kovanica.online:8000</li>
              </ul>
            </li>
          </ol>
        </section>

        {/* API reference */}
        <section id="api-reference" className="mt-10 scroll-mt-20">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <h2 className="font-display text-2xl tracking-tight text-fg">API reference</h2>
              <p className="mt-2 max-w-xl text-sm text-muted">
                JSON-RPC 2.0 endpoints for querying ledger state, headers, DAG tips, and broadcasting transactions.
              </p>
            </div>
          </div>
          <div className="mt-3 overflow-x-auto rounded-xl border border-border">
            <table className="w-full min-w-[520px] text-left text-sm">
              <thead>
                <tr className="border-b border-border">
                  <th className="px-4 py-2 text-[10px] font-medium tracking-wide text-subtle uppercase">
                    Method
                  </th>
                  <th className="px-4 py-2 text-[10px] font-medium tracking-wide text-subtle uppercase">
                    Parameters
                  </th>
                  <th className="px-4 py-2 text-[10px] font-medium tracking-wide text-subtle uppercase">
                    Description
                  </th>
                </tr>
              </thead>
              <tbody className="font-mono text-xs">
                <tr className="border-t border-border">
                  <td className="px-4 py-2.5 text-amber-400">kovanica_syncStatus</td>
                  <td className="px-4 py-2.5 text-fg">[]</td>
                  <td className="px-4 py-2.5 font-sans text-muted">Current committed height, tip round, peers, and sync status</td>
                </tr>
                <tr className="border-t border-border">
                  <td className="px-4 py-2.5 text-amber-400">kovanica_getLeaderHeader</td>
                  <td className="px-4 py-2.5 text-fg">[height?]</td>
                  <td className="px-4 py-2.5 font-sans text-muted">Committed block header, author, round, and hash</td>
                </tr>
                <tr className="border-t border-border">
                  <td className="px-4 py-2.5 text-amber-400">kovanica_getBalance</td>
                  <td className="px-4 py-2.5 text-fg">[canonicalAddress]</td>
                  <td className="px-4 py-2.5 font-sans text-muted">Account balance in atoms (1 KVNC = 10^9 atoms)</td>
                </tr>
                <tr className="border-t border-border">
                  <td className="px-4 py-2.5 text-amber-400">kovanica_getNonce</td>
                  <td className="px-4 py-2.5 text-fg">[canonicalAddress]</td>
                  <td className="px-4 py-2.5 font-sans text-muted">Current transaction sequence nonce for the account</td>
                </tr>
                <tr className="border-t border-border">
                  <td className="px-4 py-2.5 text-amber-400">kovanica_sendRawTransaction</td>
                  <td className="px-4 py-2.5 text-fg">[bincodeHex]</td>
                  <td className="px-4 py-2.5 font-sans text-muted">Broadcast signed bincode-serialized transaction to mempool</td>
                </tr>
                <tr className="border-t border-border">
                  <td className="px-4 py-2.5 text-amber-400">kovanica_getDagTips</td>
                  <td className="px-4 py-2.5 text-fg">[]</td>
                  <td className="px-4 py-2.5 font-sans text-muted">Current DAG tip block hashes for topological consensus ordering</td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>

        {/* Roadmap */}
        <section id="roadmap" className="mt-10 scroll-mt-20">
          <h2 className="font-display text-2xl tracking-tight text-fg">Roadmap</h2>
          <p className="mt-2 max-w-xl text-sm leading-relaxed text-muted">
            Explore planned protocol milestones, Chain 5 hybrid testnet verification, and mainnet deployment readiness.
          </p>
          <Button asChild variant="outline" size="sm" className="mt-4">
            <Link to="/roadmap">Open roadmap</Link>
          </Button>
        </section>
      </div>
    </div>
  );
}

function Fact({ k, v }: { k: string; v: string }) {
  return (
    <div className="rounded-lg border border-border bg-surface px-3 py-2.5 hover:bg-surface/60 transition-colors">
      <dt className="text-[10px] tracking-wide text-subtle uppercase">{k}</dt>
      <dd className="mt-1 text-fg font-medium">{v}</dd>
    </div>
  );
}

function StatusCard({ title, ok, lines }: { title: string; ok: boolean; lines: string[] }) {
  return (
    <article className="bg-bg p-4 flex items-center justify-between flex-wrap gap-4">
      <div>
        <p className="flex items-center gap-2 text-sm text-fg font-semibold">
          <span className={ok ? "size-2 rounded-full bg-emerald-400 animate-pulse" : "size-2 rounded-full bg-danger"} />
          {title}
        </p>
        <div className="flex flex-wrap items-center gap-3 mt-1.5">
          {lines.map((l) => (
            <span key={l} className="font-mono text-xs text-muted bg-surface px-2 py-0.5 rounded border border-border">
              {l}
            </span>
          ))}
        </div>
      </div>
      <div className="font-mono text-xs text-emerald-400 bg-emerald-500/10 border border-emerald-500/30 px-3 py-1.5 rounded-lg flex items-center gap-2">
        <span className="size-1.5 rounded-full bg-emerald-400" />
        Hybrid PoW + GHOSTDAG · 4s Target
      </div>
    </article>
  );
}

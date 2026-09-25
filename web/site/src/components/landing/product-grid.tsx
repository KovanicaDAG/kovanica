/**
 * Product grid — all existing surfaces kept + NFT card added.
 * Order follows notebook notes + current value.
 */
import { Link } from "@tanstack/react-router";
import {
  Activity,
  Compass,
  Eye,
  Image,
  Layers,
  Lock,
  Map,
  Route,
  Users,
  Vault,
  Wallet,
} from "lucide-react";

type ProductTo =
  | "/explorer"
  | "/wallet"
  | "/map"
  | "/multisig"
  | "/multi-asset"
  | "/network"
  | "/roadmap"
  | "/stealth"
  | "/htlc"
  | "/vaults"
  | "/nft"
  | "/faucet";

const PRODUCTS: {
  to?: ProductTo;
  href?: string;
  icon: typeof Compass;
  title: string;
  body: string;
}[] = [
  {
    to: "/explorer",
    icon: Compass,
    title: "Explorer",
    body: "GHOSTDAG graph, selected chain, live mine and pause — buttons are no longer operator-gated.",
  },
  {
    to: "/wallet",
    icon: Wallet,
    title: "Wallet",
    body: "Create or import a seed, hardware wallets, accounts 0–2, QR, faucet and Ed25519 sends.",
  },
  {
    to: "/multi-asset",
    icon: Layers,
    title: "Multi-asset",
    body: "Native multi-asset UTXOs, AssetPicker, per-asset balances and prepare — KVP-102.",
  },
  {
    to: "/nft",
    icon: Image,
    title: "NFTs",
    body: "Native NFTs (KVP-106). AssetKind::Nft, max_supply = 1, metadata URI. Design complete — ledger next.",
  },
  {
    to: "/multisig",
    icon: Users,
    title: "Multisig",
    body: "M-of-N P2SH addresses, spend proposals, partial signatures and combine — RFC-001.",
  },
  {
    to: "/stealth",
    icon: Eye,
    title: "Stealth",
    body: "One-time ECDH addresses, view tags, scan & spend — KVP-103 / RFC-003.",
  },
  {
    to: "/htlc",
    icon: Lock,
    title: "HTLC",
    body: "Hashed time-locked contracts for atomic swaps — redeem or refund — KVP-104.",
  },
  {
    to: "/vaults",
    icon: Vault,
    title: "Vaults",
    body: "CLTV / CSV time-lock vaults and treasury vesting templates — KVP-105.",
  },
  {
    to: "/network",
    icon: Activity,
    title: "Network",
    body: "Live head, peers, PoW, subsidy, finality and bootstrap seeds for the selected source.",
  },
  {
    to: "/roadmap",
    icon: Route,
    title: "Roadmap",
    body: "RFC status, client surfaces, and what is shipping next on the protocol.",
  },
  {
    to: "/map",
    icon: Map,
    title: "Origins map",
    body: "Choropleth of real origin pulses from recorded visits — record your origin to leave yours.",
  },
];

export function ProductGrid() {
  return (
    <section className="mt-12 md:mt-16" aria-labelledby="products-heading">
      <h2
        id="products-heading"
        className="font-display text-2xl tracking-tight text-fg md:text-3xl"
      >
        Surfaces
      </h2>
      <ul className="mt-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {PRODUCTS.map((p) => (
          <ProductCard key={p.title} {...p} />
        ))}
      </ul>
    </section>
  );
}

function ProductCard({
  to,
  href,
  icon: Icon,
  title,
  body,
}: {
  to?: ProductTo;
  href?: string;
  icon: typeof Compass;
  title: string;
  body: string;
}) {
  const className =
    "flex h-full flex-col rounded-xl border border-border bg-surface p-4 transition-colors duration-150 hover:bg-surface-2";
  const inner = (
    <>
      <Icon className="size-4 text-blue" />
      <h3 className="mt-3 font-display text-xl tracking-tight text-fg">{title}</h3>
      <p className="mt-1 text-sm leading-relaxed text-muted">{body}</p>
    </>
  );

  return (
    <li>
      {href ? (
        <a href={href} className={className}>
          {inner}
        </a>
      ) : (
        <Link to={to!} className={className}>
          {inner}
        </Link>
      )}
    </li>
  );
}

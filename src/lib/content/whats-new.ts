/**
 * What’s New — data for the marketing Landing only (apex).
 * Links point at public hosts (docs / testnet), never relative protocol paths
 * that would make apex look like a testnet clone.
 */

export type WhatsNewItem = {
  date: string;
  title: string;
  body: string;
  tag: "Protocol" | "Wallet" | "Infra" | "DeFi" | "Docs";
  href?: string;
};

const TESTNET = "https://testnet.kovanica.online";
const DOCS = "https://docs.kovanica.online";

export const WHATS_NEW: readonly WhatsNewItem[] = [
  {
    date: "2026-10",
    title: "Chain 4 Testnet live",
    body: "Hybrid PoW + GHOSTDAG consensus (target block 4 s, GHOSTDAG k=5, KovanicaHash PoW). Deterministic Wasmi runtime, no stake in consensus, and native KVNC.",
    tag: "Protocol",
    href: `${TESTNET}/explorer`,
  },
  {
    date: "2026-10",
    title: "Tokenomics confirmed",
    body: "90.2M total supply (9 decimals), 81M subsidy budget, 8M linear treasury, 1M genesis allocation, and 2.5 KVNC initial block reward minted to committed block authors.",
    tag: "Protocol",
    href: DOCS,
  },
  {
    date: "2026-10",
    title: "Smart contracts & Wasmi",
    body: "Deterministic WASM smart contracts: HTLC atomic swaps, CLTV/CSV vaults, M-of-N Multisig, and KVP-102 native tokens.",
    tag: "Wallet",
    href: `${TESTNET}/multisig`,
  },
  {
    date: "2026-10",
    title: "Domain & Node Ingress",
    body: "Clean host routing across kovanica.online, testnet, explorer, and seed RPC nodes with automated Let's Encrypt SSL and high-performance Nginx reverse proxy.",
    tag: "Infra",
    href: `${TESTNET}/network`,
  },
  {
    date: "2026-10",
    title: "Chain 5 Hybrid Testnet & KovanicaHash",
    body: "PoW block admission with BLAKE3 + 2 MiB memory mix (KovanicaHash, audit required before mainnet) and GHOSTDAG DAG ordering with 4 s target block interval.",
    tag: "Protocol",
    href: DOCS,
  },
  {
    date: "1.1.2027",
    title: "Public Launch Planned",
    body: "Public launch scheduled for 1.1.2027 following hybrid testnet milestones and security audits; 8.1M block halving eras with × 3/4 decay.",
    tag: "Protocol",
    href: DOCS,
  },
] as const;

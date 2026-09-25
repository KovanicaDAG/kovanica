/**
 * What’s New — data for the Landing section.
 * Keep newest first. Tag values are free-form but kept short.
 */

export type WhatsNewItem = {
  date: string;
  title: string;
  body: string;
  tag: "Protocol" | "Wallet" | "Infra" | "NFT" | "DeFi" | "Docs";
  href?: string;
};

export const WHATS_NEW: readonly WhatsNewItem[] = [
  {
    date: "2026-09",
    title: "RFC-006 live on testnet",
    body: "New genesis, full supply fields on /api/head (subsidy, circulating, burned, max_supply). Maturity rules active.",
    tag: "Protocol",
    href: "/docs",
  },
  {
    date: "2026-09",
    title: "KVP-102 Multi-asset",
    body: "Native multi-asset UTXOs, balances map, AssetPicker, prepare with explicit asset_id. Fees always in KVNC.",
    tag: "Wallet",
    href: "/multi-asset",
  },
  {
    date: "2026-09",
    title: "Domain architecture",
    body: "kovanica.online = Landing · testnet. · mainnet. · docs. · api. Host-role chrome and Cloudflare path rules live.",
    tag: "Infra",
  },
  {
    date: "2026-09",
    title: "Stealth · HTLC · Vaults",
    body: "KVP-103 / 104 / 105 surfaces on testnet: one-time addresses, atomic swaps, CLTV/CSV vaults.",
    tag: "Wallet",
  },
  {
    date: "Coming",
    title: "KVP-106 Native NFTs",
    body: "AssetKind::Nft, max_supply = 1, metadata URI. Design complete (RFC-007 draft). Ledger Phase 1 next.",
    tag: "NFT",
    href: "/nft",
  },
] as const;

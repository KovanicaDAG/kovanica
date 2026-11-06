/**
 * About / Founder content for Landing (apex only).
 * Aligns with roadmap-pack/Kovanica-Founder-Page-Outline.md
 * All hrefs absolute so apex never routes into protocol SPA paths.
 */

export const ABOUT = {
  mission:
    "Kovanica is a clean, from-scratch Layer-1 blockchain built for high-throughput payments, deterministic Wasmi smart contracts, and native KVNC currency. Early stewardship is concentrated and accountable; long-term control moves to a transparent Foundation.",

  founder: {
    name: "Antonio Đuranec",
    role: "Founder & Protocol Steward",
    bio: "Building Kovanica full-time. Focused on hybrid PoW + GHOSTDAG consensus (4s target block interval, k=5, KovanicaHash PoW), deterministic Wasmi runtime, Ed25519 cryptography, and a clean path from testnet to mainnet with progressive decentralisation.",
    location: "Istra, Hrvatska",
  },

  foundation: {
    status: "Foundation in formation",
    holds: "Domains, trademarks, treasury, and protocol IP will move under the Foundation for legal continuity and broader community governance.",
  },

  links: [
    { label: "Docs", href: "https://docs.kovanica.online" },
    { label: "GitHub", href: "https://github.com/KovanicaDAG" },
    { label: "Explorer", href: "https://testnet.kovanica.online/explorer" },
    { label: "Roadmap", href: "https://testnet.kovanica.online/roadmap" },
    { label: "Testnet", href: "https://testnet.kovanica.online" },
    { label: "Dev", href: "mailto:dev@kovanica.online" },
    { label: "Security", href: "mailto:security@kovanica.online" },
  ] as const,

  /** Personal GitHub (founder) */
  personalGithub: "https://github.com/BetterCallDzuks",
  xKovanica: "https://x.com/KovanicaDAG",
  xFounder: "https://x.com/BetterCallDzuks",

  transparency:
    "Early control is intentional and documented. Tokenomics (90.2M KVNC total supply, 81M subsidy budget, 8M treasury, 1M genesis allocation, 0.2M founder premine, 2.5 KVNC initial block reward) are public and locked. The goal is progressive decentralisation under transparent rules.",
} as const;

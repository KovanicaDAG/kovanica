# Kovanica Domain Migration — Cut-over Checklist

> Use this checklist in order. Do not skip steps.

## Phase 0 — Pre-flight (already done or confirm)

- [ ] DNS records exist and are correct:
  - `testnet.kovanica.online` → A (Proxied)
  - `mainnet.kovanica.online` → A (Proxied)
  - `faucet.testnet.kovanica.online` → A (Proxied)
  - `api.kovanica.online` → A (Proxied)
  - `kovi.kovanica.online` kept
  - Old records removed: `kovanica.kovanica.online`, `map.kovanica.online`
- [ ] Seeds remain grey-cloud (`seed`, `seed2`, `seed3`)
- [ ] `NETWORK.md` committed to the monorepo

## Phase 1 — Cloudflare Redirect Rules

- [ ] Rule 1: `map.kovanica.online` → `api.kovanica.online` (301)
- [ ] Rule 2: `kovanica.kovanica.online` → `faucet.testnet.kovanica.online` (301)
- [ ] Rule 3: `www.kovanica.online` → `kovanica.online` (301)
- [ ] Rule 4: Selected root paths → `testnet.kovanica.online` (302 for now)
- [ ] Rules are ordered correctly (highest priority first)
- [ ] Test each rule with curl or browser (see Verification Checklist)

## Phase 2 — Frontend (Web UI)

- [ ] `src/lib/network.ts` added (canonical config + detectNetwork + switchNetwork)
- [ ] `src/components/layout/source-switch.tsx` updated to hard-redirect
- [ ] `NetworkBadge` component added and shown in header
- [ ] All hard-coded network URLs / IDs replaced with `getCurrentNetwork()`
- [ ] Root landing page cleaned (see `02-ROOT-LANDING-STRUCTURE.md`)
- [ ] Build + deploy web UI to all relevant origins (root, testnet, mainnet)

## Phase 3 — Content & Documentation

- [ ] Update monorepo README links
- [ ] Update `kovanica-node` install script / README if it mentions old URLs
- [ ] Update docs.kovanica.online navigation and any hard-coded links
- [ ] Commit `NETWORK.md` (and optional public version)
- [ ] Post announcement (Telegram / Discord / GitHub)

## Phase 4 — Soft launch & monitoring

- [ ] Run full Verification Checklist
- [ ] Monitor Cloudflare analytics / errors for 24–48 h
- [ ] Watch for broken external links or community reports
- [ ] Keep Rule 4 (root path redirects) active until traffic settles

## Phase 5 — Later (Mainnet launch)

- [ ] Follow `07-MAINNET-ACTIVATION.md`
- [ ] Disable or delete temporary Rule 4 once mainnet is live and root is clean
```


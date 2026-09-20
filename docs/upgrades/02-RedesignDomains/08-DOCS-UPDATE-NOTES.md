# Suggested Documentation Updates

## Files that should be reviewed / patched

### 1. Monorepo root README
- Replace any hard-coded `https://kovanica.online/explorer` style links
- Point interactive surfaces to `testnet.kovanica.online`
- Add a short “Domains” section that links to `NETWORK.md`

### 2. kovanica-node README / install script
- Seed list should mention `seed.kovanica.online`, `seed2`, `seed3`
- Any “connect to testnet” instructions should use the new hostnames
- Avoid assuming the root domain serves the live API

### 3. docs.kovanica.online
- Add or update a “Network & Domains” page using the content from `05-NETWORK-PUBLIC.md`
- Make sure RFC and KVP pages do not deep-link into old root paths that no longer exist

### 4. TESTNET-RFC006.md / RFC-006 related docs
- Update any URLs that still point at the old mixed root
- Clarify that post-migration the live testnet UI lives on the testnet subdomain

### 5. LEGIT-BOARD.md or roadmap pages
- No structural change required, but links to live surfaces should be checked

## Principle
Prefer linking to the **subdomain** that owns the feature rather than to a path on the root.
```


# Seed Topology Recommendations

## Current state (from DNS export)

| Seed                     | IP                 | Proxied | Notes                    |
|--------------------------|--------------------|---------|--------------------------|
| seed.kovanica.online     | 145.223.116.178    | No      | Primary                  |
| seed2.kovanica.online    | 145.223.116.178    | No      | Same machine as primary  |
| seed3.kovanica.online    | 3.70.236.3         | No      | Different machine (good) |

## Recommendations

1. **Keep all seeds grey-cloud** (never proxy through Cloudflare).  
   P2P on TCP 9000 must see the real IP.

2. **Improve diversity**  
   `seed` and `seed2` currently share the same IP. Ideal end state:
   - seed  → machine A
   - seed2 → machine B
   - seed3 → machine C (already different)

3. **Geographic / provider diversity** (nice to have)  
   Different data centres or providers reduce correlated failures.

4. **Documentation**  
   Publish the seed list in:
   - NETWORK.md
   - docs.kovanica.online
   - node README / install script

5. **Health**  
   Consider a simple external check that verifies TCP 9000 is open on each seed and surfaces it on status.kovanica.online.

## Minimal action right now

- No urgent change required for the domain migration.
- When convenient, move `seed2` to a second machine so it is no longer an alias of `seed`.
```


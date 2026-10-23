# GitHub Actions — currently disabled

**Status: all five workflows are `disabled_manually` on GitHub since 2026-10-23.**

Reason: GitHub Actions is blocked at the account level. Every job of every run fails in
2–3 s with no runner assigned and one annotation:

> The job was not started because recent account payments have failed or your spending
> limit needs to be increased. Please check the 'Billing & plans' section in your settings

The YAML here is not broken. Re-running or editing it will not help until billing is fixed.
Tracked as **PRO-14** in Linear.

## What is off

| Workflow | File | Lost while disabled |
| -- | -- | -- |
| CI | `ci.yml` | fmt/clippy/build/test, MSRV, security audit, **faucet deploy on release** |
| Secrets Scan | `secrets.yml` | nightly full-history gitleaks scan |
| Soak | `soak.yml` | nightly docker-compose soak |
| chaos / fault-injection | `chaos.yml` | manual fault-injection runs (already `workflow_dispatch` only) |
| Deploy Web to Cloudflare Pages | `web-deploy.yml` | automatic `packages/web` deploys — publish manually |

## Re-enabling

After the billing/spending limit is cleared:

```sh
for wf in ci secrets soak chaos web-deploy; do
  gh workflow enable ".github/workflows/$wf.yml"
done
gh workflow list --all
```

Then re-run `main` and compare against the local baseline established on `8bb3929c`:

| Gate | Result |
| -- | -- |
| `cargo fmt --all --check` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass, 0 warnings |
| `cargo test --workspace` | 1035 passed, 0 failed, 1 ignored (83 suites) |

Never validated by any runner: MSRV 1.90, `--all-features`, docker-compose soak, the
gitleaks workflow itself (the local pre-commit hook does run and passes).

## While disabled

- A PR showing "no checks reported" is **unverified**, not green.
- Run the three gates above locally before merging consensus, ledger or wire changes.
- Do not delete these files to silence badges — disable is reversible, deletion is not.

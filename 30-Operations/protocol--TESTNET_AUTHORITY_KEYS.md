# Testnet Authority Keys — BURNED AND REMOVED

> ## These keys are burned. Do not use, deploy, or restore them.
>
> The PoA authority **signing secrets** formerly stored in this file were
> committed to a **public** repository on 2026-09-26 (P0 secret-leak
> incident). With `KOVANICA_AUTHORITY_THRESHOLD=2` of 3, anyone who cloned
> the repository could forge authority-signed blocks for the whole set.
>
> **Rotation is the remediation. Rewriting git history is not** — by the time
> a secret reaches a public remote, assume every byte is already copied.

**Status of the burned set**

- All three keys rotated out of service on 2026-09-26.
- The host that was running a burned key was stopped and its unit `disable`d.
- The burned unit file is retained on that host only, as
  `kovanica-seed2.service.burned-key-20260926-062642` (mode 0600), for forensics.
- The installed replacement key was verified to be *not* one of the burned three.

**Where the replacement keys are**

Replacement keys were generated on 2026-09-26 and are held **outside this
repository**, one secret per host, in mode-`0600` `EnvironmentFile`s. They
appear in no commit, on no branch, in no document, and in no chat. Their
distribution is governed by [`docs/AUTHORITY-KEY-CEREMONY.md`](docs/AUTHORITY-KEY-CEREMONY.md).

## Generating key material

Run the generator **on the host that will hold the secret**, then move the
resulting secret off any machine that also holds the other secrets:

```sh
cargo run --release --example generate_authority_keys -- \
  --out-dir /etc/kovanica/authority-keys \
  --count 3 \
  --threshold 2 \
  --slot-duration 3000
```

Flags: `--out-dir <DIR>` (default `./authority-keys`), `--count <N>`
(default 3), `--threshold <T>` (default: a strict majority of `--count`,
i.e. `count/2 + 1`; `AuthoritySet` enforces `MIN_THRESHOLD=2`), and
`--slot-duration <MS>` (default 3000). Secrets are written to mode-`0600`
files and **never to stdout**. The generator exits non-zero if an output file
already exists.

## The one rule

A **public** key set (`KOVANICA_AUTHORITIES`) is safe to commit — it is
published on-chain by `AuthorityUpdateTx` anyway.

A **signing** key (`KOVANICA_AUTHORITY_KEY`) must never be committed,
pasted into a document, issue, or chat, or written into a systemd `unit`
file. Put it in a `0600` `EnvironmentFile=` and move on.

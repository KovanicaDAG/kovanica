# VERIFIED-FACTS — verified atomic facts

One fact per bullet. Every fact carries its **source**: where and how it was
verified (file, command output, commit). No source → tag `[unverified]` or
leave it out. Stale facts get corrected in place with a new source — never
silently left to rot.

## Runtime & services

- API listens on `:PORT` — verified: `<file/config> @ <date>`

## Paths & layout

- Data lives in `<path>` — verified: `<command> @ <date>`

## Environment variables

- `VAR_NAME` — purpose, where set, default — verified: `<source>`

## Versions & dependencies

- Language/runtime version — verified: `<command> @ <date>`

## External systems

- Service X at <url/host>, auth via <mechanism, never the secret itself>

---

*From the MARKDOWN.god doctrine: The Laws, Law 0 (*Reality is the only source*) — loaded at session start (Session Operating Loop, Bootstrap step 3)*

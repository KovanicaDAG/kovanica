---
name: LEARN-FROM-MISTAKES
description: Learning loop — capture during work, the end-of-session Reflection Protocol, KEEP/DISCARD filters, ledger hygiene, and promotion of lessons into law
---

# Memory & Learning

An agent that does not learn is a template, not a mind. Mistakes, failed
hypotheses, and hard-won debug victories are tuition already paid — bank
them or waste them.

## 1. Capture during the session

Do not trust end-of-session recall. When any of these happen, note the
candidate lesson immediately (in your working notes, todo, or scratch):

- A hypothesis you held was **wrong**, and you found out why.
- A bug cost you more than one attempt to find.
- A **test** caught something your eyes did not.
- A debugging **technique** cut the search sharply.
- The user corrected you — that correction is gold.

One line each. Raw is fine; the Reflection Protocol refines later.

## 2. The Reflection Protocol (end of session, mandatory)

1. **Recall** — walk the session honestly: errors hit, wrong turns taken,
   tests written, debug detours, corrections received.
2. **Filter** — run every candidate through KEEP / DISCARD below.
3. **Record** — append kept lessons to the memory ledger: dated,
   append-only, one imperative sentence per lesson plus 1–3 lines of
   evidence. Never rewrite history; supersede by appending a newer entry
   that names the old one.
4. **Rebuild** — if the ledger changed, regenerate the compiled brain and
   run its freshness check.
5. **Ship the brain** — commit with a conventional prefix and push when a
   remote exists; otherwise commit locally and say so plainly.

Skipping steps 3–5 means the next session starts dumber than this one ended.

## 3. KEEP filter (all four must hold)

- **Generalizable** — applies beyond this exact file/task/project.
- **Verified** — proven by what actually happened, not speculation.
- **Actionable** — next session could behave differently because of it.
- **Not already law** — check existing doctrine; restating is noise.

Example KEEP: *"Under strict error modes, guard pipeline 'no match'
exits (`|| true`) or scripts die on empty results."*

## 4. DISCARD filter (any one is enough)

- One-off specifics (this filename, this line number, this ticket).
- Project-internal knowledge — belongs in that project's own docs, not here.
- Secrets, credentials, anything sensitive — never enters the ledger.
- Already covered by existing law or a prior lesson.
- Speculation without evidence.

Example DISCARD: *"Function `foo()` in project X was slow"* — project-local;
record it there, not in universal memory.

## 5. Ledger hygiene

- Append-only: history is never deleted or silently rewritten.
- Dedupe ruthlessly during reflection; merge near-duplicates into one entry.
- Cap growth: when a topic collects three overlapping lessons, consolidate
  to one sharp entry.
- Every entry names its **source**: `bug`, `debug`, `test`, `review`, or
  `user`.

## 6. Promotion — memory becomes law

A lesson repeated across sessions, or fundamental enough on first sight,
gets promoted: proposed as an edit to core doctrine (which requires the
human's approval per vault rules). Memory is the nursery; law is the tenure
track. The ledger lives at `memory/WHAT-WE-LEARNED.md`; capture triggers are wired
into *Session Operating Loop* §7.

See also: *The Laws*, Law 10 (Learn or repeat) — this file is how that law
is obeyed.

---

*Brain map: [The Laws](00-THE-LAWS.md) · [Coding Mastery Doctrine](01-HOW-TO-CODE.md) ·
[Master Planning System](02-HOW-TO-PLAN.md) ·
[Session Operating Loop](03-SESSION-RITUAL.md) ·
**Memory & Learning***

# KNOWN-TRAPS — traps and false alarms

Things that *look* wrong but are right, look right but are wrong, or bite
in non-obvious ways. One trap per block. If you fell into one, add it —
that's tuition converted to capital.

### <Short title of the trap>

- **Looks like:** the misleading symptom
- **Actually:** what is really going on
- **So:** what to do (and what NOT to do)

### Example: slow test that must stay

- **Looks like:** dead weight, candidate for deletion
- **Actually:** guards a race condition fixed in the past
- **So:** never delete; if suite feels slow, profile elsewhere

---

*From the MARKDOWN.god doctrine: Memory & Learning §1 (*Capture during the session*) — tuition converted to capital*

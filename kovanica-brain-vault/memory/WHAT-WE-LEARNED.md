# Lessons Ledger

Append-only memory of what the agent learned. Format per entry:

```
### YYYY-MM-DD — <short-slug>
- **Source:** bug | debug | test | review | user
- **Lesson:** <one imperative sentence>
- **Evidence:** <1–3 lines, what actually happened>
```

Rules live in core doctrine (*Memory Rules live in core doctrine (`Memory & Learning`). Never delete history — Learning*). Never delete history —
supersede by appending. Promote recurring lessons into core law.

---

### 2026-08-24 — guard-no-match-exits
- **Source:** bug
- **Lesson:** Under strict error modes, guard pipeline "no match" exits (`|| true`) or scripts die on empty results.
- **Evidence:** Registry scan loops aborted mid-run when a glob matched nothing; `grep -v` returning exit 1 killed scripts under `set -euo pipefail` until guarded.

### 2026-08-24 — defensive-config-autosave
- **Source:** debug
- **Lesson:** When auto-editing structured configs, parse defensively, preserve unknown syntax, and fall back to printing manual instructions rather than corrupting the file.
- **Evidence:** Merging a settings file with comments required stripping only line comments; full rewrite would have destroyed any syntax the parser didn't understand, so the merge fails soft with instructions instead.

### 2026-08-24 — compile-then-diff-drift-test
- **Source:** test
- **Lesson:** Make artifact freshness a testable property: rebuild to temp, diff against the shipped artifact, fail loudly on drift.
- **Evidence:** Generated brain drifted from sources between edits; a `--check` mode (compile → diff → exit status) turned silent staleness into a loud, greppable failure.

### 2026-08-24 — dht-lookup-target-preservation
- **Source:** bug
- **Lesson:** Never filter out the query target from iterative graph or DHT routing results.
- **Evidence:** `NodeLookup::add_results` discarded candidates whose NodeId matched the search target, returning only intermediate neighbours instead of the exact target contact.

### 2026-08-24 — peer-cache-endpoint-refresh
- **Source:** debug
- **Lesson:** Always refresh network endpoint addresses alongside timestamps when updating existing peer routing entries.
- **Evidence:** K-bucket contact updates refreshed `last_seen_ms` but preserved stale addresses, causing routing failure when peers reconnected from dynamic addresses.

### 2026-08-24 — supervisor-port-isolation
- **Source:** bug
- **Lesson:** Never run overlapping process supervisors over the same network ports, and use atomic swap operations when deploying shared running binaries.
- **Evidence:** Dual supervision under pm2 and systemd caused zombie processes to contend for P2P ports, while direct overwrites of running node binaries triggered ETXTBSY until atomic install/mv was used.

### 2026-08-24 — peer-replenish-candidate-filtering
- **Source:** bug
- **Lesson:** When replenishing connections from a routing table, query candidates broadly and filter already connected peers rather than truncating candidate queries to the deficit count.
- **Evidence:** Truncating DHT peer queries to `needed` returned the already-connected closest peers, preventing discovery of further unconnected routing table contacts and leaving nodes under-connected.

### 2026-08-24 — dns-resolver-host-port-normalization
- **Source:** bug
- **Lesson:** Robustly parse and strip embedded ports from DNS seed hostnames before formatting with default ports.
- **Evidence:** Seed hostnames specifying explicit ports (e.g., `seed.kovanica.online:9000`) caused double-port formatting (`:9000:9000`), breaking socket address resolution in `StdDnsResolver`.

### 2026-08-24 — artifact-build-timestamp-determinism
- **Source:** test
- **Lesson:** Use source file modification timestamps instead of dynamic system time for compiled artifact headers in drift checks.
- **Evidence:** `Obsidian-Vault/scripts/build-god-brain.sh --check` constantly failed drift validation because dynamic UTC timestamps produced false positives between build time and test time.

### 2026-08-24 — dns-resolver-ipv6-bracket-and-raw-ip-parsing
- **Source:** review
- **Lesson:** Strip brackets and isolate ports before delegating to `ToSocketAddrs` to handle both bracketed IPv6 and raw IP strings consistently.
- **Evidence:** IPv6 addresses with explicit ports (e.g., `[::1]:9000`) caused double-port concatenation (`[::1]:9000:9000`) when `host.starts_with('[')` bypassed port stripping in `StdDnsResolver`.

### 2026-08-24 — dht-message-response-sender-identity
- **Source:** review
- **Lesson:** Wire response frames must identify the responding node rather than echoing the requester's NodeId.
- **Evidence:** `handle_dht_msg` and `handle_relay_query` echoed the incoming request's `sender` NodeId in `Pong` and `Nodes` responses instead of returning the local node's `local_id`.

### 2026-08-24 — p2p-overlay-saturation-in-dht-tests
- **Source:** test
- **Lesson:** When testing DHT connection replenishment, inject isolated unpeered nodes to avoid false negatives from concurrent gossip-flood peer saturation.
- **Evidence:** Hello message gossip connected all peers across the graph diameter during drain phases, causing replenishment to no-op until an isolated node with zero initial connections was tested.

### 2026-08-24 — kbucket-prune-replacement-promotion
- **Source:** review
- **Lesson:** When batch pruning dead routing table contacts, promote waiting replacement cache candidates up to bucket capacity to prevent under-capacity degradation.
- **Evidence:** `KBucket::prune_dead` cleared unresponsive peers without promoting available candidates from `replacement_cache`, leaving buckets under-filled until subsequent individual lookups occurred.

### 2026-08-24 — dht-lookup-shortlist-endpoint-refresh
- **Source:** review
- **Lesson:** Update cached contact network endpoints and timestamps when duplicate candidates return with fresher data during iterative DHT lookups.
- **Evidence:** `NodeLookup::add_results` dropped responses for existing candidates, retaining stale endpoint addresses during active IP churn.

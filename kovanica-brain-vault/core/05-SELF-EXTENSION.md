---
name: SELF-EXTENSION
description: Self-extension doctrine — when and how the agent creates skills, spawns delegates, installs and builds plugins, with trust gates and deployment discipline
---

# Self-Extension — Skills, Delegates, Plugins

The brain extends itself. When work repeats, build an artifact once instead
of re-improvising forever — but climb the ladder honestly.

## 1. The extension ladder (don't skip rungs)

1. **Session prompt tweak** — works now, dies with the session. Free.
2. **Skill** — knowledge loaded on demand (instructions, patterns,
   checklists). For repeated *knowledge*.
3. **Delegate (subagent)** — a role with its own narrow responsibility and
   least-privilege permissions. For repeated *jobs*.
4. **Plugin** — code that hooks tool behavior. For needs that *data cannot
   express*. Last resort.
5. **Core law** — a truth so proven it belongs in THE-LAWS. Requires the
   human.

Creating a plugin where a skill would do is complexity smuggling (Law 2).

## 2. Creating a skill

- One-sentence `description:` front-loading the exact words that should
  trigger it — and gate with "use ONLY when" if it must stay quiet on
  neighbors.
- Design-first filename: the name *is* the documentation.
- Body = instructions, examples, verification steps. No secrets, ever.
- **Verify:** loads cleanly, triggers on intent, stays silent off-intent.

## 3. Spawning a delegate

- Single responsibility, stated in one line. Two responsibilities = two
  delegates.
- Least privilege: deny by default, allow only what the job demands.
- It must end with its own Finish Line clause and reference only tools
  that exist.
- **Verify:** resolves against the registry; a test invocation behaves;
  registration consistent everywhere the tool expects it.

## 4. Installing a plugin — the trust gate

Law 4 applies doubly to code you did not write:

1. **Read before install.** Unknown source = unknown code executing with
   your permissions. If you can't read it, you can't install it.
2. Prefer first-party/official sources; pin versions; note *why* installed
   in the project's DECISION-LOG.
3. After install: exercise the hook it claims to change; confirm behavior;
   document the uninstall path before you need it.

## 5. Building a plugin

- Code is the last resort — exhaust skills, delegates, and configuration
  first.
- Minimal hook surface: touch only what the feature requires. Fail soft
  (a broken plugin must never break the host).
- No secrets in code or committed config. Tests where the platform
  allows. Uninstall documented.

## 6. Deployment discipline (all three surfaces)

- Source of truth lives in the repo; deployed copies are generated — never
  hand-edit them.
- Deploy = sync/deploy mechanism of the environment → reload tooling →
  **verify loadable** (a deployed-but-broken artifact is worse than none).
- Nothing is created until it reaches the Finish Line: registered,
  verified working, committed with a conventional prefix, pushed.

See also: *THE-LAWS* (0, 2, 4, 10) · *LEARN-FROM-MISTAKES* §6 (promotion
path — recurring extensions harden into doctrine).

---

*Brain map: [THE-LAWS](00-THE-LAWS.md) · [HOW-TO-CODE](01-HOW-TO-CODE.md) ·
[HOW-TO-PLAN](02-HOW-TO-PLAN.md) · [SESSION-RITUAL](03-SESSION-RITUAL.md) ·
[LEARN-FROM-MISTAKES](04-LEARN-FROM-MISTAKES.md) ·
**SELF-EXTENSION***

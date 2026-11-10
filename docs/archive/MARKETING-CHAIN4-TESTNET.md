# Chain 4 testnet — launch pack for X, Facebook and Discord

Everything here is copy-paste ready. No placeholders: the links are live and were
checked on 2026-10-28, the images are in `docs/marketing/`.

**What is being announced:** the public testnet (chain id **4**) is now the
default network on a fresh install of every Kovanica app — Android wallet,
lightnode, comms — and the desktop console and comms app got the same network
switch. Previously you had to paste an RPC endpoint by hand, and the faucet
button pointed at a host that did not resolve.

**Download links used below** (short, branded, they 302 to the pinned release
asset, so they always serve the current build):

| App | Link |
|---|---|
| Wallet 0.3.3 | https://kovanica.online/download/kovanica-wallet-0.3.3.apk |
| Lightnode 0.8.4 | https://kovanica.online/download/kovanica-lightnode-0.8.4.apk |
| Comms 0.2.0 | https://kovanica.online/download/kovanica-comms-0.2.0.apk |
| All releases | https://github.com/KovanicaDAG/kovanica/releases |

Canonical GitHub assets, if someone wants to verify a hash:

| App | Asset | sha256 |
|---|---|---|
| Wallet 0.3.3 | `v0.3.3-wallet/Kovanica-Wallet-0.3.3-arm64-v8a.apk` | `945aede119f650a382c6d6eda0e1902a1d98a7dd2ea72f3d045f2a7d10b95d4c` |
| Lightnode 0.8.4 | `v0.8.4-android/Kovanica-Lightnode-0.8.4-arm64-v8a.apk` | `b288873eeda80fd50745b676575345a661fe12005d4e3a08ad887f69a139be9b` |
| Comms 0.2.0 | `v0.2.0-comms/kovanica-comms-0.2.0.apk` | `0255fa08e14ac430dceae9fd4fd435f5df97b362d3693221043f96ff167cd902` |

All three are signed with the same release certificate as earlier builds
(SHA-256 `630a6308e7790287e32f396e97bf5555e355031b1cef2b9c3a263a92673c4562`), so
they install over 0.8.3 / 0.3.2 / 0.1.0 without uninstalling and the phone wallet
survives.

## Guardrails (do not break these)

- Chain **4** is testnet. Mainnet is chain **1** and is **not open** — say that
  plainly, never imply a date.
- Testnet KUNA is **worthless**. Contracts are **not audited**. Say both in every
  long post; it buys credibility and pre-empts the "when moon" replies.
- No price talk, no listing talk, no "production ready", no audit claims.
- The coin is **KUNA**. Never write `KVNC` as the currency. Addresses are
  `kvnc…dag`.
- Numbers you may use: 4 s round, committed leader every 12 s, 15–21 active
  validators, minimum stake 50 000 KUNA, 9 decimals, 90.2 M total supply,
  2.5 KUNA initial block reward, faucet 1 KUNA per request / 10 KUNA lifetime.

---

## X (Twitter)

Character counts below are Twitter-weighted (URL = 23, emoji = 2). All fit a
single tweet.

### Single post — 249 chars

```
Kovanica Chain 4 testnet is live — and now the default network in every app we ship.

4s rounds, committed leader every 12s, KUNA at 9 decimals, deterministic WASM execution.

Install, open, you're on testnet. No RPC config.

https://kovanica.online
```

Attach: `chain4-x-1600x900.png`.

### Thread — post 1 of 5, 198 chars

```
Kovanica Chain 4 testnet is live.

And as of today it's the default network in every app we ship: mobile wallet, lightnode, comms, desktop console.

No RPC hunting. Install, open, you're on chain 4.
```

### 2/5 — 247 chars

```
What changed in the apps:

• Testnet (chain 4) preselected on a fresh install
• Mainnet (chain 1) listed and clearly closed
• Local (1337) stays one tap away for devs
• Custom RPC still there

Plus a faucet button that actually reaches the faucet.
```

### 3/5 — 205 chars

```
Start with the wallet:

→ create a PRO-21 address (kvnc…dag)
→ claim 1 KUNA from the faucet, 10 KUNA lifetime cap per address
→ send, receive, check history

No install? Web wallet: https://kovanica.online/wallet
```

### 4/5 — 235 chars

```
Want to verify instead of trust?

The lightnode follows the DAG from your phone: committed leader height, epoch, validator set, balances. 4s rounds, 12s committed leaders.

Comms puts encrypted peer-to-peer messaging on the same chain.
```

### 5/5 — 222 chars

```
Everything is open:

Explorer https://explorer.kovanica.online
Faucet https://faucet.kovanica.online/faucet
Docs https://docs.kovanica.online
Code https://github.com/KovanicaDAG/kovanica

Testnet coins are worthless and contracts are unaudited. Break it and tell us.
```

### Replies to have ready

> **"Is this mainnet?"** — No. Chain 4 is the public testnet. Mainnet is chain 1
> and it is not open. No date announced.

> **"How do I get coins?"** — Faucet: https://faucet.kovanica.online/faucet —
> 1 KUNA per request, 10 KUNA lifetime per address. It is testnet money.

> **"iOS?"** — Android arm64 today. The web wallet runs in any mobile browser:
> https://kovanica.online/wallet

> **"Why no app store?"** — Sideload only for now, signed with our release key,
> source is on GitHub.

---

## Facebook

### English

```
Kovanica Chain 4 testnet is live — and it is now the default in every app.

If you installed an earlier build, you had to type an RPC endpoint by hand. That
is gone. Open the wallet, the lightnode, the comms app or the desktop console and
you are already on Testnet (chain 4). Mainnet is visible but closed, and Local
(1337) is still one tap away if you run your own node.

What you can do right now, for free:

• Create a wallet with a native PRO-21 address (kvnc…dag)
• Claim test KUNA from the faucet — 1 KUNA per request, 10 KUNA lifetime cap
• Send and receive KUNA in seconds
• Follow the DAG from your phone with the lightnode
• Send encrypted peer-to-peer messages with Comms
• Deploy HTLC, Vault, Multisig and Token contracts

Under the hood: Mysticeti-style uncertified DAG, a 4-second round with a
committed leader every 12 seconds, deterministic WASM execution, 15–21 staked
validators, and KUNA with 9 decimals.

This is still testnet. Coins are worthless, contracts are not audited, and things
will break. That is what we are here for — try to break it and tell us what you
find.

Wallet: https://kovanica.online/download/kovanica-wallet-0.3.3.apk
Lightnode: https://kovanica.online/download/kovanica-lightnode-0.8.4.apk
Comms: https://kovanica.online/download/kovanica-comms-0.2.0.apk
Web wallet: https://kovanica.online/wallet
Explorer: https://explorer.kovanica.online
Faucet: https://faucet.kovanica.online/faucet
Docs: https://docs.kovanica.online
```

Attach: `chain4-launch-1200x630.png` (or the square for a two-image post).

### Hrvatski

```
Kovanica Chain 4 testnet je živ — i od sada je zadana mreža u svakoj aplikaciji.

Ako ste instalirali stariju verziju, RPC endpoint ste morali upisivati ručno. To
više ne treba. Otvorite wallet, lightnode, comms ili desktop konzolu i već ste na
Testnetu (chain 4). Mainnet je vidljiv, ali zatvoren, a Local (1337) ostaje na
jedan klik ako vrtite vlastiti node.

Što možete odmah, besplatno:

• Napraviti wallet s PRO-21 adresom (kvnc…dag)
• Podići testne KUNE s fauceta — 1 KUNA po zahtjevu, 10 KUNA ukupno po adresi
• Slati i primati KUNE u nekoliko sekundi
• Pratiti DAG na mobitelu preko lightnodea
• Slati šifrirane peer-to-peer poruke preko Commsa
• Deployati HTLC, Vault, Multisig i Token ugovore

Ispod haube: Mysticeti-stil neovjereni DAG, runda 4 s (committed leader svakih
12 s), determinističko WASM izvršavanje, 15–21 stakani validator i KUNA s 9
decimala.

Ovo je još testnet. Novčići nemaju vrijednost, ugovori nisu auditirani i stvari
će pucati. Zato smo tu — pokušajte ga slomiti i javite nam što nađete.

Wallet: https://kovanica.online/download/kovanica-wallet-0.3.3.apk
Lightnode: https://kovanica.online/download/kovanica-lightnode-0.8.4.apk
Comms: https://kovanica.online/download/kovanica-comms-0.2.0.apk
Web wallet: https://kovanica.online/wallet
Explorer: https://explorer.kovanica.online
Faucet: https://faucet.kovanica.online/faucet
Dokumentacija: https://docs.kovanica.online
```

---

## Discord

### Announcement — #announcements

```
@here **Kovanica Chain 4 testnet is live — and it's now the default in every app**

No more pasting an RPC endpoint. A fresh install opens on **Testnet (chain 4)**.
Mainnet (chain 1) is listed and closed, Local (1337) is one tap away for devs, and
Custom RPC is still there.

**What shipped**
• Wallet 0.3.3 — testnet preselected, faucet button now reaches the live faucet
• Lightnode 0.8.4 — same, plus chain / epoch / validator view
• Comms 0.2.0 — network switch for the relay RPC
• Desktop console + Comms desktop — network switch above the endpoint field

**Try it in 3 steps**
1. Install the wallet: <https://kovanica.online/download/kovanica-wallet-0.3.3.apk>
2. Create a wallet, copy your `kvnc…dag` address
3. Claim 1 KUNA: <https://faucet.kovanica.online/faucet> (10 KUNA lifetime cap per address)

**Network**
• Chain id: **4**
• RPC: `https://rpc-testnet.kovanica.online/rpc`
• Explorer: <https://explorer.kovanica.online>
• Docs: <https://docs.kovanica.online>

Testnet coins are worthless and contracts are unaudited. Break things and report
them in #bugs — include app version, phone/OS, what you tapped and the exact
error text.
```

Attach: `chain4-discord-1200x400.png`.

### Short version — other servers / #general

```
Kovanica Chain 4 testnet is live and now the default network in every app —
wallet, lightnode, comms, desktop console. Install, open, you're on testnet.

Wallet <https://kovanica.online/download/kovanica-wallet-0.3.3.apk>
Lightnode <https://kovanica.online/download/kovanica-lightnode-0.8.4.apk>
Comms <https://kovanica.online/download/kovanica-comms-0.2.0.apk>
Faucet <https://faucet.kovanica.online/faucet>
Explorer <https://explorer.kovanica.online>
```

### Embed (webhook / bot)

```
title:       Chain 4 testnet is now the default
description: Every Kovanica app opens on Testnet (chain 4) — wallet, lightnode, comms, desktop console. Faucet fixed, mainnet still closed, local dev untouched.
color:       0xF2A900
fields:
  Wallet 0.3.3     · Testnet preselected, faucet button reaches the live faucet
  Lightnode 0.8.4  · Chain / epoch / validator view, 4s DAG sync
  Comms 0.2.0      · Relay RPC network switch
  Desktop          · Same switch in console and comms desktop
footer:      kovanica.online · chain id 4
```

### Pinned message — #testnet

```
**Testnet quickstart**
1. Wallet → create → copy your `kvnc…dag` address
2. Faucet → <https://faucet.kovanica.online/faucet> → 1 KUNA (10 KUNA lifetime cap)
3. Explorer → <https://explorer.kovanica.online>

Network: Testnet (chain 4) · `https://rpc-testnet.kovanica.online/rpc`
Local dev: Local (1337) · `http://127.0.0.1:8545/rpc` (`10.0.2.2` on the emulator)
Mainnet (chain 1) is closed.
```

---

## Images

Generated from the site palette (`#09090b` background, gold `#F2A900`, teal
`#2fbaa4`, blue `#8aa0b4`, text `#f2f1ee`). Re-render at any size with
`docs/marketing/make-cards.sh` (ImageMagick + DejaVu fonts).

| File | Size | Use |
|---|---|---|
| `chain4-launch-1200x630.png` | 1200×630 | Facebook post, LinkedIn, X link card, Discord embed image |
| `chain4-x-1600x900.png` | 1600×900 | X single post / thread header |
| `chain4-discord-1200x400.png` | 1200×400 | Discord announcement banner |
| `chain4-square-1080x1080.png` | 1080×1080 | Instagram, Facebook square, Discord embed |

Alt text (use it — it is also what screen readers announce):

> Dark card with a gold top rule. Headline: "Chain 4 testnet is now the default."
> Subline lists Wallet, Lightnode, Comms and Desktop console, with
> kovanica.online and "chain id 4 · 4s rounds · 12s committed leader" underneath.

## Posting plan

1. **Discord first** — #announcements with the banner, then pin the quickstart in
   #testnet. The community gets the links before the public.
2. **X** — single post + the 5-tweet thread an hour later, thread pinned for the
   week. Reply to your own single post with the thread link.
3. **Facebook** — 1–2 hours after X, English first, Croatian the same day (or the
   next morning for reach).
4. **Follow-up** — 48 h later, quote-post the best user screenshot with the first
   bug that got fixed. That is the post that convinces developers.

Hashtags, sparingly: `#Kovanica #KUNA #Layer1 #DAG #Testnet`.

## After posting, watch

- Faucet queue length and 10-KUNA cap complaints (`seed3` logs).
- Crash reports naming an app version — check `adb logcat` output pasted in #bugs.
- Explorer search queries for chain 1 addresses (people testing "is mainnet on?").

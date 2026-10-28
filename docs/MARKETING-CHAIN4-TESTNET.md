# Chain 4 testnet â€” launch copy for Facebook, X and Discord

Ready-to-paste copy for the release that makes **Testnet (chain 4)** the default
network in every Kovanica app. Replace the bracketed placeholders once the APKs
are uploaded.

## Fill these in first

| Placeholder | Value |
|---|---|
| `[LIGHTNODE_APK]` | https://github.com/KovanicaDAG/kovanica/releases/download/v0.8.4-android/Kovanica-Lightnode-0.8.4-arm64-v8a.apk |
| `[WALLET_APK]` | https://github.com/KovanicaDAG/kovanica/releases/download/v0.3.3-wallet/Kovanica-Wallet-0.3.3-arm64-v8a.apk |
| `[COMMS_APK]` | https://github.com/KovanicaDAG/kovanica/releases/download/v0.2.0-comms/kovanica-comms-0.2.0.apk |
| `[RELEASES]` | `https://github.com/KovanicaDAG/kovanica/releases` |
| Website | `https://kovanica.online` |
| Web wallet | `https://kovanica.online/wallet` |
| Explorer | `https://explorer.kovanica.online` |
| Faucet | `https://faucet.kovanica.online/faucet` |
| Docs | `https://docs.kovanica.online` |

Facts used below, so nothing gets oversold:

- Chain id **4** is the live public testnet. Mainnet is chain **1** and is **not open**.
- Round interval **4 s**, so a committed leader every **12 s** (Mysticeti-style
  uncertified DAG, wave = 3).
- Native coin **KUNA**, **9 decimals**, **90.2 M** total supply, **2.5 KUNA**
  initial block reward, era **8.1 M** committed leaders, decay **Ă—3/4**.
- **15â€“21** active validators, minimum stake **50 000 KUNA**.
- Faucet: **1 KUNA** per request, **10 KUNA lifetime cap per address**.
- No audit claims, no mainnet date, no price talk.

---

## X (Twitter)

### Single post

```
Kovanica Chain 4 testnet is live â€” and it's now the default in every app.

4s rounds, a committed leader every 12s, KUNA at 9 decimals, deterministic WASM execution.

Wallet, lightnode and comms: install, open, you're on testnet. No config.

https://kovanica.online
```

### Thread

**1/**

```
Kovanica Chain 4 testnet is live.

And as of today it's the default network in every app we ship: mobile wallet, lightnode, comms, desktop console.

No RPC hunting. Install, open, you're on chain 4.
```

**2/**

```
What changed in the apps:

â€˘ Testnet (chain 4) is preselected on a fresh install
â€˘ Mainnet (chain 1) is listed and clearly closed
â€˘ Local (1337) stays one tap away for devs
â€˘ Custom RPC still there for anyone who wants it
```

**3/**

```
Start with the wallet:

â†’ create a PRO-21 address (kvncâ€¦dag)
â†’ claim 1 KUNA from the faucet, 10 KUNA lifetime cap per address
â†’ send, receive, check history

Web wallet if you don't want to install anything: https://kovanica.online/wallet
```

**4/**

```
Want to verify instead of trust?

The lightnode follows the DAG on your phone: committed leader height, epoch, validators, balances. 4s rounds, 12s committed leaders.

Comms adds encrypted peer-to-peer messaging on top of the same chain.
```

**5/**

```
Everything is open:

Explorer  https://explorer.kovanica.online
Faucet    https://faucet.kovanica.online/faucet
Docs      https://docs.kovanica.online
Code      https://github.com/KovanicaDAG/kovanica

Found a bug? Tell us â€” testnet feedback is the whole point.
```

---

## Facebook

### English

```
Kovanica Chain 4 testnet is live â€” and it's now the default in every app.

If you installed an earlier build, you had to type an RPC endpoint by hand.
That's gone. Open the wallet, the lightnode, the comms app or the desktop
console and you are already on Testnet (chain 4). Mainnet is visible but
closed, and Local (1337) is still one tap away if you run your own node.

What you can do right now, for free:

â€˘ Create a wallet with a native PRO-21 address (kvncâ€¦dag)
â€˘ Claim test KUNA from the faucet â€” 1 KUNA per request, 10 KUNA lifetime cap
â€˘ Send and receive KUNA in seconds
â€˘ Run a lightnode and follow the DAG on your phone
â€˘ Send encrypted peer-to-peer messages with Comms
â€˘ Deploy HTLC, Vault, Multisig and Token contracts

Under the hood: Mysticeti-style uncertified DAG, a 4-second round with a
committed leader every 12 seconds, deterministic WASM execution, 15â€“21 staked
validators, and KUNA with 9 decimals.

This is still testnet. Coins are worthless, contracts are not audited, and
things will break. That's what we're here for â€” try to break it and tell us
what you find.

Downloads: [RELEASES]
Web wallet: https://kovanica.online/wallet
Explorer: https://explorer.kovanica.online
Faucet: https://faucet.kovanica.online/faucet
Docs: https://docs.kovanica.online
```

### Hrvatski

```
Kovanica Chain 4 testnet je Ĺľiv â€” i od sada je zadana mreĹľa u svakoj aplikaciji.

Ako ste instalirali stariju verziju, RPC endpoint ste morali upisivati ruÄŤno.
To viĹˇe ne treba. Otvorite wallet, lightnode, comms ili desktop konzolu i veÄ‡
ste na Testnetu (chain 4). Mainnet je vidljiv, ali zatvoren, a Local (1337)
ostaje na jedan klik ako vrtite vlastiti node.

Ĺ to moĹľete odmah, besplatno:

â€˘ Napraviti wallet s PRO-21 adresom (kvncâ€¦dag)
â€˘ PodiÄ‡i testne KUNE s fauceta â€” 1 KUNA po zahtjevu, 10 KUNA ukupno po adresi
â€˘ Slati i primati KUNE u nekoliko sekundi
â€˘ Vrtjeti lightnode i pratiti DAG na mobitelu
â€˘ Slati Ĺˇifrirane peer-to-peer poruke preko Commsa
â€˘ Deployati HTLC, Vault, Multisig i Token ugovore

Ispod haube: Mysticeti-stil neovjereni DAG, runda 4 s (committed leader svakih
12 s), deterministiÄŤko WASM izvrĹˇavanje, 15â€“21 stakani validator i KUNA s 9
decimala.

Ovo je joĹˇ testnet. NovÄŤiÄ‡i nemaju vrijednost, ugovori nisu auditirani i stvari
Ä‡e pucati. Zato smo tu â€” pokuĹˇajte ga slomiti i javite nam Ĺˇto naÄ‘ete.

Download: [RELEASES]
Web wallet: https://kovanica.online/wallet
Explorer: https://explorer.kovanica.online
Faucet: https://faucet.kovanica.online/faucet
Dokumentacija: https://docs.kovanica.online
```

---

## Discord

### Announcement (main channel)

```
@here Kovanica Chain 4 testnet is live â€” and it's now the default in every app

You no longer have to paste an RPC endpoint. Fresh installs open on
**Testnet (chain 4)**. Mainnet (chain 1) is listed and closed, Local (1337)
is one tap away for devs, and Custom RPC is still there.

**What's new**
â€˘ Wallet 0.3.3 â€” testnet preselected, faucet button now reaches the live faucet
â€˘ Lightnode 0.8.4 â€” same, plus chain/epoch/faucet prefill
â€˘ Comms 0.2.0 â€” new network switch for the relay RPC (Testnet / Local)
â€˘ Desktop console + Comms desktop â€” network switch above the endpoint field

**Try it in 3 steps**
1. Install: [WALLET_APK] (or the web wallet: https://kovanica.online/wallet)
2. Create a wallet â†’ copy your kvncâ€¦dag address
3. Claim 1 KUNA: https://faucet.kovanica.online/faucet (10 KUNA lifetime cap per address)

**Network**
â€˘ Chain id: 4
â€˘ RPC: https://rpc-testnet.kovanica.online/rpc
â€˘ Explorer: https://explorer.kovanica.online
â€˘ Docs: https://docs.kovanica.online

Testnet coins are worthless and contracts are unaudited. Break things and
report them in #bugs â€” include app version, phone/OS, what you tapped and the
exact error text.
```

### Short version (#general / other servers)

```
Kovanica Chain 4 testnet is live and now the default network in every app â€”
wallet, lightnode, comms, desktop console. Install, open, you're on testnet.

Wallet: [WALLET_APK]
Lightnode: [LIGHTNODE_APK]
Comms: [COMMS_APK]
Faucet: https://faucet.kovanica.online/faucet
Explorer: https://explorer.kovanica.online
```

### Embed-ready fields

```
Title:       Chain 4 testnet is now the default
Description: Every app ships with Testnet (chain 4) preselected â€” wallet, lightnode, comms, desktop console. Faucet fixed, mainnet still closed, local dev untouched.
Fields:
  Wallet 0.3.3      Â· Testnet preselected, reachable faucet
  Lightnode 0.8.4   Â· Chain/epoch/faucet prefill, 4s DAG sync
  Comms 0.2.0       Â· Relay RPC network switch
  Desktop 0.1.0     Â· Network switch in console + comms
Footer:        https://kovanica.online Â· chain id 4 Â· KUNA, 9 decimals
```

### Pinned message

```
Testnet quickstart
1. Wallet â†’ create â†’ copy kvncâ€¦dag address
2. Faucet â†’ https://faucet.kovanica.online/faucet â†’ 1 KUNA (10 KUNA lifetime cap)
3. Explorer â†’ https://explorer.kovanica.online

Network: Testnet (chain 4) Â· https://rpc-testnet.kovanica.online/rpc
Local dev: Local (1337) Â· http://127.0.0.1:8545/rpc (10.0.2.2 on the emulator)
Mainnet (chain 1) is closed.
```

---

## Visuals

| Asset | Size | Idea |
|---|---|---|
| Facebook / LinkedIn | 1200Ă—630 | Dark brand background, KUNA mark, headline "Chain 4 testnet is now the default", three app names |
| X post | 1600Ă—900 | Phone screenshot of the network switch with Testnet (chain 4) active, headline in the top third |
| X thread | 1600Ă—900 | #2: switch screenshot Â· #3: wallet balance after the faucet Â· #4: lightnode status card Â· #5: link card |
| Discord banner | 1200Ă—400 | "Testnet (chain 4)" plus the RPC host, thin mono line |
| Discord embed image | 1200Ă—630 | Same as the Facebook card |

Alt text for the switch screenshot: "Kovanica app network switch with Testnet
(chain 4), Mainnet (Chain 1), Local (1337) and Custom buttons, Testnet active."

## Posting order

1. Discord announcement first â€” the community gets the links before the public.
2. X thread, pinned to the profile for the week.
3. Facebook post 1â€“2 hours later, same day.
4. Reply to the X thread with the first user screenshot or a fix shipped that day.

Hashtags, sparingly: `#Kovanica #KUNA #Layer1 #DAG #Testnet`


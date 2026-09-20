# Public Announcement Texts

## Telegram / Discord (short)

```
Domain structure cleaned up.

• kovanica.online          → project homepage
• testnet.kovanica.online  → full testnet (explorer, wallet, faucet, tools)
• mainnet.kovanica.online  → ready for mainnet

Old links are automatically redirected.
Faucet is now at faucet.testnet.kovanica.online
Docs stay at docs.kovanica.online
```

## GitHub README / Release note

```
### Domain architecture (2026-09)

The public domains have been reorganised for the pre-mainnet phase:

- `kovanica.online` is now the pure project landing page
- Full network surfaces live on `testnet.kovanica.online` and `mainnet.kovanica.online`
- Faucet moved to `faucet.testnet.kovanica.online`
- Public API entry: `api.kovanica.online`

See `NETWORK.md` for the complete map and client configuration.
```

## docs.kovanica.online (slightly longer)

```
## Network & Domain Layout

Kovanica uses a clean subdomain split:

| Site                         | Purpose                          |
|-----------------------------|----------------------------------|
| kovanica.online             | Project homepage                 |
| testnet.kovanica.online     | Live testnet                     |
| mainnet.kovanica.online     | Mainnet (ready)                  |
| faucet.testnet.kovanica.online | Testnet faucet                |
| docs.kovanica.online        | Specs & RFCs                     |
| explorer / wallet           | Shared for now                   |

Client applications should detect the network from the hostname and use the configuration in `NETWORK.md` / `src/lib/network.ts`.
```
```


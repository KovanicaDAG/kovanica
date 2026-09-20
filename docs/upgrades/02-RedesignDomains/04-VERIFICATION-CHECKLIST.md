# Verification Checklist

Run these after DNS + Redirect Rules + frontend deploy.

## DNS

```bash
dig +short testnet.kovanica.online A
dig +short mainnet.kovanica.online A
dig +short faucet.testnet.kovanica.online A
dig +short api.kovanica.online A
dig +short seed.kovanica.online A          # should be grey-cloud IP
```

## Redirects (curl)

```bash
# map → api
curl -I https://map.kovanica.online

# old kovanica.kovanica → faucet
curl -I https://kovanica.kovanica.online

# www → apex
curl -I https://www.kovanica.online

# root path → testnet
curl -I https://kovanica.online/explorer
curl -I https://kovanica.online/wallet
curl -I https://kovanica.online/faucet
```

Expect `301` or `302` with correct `Location` header.

## Browser checks

- [ ] https://kovanica.online loads clean landing (no live explorer / faucet)
- [ ] Network switcher on root redirects to testnet. or mainnet. hostname
- [ ] https://testnet.kovanica.online shows Testnet badge and full UI
- [ ] https://mainnet.kovanica.online shows Mainnet badge
- [ ] Faucet link points to faucet.testnet.kovanica.online
- [ ] Switching networks changes the hostname in the address bar
- [ ] localStorage / cookies do not leak between testnet and mainnet

## API smoke

```bash
curl -s https://api.kovanica.online/api/head | jq .
# or whatever the current public head endpoint is
```

## Seeds

```bash
# Should resolve and be reachable on TCP 9000 (from a machine that can reach it)
nc -zv seed.kovanica.online 9000
```
```


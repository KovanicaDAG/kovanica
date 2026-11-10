# Chain 2 public testnet seeds (seed1..3)
1. `./gen-p2p-key.sh /etc/kovanica/seedN.p2p.key` on each host (keys are never stored in git).
2. Copy `seedN.env.example` -> `/etc/kovanica/seedN.env`, install `ops/systemd/kovanica-seed@.service`.
3. `sudo ./firewall.sh`; include `nginx-ratelimit.conf` and `ops/ingress/setup-nginx.sh seedN`.
4. Enable the seeder on one seed; publish the generated zone file for `seed.kovanica.online`.
5. Faucet: `kovanica-faucet --chain-id 2 --max-requests 3 --rate-limit-window 3600` (limits per IP and per address).
Credentials are not kept in this repo.

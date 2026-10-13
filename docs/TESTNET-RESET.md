# Testnet reset after the light-client merge

Recorded 2026-10-14. `main` is `f012c58` (merge of pull request #56).

A reset changes the live mesh only when all three seeds are rebuilt from that commit and started again together. Restarting the binary that is running now starts a fresh chain under the old rules.

## What is running

The live process is still `/usr/local/bin/kvnc-node`, built 2026-10-13 20:02, before the merge. On this machine its `consensus.redb` and `state.redb` are already deleted on disk. The process keeps those files open. `seed2` and `seed3` are still on that older build.

`KVNC_ALLOW_RESET=0` does not apply this code. This node has no reset command.

## Fresh start of the merged binary

Stop all three seeds, install a binary built from `f012c58` on each of them, wipe the chain databases, and start them together.

- MysticGhost is on, so commit colouring follows scoped GHOSTDAG.
- Each committed leader still mints `block_reward`. Twenty percent sits in the light escrow until the epoch closes (7 200 leader blocks, about a day). With no light nodes bonded, that share returns to the leaders who contributed it.
- VRF leader election is on, and the schedule stays round-robin. The genesis file under `/var/lib/kvnc` has no `vrf_public_key`, so nobody publishes a carrier and the election does not leave round-robin.
- Chain id stays 1337. The light RPC methods are available. Old height, balances, and the mempool are gone.

One seed restarted alone splits from the other two. Nothing has been rebuilt or restarted for this note.

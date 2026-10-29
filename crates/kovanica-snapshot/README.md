# kovanica-snapshot

Operator tooling for exporting/importing Kovanica state snapshots (magic `KVSNSP01`).

## Usage

```sh
# Export from a node data dir
kovanica-snapshot export --data-dir ./data --out snapshot.bin

# Verify magic + header
kovanica-snapshot verify --file snapshot.bin

# Import into empty data dir (requires --force)
kovanica-snapshot import --data-dir ./data2 --file snapshot.bin --force
```

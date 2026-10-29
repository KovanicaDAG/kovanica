# Fuzzing

Run with cargo-fuzz (nightly):

```
cargo fuzz run statement_block
cargo fuzz run mempool_admission
```

Targets are optional and not part of default build.
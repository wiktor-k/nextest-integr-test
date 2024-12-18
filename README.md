# nextest integration tests

This repository showcases issues with:

```rust
.with_wait_strategy(WaitStrategy::stderr_contains(
        "listening on 8443/TCP for HTTPS",
))
```

which happen only when high concurrency is requested, e.g. via:

```sh
cargo nextest run
```

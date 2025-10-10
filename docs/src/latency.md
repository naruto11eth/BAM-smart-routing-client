# Latency & Routing Strategy

Latency probes run concurrently using the configured [`ProbeMethod`]. Each
probe uses the async `solana_rpc_client` under the hood, so the calls honour the
same request semantics as the official client.

## Probe methods

| Method               | Description                               | RPC call           |
| -------------------- | ----------------------------------------- | ------------------ |
| `GetHealth`          | Default, ensures the node reports `ok`.   | `getHealth`        |
| `GetSlot`            | Faster, checks slot progression.          | `getSlot`          |
| `GetLatestBlockhash` | Verifies recent blockhash availability.   | `getLatestBlockhash` |

Use `RouterConfig::probe_method` to change the behaviour:

```rust
use bam_smart_routing_client::{BamSmartRouter, RouterConfig, ProbeMethod};

let mut config = RouterConfig::default();
config.probe_method = ProbeMethod::GetSlot;
let router = BamSmartRouter::new(config)?;
```

## Timeouts

Two independent timeouts keep the router responsive:

- `probe_timeout`: wraps each latency probe via `tokio::time::timeout`.
- `rpc_timeout`: applied when constructing the underlying `RpcClient`.

Both default to conservative values (1.5s and 5s respectively) and can be tuned
in the `RouterConfig`.

## Retry policy

The router retries transient network failures signalled by `reqwest` or I/O
errors up to `retries` times (default: 1 extra attempt). RPC logic errors are
not retried. The count of attempts is included in the
[`TransactionSendOutcome`].

[`ProbeMethod`]: crate::ProbeMethod
[`TransactionSendOutcome`]: crate::TransactionSendOutcome

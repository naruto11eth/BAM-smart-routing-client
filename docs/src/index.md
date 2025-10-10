# BAM Smart Routing Client

Welcome to the documentation for the **BAM Smart Routing Client** &ndash; a Rust
library that discovers Blockspace Assembly Marketplace (BAM) regions, measures
RPC latency in parallel, and submits Solana transactions to the fastest healthy
endpoint.

This is intended for automated routing on BAM's
public testnet. It combines:

- Region metadata baked directly into the crate.
- Pluggable latency probes (`getHealth`, `getSlot`, or `getLatestBlockhash`).
- Intelligent retries when submitting transactions.
- Helpers for base64/base58 encoded payloads.

The crate focuses on correctness, sensible defaults, and an ergonomic async API
that fits naturally into existing `tokio` applications.

## Key Features

- **Parallel latency probing** with tunable timeouts and RPC methods.
- **Fastest-route selection** before dispatching transactions.
- **Transaction helpers** for raw `VersionedTransaction`, legacy `Transaction`,
  base64, base58, or raw bytes.
- **Retry logic** for transient transport failures.
- **Configurable submission target** (defaults to Solana public RPC with a TPU hook ready to wire in).
- **Test-friendly configuration** that allows overriding regions.

Continue with the [Quick Start](./quickstart.md) guide to integrate the router
into your project.

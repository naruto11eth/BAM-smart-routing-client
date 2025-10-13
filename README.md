# BAM Smart Routing Client

A Rust library that discovers BAM (Blockspace Assembly Marketplace) RPC regions,
measures their latency in parallel, and sends Solana transactions to the
fastest healthy endpoint built by naruto11.

## Highlights

- Built-in metadata for BAM mainnet and testnet regions.
- Concurrent latency probes via async Solana RPC calls.
- Intelligent retries for transient network failures.
- Helpers for `VersionedTransaction`, legacy `Transaction`, base64, base58, or raw bytes.
- Default submission via public Solana RPC (testnet/mainnet) with hooks for future TPU submission.
- Docs through mdbook.

## Getting started

The crate targets Rust 1.75+ and depends on `tokio` for async runtime support.

```toml
[dependencies]
bam-smart-routing-client = { path = "." }
tokio = { version = "1.40", features = ["macros", "rt-multi-thread"] }
solana-sdk = "1.18.26"
```

Quick probe example:

```rust,no_run
use bam_smart_routing_client::{BamSmartRouter, RouterConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router = BamSmartRouter::new(RouterConfig::default())?;
    let selection = router.select_fastest().await?;

    println!(
        "{} ({:?}) in {} ms",
        selection.region.endpoint,
        selection.region.network,
        selection.latency.as_millis()
    );
    Ok(())
}
```

By default txns go to the public Solana RPC that matches the network
filter (testnet -> `https://api.testnet.solana.com`, mainnet ->
`https://api.mainnet-beta.solana.com`). Override this behaviour as needed:

```rust,no_run
use bam_smart_routing_client::{
    BamSmartRouter, RouterConfig, SubmissionEndpoint,
};

let mut config = RouterConfig::default();
config.submission_endpoint = SubmissionEndpoint::FastestRegion;
let router = BamSmartRouter::new(config)?;
```

To target the validator TPU directly once QUIC credentials are available:

```rust
use bam_smart_routing_client::{BamSmartRouter, RouterConfig, SubmissionEndpoint};

let mut config = RouterConfig::default();
config.submission_endpoint = SubmissionEndpoint::Tpu {
    host: "ny.testnet.bam.jito.wtf".into(),
    port: 10000, // replace with actual TPU port
};
// NOTE: the current implementation returns an error for `Tpu`. good TODO.
let router = BamSmartRouter::new(config)?;
```

### CLI probe example

```
cargo run --example probe

# Submit a signed transaction (base64) through the configured RPC
cargo run --example probe -- --tx-base64 <BASE64_TX>
```

### Generate a signed transaction for testing

Use the helper example to mint a fresh base64 payload without juggling CLI
flags manually:

```
cargo run --example make_tx -- <RECIPIENT_PUBKEY> <AMOUNT_SOL|LAMPORTS> [KEYPAIR_PATH] [RPC_URL]

# Example: send 0.01 SOL from a local keypair on testnet
cargo run --example make_tx -- <RECIPIENT_PUBKEY> 0.01 ~/solana-test.json

# Pipe the output into the router example
BASE64_TX=$(cargo run --example make_tx -- <RECIPIENT_PUBKEY> 0.01 ~/solana-test.json | tail -n1)
cargo run --example probe -- --tx-base64 "$BASE64_TX"
```

The helper fetches a fresh blockhash, signs the transfer, and prints the
serialized transaction as base64 without broadcasting it.

See the [docs](docs/src/index.md) for more examples and API details.

## Development workflow

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

The latency and routing logic is covered by async unit tests that leverage
`httpmock` to control simulated RPC delays.

### Documentation site

This repo ships an [mdBook](https://rust-lang.github.io/mdBook/) site under
`docs/`.

```bash
mdbook serve docs
-> http://localhost:3000 for live preview
```

### Structure

```
├── Cargo.toml
├── src/
│   ├── config.rs
│   ├── error.rs
│   ├── lib.rs
│   ├── probe.rs
│   ├── regions.rs
│   └── router.rs
├── docs/
│   ├── book.toml
│   └── src/
│       ├── SUMMARY.md
│       ├── index.md
│       ├── quickstart.md
│       ├── latency.md
│       └── api.md
└── README.md
```

## Future work

- Optional dynamic region discovery over HTTPS.
- Metrics integration for reporting latency trends.
- CLI wrapper for manual transaction dispatch.

## License

MIT

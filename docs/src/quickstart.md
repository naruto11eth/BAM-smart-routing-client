# Quick Start

This section shows the fastest way to probe BAM regions and submit a
transaction using the Rust crate.

## Installation

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
bam-smart-routing-client = { path = "../" }
solana-sdk = "=3.0.0"
tokio = { version = "1.40", features = ["macros", "rt-multi-thread"] }
```

Replace the local path with the published crate name once available.

## Probing regions

```rust,no_run
use bam_smart_routing_client::{BamSmartRouter, RouterConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router = BamSmartRouter::new(RouterConfig::default())?;
    let probes = router.probe_regions().await;

    for record in probes {
        println!("{} => {}", record.region.code, record.status);
    }

    Ok(())
}
```

## Submitting a transaction

Before submitting you need a signed `VersionedTransaction`.

```rust,no_run
use bam_smart_routing_client::{BamSmartRouter, RouterConfig};
use solana_sdk::transaction::VersionedTransaction;

async fn submit(tx: VersionedTransaction) -> anyhow::Result<()> {
    let router = BamSmartRouter::new(RouterConfig::default())?;
    let outcome = router.submit_versioned_transaction(&tx).await?;

    println!(
        "Sent {:?} via {} in {} ms (attempts: {})",
        outcome.signature,
        outcome.region.code,
        outcome.latency.as_millis(),
        outcome.attempts,
    );
    Ok(())
}
```

Transactions are forwarded to the public Solana RPC that matches the router’s
network filter. Override this by setting `submission_endpoint` to either a
custom URL or `SubmissionEndpoint::FastestRegion` if you want to submit back to
the probed node.

The helper methods `submit_transaction_base64`,
`submit_transaction_base58`, and `submit_transaction_bytes` cover the common
encoding formats used by wallets and SDKs.

### CLI example

```
cargo run --example probe
cargo run --example probe -- --tx-base64 <BASE64_TX>
```

### Create a base64 transaction payload

```
cargo run --example make_tx -- <RECIPIENT_PUBKEY> <AMOUNT_SOL|LAMPORTS> [KEYPAIR_PATH] [RPC_URL]
```

The helper fetches a fresh blockhash, signs the transfer with the supplied
keypair, and prints the serialized transaction as base64 without broadcasting
it. Pipe the final line straight into the probe example when you want to submit
via the router.

# API Reference

This page highlights the most important structs and methods. See the generated
Rust docs for exhaustive details.

## Regions

```rust
use bam_smart_routing_client::{BAM_TESTNET_REGIONS, BamRegion, BamNetwork};

for region in BAM_TESTNET_REGIONS {
    println!("{} -> {}", region.code, region.endpoint);
}
```

- [`BamRegion`] exposes the region metadata.
- `BAM_TESTNET_REGIONS` and `BAM_MAINNET_REGIONS` list known endpoints.
- [`NetworkFilter`] controls which set is used by the router.

## Router configuration

```rust
use bam_smart_routing_client::{NetworkFilter, ProbeMethod, RouterConfig};

let config = RouterConfig {
    network_filter: NetworkFilter::Testnet,
    probe_method: ProbeMethod::GetHealth,
    probe_timeout: std::time::Duration::from_millis(1500),
    rpc_timeout: std::time::Duration::from_secs(5),
    retries: 1,
    commitment: solana_sdk::commitment_config::CommitmentConfig::confirmed(),
};
```

## Selecting the fastest region

```rust,no_run
use bam_smart_routing_client::{BamSmartRouter, RouterConfig};

# async fn demo() -> anyhow::Result<()> {
let router = BamSmartRouter::new(RouterConfig::default())?;
let selection = router.select_fastest().await?;
println!(
    "Fastest region: {} ({} ms)",
    selection.region.endpoint,
    selection.latency.as_millis()
);
# Ok(())
# }
```

The returned [`RouteSelection`] retains the full probe table for debugging.

## Submitting transactions

```rust,no_run
use bam_smart_routing_client::{BamSmartRouter, RouterConfig};
use solana_sdk::transaction::VersionedTransaction;

# async fn submit(tx: VersionedTransaction) -> anyhow::Result<()> {
let router = BamSmartRouter::new(RouterConfig::default())?;
let outcome = router.submit_versioned_transaction(&tx).await?;
println!("Signature {}", outcome.signature);
# Ok(())
# }
```

Alternative helpers:

- `submit_transaction_base64`
- `submit_transaction_base58`
- `submit_transaction_bytes`
- `submit_legacy_transaction`

Each returns a [`TransactionSendOutcome`] with the signature, chosen region,
latency, and retry count.

[`BamRegion`]: crate::BamRegion
[`NetworkFilter`]: crate::NetworkFilter
[`RouteSelection`]: crate::RouteSelection
[`TransactionSendOutcome`]: crate::TransactionSendOutcome

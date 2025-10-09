//! BAM Smart Routing Client
//!
//! This crate provides a latency-aware Solana RPC routing client that targets
//! the BAM (Blockspace Assembly Marketplace) testnet and mainnet regions. It
//! offers helpers to list known regions, probe their health, pick the fastest
//! endpoint, and submit raw Solana transactions with retries.

mod config;
mod error;
mod probe;
mod regions;
mod router;

pub use crate::config::{NetworkFilter, ProbeMethod, RouterConfig};
pub use crate::error::RouterError;
pub use crate::probe::{ProbeRecord, ProbeStatus};
pub use crate::regions::{
    BAM_MAINNET_REGIONS, BAM_TESTNET_REGIONS, BamNetwork, BamRegion, RegionMetadata, all_regions,
    regions_for_network,
};
pub use crate::router::{BamSmartRouter, RouteSelection, TransactionSendOutcome};

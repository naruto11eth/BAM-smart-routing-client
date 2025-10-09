use std::time::Duration;

use solana_sdk::commitment_config::CommitmentConfig;

use crate::regions::{BamNetwork, BamRegion, all_regions, regions_for_network};

/// Controls which regions the router considers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkFilter {
    /// Restrict routing to BAM testnet regions (default for this crate).
    Testnet,
    /// Restrict routing to BAM mainnet regions.
    Mainnet,
    /// Consider both testnet and mainnet regions.
    All,
}

impl NetworkFilter {
    pub(crate) fn select_regions(&self) -> Vec<&'static BamRegion> {
        match self {
            NetworkFilter::Testnet => regions_for_network(BamNetwork::Testnet).iter().collect(),
            NetworkFilter::Mainnet => regions_for_network(BamNetwork::Mainnet).iter().collect(),
            NetworkFilter::All => all_regions().iter().collect(),
        }
    }
}

/// RPC method used for latency probing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeMethod {
    /// Call `getHealth` to ensure the node reports `ok`.
    GetHealth,
    /// Call `getSlot` as a lightweight RPC probe.
    GetSlot,
    /// Call `getLatestBlockhash` to ensure recent blockhash availability.
    GetLatestBlockhash,
}

impl Default for ProbeMethod {
    fn default() -> Self {
        ProbeMethod::GetHealth
    }
}

/// Configuration for the smart router.
#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub network_filter: NetworkFilter,
    pub probe_method: ProbeMethod,
    pub probe_timeout: Duration,
    pub rpc_timeout: Duration,
    pub retries: usize,
    pub commitment: CommitmentConfig,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            network_filter: NetworkFilter::Testnet,
            probe_method: ProbeMethod::GetHealth,
            probe_timeout: Duration::from_millis(1500),
            rpc_timeout: Duration::from_secs(5),
            retries: 1,
            commitment: CommitmentConfig::confirmed(),
        }
    }
}

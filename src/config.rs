use std::time::Duration;

use solana_commitment_config::CommitmentConfig;

use crate::regions::{BamNetwork, BamRegion, all_regions, regions_for_network};

/// Controls where transactions are ultimately submitted
#[derive(Debug, Clone)]
pub enum SubmissionEndpoint {
    /// Submit directly to the region selected via latency probes
    FastestRegion,
    /// Submit to a fixed Solana RPC endpoint irrespective of the probe winner
    Fixed(String),
    /// Submit directly to the validator TPU
    Tpu { host: String, port: u16 },
}

/// Controls which regions the router considers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkFilter {
    /// Restrict routing to BAM testnet regions (default for this crate)
    Testnet,
    /// Restrict routing to BAM mainnet regions
    Mainnet,
    /// Consider both testnet and mainnet regions
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

/// Probing strategy used for latency measurement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeMethod {
    /// Measure TCP connect latency without issuing a JSON-RPC request
    TcpConnect,
    /// Call `getHealth` to ensure the node reports `ok`
    GetHealth,
    /// Call `getSlot` as a lightweight RPC probe
    GetSlot,
    /// Call `getLatestBlockhash` to ensure recent blockhash availability
    GetLatestBlockhash,
}

impl Default for ProbeMethod {
    fn default() -> Self {
        ProbeMethod::TcpConnect
    }
}

/// Configuration for the smart router
#[derive(Debug, Clone)]
pub struct RouterConfig {
    pub network_filter: NetworkFilter,
    pub probe_method: ProbeMethod,
    pub probe_timeout: Duration,
    pub rpc_timeout: Duration,
    pub retries: usize,
    pub commitment: CommitmentConfig,
    pub submission_endpoint: SubmissionEndpoint,
}

impl Default for RouterConfig {
    fn default() -> Self {
        let network_filter = NetworkFilter::Testnet;
        Self {
            network_filter,
            probe_method: ProbeMethod::default(),
            probe_timeout: Duration::from_millis(1500),
            rpc_timeout: Duration::from_secs(5),
            retries: 1,
            commitment: CommitmentConfig::confirmed(),
            submission_endpoint: default_submission_endpoint(network_filter),
        }
    }
}

fn default_submission_endpoint(filter: NetworkFilter) -> SubmissionEndpoint {
    match filter {
        NetworkFilter::Testnet => {
            SubmissionEndpoint::Fixed("https://api.testnet.solana.com".to_string())
        }
        NetworkFilter::Mainnet => {
            SubmissionEndpoint::Fixed("https://api.mainnet-beta.solana.com".to_string())
        }
        NetworkFilter::All => SubmissionEndpoint::FastestRegion,
    }
}

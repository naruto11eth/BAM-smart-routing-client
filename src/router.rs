use std::time::{Duration, Instant};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use futures::future::join_all;
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    nonblocking::rpc_client::RpcClient,
    rpc_client::SerializableTransaction,
};
use solana_sdk::{
    signature::Signature,
    transaction::{Transaction, VersionedTransaction},
};
use tokio::{net::TcpStream, time::timeout};
use url::Url;

use crate::{
    config::{ProbeMethod, RouterConfig, SubmissionEndpoint},
    error::RouterError,
    probe::{ProbeRecord, ProbeStatus},
    regions::BamRegion,
};

/// High level latency finder router for BAM RPC regions
#[derive(Debug, Clone)]
pub struct BamSmartRouter {
    config: RouterConfig,
    regions: Vec<&'static BamRegion>,
}

impl BamSmartRouter {
    /// Create a router with the provided configuration filtering the built-in BAM regions
    pub fn new(config: RouterConfig) -> Result<Self, RouterError> {
        let regions = config.network_filter.select_regions();
        if regions.is_empty() {
            return Err(RouterError::NoRegions(config.network_filter));
        }
        Ok(Self { config, regions })
    }

    /// Create a router with custom regions if we ever get something, this is not required tbh
    pub fn with_regions(
        config: RouterConfig,
        regions: Vec<&'static BamRegion>,
    ) -> Result<Self, RouterError> {
        if regions.is_empty() {
            return Err(RouterError::NoRegions(config.network_filter));
        }
        Ok(Self { config, regions })
    }

    /// Returns the configuration in use
    pub fn config(&self) -> &RouterConfig {
        &self.config
    }

    /// Returns the regions the router will consider
    pub fn regions(&self) -> &[&'static BamRegion] {
        &self.regions
    }

    // Executes the configured latency probe against every region in parallel -- the join_all is used to run them in parallel
    // this iwll return a vector of ProbeRecord
    pub async fn probe_regions(&self) -> Vec<ProbeRecord> {
        let futures = self
            .regions
            .iter()
            .map(|region| self.probe_single_region(*region));
        join_all(futures).await
    }

    /// Selects the fastest healthy region based on the configured probe
    pub async fn select_fastest(&self) -> Result<RouteSelection, RouterError> {
        let probes = self.probe_regions().await;
        let mut best: Option<(usize, Duration)> = None;

        for (idx, record) in probes.iter().enumerate() {
            if let ProbeStatus::Healthy { latency } = record.status {
                if best
                    .map(|(_, best_latency)| latency < best_latency)
                    .unwrap_or(true)
                {
                    best = Some((idx, latency));
                }
            }
        }

        match best {
            Some((idx, latency)) => Ok(RouteSelection {
                region: probes[idx].region,
                latency,
                probes,
            }),
            None => Err(RouterError::NoHealthyRegion { probes }),
        }
    }

    /// Submit any serializable Solana transaction, routing it to the fastest region
    /// need to have a txnSend result as well just how we had for route selection
    /// added send_txn_to_region as well
    pub async fn submit_serializable_transaction<T>(
        &self,
        transaction: &T,
    ) -> Result<TransactionSendOutcome, RouterError>
    where
        T: SerializableTransaction + Sync,
    {
        let selection = self.select_fastest().await?;
        let (signature, attempts) = self
            .send_transaction_to_region(transaction, selection.region)
            .await?;

        Ok(TransactionSendOutcome {
            signature,
            region: selection.region,
            latency: selection.latency,
            probes: selection.probes,
            attempts,
        })
    }

    async fn probe_single_region(&self, region: &'static BamRegion) -> ProbeRecord {}
}

// need a struct to store result of selecting a region for rounting, that's why added RouteSelection above
#[derive(Debug)]
pub struct RouteSelection {
    pub region: &'static BamRegion,
    pub latency: Duration,
    pub probes: Vec<ProbeRecord>,
}

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

    /// Convenience wrapper for `VersionedTransaction` objects
    pub async fn submit_versioned_transaction(
        &self,
        transaction: &VersionedTransaction,
    ) -> Result<TransactionSendOutcome, RouterError> {
        self.submit_serializable_transaction(transaction).await
    }

    /// Convenience wrapper for legacy `Transaction` objects
    pub async fn submit_legacy_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<TransactionSendOutcome, RouterError> {
        self.submit_serializable_transaction(transaction).await
    }

    /// Decode a base64-encoded transaction and submit it
    pub async fn submit_transaction_base64(
        &self,
        base64_tx: &str,
    ) -> Result<TransactionSendOutcome, RouterError> {
        let tx = decode_base64_transaction(base64_tx)?; // need to write the decode_base fn
        self.submit_versioned_transaction(&tx).await
    }

    /// Decode a base58-encoded transaction and submit it
    pub async fn submit_transaction_base58(
        &self,
        base58_tx: &str,
    ) -> Result<TransactionSendOutcome, RouterError> {
        let tx = decode_base58_transaction(base58_tx)?;
        self.submit_versioned_transaction(&tx).await
    }

    /// Submit raw bincode-serialized transaction bytes
    pub async fn submit_transaction_bytes(
        &self,
        bytes: &[u8],
    ) -> Result<TransactionSendOutcome, RouterError> {
        let tx = decode_transaction_bytes(bytes)?;
        self.submit_versioned_transaction(&tx).await
    }

    async fn probe_single_region(&self, region: &'static BamRegion) -> ProbeRecord {
        match self.config.probe_method {
            ProbeMethod::TcpConnect => self.probe_tcp_connect(region).await,
            method => {
                let client = self.build_client(region);
                let start = Instant::now();
                let outcome = timeout(
                    self.config.probe_timeout,
                    self.execute_probe(&client, method),
                )
                .await;

                let status = match outcome {
                    Ok(Ok(())) => ProbeStatus::Healthy {
                        latency: start.elapsed(),
                    },
                    Ok(Err(err)) => ProbeStatus::RpcError { error: err },
                    Err(_) => ProbeStatus::Timeout {
                        timeout: self.config.probe_timeout,
                    },
                };

                ProbeRecord { region, status }
            }
        }
    }

    /// this is the main/default probe fn that should be invoked. we are doing a simple tcp connect, something as simple as curl command
    async fn probe_tcp_connect(&self, region: &'static BamRegion) -> ProbeRecord {
        let status = match Url::parse(region.endpoint) {
            Ok(url) => {
                if let (Some(host), Some(port)) = (url.host_str(), url.port_or_known_default()) {
                    let address = format!("{}:{}", host, port);
                    let start = Instant::now();
                    match timeout(self.config.probe_timeout, TcpStream::connect(address)).await {
                        Ok(Ok(stream)) => {
                            drop(stream);
                            ProbeStatus::Healthy {
                                latency: start.elapsed(),
                            }
                        }
                        Ok(Err(err)) => ProbeStatus::OtherError {
                            reason: err.to_string(),
                        },
                        Err(_) => ProbeStatus::Timeout {
                            timeout: self.config.probe_timeout,
                        },
                    }
                } else {
                    ProbeStatus::OtherError {
                        reason: "missing host or port".to_string(),
                    }
                }
            }
            Err(err) => ProbeStatus::OtherError {
                reason: err.to_string(),
            },
        };

        ProbeRecord { region, status }
    }

    async fn execute_probe(
        &self,
        client: &RpcClient,
        method: ProbeMethod,
    ) -> Result<(), ClientError> {
        match method {
            ProbeMethod::TcpConnect => unreachable!("TCP connect probing handled separately"),
            ProbeMethod::GetHealth => client.get_health().await,
            ProbeMethod::GetSlot => client.get_slot().await.map(|_| ()),
            ProbeMethod::GetLatestBlockhash => client.get_latest_blockhash().await.map(|_| ()),
        }
    }

    /// this is helper when we pick json-rpc probe, and constructs a solana async rpc client pointing at that region's endpoint
    fn build_client(&self, region: &BamRegion) -> RpcClient {
        RpcClient::new_with_timeout_and_commitment(
            region.endpoint.to_string(),
            self.config.rpc_timeout,
            self.config.commitment.clone(),
        )
    }
    /// used whenever we send the actual txns but it looks at config.submission_endpoints. ot chooses where to submit.
    fn build_submission_client(&self, region: &BamRegion) -> Result<RpcClient, RouterError> {
        match &self.config.submission_endpoint {
            SubmissionEndpoint::FastestRegion => Ok(RpcClient::new_with_timeout_and_commitment(
                region.endpoint.to_string(),
                self.config.rpc_timeout,
                self.config.commitment.clone(),
            )),
            SubmissionEndpoint::Fixed(endpoint) => Ok(RpcClient::new_with_timeout_and_commitment(
                endpoint.clone(),
                self.config.rpc_timeout,
                self.config.commitment.clone(),
            )),
            SubmissionEndpoint::Tpu { host, port } => {
                Err(RouterError::UnsupportedSubmissionEndpoint {
                    description: format!("TPU submission to {host}:{port}"),
                })
            }
        }
    }

    /// actually what sends the signed txns. it will call build_submission_client to get teh right client first and then send_transaction(). it sends during and chooses logic.
    async fn send_transaction_to_region<T>(
        &self,
        transaction: &T,
        region: &'static BamRegion,
    ) -> Result<(Signature, usize), RouterError>
    where
        T: SerializableTransaction + Sync,
    {
        let client = self.build_submission_client(region)?;
        let attempts = self.config.retries.saturating_add(1);
        let mut last_error: Option<ClientError> = None;

        for attempt in 1..=attempts {
            match client.send_transaction(transaction).await {
                Ok(signature) => return Ok((signature, attempt)),
                Err(err) => {
                    if !Self::should_retry(&err) || attempt == attempts {
                        return Err(RouterError::TransactionFailed {
                            region_code: region.code,
                            source: err,
                        });
                    }
                    last_error = Some(err);
                }
            }
        }

        let error = last_error.unwrap_or_else(|| {
            ClientError::from(ClientErrorKind::Custom(
                "transaction failed without specific error".to_string(),
            ))
        });
        Err(RouterError::TransactionFailed {
            region_code: region.code,
            source: error,
        })
    }

    // if user gets any error
    fn should_retry(error: &ClientError) -> bool {
        match error.kind() {
            ClientErrorKind::Reqwest(reqwest_error) => {
                reqwest_error.is_timeout() || reqwest_error.is_connect()
            }
            ClientErrorKind::Io(_) => true,
            ClientErrorKind::RpcError(_) => false,
            ClientErrorKind::SerdeJson(_) => false,
            ClientErrorKind::SigningError(_) => false,
            ClientErrorKind::TransactionError(_) => false,
            ClientErrorKind::Custom(_) => false,
        }
    }
}

// need a struct to store result of selecting a region for rounting, that's why added RouteSelection above
#[derive(Debug)]
pub struct RouteSelection {
    pub region: &'static BamRegion,
    pub latency: Duration,
    pub probes: Vec<ProbeRecord>,
}

/// Response returned after submitting a transaction
#[derive(Debug)]
pub struct TransactionSendOutcome {
    pub signature: Signature,
    pub region: &'static BamRegion,
    pub latency: Duration,
    pub probes: Vec<ProbeRecord>,
    pub attempts: usize,
}

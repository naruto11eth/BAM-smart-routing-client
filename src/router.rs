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
}

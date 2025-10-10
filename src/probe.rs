use std::{fmt, time::Duration};

use solana_client::client_error::ClientError;

use crate::regions::BamRegion;

/// Latency probe outcome for a single region
#[derive(Debug)]
pub struct ProbeRecord {
    pub region: &'static BamRegion,
    pub status: ProbeStatus,
}

impl ProbeRecord {
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, ProbeStatus::Healthy { .. })
    }

    pub fn latency(&self) -> Option<Duration> {
        match self.status {
            ProbeStatus::Healthy { latency } => Some(latency),
            _ => None,
        }
    }
}

/// Compact status for a probe
#[derive(Debug)]
pub enum ProbeStatus {
    Healthy { latency: Duration },
    Timeout { timeout: Duration },
    RpcError { error: ClientError },
}

impl fmt::Display for ProbeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProbeStatus::Healthy { latency } => {
                write!(f, "healthy ({} ms)", latency.as_millis())
            }
            ProbeStatus::Timeout { timeout } => {
                write!(f, "timeout after {} ms", timeout.as_millis())
            }
            ProbeStatus::RpcError { error } => {
                write!(f, "rpc error: {}", error)
            }
        }
    }
}

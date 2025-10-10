use std::fmt;

use solana_client::client_error::ClientError;

use crate::{config::NetworkFilter, probe::ProbeRecord};

/// Error returned by the smart router.
#[derive(Debug)]
pub enum RouterError {
    NoRegions(NetworkFilter),
    NoHealthyRegion {
        probes: Vec<ProbeRecord>,
    },
    TransactionFailed {
        region_code: &'static str,
        source: ClientError,
    },
    InvalidEncoding {
        encoding: &'static str,
        reason: String,
    },
    Deserialization(String),
    UnsupportedSubmissionEndpoint {
        description: String,
    },
}

impl std::error::Error for RouterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RouterError::TransactionFailed { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouterError::NoRegions(filter) => {
                write!(f, "no regions available for network filter {filter:?}")
            }
            RouterError::NoHealthyRegion { probes } => {
                write!(
                    f,
                    "no healthy regions found after probing {} endpoints",
                    probes.len()
                )
            }
            RouterError::TransactionFailed {
                region_code,
                source,
            } => {
                write!(f, "transaction failed for region {region_code}: {source}")
            }
            RouterError::InvalidEncoding { encoding, reason } => {
                write!(f, "failed to decode {encoding} transaction: {reason}")
            }
            RouterError::Deserialization(reason) => {
                write!(f, "failed to deserialize transaction bytes: {reason}")
            }
            RouterError::UnsupportedSubmissionEndpoint { description } => {
                write!(f, "submission strategy not yet implemented: {description}")
            }
        }
    }
}

impl RouterError {
    /// Returns the probing records if available.
    pub fn probes(&self) -> Option<&[ProbeRecord]> {
        match self {
            RouterError::NoHealthyRegion { probes } => Some(probes),
            _ => None,
        }
    }
}

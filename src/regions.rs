use once_cell::sync::Lazy;

/// Identifies the Solana network a BAM region belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BamNetwork {
    /// Solana mainnet-beta.
    Mainnet,
    /// Solana public testnet.
    Testnet,
}

/// Supplemental metadata for a region used in documentation or UIs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RegionMetadata {
    pub city: Option<&'static str>,
    pub country: Option<&'static str>,
    pub country_code: Option<&'static str>,
    pub notes: Option<&'static str>,
}

/// A BAM RPC region description
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BamRegion {
    pub code: &'static str,
    pub name: &'static str,
    pub endpoint: &'static str,
    pub network: BamNetwork,
    pub metadata: RegionMetadata,
}

pub const BAM_MAINNET_REGIONS: &[BamRegion] = &[
    BamRegion {
        code: "ams",
        name: "Amsterdam",
        endpoint: "http://amsterdam.mainnet.bam.jito.wtf",
        network: BamNetwork::Mainnet,
        metadata: RegionMetadata {
            city: Some("Amsterdam"),
            country: Some("Netherlands"),
            country_code: Some("NL"),
            notes: None,
        },
    },
    BamRegion {
        code: "fra",
        name: "Frankfurt",
        endpoint: "http://frankfurt.mainnet.bam.jito.wtf",
        network: BamNetwork::Mainnet,
        metadata: RegionMetadata {
            city: Some("Frankfurt"),
            country: Some("Germany"),
            country_code: Some("DE"),
            notes: None,
        },
    },
    BamRegion {
        code: "ny-mainnet",
        name: "New York",
        endpoint: "http://ny.mainnet.bam.jito.wtf",
        network: BamNetwork::Mainnet,
        metadata: RegionMetadata {
            city: Some("New York"),
            country: Some("United States"),
            country_code: Some("US"),
            notes: None,
        },
    },
];

pub const BAM_TESTNET_REGIONS: &[BamRegion] = &[
    BamRegion {
        code: "ny-testnet",
        name: "New York",
        endpoint: "http://ny.testnet.bam.jito.wtf",
        network: BamNetwork::Testnet,
        metadata: RegionMetadata {
            city: Some("New York"),
            country: Some("United States"),
            country_code: Some("US"),
            notes: None,
        },
    },
    BamRegion {
        code: "dal",
        name: "Dallas",
        endpoint: "http://dallas.testnet.bam.jito.wtf",
        network: BamNetwork::Testnet,
        metadata: RegionMetadata {
            city: Some("Dallas"),
            country: Some("United States"),
            country_code: Some("US"),
            notes: None,
        },
    },
    BamRegion {
        code: "slc",
        name: "Salt Lake City",
        endpoint: "http://slc.testnet.bam.jito.wtf",
        network: BamNetwork::Testnet,
        metadata: RegionMetadata {
            city: Some("Salt Lake City"),
            country: Some("United States"),
            country_code: Some("US"),
            notes: None,
        },
    },
];

static ALL_REGIONS: Lazy<Vec<BamRegion>> = Lazy::new(|| {
    let mut regions = Vec::with_capacity(BAM_MAINNET_REGIONS.len() + BAM_TESTNET_REGIONS.len());
    regions.extend_from_slice(BAM_MAINNET_REGIONS);
    regions.extend_from_slice(BAM_TESTNET_REGIONS);
    regions
});

/// Returns every known BAM region
pub fn all_regions() -> &'static [BamRegion] {
    ALL_REGIONS.as_slice()
}

/// Returns the regions that belong to the selected network
pub fn regions_for_network(network: BamNetwork) -> &'static [BamRegion] {
    match network {
        BamNetwork::Mainnet => BAM_MAINNET_REGIONS,
        BamNetwork::Testnet => BAM_TESTNET_REGIONS,
    }
}

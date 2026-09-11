use serde::{Deserialize, Serialize};

/// Transport selection is intentionally below TransferManager and above E2E.
/// Every transport must carry the same encrypted transfer frames.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransportType {
    DirectLan,
    NearbyHotspot,
    Bluetooth,
    Relay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportTarget {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub relay_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportCandidate {
    pub transport: TransportType,
    pub target: TransportTarget,
    pub available: bool,
    pub score: u8,
    pub reason: Option<String>,
}

pub fn select_best(candidates: &[TransportCandidate]) -> Option<TransportCandidate> {
    candidates
        .iter()
        .filter(|c| c.available)
        .max_by_key(|c| c.score)
        .cloned()
}

pub fn local_candidates(host: impl Into<String>, port: u16) -> Vec<TransportCandidate> {
    let host = host.into();
    vec![
        TransportCandidate {
            transport: TransportType::DirectLan,
            target: TransportTarget { host: Some(host.clone()), port: Some(port), relay_url: None },
            available: true,
            score: 100,
            reason: None,
        },
        TransportCandidate {
            transport: TransportType::NearbyHotspot,
            target: TransportTarget { host: Some(host), port: Some(port), relay_url: None },
            available: true,
            score: 80,
            reason: Some("Uses the same local transfer protocol over a hotspot/private network".into()),
        },
    ]
}

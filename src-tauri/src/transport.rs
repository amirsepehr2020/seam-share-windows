use std::net::SocketAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransportType {
    DirectLan,
    NearbyHotspot,
    Bluetooth,
    Relay,
}

#[derive(Debug, Clone)]
pub struct TransportCandidate {
    pub kind: TransportType,
    pub address: Option<SocketAddr>,
    pub available: bool,
    pub priority: u8,
}

pub trait SeamTransport {
    fn kind(&self) -> TransportType;
    fn is_available(&self) -> bool;
}

/// Keeps transport selection independent from TransferManager and E2E crypto.
pub fn rank(mut candidates: Vec<TransportCandidate>) -> Vec<TransportCandidate> {
    candidates.retain(|c| c.available);
    candidates.sort_by_key(|c| c.priority);
    candidates
}

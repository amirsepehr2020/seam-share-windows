use serde::Serialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};
use tauri::command;

const DISCOVERY_PORT: u16 = 38947;
const MAGIC: &[u8] = b"SEAM_SHARE_DISCOVER_V1";

#[derive(Debug, Serialize, Clone)]
pub struct NetworkInfo {
    pub local_ip: String,
    pub discovery_port: u16,
}

#[derive(Debug, Serialize, Clone)]
pub struct NearbyDevice {
    pub name: String,
    pub ip: String,
    pub port: u16,
}

#[command]
fn network_info() -> Result<NetworkInfo, String> {
    let ip = local_ip_address::local_ip().map_err(|e| e.to_string())?;
    Ok(NetworkInfo { local_ip: ip.to_string(), discovery_port: DISCOVERY_PORT })
}

#[command]
fn discover_devices(device_name: String, timeout_ms: u64) -> Result<Vec<NearbyDevice>, String> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).map_err(|e| e.to_string())?;
    socket.set_broadcast(true).map_err(|e| e.to_string())?;
    socket.set_read_timeout(Some(Duration::from_millis(120))).map_err(|e| e.to_string())?;

    let message = format!("{}|{}", String::from_utf8_lossy(MAGIC), device_name);
    socket.send_to(message.as_bytes(), SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), DISCOVERY_PORT)).map_err(|e| e.to_string())?;

    let deadline = Instant::now() + Duration::from_millis(timeout_ms.clamp(250, 3000));
    let mut found = Vec::new();
    let mut buffer = [0u8; 1024];
    while Instant::now() < deadline {
        match socket.recv_from(&mut buffer) {
            Ok((len, addr)) => {
                let text = String::from_utf8_lossy(&buffer[..len]);
                if let Some(name) = text.strip_prefix(&format!("{}|", String::from_utf8_lossy(MAGIC))) {
                    found.push(NearbyDevice { name: name.to_owned(), ip: addr.ip().to_string(), port: DISCOVERY_PORT });
                }
            }
            Err(_) => {}
        }
    }
    found.sort_by(|a, b| a.ip.cmp(&b.ip));
    found.dedup_by(|a, b| a.ip == b.ip);
    Ok(found)
}

#[command]
fn respond_to_discovery(device_name: String) -> Result<(), String> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, DISCOVERY_PORT)).map_err(|e| e.to_string())?;
    socket.set_broadcast(true).map_err(|e| e.to_string())?;
    let mut buffer = [0u8; 1024];
    loop {
        let (len, addr) = socket.recv_from(&mut buffer).map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&buffer[..len]);
        if text.starts_with(&format!("{}|", String::from_utf8_lossy(MAGIC))) {
            let reply = format!("{}|{}", String::from_utf8_lossy(MAGIC), device_name);
            socket.send_to(reply.as_bytes(), addr).map_err(|e| e.to_string())?;
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![network_info, discover_devices, respond_to_discovery])
        .run(tauri::generate_context!())
        .expect("error while running SEAM Share");
}

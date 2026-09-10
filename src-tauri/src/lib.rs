use serde::{Deserialize, Serialize};
use std::{fs, io::{Read, Write}, net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket}, path::PathBuf, sync::{Arc, Mutex}, thread, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use tauri::command;

const DISCOVERY_PORT: u16 = 38947;
const TRANSFER_PORT: u16 = 38948;
const MAGIC: &str = "SEAM_SHARE_DISCOVER_V2";
const CHUNK: usize = 1024 * 256;

#[derive(Debug, Serialize, Clone)] pub struct NetworkInfo { pub local_ip: String, pub discovery_port: u16, pub transfer_port: u16 }
#[derive(Debug, Serialize, Clone)] pub struct NearbyDevice { pub name: String, pub ip: String, pub port: u16 }
#[derive(Debug, Serialize, Deserialize, Clone)] pub struct PairRequest { pub device_id: String, pub device_name: String, pub token: String }
#[derive(Debug, Serialize, Clone)] pub struct PairingInfo { pub device_id: String, pub device_name: String, pub ip: String, pub port: u16, pub token: String }
#[derive(Clone)] struct AppState { device_id: String, device_name: String, token: String, paired: Arc<Mutex<Vec<String>>> }

fn local_ip() -> Result<IpAddr, String> { local_ip_address::local_ip().map_err(|e| e.to_string()) }
fn make_id() -> String { format!("seam-{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos()) }
fn safe_filename(value: &str) -> String { let decoded = urlencoding::decode(value).unwrap_or_else(|_| value.into()); let safe = decoded.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c,'.'|'_'|'-'|' ')).collect::<String>(); if safe.is_empty() { "received-file".into() } else { safe } }

#[command] fn network_info() -> Result<NetworkInfo, String> { Ok(NetworkInfo { local_ip: local_ip()?.to_string(), discovery_port: DISCOVERY_PORT, transfer_port: TRANSFER_PORT }) }
#[command] fn pairing_info(state: tauri::State<AppState>) -> Result<PairingInfo, String> { Ok(PairingInfo { device_id: state.device_id.clone(), device_name: state.device_name.clone(), ip: local_ip()?.to_string(), port: TRANSFER_PORT, token: state.token.clone() }) }

#[command]
fn discover_devices(device_name: String, timeout_ms: u64) -> Result<Vec<NearbyDevice>, String> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).map_err(|e| e.to_string())?; socket.set_broadcast(true).map_err(|e| e.to_string())?; socket.set_read_timeout(Some(Duration::from_millis(150))).map_err(|e| e.to_string())?;
    let message = format!("{}|{}|{}", MAGIC, device_name, TRANSFER_PORT); socket.send_to(message.as_bytes(), (Ipv4Addr::BROADCAST, DISCOVERY_PORT)).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_millis(timeout_ms.clamp(300, 3000)); let mut found = Vec::new(); let mut buffer=[0u8;1024];
    while Instant::now()<deadline { if let Ok((len,addr))=socket.recv_from(&mut buffer) { let p:Vec<&str>=String::from_utf8_lossy(&buffer[..len]).split('|').collect(); if p.len()==3 && p[0]==MAGIC { if let Ok(port)=p[2].parse::<u16>() { found.push(NearbyDevice{name:p[1].into(),ip:addr.ip().to_string(),port}); } } } }
    found.sort_by(|a,b|a.ip.cmp(&b.ip)); found.dedup_by(|a,b|a.ip==b.ip); Ok(found)
}

fn run_discovery_responder(name: String) { if let Ok(socket)=UdpSocket::bind((Ipv4Addr::UNSPECIFIED,DISCOVERY_PORT)) { let _=socket.set_broadcast(true); let mut buffer=[0u8;1024]; loop { if let Ok((len,addr))=socket.recv_from(&mut buffer) { let p:Vec<&str>=String::from_utf8_lossy(&buffer[..len]).split('|').collect(); if p.len()>=3 && p[0]==MAGIC { let reply=format!("{}|{}|{}",MAGIC,name,TRANSFER_PORT); let _=socket.send_to(reply.as_bytes(),addr); } } } } }

fn read_headers(stream:&mut TcpStream)->Result<(String,Vec<(String,String)>,Vec<u8>),String>{ let mut data=Vec::new(); let mut buf=[0u8;4096]; loop { let n=stream.read(&mut buf).map_err(|e|e.to_string())?; if n==0{return Err("connection closed".into())} data.extend_from_slice(&buf[..n]); if let Some(pos)=data.windows(4).position(|w|w==b"\r\n\r\n") { let head=data[..pos+4].to_vec(); let body=data[pos+4..].to_vec(); let text=String::from_utf8_lossy(&head); let mut lines=text.split("\r\n"); let request=lines.next().unwrap_or_default().to_string(); let headers=lines.filter_map(|l|l.split_once(':')).map(|(k,v)|(k.trim().to_ascii_lowercase(),v.trim().to_string())).collect(); return Ok((request,headers,body)); } if data.len()>64*1024{return Err("headers too large".into())} } }
fn header(headers:&[(String,String)],key:&str)->Option<String>{headers.iter().find(|(k,_)|k==key).map(|(_,v)|v.clone())}

fn transfer_server(state:AppState){ let listener=match TcpListener::bind((Ipv4Addr::UNSPECIFIED,TRANSFER_PORT)){Ok(v)=>v,Err(_)=>return}; for incoming in listener.incoming(){ if let Ok(mut stream)=incoming { let s=state.clone(); thread::spawn(move||{ let (request,headers,body)=match read_headers(&mut stream){Ok(v)=>v,Err(_)=>return}; let token=header(&headers,"x-seam-token").unwrap_or_default(); if token!=s.token { let _=stream.write_all(b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n"); return; }
 if request.starts_with("POST /pair "){ if let Ok(text)=String::from_utf8(body) { if let Ok(pair)=serde_json::from_str::<PairRequest>(&text) { if let Ok(mut list)=s.paired.lock(){ if !list.contains(&pair.device_id){list.push(pair.device_id);} } let _=stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: application/json\r\n\r\n{}"); return; } } let _=stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n"); return; }
 if request.starts_with("POST /receive "){ let name=header(&headers,"x-file-name").unwrap_or_else(||"received-file".into()); let size=header(&headers,"content-length").and_then(|v|v.parse::<u64>().ok()).unwrap_or(0); let mut path=PathBuf::from(std::env::var("USERPROFILE").unwrap_or_else(|_|".".into())); path.push("Downloads"); path.push("SEAM Share"); let _=fs::create_dir_all(&path); path.push(safe_filename(&name)); if let Ok(mut file)=fs::File::create(path){ let mut total=body.len() as u64; let _=file.write_all(&body); let mut buf=[0u8;CHUNK]; while total<size { match stream.read(&mut buf){Ok(0)=>break,Ok(n)=>{let _=file.write_all(&buf[..n]);total+=n as u64},Err(_)=>break} } let _=stream.write_all(b"HTTP/1.1 201 Created\r\nContent-Length: 2\r\n\r\nOK"); return; } let _=stream.write_all(b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n"); }
 }); } } }
}

#[cfg_attr(mobile,tauri::mobile_entry_point)] pub fn run(){ let state=AppState{device_id:make_id(),device_name:"This PC".into(),token:make_id(),paired:Arc::new(Mutex::new(Vec::new()))}; let d=state.device_name.clone(); let t=state.clone(); thread::spawn(move||run_discovery_responder(d)); thread::spawn(move||transfer_server(t)); tauri::Builder::default().manage(state).invoke_handler(tauri::generate_handler![network_info,discover_devices,pairing_info]).run(tauri::generate_context!()).expect("error while running SEAM Share"); }

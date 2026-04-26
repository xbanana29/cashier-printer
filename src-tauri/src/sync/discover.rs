use std::net::UdpSocket;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{PeerInfo, PeerMap};

/// UDP port used for peer discovery broadcasts.
const DISCOVERY_PORT: u16 = 47295;
/// Interval between "I'm alive" broadcasts.
const BROADCAST_INTERVAL_SECS: u64 = 15;

/// Start UDP broadcast (announce) and listen threads.
pub fn start(peers: PeerMap, device_id: String, pc_name: String, http_port: u16) {
    // Announce thread: broadcast our presence on the LAN every 15 s
    {
        let device_id = device_id.clone();
        let pc_name = pc_name.clone();
        std::thread::spawn(move || broadcast_loop(device_id, pc_name, http_port));
    }

    // Listen thread: receive announcements from other devices
    {
        let device_id = device_id.clone();
        std::thread::spawn(move || listen_loop(peers, device_id));
    }
}

fn broadcast_loop(device_id: String, pc_name: String, http_port: u16) {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[sync] UDP broadcast bind failed: {e}");
            return;
        }
    };
    if let Err(e) = socket.set_broadcast(true) {
        eprintln!("[sync] set_broadcast failed: {e}");
    }

    let msg = serde_json::json!({
        "v": 1,
        "device_id": device_id,
        "pc_name": pc_name,
        "port": http_port
    })
    .to_string();
    let msg_bytes = msg.as_bytes().to_vec();

    loop {
        let dest = format!("255.255.255.255:{DISCOVERY_PORT}");
        if let Err(e) = socket.send_to(&msg_bytes, &dest) {
            eprintln!("[sync] broadcast send failed: {e}");
        }
        std::thread::sleep(std::time::Duration::from_secs(BROADCAST_INTERVAL_SECS));
    }
}

fn listen_loop(peers: PeerMap, my_device_id: String) {
    let socket = match UdpSocket::bind(format!("0.0.0.0:{DISCOVERY_PORT}")) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[sync] UDP listen bind :{DISCOVERY_PORT} failed: {e}");
            return;
        }
    };
    // Short read timeout so we don't block the thread forever on shutdown
    let _ = socket.set_read_timeout(Some(std::time::Duration::from_secs(5)));

    let mut buf = [0u8; 2048];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, from)) => {
                if let Ok(msg) = serde_json::from_slice::<serde_json::Value>(&buf[..len]) {
                    let peer_id = msg["device_id"].as_str().unwrap_or("").to_string();
                    if peer_id.is_empty() || peer_id == my_device_id {
                        continue;
                    }
                    let peer_pc = msg["pc_name"].as_str().unwrap_or("").to_string();
                    let peer_port = msg["port"].as_u64().unwrap_or(47289) as u16;
                    let peer_ip = from.ip().to_string();

                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    let mut map = peers.lock().expect("peers lock poisoned");
                    let entry = map.entry(peer_id.clone()).or_insert(PeerInfo {
                        device_id: peer_id.clone(),
                        pc_name: peer_pc.clone(),
                        addr: format!("{peer_ip}:{peer_port}"),
                        last_seen: now,
                        orders_synced: 0,
                    });
                    // Refresh address and timestamp on every heartbeat
                    entry.pc_name = peer_pc;
                    entry.addr = format!("{peer_ip}:{peer_port}");
                    entry.last_seen = now;
                }
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => eprintln!("[sync] UDP recv error: {e}"),
        }
    }
}

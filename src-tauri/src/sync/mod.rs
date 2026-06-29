pub mod client;
pub mod discover;
pub mod server;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::db::DbConn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub device_id: String,
    pub pc_name: String,
    /// "ip:port" of the peer's HTTP sync server
    pub addr: String,
    /// Unix timestamp of last UDP heartbeat
    pub last_seen: u64,
    /// Cumulative orders pulled from this peer
    pub orders_synced: u32,
    /// True if added manually by IP (not via UDP discovery).
    #[serde(default)]
    pub manual: bool,
}

pub type PeerMap = Arc<Mutex<HashMap<String, PeerInfo>>>;

/// Settings key holding the JSON array of manually-added peer addresses.
const MANUAL_PEERS_KEY: &str = "manual_peers";

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn normalize_addr(addr: &str) -> String {
    let a = addr.trim();
    if a.contains(':') {
        a.to_string()
    } else {
        format!("{a}:{}", server::BASE_PORT)
    }
}

/// Read the persisted list of manual peer addresses.
pub fn manual_peer_addrs(db: &DbConn) -> Vec<String> {
    let conn = match db.lock() {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    let raw = crate::db::settings::get_setting(&conn, MANUAL_PEERS_KEY).unwrap_or_default();
    if raw.is_empty() {
        return vec![];
    }
    serde_json::from_str(&raw).unwrap_or_default()
}

fn save_manual_peer_addrs(db: &DbConn, addrs: &[String]) {
    if let Ok(conn) = db.lock() {
        let raw = serde_json::to_string(addrs).unwrap_or_else(|_| "[]".into());
        let _ = crate::db::settings::set_setting(&conn, MANUAL_PEERS_KEY, &raw);
    }
}

/// Add a peer by address: ping to validate, insert into the live map, persist.
pub fn add_manual_peer(db: &DbConn, peers: &PeerMap, addr: &str) -> Result<PeerInfo, String> {
    let addr = normalize_addr(addr);
    let (device_id, pc_name) = client::ping_peer(&addr)?;
    let info = PeerInfo {
        device_id: device_id.clone(),
        pc_name,
        addr: addr.clone(),
        last_seen: now_secs(),
        orders_synced: 0,
        manual: true,
    };
    {
        let mut map = peers.lock().map_err(|_| "peers lock poisoned".to_string())?;
        map.insert(device_id, info.clone());
    }
    let mut addrs = manual_peer_addrs(db);
    if !addrs.iter().any(|a| a == &addr) {
        addrs.push(addr);
        save_manual_peer_addrs(db, &addrs);
    }
    Ok(info)
}

/// Remove a manually-added peer by address (from persistence + live map).
pub fn remove_manual_peer(db: &DbConn, peers: &PeerMap, addr: &str) -> Result<(), String> {
    let addr = normalize_addr(addr);
    let mut addrs = manual_peer_addrs(db);
    addrs.retain(|a| a != &addr);
    save_manual_peer_addrs(db, &addrs);
    let mut map = peers.lock().map_err(|_| "peers lock poisoned".to_string())?;
    map.retain(|_, p| !(p.manual && p.addr == addr));
    Ok(())
}

/// Re-insert persisted manual peers into the map at startup. Offline ones get a
/// placeholder entry so they still show and get retried by the sync loop.
// ponytail: an offline placeholder can briefly duplicate with a discovered entry once the peer comes online; harmless, dedupes itself on next add/remove.
fn load_manual_peers(db: &DbConn, peers: &PeerMap) {
    for addr in manual_peer_addrs(db) {
        match client::ping_peer(&addr) {
            Ok((device_id, pc_name)) => {
                let mut map = peers.lock().expect("peers lock poisoned");
                map.insert(
                    device_id.clone(),
                    PeerInfo { device_id, pc_name, addr, last_seen: now_secs(), orders_synced: 0, manual: true },
                );
            }
            Err(_) => {
                let key = format!("manual:{addr}");
                let mut map = peers.lock().expect("peers lock poisoned");
                map.insert(
                    key.clone(),
                    PeerInfo { device_id: key, pc_name: String::new(), addr, last_seen: 0, orders_synced: 0, manual: true },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_addr, server::BASE_PORT};

    #[test]
    fn normalize_appends_default_port_when_missing() {
        assert_eq!(normalize_addr("192.168.1.50"), format!("192.168.1.50:{BASE_PORT}"));
    }

    #[test]
    fn normalize_keeps_explicit_port_and_trims() {
        assert_eq!(normalize_addr("  192.168.1.50:47290 "), "192.168.1.50:47290");
    }
}

/// Start background sync threads and return the shared peer map.
pub fn start(db: DbConn, device_id: String, pc_name: String) -> PeerMap {
    let peers: PeerMap = Arc::new(Mutex::new(HashMap::new()));

    // Bind HTTP server first so we know which port to advertise
    let server_port = server::start(db.clone(), device_id.clone(), pc_name.clone());

    // Channel: discover thread signals sync thread when a new peer is first seen
    let (new_peer_tx, new_peer_rx) = std::sync::mpsc::channel::<()>();

    // UDP discovery: broadcast our info + listen for peers
    discover::start(
        peers.clone(),
        device_id.clone(),
        pc_name.clone(),
        server_port,
        new_peer_tx,
    );

    // Re-add manually configured peers (survives across restarts)
    load_manual_peers(&db, &peers);

    // Periodic sync thread: syncs immediately on new-peer signal, or every 30s otherwise
    {
        let db = db.clone();
        let peers = peers.clone();
        std::thread::spawn(move || {
            // Short initial delay so UDP discovery can hear the first heartbeats
            std::thread::sleep(std::time::Duration::from_secs(3));
            client::sync_from_all_peers(&db, &peers);
            loop {
                // Block until new peer found OR 30s timeout — whichever comes first
                let _ = new_peer_rx.recv_timeout(std::time::Duration::from_secs(30));
                // Drain any extra signals that arrived while we were syncing
                while new_peer_rx.try_recv().is_ok() {}
                client::sync_from_all_peers(&db, &peers);
            }
        });
    }

    peers
}

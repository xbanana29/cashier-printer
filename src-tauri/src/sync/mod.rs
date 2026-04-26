pub mod client;
pub mod discover;
pub mod server;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
}

pub type PeerMap = Arc<Mutex<HashMap<String, PeerInfo>>>;

/// Start background sync threads and return the shared peer map.
pub fn start(db: DbConn, device_id: String, pc_name: String) -> PeerMap {
    let peers: PeerMap = Arc::new(Mutex::new(HashMap::new()));

    // Bind HTTP server first so we know which port to advertise
    let server_port = server::start(db.clone(), device_id.clone(), pc_name.clone());

    // UDP discovery: broadcast our info + listen for peers
    discover::start(
        peers.clone(),
        device_id.clone(),
        pc_name.clone(),
        server_port,
    );

    // Periodic sync: wait 4s for initial peer discovery, then sync every 30s
    {
        let db = db.clone();
        let peers = peers.clone();
        let device_id = device_id.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(4));
            client::sync_from_all_peers(&db, &peers, &device_id);
            loop {
                std::thread::sleep(std::time::Duration::from_secs(30));
                client::sync_from_all_peers(&db, &peers, &device_id);
            }
        });
    }

    peers
}

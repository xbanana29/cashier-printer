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

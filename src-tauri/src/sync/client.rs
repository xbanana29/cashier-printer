use crate::db::DbConn;
use crate::db::orders::{get_orders_for_sync, insert_sync_order, SyncOrder};
use super::{PeerInfo, PeerMap};

/// Pull orders from every known peer. Returns total orders inserted.
pub fn sync_from_all_peers(db: &DbConn, peers: &PeerMap) -> usize {
    let peer_list: Vec<PeerInfo> = {
        let map = peers.lock().expect("peers lock poisoned");
        map.values().cloned().collect()
    };

    let mut total = 0;
    for peer in &peer_list {
        match pull_from_peer(db, peer) {
            Ok(n) => {
                total += n;
                if n > 0 {
                    let mut map = peers.lock().expect("peers lock poisoned");
                    if let Some(p) = map.get_mut(&peer.device_id) {
                        p.orders_synced += n as u32;
                    }
                }
            }
            Err(e) => eprintln!("[sync] pull from {} ({}) failed: {}", peer.pc_name, peer.addr, e),
        }

        // Also push our orders to each peer
        if let Err(e) = push_to_peer(db, peer) {
            eprintln!("[sync] push to {} ({}) failed: {}", peer.pc_name, peer.addr, e);
        }
    }
    total
}

fn pull_from_peer(db: &DbConn, peer: &PeerInfo) -> Result<usize, String> {
    let url = format!("http://{}/sync", peer.addr);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(3))
        .timeout(std::time::Duration::from_secs(8))
        .build();

    let response = agent
        .get(&url)
        .call()
        .map_err(|e| e.to_string())?;

    let orders: Vec<SyncOrder> = response
        .into_json()
        .map_err(|e| format!("JSON parse: {e}"))?;

    let mut inserted = 0;
    let conn = db.lock().map_err(|e| format!("DB lock: {e}"))?;
    for order in &orders {
        if insert_sync_order(&conn, order).unwrap_or(false) {
            inserted += 1;
        }
    }
    Ok(inserted)
}

fn push_to_peer(db: &DbConn, peer: &PeerInfo) -> Result<(), String> {
    let url = format!("http://{}/push", peer.addr);
    let orders = {
        let conn = db.lock().map_err(|e| format!("DB lock: {e}"))?;
        get_orders_for_sync(&conn).map_err(|e| e.to_string())?
    };
    if orders.is_empty() {
        return Ok(());
    }
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(3))
        .timeout(std::time::Duration::from_secs(8))
        .build();

    agent
        .post(&url)
        .set("Content-Type", "application/json")
        .send_json(ureq::serde_json::json!(orders))
        .map_err(|e| e.to_string())?;
    Ok(())
}

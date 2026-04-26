use tauri::State;

use crate::db::DbConn;
use crate::error::AppError;
use crate::sync::{client, PeerInfo, PeerMap};

#[tauri::command]
pub async fn get_peers(peers: State<'_, PeerMap>) -> Result<Vec<PeerInfo>, AppError> {
    let map = peers
        .lock()
        .map_err(|_| AppError::Database("peers lock poisoned".into()))?;
    Ok(map.values().cloned().collect())
}

#[tauri::command]
pub async fn sync_now(
    db: State<'_, DbConn>,
    peers: State<'_, PeerMap>,
) -> Result<usize, AppError> {
    let count = client::sync_from_all_peers(&db, &peers, "");
    Ok(count)
}

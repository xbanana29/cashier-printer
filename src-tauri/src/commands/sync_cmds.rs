use serde::Serialize;
use std::net::UdpSocket;
use tauri::State;

use crate::db::DbConn;
use crate::error::AppError;
use crate::sync::{client, PeerInfo, PeerMap};

/// Diagnostic info shown in Settings so the user can see why sync may not work.
#[derive(Debug, Serialize)]
pub struct SyncInfo {
    /// This PC's LAN IP (the interface used to reach the internet).
    pub local_ip: String,
    pub discovery_port: u16,
    /// HTTP sync port range, e.g. "47289-47294".
    pub sync_port: String,
    pub device_id: String,
    /// Windows network profile (Public/Private/...). "—" elsewhere.
    pub network_profile: String,
}

/// Best-effort local IP: connect a throwaway UDP socket so the OS picks the
/// outbound interface, then read its address. No packet is actually sent.
fn local_ip() -> String {
    if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
        if sock.connect("8.8.8.8:53").is_ok() {
            if let Ok(addr) = sock.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "—".to_string()
}

#[cfg(target_os = "windows")]
fn network_profile() -> String {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", "(Get-NetConnectionProfile).NetworkCategory"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    match out {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout);
            let v: Vec<&str> = s.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
            if v.is_empty() { "—".to_string() } else { v.join(", ") }
        }
        Err(_) => "—".to_string(),
    }
}

// ponytail: profile detection is Windows-only; other OSes don't have the Public/Private firewall split that breaks broadcast.
#[cfg(not(target_os = "windows"))]
fn network_profile() -> String {
    "—".to_string()
}

#[tauri::command]
pub async fn get_sync_info(db: State<'_, DbConn>) -> Result<SyncInfo, AppError> {
    let device_id = {
        let conn = db
            .lock()
            .map_err(|_| AppError::Database("db lock poisoned".into()))?;
        crate::db::settings::get_or_create_device_id(&conn)
    };
    let base = crate::sync::server::BASE_PORT;
    Ok(SyncInfo {
        local_ip: local_ip(),
        discovery_port: crate::sync::discover::DISCOVERY_PORT,
        sync_port: format!("{}-{}", base, base + 5),
        device_id,
        network_profile: network_profile(),
    })
}

/// Add inbound Windows Firewall rules (Profile=Any) for the discovery + sync
/// ports, so peers are found even when the network is set to "Public".
/// Self-elevates via UAC; we can't observe the elevated result, so success here
/// just means the prompt was launched.
#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn fix_firewall() -> Result<(), AppError> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let d = crate::sync::discover::DISCOVERY_PORT;
    let base = crate::sync::server::BASE_PORT;
    let end = base + 5;
    let script = format!(
        "Get-NetFirewallRule -DisplayName 'Cashier Printer Sync*' -ErrorAction SilentlyContinue | Remove-NetFirewallRule -ErrorAction SilentlyContinue\r\n\
         New-NetFirewallRule -DisplayName 'Cashier Printer Sync (UDP {d})' -Direction Inbound -Action Allow -Protocol UDP -LocalPort {d} -Profile Any | Out-Null\r\n\
         New-NetFirewallRule -DisplayName 'Cashier Printer Sync (TCP {base}-{end})' -Direction Inbound -Action Allow -Protocol TCP -LocalPort {base}-{end} -Profile Any | Out-Null\r\n"
    );
    let mut path = std::env::temp_dir();
    path.push("cashier_printer_firewall.ps1");
    std::fs::write(&path, script)
        .map_err(|e| AppError::Settings(format!("Gagal menulis skrip firewall: {e}")))?;

    // Launch an elevated PowerShell that runs the script (UAC prompt). Path may
    // contain spaces — single quotes in the ArgumentList handle that.
    let outer = format!(
        "Start-Process powershell -Verb RunAs -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File','{}'",
        path.display()
    );
    std::process::Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &outer])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| AppError::Settings(format!("Gagal menjalankan perbaikan firewall: {e}")))?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub async fn fix_firewall() -> Result<(), AppError> {
    Err(AppError::Settings(
        "Perbaikan firewall otomatis hanya tersedia di Windows.".into(),
    ))
}

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
    let count = client::sync_from_all_peers(&db, &peers);
    Ok(count)
}

#[tauri::command]
pub async fn add_manual_peer(
    addr: String,
    db: State<'_, DbConn>,
    peers: State<'_, PeerMap>,
) -> Result<(), AppError> {
    crate::sync::add_manual_peer(&db, &peers, &addr)
        .map_err(|e| AppError::Settings(format!("Tidak bisa terhubung ke {}: {e}", addr.trim())))?;
    // Pull immediately so orders appear right away
    let _ = client::sync_from_all_peers(&db, &peers);
    Ok(())
}

#[tauri::command]
pub async fn remove_manual_peer(
    addr: String,
    db: State<'_, DbConn>,
    peers: State<'_, PeerMap>,
) -> Result<(), AppError> {
    crate::sync::remove_manual_peer(&db, &peers, &addr).map_err(AppError::Settings)?;
    Ok(())
}

//! Data types mirroring the Tauri backend DTOs. Field names use snake_case to
//! match the backend's default serde serialization exactly.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Order {
    pub id: i64,
    pub customer_name: String,
    pub content: String,
    pub order_type: String,
    pub created_at: String,
    /// Workstation name that created this order.
    pub pc_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSettings {
    pub default_printer: String,
    pub paper_size: String,
    pub store_name: String,
    pub footer_text: String,
    pub serial_baud_rate: u32,
    pub auto_cut: bool,
    pub pc_name: String,
    pub extra_feeds: u8,
    pub bold_items: bool,
    pub large_font: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            default_printer: String::new(),
            paper_size: "80mm".to_string(),
            store_name: String::new(),
            footer_text: String::new(),
            serial_baud_rate: 9600,
            auto_cut: true,
            pc_name: String::new(),
            extra_feeds: 0,
            bold_items: false,
            large_font: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PrinterInfo {
    pub name: String,
    pub is_default: bool,
    #[allow(dead_code)]
    pub connection_type: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PeerInfo {
    pub device_id: String,
    pub pc_name: String,
    pub addr: String,
    pub last_seen: u64,
    pub orders_synced: u32,
    #[serde(default)]
    pub manual: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SyncInfo {
    pub local_ip: String,
    pub discovery_port: u16,
    pub sync_port: String,
    pub device_id: String,
    pub network_profile: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub body: String,
}

/// Character column width for each supported paper size.
pub fn char_width_for(paper_size: &str) -> i32 {
    match paper_size {
        "58mm" => 32,
        "75mm" => 42,
        _ => 48,
    }
}

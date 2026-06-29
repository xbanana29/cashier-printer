//! Typed wrappers around the Tauri `invoke` bridge.
//!
//! The Dioxus frontend runs as WASM inside the Tauri webview. With
//! `app.withGlobalTauri = true` (tauri.conf.json) the IPC entry point is
//! exposed at `window.__TAURI__.core.invoke`, which we bind here via
//! `wasm-bindgen`. Argument keys are camelCase to match Tauri's automatic
//! camelCase→snake_case parameter mapping (same contract the old TS used).

use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::types::{AppSettings, Order, PeerInfo, PrinterInfo, SyncInfo, UpdateInfo};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke, catch)]
    async fn tauri_invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// A backend error surfaced to the UI. Mirrors the serialized `AppError`
/// (`{ "type": ..., "message": ... }`); falls back gracefully for anything else.
#[derive(Debug, Clone)]
pub struct AppError {
    pub kind: String,
    pub message: Option<String>,
}

impl AppError {
    /// Human-readable text for a toast (message if present, else the kind).
    pub fn display(&self) -> String {
        self.message
            .clone()
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| self.kind.clone())
    }

    fn from_js(err: JsValue) -> Self {
        #[derive(serde::Deserialize)]
        struct Raw {
            #[serde(rename = "type")]
            kind: Option<String>,
            message: Option<String>,
        }
        if let Ok(raw) = serde_wasm_bindgen::from_value::<Raw>(err.clone()) {
            if raw.kind.is_some() || raw.message.is_some() {
                return AppError {
                    kind: raw.kind.unwrap_or_else(|| "Error".to_string()),
                    message: raw.message,
                };
            }
        }
        AppError {
            kind: "Error".to_string(),
            message: Some(err.as_string().unwrap_or_else(|| "Terjadi kesalahan".to_string())),
        }
    }
}

fn no_args() -> JsValue {
    JsValue::from(js_sys::Object::new())
}

fn to_args<T: Serialize>(value: &T) -> JsValue {
    serde_wasm_bindgen::to_value(value).unwrap_or(JsValue::NULL)
}

/// Invoke a command that returns a deserializable value.
async fn invoke<T: DeserializeOwned>(cmd: &str, args: JsValue) -> Result<T, AppError> {
    match tauri_invoke(cmd, args).await {
        Ok(val) => serde_wasm_bindgen::from_value(val).map_err(|e| AppError {
            kind: "Deserialize".to_string(),
            message: Some(e.to_string()),
        }),
        Err(err) => Err(AppError::from_js(err)),
    }
}

/// Invoke a command whose result we discard (returns `()` / void).
async fn invoke_unit(cmd: &str, args: JsValue) -> Result<(), AppError> {
    tauri_invoke(cmd, args)
        .await
        .map(|_| ())
        .map_err(AppError::from_js)
}

// ── Orders ───────────────────────────────────────────────────────────────────

pub async fn create_order(
    customer_name: &str,
    content: &str,
    order_type: &str,
) -> Result<i64, AppError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        customer_name: &'a str,
        content: &'a str,
        order_type: &'a str,
    }
    invoke(
        "create_order",
        to_args(&Args {
            customer_name,
            content,
            order_type,
        }),
    )
    .await
}

pub async fn get_orders(order_type: &str) -> Result<Vec<Order>, AppError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        order_type: &'a str,
    }
    invoke("get_orders", to_args(&Args { order_type })).await
}

pub async fn get_order(id: i64) -> Result<Order, AppError> {
    #[derive(Serialize)]
    struct Args {
        id: i64,
    }
    invoke("get_order", to_args(&Args { id })).await
}

pub async fn update_order(id: i64, customer_name: &str, content: &str) -> Result<(), AppError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args<'a> {
        id: i64,
        customer_name: &'a str,
        content: &'a str,
    }
    invoke_unit(
        "update_order",
        to_args(&Args {
            id,
            customer_name,
            content,
        }),
    )
    .await
}

pub async fn delete_order(id: i64) -> Result<(), AppError> {
    #[derive(Serialize)]
    struct Args {
        id: i64,
    }
    invoke_unit("delete_order", to_args(&Args { id })).await
}

pub async fn purge_old_orders() -> Result<u64, AppError> {
    invoke("purge_old_orders", no_args()).await
}

// ── Printing ─────────────────────────────────────────────────────────────────

pub async fn list_printers() -> Result<Vec<PrinterInfo>, AppError> {
    invoke("list_printers", no_args()).await
}

pub async fn list_serial_ports() -> Result<Vec<String>, AppError> {
    invoke("list_serial_ports", no_args()).await
}

pub async fn print_order(order_id: i64) -> Result<(), AppError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args {
        order_id: i64,
    }
    invoke_unit("print_order", to_args(&Args { order_id })).await
}

pub async fn reprint_order(order_id: i64) -> Result<(), AppError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args {
        order_id: i64,
    }
    invoke_unit("reprint_order", to_args(&Args { order_id })).await
}

pub async fn preview_receipt(order_id: i64) -> Result<String, AppError> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Args {
        order_id: i64,
    }
    invoke("preview_receipt", to_args(&Args { order_id })).await
}

pub async fn test_print() -> Result<(), AppError> {
    invoke_unit("test_print", no_args()).await
}

// ── Settings ─────────────────────────────────────────────────────────────────

pub async fn get_settings() -> Result<AppSettings, AppError> {
    invoke("get_settings", no_args()).await
}

pub async fn save_settings(settings: &AppSettings) -> Result<(), AppError> {
    #[derive(Serialize)]
    struct Args<'a> {
        settings: &'a AppSettings,
    }
    invoke_unit("save_settings", to_args(&Args { settings })).await
}

// ── LAN sync ─────────────────────────────────────────────────────────────────

pub async fn get_peers() -> Result<Vec<PeerInfo>, AppError> {
    invoke("get_peers", no_args()).await
}

pub async fn sync_now() -> Result<u64, AppError> {
    invoke("sync_now", no_args()).await
}

pub async fn get_sync_info() -> Result<SyncInfo, AppError> {
    invoke("get_sync_info", no_args()).await
}

pub async fn fix_firewall() -> Result<(), AppError> {
    invoke_unit("fix_firewall", no_args()).await
}

pub async fn add_manual_peer(addr: &str) -> Result<(), AppError> {
    #[derive(Serialize)]
    struct Args<'a> {
        addr: &'a str,
    }
    invoke_unit("add_manual_peer", to_args(&Args { addr })).await
}

pub async fn remove_manual_peer(addr: &str) -> Result<(), AppError> {
    #[derive(Serialize)]
    struct Args<'a> {
        addr: &'a str,
    }
    invoke_unit("remove_manual_peer", to_args(&Args { addr })).await
}

// ── App / updater / opener (moved from JS plugins to backend commands) ───────

pub async fn get_app_version() -> Result<String, AppError> {
    invoke("get_app_version", no_args()).await
}

pub async fn open_external(url: &str) -> Result<(), AppError> {
    #[derive(Serialize)]
    struct Args<'a> {
        url: &'a str,
    }
    invoke_unit("open_external", to_args(&Args { url })).await
}

pub async fn check_for_update() -> Result<Option<UpdateInfo>, AppError> {
    invoke("check_for_update", no_args()).await
}

pub async fn install_update() -> Result<(), AppError> {
    invoke_unit("install_update", no_args()).await
}

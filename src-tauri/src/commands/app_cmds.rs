use crate::error::AppError;
use serde::Serialize;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::UpdaterExt;

/// Return the running application version (from Cargo/tauri.conf.json).
/// Replaces the frontend `@tauri-apps/api/app` `getVersion()` call so the
/// Dioxus/WASM frontend only ever talks to the backend through `invoke`.
#[tauri::command]
pub async fn get_app_version(app: tauri::AppHandle) -> Result<String, AppError> {
    Ok(app.package_info().version.to_string())
}

/// Open an external URL in the user's default browser.
/// Replaces the frontend `@tauri-apps/plugin-opener` `openUrl()` call.
#[tauri::command]
pub async fn open_external(app: tauri::AppHandle, url: String) -> Result<(), AppError> {
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| AppError::Settings(e.to_string()))
}

#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub body: String,
}

/// Check the configured updater endpoint for a newer release.
///
/// Mirrors the previous frontend UX: any error (no release published yet,
/// offline, etc.) is treated as "up to date" by returning `None` rather than
/// surfacing a scary error to the cashier.
#[tauri::command]
pub async fn check_for_update(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, AppError> {
    let updater = match app.updater() {
        Ok(u) => u,
        Err(_) => return Ok(None),
    };
    match updater.check().await {
        Ok(Some(update)) => Ok(Some(UpdateInfo {
            version: update.version.clone(),
            body: update.body.clone().unwrap_or_default(),
        })),
        Ok(None) => Ok(None),
        Err(_) => Ok(None),
    }
}

/// Download + install the pending update, then relaunch the app.
///
/// On success the process restarts and this command never returns; on failure
/// it returns an error the frontend can show as a toast.
#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), AppError> {
    let updater = app
        .updater()
        .map_err(|e| AppError::Settings(e.to_string()))?;
    let update = updater
        .check()
        .await
        .map_err(|e| AppError::Settings(e.to_string()))?;
    match update {
        Some(update) => {
            update
                .download_and_install(|_chunk, _total| {}, || {})
                .await
                .map_err(|e| AppError::Settings(e.to_string()))?;
            app.restart();
        }
        None => Ok(()),
    }
}

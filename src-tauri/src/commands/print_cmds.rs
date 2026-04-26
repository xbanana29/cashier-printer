use crate::db::{orders, settings, DbConn};
use crate::error::AppError;
use crate::print::{builder, driver, list_available_printers, PrinterInfo};

#[tauri::command]
pub async fn list_printers() -> Result<Vec<PrinterInfo>, AppError> {
    Ok(list_available_printers())
}

#[tauri::command]
pub async fn list_serial_ports() -> Result<Vec<String>, AppError> {
    let ports = serialport::available_ports()
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.port_name)
        .collect();
    Ok(ports)
}

#[tauri::command]
pub async fn print_order(
    state: tauri::State<'_, DbConn>,
    order_id: i64,
) -> Result<(), AppError> {
    let (order, app_settings) = {
        let conn = state.lock().map_err(|_| AppError::Database("lock poisoned".into()))?;
        let order = orders::get_order(&conn, order_id).map_err(AppError::from)?;
        let app_settings = settings::get_all_settings(&conn).map_err(AppError::from)?;
        (order, app_settings)
    };

    let bytes = builder::build_receipt(&order, &app_settings);
    driver::dispatch_print(&app_settings.default_printer, &bytes, app_settings.serial_baud_rate)
}

#[tauri::command]
pub async fn reprint_order(
    state: tauri::State<'_, DbConn>,
    order_id: i64,
) -> Result<(), AppError> {
    print_order(state, order_id).await
}

#[tauri::command]
pub async fn preview_receipt(
    state: tauri::State<'_, DbConn>,
    order_id: i64,
) -> Result<String, AppError> {
    let conn = state.lock().map_err(|_| AppError::Database("lock poisoned".into()))?;
    let order = orders::get_order(&conn, order_id).map_err(AppError::from)?;
    let app_settings = settings::get_all_settings(&conn).map_err(AppError::from)?;
    Ok(builder::build_receipt_preview(&order, &app_settings))
}

#[tauri::command]
pub async fn test_print(state: tauri::State<'_, DbConn>) -> Result<(), AppError> {
    let app_settings = {
        let conn = state.lock().map_err(|_| AppError::Database("lock poisoned".into()))?;
        settings::get_all_settings(&conn).map_err(AppError::from)?
    };

    let bytes = builder::build_test_receipt(&app_settings);
    driver::dispatch_print(&app_settings.default_printer, &bytes, app_settings.serial_baud_rate)
}

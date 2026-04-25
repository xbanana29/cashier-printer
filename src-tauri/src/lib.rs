pub mod commands;
pub mod db;
pub mod error;
pub mod print;

use tauri::Manager;

/// Retrieve the machine hostname via the OS `hostname` command.
/// Used as a default value for the `pc_name` setting on first run.
fn get_hostname() -> String {
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "PC".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("could not resolve app data dir");
            std::fs::create_dir_all(&app_data_dir)?;
            let conn = db::init(&app_data_dir).expect("failed to initialize database");

            // Auto-populate pc_name with the system hostname on first run
            {
                let c = conn.lock().expect("db lock poisoned");
                let current = db::settings::get_setting(&c, "pc_name").unwrap_or_default();
                if current.is_empty() {
                    let _ = db::settings::set_setting(&c, "pc_name", &get_hostname());
                }
            }

            app.manage(conn);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::order_cmds::create_order,
            commands::order_cmds::get_orders,
            commands::order_cmds::get_order,
            commands::order_cmds::update_order,
            commands::order_cmds::delete_order,
            commands::order_cmds::purge_old_orders,
            commands::print_cmds::list_printers,
            commands::print_cmds::print_order,
            commands::print_cmds::reprint_order,
            commands::print_cmds::preview_receipt,
            commands::print_cmds::test_print,
            commands::settings_cmds::get_settings,
            commands::settings_cmds::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

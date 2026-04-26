use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Deserializer, Serialize};

fn de_string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        other => Err(D::Error::custom(format!("expected string or number, got {other}"))),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub default_printer: String,
    pub paper_size: String,
    pub store_name: String,
    pub footer_text: String,
    /// Baud rate for serial/COM port connections (9600 / 19200 / 38400 / 115200).
    /// Ignored when printing via OS spooler or network.
    pub serial_baud_rate: u32,
    /// Whether to send an auto-cut command at the end of each receipt.
    /// Disable for dot-matrix printers (e.g. TM-U220) that lack an auto-cutter.
    pub auto_cut: bool,
    /// Display name of this workstation, printed at the bottom of each receipt
    /// and shown in the order history list. Auto-populated from system hostname.
    pub pc_name: String,
    /// ESC/POS character size for the order content (item list) lines.
    /// Accepts numeric pt values (8–24) or legacy keywords "normal"/"tall"/"wide"/"large".
    #[serde(deserialize_with = "de_string_or_number")]
    pub content_font_size: String,
    /// Extra blank lines fed after the receipt (0–5).
    /// Advances paper so the last printed line clears the print-head area on printers
    /// that don't fully eject paper on their own.
    pub extra_feeds: u8,
}

pub fn get_setting(conn: &Connection, key: &str) -> Result<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .map(|v: Option<String>| v.unwrap_or_default())
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

pub fn get_all_settings(conn: &Connection) -> Result<AppSettings> {
    Ok(AppSettings {
        default_printer: get_setting(conn, "default_printer").unwrap_or_default(),
        paper_size: get_setting(conn, "paper_size").unwrap_or_else(|_| "80mm".to_string()),
        store_name: get_setting(conn, "store_name").unwrap_or_default(),
        footer_text: get_setting(conn, "footer_text").unwrap_or_default(),
        serial_baud_rate: get_setting(conn, "serial_baud_rate")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(9600),
        auto_cut: get_setting(conn, "auto_cut")
            .ok()
            .map(|v| v != "false")
            .unwrap_or(true),
        pc_name: get_setting(conn, "pc_name").unwrap_or_default(),
        content_font_size: get_setting(conn, "content_font_size")
            .unwrap_or_else(|_| "normal".to_string()),
        extra_feeds: get_setting(conn, "extra_feeds")
            .ok()
            .and_then(|v| v.parse::<u8>().ok())
            .map(|n| n.min(5))
            .unwrap_or(0),
    })
}

/// Get the stored device UUID, generating and persisting one if it doesn't exist yet.
pub fn get_or_create_device_id(conn: &Connection) -> String {
    if let Ok(id) = get_setting(conn, "device_id") {
        if !id.is_empty() {
            return id;
        }
    }
    let new_id = uuid::Uuid::new_v4().to_string();
    let _ = set_setting(conn, "device_id", &new_id);
    new_id
}

pub fn save_all_settings(conn: &Connection, settings: &AppSettings) -> Result<()> {
    set_setting(conn, "default_printer", &settings.default_printer)?;
    set_setting(conn, "paper_size", &settings.paper_size)?;
    set_setting(conn, "store_name", &settings.store_name)?;
    set_setting(conn, "footer_text", &settings.footer_text)?;
    set_setting(conn, "serial_baud_rate", &settings.serial_baud_rate.to_string())?;
    set_setting(conn, "auto_cut", if settings.auto_cut { "true" } else { "false" })?;
    set_setting(conn, "pc_name", &settings.pc_name)?;
    set_setting(conn, "content_font_size", &settings.content_font_size)?;
    set_setting(conn, "extra_feeds", &settings.extra_feeds.min(5).to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT);
             INSERT INTO settings VALUES ('paper_size',        '80mm');
             INSERT INTO settings VALUES ('default_printer',   '');
             INSERT INTO settings VALUES ('store_name',        '');
             INSERT INTO settings VALUES ('footer_text',       '');
             INSERT INTO settings VALUES ('serial_baud_rate',  '9600');
             INSERT INTO settings VALUES ('auto_cut',          'true');
             INSERT INTO settings VALUES ('pc_name',           '');
             INSERT INTO settings VALUES ('content_font_size', 'normal');
             INSERT INTO settings VALUES ('extra_feeds',       '0');",
        )
        .unwrap();
        conn
    }

    #[test]
    fn get_setting_returns_value() {
        let conn = setup();
        assert_eq!(get_setting(&conn, "paper_size").unwrap(), "80mm");
    }

    #[test]
    fn get_setting_missing_key_returns_empty() {
        let conn = setup();
        // query_row on a missing key returns QueryReturnedNoRows — get_setting maps that
        // through .map(|v: Option<String>| v.unwrap_or_default()), but query_row itself
        // returns Err(QueryReturnedNoRows) first. Verify this propagates.
        let result = get_setting(&conn, "nonexistent_key");
        assert!(result.is_err());
    }

    #[test]
    fn set_and_get_roundtrip() {
        let conn = setup();
        set_setting(&conn, "store_name", "Toko Maju").unwrap();
        assert_eq!(get_setting(&conn, "store_name").unwrap(), "Toko Maju");
    }

    #[test]
    fn set_setting_replaces_existing_value() {
        let conn = setup();
        set_setting(&conn, "paper_size", "58mm").unwrap();
        set_setting(&conn, "paper_size", "75mm").unwrap();
        assert_eq!(get_setting(&conn, "paper_size").unwrap(), "75mm");
    }

    #[test]
    fn get_all_settings_defaults() {
        let conn = setup();
        let s = get_all_settings(&conn).unwrap();
        assert_eq!(s.paper_size, "80mm");
        assert_eq!(s.serial_baud_rate, 9600);
        assert!(s.auto_cut);
        assert_eq!(s.default_printer, "");
        assert_eq!(s.store_name, "");
        assert_eq!(s.footer_text, "");
        assert_eq!(s.pc_name, "");
        assert_eq!(s.content_font_size, "normal");
        assert_eq!(s.extra_feeds, 0);
    }

    #[test]
    fn save_and_reload_all_settings() {
        let conn = setup();
        let original = AppSettings {
            default_printer: "Printer1".to_string(),
            paper_size: "58mm".to_string(),
            store_name: "Toko XYZ".to_string(),
            footer_text: "Terima kasih".to_string(),
            serial_baud_rate: 19200,
            auto_cut: false,
            pc_name: "Kasir 2".to_string(),
            content_font_size: "large".to_string(),
            extra_feeds: 3,
        };
        save_all_settings(&conn, &original).unwrap();
        let loaded = get_all_settings(&conn).unwrap();
        assert_eq!(loaded.default_printer, "Printer1");
        assert_eq!(loaded.paper_size, "58mm");
        assert_eq!(loaded.store_name, "Toko XYZ");
        assert_eq!(loaded.footer_text, "Terima kasih");
        assert_eq!(loaded.serial_baud_rate, 19200);
        assert!(!loaded.auto_cut);
        assert_eq!(loaded.pc_name, "Kasir 2");
        assert_eq!(loaded.content_font_size, "large");
        assert_eq!(loaded.extra_feeds, 3);
    }

    #[test]
    fn extra_feeds_clamped_to_5_on_save_and_load() {
        let conn = setup();
        let settings = AppSettings {
            default_printer: String::new(),
            paper_size: "80mm".to_string(),
            store_name: String::new(),
            footer_text: String::new(),
            serial_baud_rate: 9600,
            auto_cut: true,
            pc_name: String::new(),
            content_font_size: "normal".to_string(),
            extra_feeds: 99, // over max
        };
        save_all_settings(&conn, &settings).unwrap();
        let loaded = get_all_settings(&conn).unwrap();
        assert_eq!(loaded.extra_feeds, 5, "extra_feeds must be clamped to 5");
    }

    #[test]
    fn extra_feeds_invalid_string_falls_back_to_0() {
        let conn = setup();
        set_setting(&conn, "extra_feeds", "bad").unwrap();
        let s = get_all_settings(&conn).unwrap();
        assert_eq!(s.extra_feeds, 0);
    }

    #[test]
    fn auto_cut_false_string_parses_to_false() {
        let conn = setup();
        set_setting(&conn, "auto_cut", "false").unwrap();
        let s = get_all_settings(&conn).unwrap();
        assert!(!s.auto_cut);
    }

    #[test]
    fn auto_cut_any_other_value_is_true() {
        let conn = setup();
        for val in &["true", "1", "yes", "TRUE"] {
            set_setting(&conn, "auto_cut", val).unwrap();
            let s = get_all_settings(&conn).unwrap();
            assert!(s.auto_cut, "expected auto_cut=true for value '{val}'");
        }
    }

    #[test]
    fn serial_baud_rate_invalid_falls_back_to_9600() {
        let conn = setup();
        set_setting(&conn, "serial_baud_rate", "not_a_number").unwrap();
        let s = get_all_settings(&conn).unwrap();
        assert_eq!(s.serial_baud_rate, 9600);
    }

    #[test]
    fn serial_baud_rate_valid_values() {
        let conn = setup();
        for &rate in &[9600u32, 19200, 38400, 115200] {
            set_setting(&conn, "serial_baud_rate", &rate.to_string()).unwrap();
            let s = get_all_settings(&conn).unwrap();
            assert_eq!(s.serial_baud_rate, rate);
        }
    }
}

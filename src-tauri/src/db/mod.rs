use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

pub mod orders;
pub mod settings;

pub type DbConn = Arc<Mutex<Connection>>;

const SCHEMA_SQL: &str = "
    PRAGMA journal_mode=WAL;

    CREATE TABLE IF NOT EXISTS orders (
        id            INTEGER PRIMARY KEY AUTOINCREMENT,
        customer_name TEXT NOT NULL,
        content       TEXT NOT NULL,
        order_type    TEXT NOT NULL DEFAULT 'order',
        created_at    DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS settings (
        key   TEXT PRIMARY KEY,
        value TEXT
    );

    INSERT OR IGNORE INTO settings (key, value) VALUES ('paper_size',        '80mm');
    INSERT OR IGNORE INTO settings (key, value) VALUES ('default_printer',   '');
    INSERT OR IGNORE INTO settings (key, value) VALUES ('store_name',        'CV REJEKI AMERTA JAYA');
    INSERT OR IGNORE INTO settings (key, value) VALUES ('footer_text',       '');
    INSERT OR IGNORE INTO settings (key, value) VALUES ('serial_baud_rate',  '9600');
    INSERT OR IGNORE INTO settings (key, value) VALUES ('auto_cut',           'true');
    INSERT OR IGNORE INTO settings (key, value) VALUES ('pc_name',            '');
    INSERT OR IGNORE INTO settings (key, value) VALUES ('content_font_size',  'normal');
    INSERT OR IGNORE INTO settings (key, value) VALUES ('extra_feeds',        '0');
";

pub fn init(app_data_dir: &std::path::Path) -> Result<DbConn> {
    let db_path = app_data_dir.join("orders.db");
    let conn = Connection::open(&db_path)?;
    conn.execute_batch(SCHEMA_SQL)?;
    // Migration for existing DBs: no-op if column already exists
    let _ = conn.execute_batch(
        "ALTER TABLE orders ADD COLUMN order_type TEXT NOT NULL DEFAULT 'order';",
    );
    Ok(Arc::new(Mutex::new(conn)))
}

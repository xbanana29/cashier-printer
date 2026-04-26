use rusqlite::{Connection, Result, params};
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
    INSERT OR IGNORE INTO settings (key, value) VALUES ('device_id',          '');
";

pub fn init(app_data_dir: &std::path::Path) -> Result<DbConn> {
    let db_path = app_data_dir.join("orders.db");
    let conn = Connection::open(&db_path)?;
    conn.execute_batch(SCHEMA_SQL)?;

    // Migration: add order_type if missing (no-op when already exists)
    let _ = conn.execute_batch(
        "ALTER TABLE orders ADD COLUMN order_type TEXT NOT NULL DEFAULT 'order';",
    );
    // Migration: add sync_id if missing
    let _ = conn.execute_batch(
        "ALTER TABLE orders ADD COLUMN sync_id TEXT;",
    );

    // Generate sync_ids for existing orders that don't have one
    let ids_without_sync_id: Vec<i64> = {
        let mut stmt = conn.prepare("SELECT id FROM orders WHERE sync_id IS NULL")?;
        let ids: Vec<i64> = stmt.query_map([], |r| r.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        ids
    };
    for id in ids_without_sync_id {
        let sync_id = uuid::Uuid::new_v4().to_string();
        let _ = conn.execute(
            "UPDATE orders SET sync_id = ?1 WHERE id = ?2",
            params![sync_id, id],
        );
    }

    Ok(Arc::new(Mutex::new(conn)))
}

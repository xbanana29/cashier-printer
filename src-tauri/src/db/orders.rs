use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};

fn fmt_date(raw: &str) -> String {
    // "2026-04-25 14:32:00" → "25/04/26 14:32"
    if raw.len() >= 16 {
        format!(
            "{}/{}/{} {}:{}",
            &raw[8..10],
            &raw[5..7],
            &raw[2..4],
            &raw[11..13],
            &raw[14..16]
        )
    } else if raw.len() >= 10 {
        format!("{}/{}/{}", &raw[8..10], &raw[5..7], &raw[2..4])
    } else {
        raw.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: i64,
    pub customer_name: String,
    pub content: String,
    pub order_type: String,
    pub created_at: String,
}

/// Flat order struct used for LAN sync (raw ISO created_at, no formatting).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SyncOrder {
    pub sync_id: String,
    pub customer_name: String,
    pub content: String,
    pub order_type: String,
    pub created_at: String,
}

pub fn create_order(
    conn: &Connection,
    customer_name: &str,
    content: &str,
    order_type: &str,
) -> Result<i64> {
    let sync_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO orders (customer_name, content, order_type, sync_id) VALUES (?1, ?2, ?3, ?4)",
        params![customer_name, content, order_type, sync_id],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Return orders created within the last 30 days for sync (raw ISO dates).
pub fn get_orders_for_sync(conn: &Connection) -> Result<Vec<SyncOrder>> {
    let mut stmt = conn.prepare(
        "SELECT sync_id, customer_name, content, order_type, created_at \
         FROM orders \
         WHERE sync_id IS NOT NULL \
           AND created_at >= datetime('now', '-30 days') \
         ORDER BY created_at DESC \
         LIMIT 500",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(SyncOrder {
            sync_id: row.get(0)?,
            customer_name: row.get(1)?,
            content: row.get(2)?,
            order_type: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    rows.collect()
}

/// Insert a synced order from a remote peer. Returns true if the row was inserted (not a duplicate).
pub fn insert_sync_order(conn: &Connection, order: &SyncOrder) -> Result<bool> {
    let affected = conn.execute(
        "INSERT OR IGNORE INTO orders (sync_id, customer_name, content, order_type, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            order.sync_id,
            order.customer_name,
            order.content,
            order.order_type,
            order.created_at
        ],
    )?;
    Ok(affected > 0)
}

pub fn get_orders(conn: &Connection, order_type: &str) -> Result<Vec<Order>> {
    let mut stmt = conn.prepare(
        "SELECT id, customer_name, content, order_type, created_at \
         FROM orders WHERE order_type = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([order_type], |row| {
        Ok(Order {
            id: row.get(0)?,
            customer_name: row.get(1)?,
            content: row.get(2)?,
            order_type: row.get(3)?,
            created_at: fmt_date(&row.get::<_, String>(4)?),
        })
    })?;
    rows.collect()
}

pub fn get_order(conn: &Connection, id: i64) -> Result<Order> {
    conn.query_row(
        "SELECT id, customer_name, content, order_type, created_at FROM orders WHERE id = ?1",
        params![id],
        |row| {
            Ok(Order {
                id: row.get(0)?,
                customer_name: row.get(1)?,
                content: row.get(2)?,
                order_type: row.get(3)?,
                created_at: fmt_date(&row.get::<_, String>(4)?),
            })
        },
    )
}

pub fn update_order(
    conn: &Connection,
    id: i64,
    customer_name: &str,
    content: &str,
) -> Result<()> {
    let affected = conn.execute(
        "UPDATE orders SET customer_name = ?1, content = ?2 WHERE id = ?3",
        params![customer_name, content, id],
    )?;
    if affected == 0 {
        Err(rusqlite::Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

pub fn delete_order(conn: &Connection, id: i64) -> Result<()> {
    let affected = conn.execute("DELETE FROM orders WHERE id = ?1", params![id])?;
    if affected == 0 {
        Err(rusqlite::Error::QueryReturnedNoRows)
    } else {
        Ok(())
    }
}

pub fn delete_orders_older_than(conn: &Connection, days: u32) -> Result<usize> {
    let affected = conn.execute(
        "DELETE FROM orders WHERE created_at < datetime('now', ?1)",
        params![format!("-{} days", days)],
    )?;
    Ok(affected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE orders (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                customer_name TEXT NOT NULL,
                content       TEXT NOT NULL,
                order_type    TEXT NOT NULL DEFAULT 'order',
                created_at    DATETIME DEFAULT CURRENT_TIMESTAMP,
                sync_id       TEXT
            );",
        )
        .unwrap();
        conn
    }

    #[test]
    fn create_and_get_order() {
        let conn = setup();
        let id = create_order(&conn, "Pak Budi", "2 sak beras", "order").unwrap();
        assert!(id > 0);
        let order = get_order(&conn, id).unwrap();
        assert_eq!(order.customer_name, "Pak Budi");
        assert_eq!(order.content, "2 sak beras");
        assert_eq!(order.order_type, "order");
        assert_eq!(order.id, id);
        assert!(!order.created_at.is_empty());
    }

    #[test]
    fn create_and_get_receipt() {
        let conn = setup();
        let id = create_order(&conn, "Toko Maju", "Jenis : Retur\nGudang : A", "receipt").unwrap();
        let order = get_order(&conn, id).unwrap();
        assert_eq!(order.order_type, "receipt");
        assert_eq!(order.customer_name, "Toko Maju");
    }

    #[test]
    fn get_order_not_found() {
        let conn = setup();
        let err = get_order(&conn, 9999).unwrap_err();
        assert_eq!(err, rusqlite::Error::QueryReturnedNoRows);
    }

    #[test]
    fn get_orders_empty() {
        let conn = setup();
        let orders = get_orders(&conn, "order").unwrap();
        assert!(orders.is_empty());
    }

    #[test]
    fn get_orders_filters_by_type() {
        let conn = setup();
        create_order(&conn, "Alpha", "item a", "order").unwrap();
        create_order(&conn, "Beta", "item b", "receipt").unwrap();
        let orders = get_orders(&conn, "order").unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].customer_name, "Alpha");
        let receipts = get_orders(&conn, "receipt").unwrap();
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].customer_name, "Beta");
    }

    #[test]
    fn get_orders_returns_all_of_same_type() {
        let conn = setup();
        let id1 = create_order(&conn, "Alpha", "item a", "order").unwrap();
        let id2 = create_order(&conn, "Beta", "item b", "order").unwrap();
        let orders = get_orders(&conn, "order").unwrap();
        assert_eq!(orders.len(), 2);
        let ids: Vec<i64> = orders.iter().map(|o| o.id).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn update_order_success() {
        let conn = setup();
        let id = create_order(&conn, "Old Name", "old content", "order").unwrap();
        update_order(&conn, id, "New Name", "new content").unwrap();
        let order = get_order(&conn, id).unwrap();
        assert_eq!(order.customer_name, "New Name");
        assert_eq!(order.content, "new content");
    }

    #[test]
    fn update_order_not_found() {
        let conn = setup();
        let err = update_order(&conn, 9999, "X", "Y").unwrap_err();
        assert_eq!(err, rusqlite::Error::QueryReturnedNoRows);
    }

    #[test]
    fn delete_order_success() {
        let conn = setup();
        let id = create_order(&conn, "To Delete", "stuff", "order").unwrap();
        delete_order(&conn, id).unwrap();
        let err = get_order(&conn, id).unwrap_err();
        assert_eq!(err, rusqlite::Error::QueryReturnedNoRows);
    }

    #[test]
    fn delete_order_not_found() {
        let conn = setup();
        let err = delete_order(&conn, 9999).unwrap_err();
        assert_eq!(err, rusqlite::Error::QueryReturnedNoRows);
    }

    #[test]
    fn delete_orders_older_than_removes_old() {
        let conn = setup();
        conn.execute(
            "INSERT INTO orders (customer_name, content, order_type, created_at) VALUES ('Old', 'stuff', 'order', datetime('now', '-400 days'))",
            [],
        )
        .unwrap();
        create_order(&conn, "Recent", "stuff", "order").unwrap();

        let deleted = delete_orders_older_than(&conn, 365).unwrap();
        assert_eq!(deleted, 1);

        let remaining = get_orders(&conn, "order").unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].customer_name, "Recent");
    }

    #[test]
    fn delete_orders_older_than_keeps_recent() {
        let conn = setup();
        create_order(&conn, "New", "item", "order").unwrap();
        let deleted = delete_orders_older_than(&conn, 365).unwrap();
        assert_eq!(deleted, 0);
        assert_eq!(get_orders(&conn, "order").unwrap().len(), 1);
    }
}

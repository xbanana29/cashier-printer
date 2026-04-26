use crate::db::DbConn;
use crate::db::orders::{get_orders_for_sync, insert_sync_order, SyncOrder};

const BASE_PORT: u16 = 47289;

/// Bind the HTTP sync server on the first available port in [47289, 47295).
/// Returns the bound port, or 0 if all ports are busy.
pub fn start(db: DbConn, device_id: String, pc_name: String) -> u16 {
    for port in BASE_PORT..(BASE_PORT + 6) {
        if let Ok(server) = tiny_http::Server::http(format!("0.0.0.0:{}", port)) {
            std::thread::spawn(move || serve(server, db, device_id, pc_name));
            return port;
        }
    }
    eprintln!("[sync] Could not bind HTTP server on ports {BASE_PORT}-{}", BASE_PORT + 5);
    0
}

fn serve(server: tiny_http::Server, db: DbConn, device_id: String, pc_name: String) {
    for request in server.incoming_requests() {
        handle(&db, &device_id, &pc_name, request);
    }
}

fn handle(
    db: &DbConn,
    device_id: &str,
    pc_name: &str,
    mut request: tiny_http::Request,
) {
    let url = request.url().to_string();

    let body = if url == "/ping" {
        serde_json::json!({
            "ok": true,
            "device_id": device_id,
            "pc_name": pc_name
        })
        .to_string()
    } else if url.starts_with("/sync") {
        let orders = {
            let conn = match db.lock() {
                Ok(c) => c,
                Err(_) => {
                    let _ = request.respond(
                        tiny_http::Response::from_string("error").with_status_code(
                            tiny_http::StatusCode(500),
                        ),
                    );
                    return;
                }
            };
            get_orders_for_sync(&conn).unwrap_or_default()
        };
        serde_json::to_string(&orders).unwrap_or_else(|_| "[]".into())
    } else if url == "/push" {
        // Receive orders pushed by a peer
        let mut content = String::new();
        if request.as_reader().read_to_string(&mut content).is_ok() {
            if let Ok(orders) = serde_json::from_str::<Vec<SyncOrder>>(&content) {
                if let Ok(conn) = db.lock() {
                    for order in &orders {
                        let _ = insert_sync_order(&conn, order);
                    }
                }
            }
        }
        let _ = request.respond(tiny_http::Response::from_string(r#"{"ok":true}"#));
        return;
    } else {
        let _ = request.respond(
            tiny_http::Response::from_string("Not Found")
                .with_status_code(tiny_http::StatusCode(404)),
        );
        return;
    };

    let response = tiny_http::Response::from_string(body).with_header(
        tiny_http::Header::from_bytes("Content-Type", "application/json")
            .expect("valid header"),
    );
    let _ = request.respond(response);
}

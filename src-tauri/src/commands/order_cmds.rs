use crate::db::{orders, DbConn};
use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderDto {
    pub id: i64,
    pub customer_name: String,
    pub content: String,
    pub order_type: String,
    pub created_at: String,
}

impl From<orders::Order> for OrderDto {
    fn from(o: orders::Order) -> Self {
        OrderDto {
            id: o.id,
            customer_name: o.customer_name,
            content: o.content,
            order_type: o.order_type,
            created_at: o.created_at,
        }
    }
}

#[tauri::command]
pub async fn create_order(
    state: tauri::State<'_, DbConn>,
    customer_name: String,
    content: String,
    order_type: String,
) -> Result<i64, AppError> {
    let conn = state.lock().map_err(|_| AppError::Database("lock poisoned".into()))?;
    orders::create_order(&conn, &customer_name, &content, &order_type).map_err(AppError::from)
}

#[tauri::command]
pub async fn get_orders(
    state: tauri::State<'_, DbConn>,
    order_type: String,
) -> Result<Vec<OrderDto>, AppError> {
    let conn = state.lock().map_err(|_| AppError::Database("lock poisoned".into()))?;
    orders::get_orders(&conn, &order_type)
        .map(|v| v.into_iter().map(OrderDto::from).collect())
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn get_order(
    state: tauri::State<'_, DbConn>,
    id: i64,
) -> Result<OrderDto, AppError> {
    let conn = state.lock().map_err(|_| AppError::Database("lock poisoned".into()))?;
    orders::get_order(&conn, id)
        .map(OrderDto::from)
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn update_order(
    state: tauri::State<'_, DbConn>,
    id: i64,
    customer_name: String,
    content: String,
) -> Result<(), AppError> {
    let conn = state.lock().map_err(|_| AppError::Database("lock poisoned".into()))?;
    orders::update_order(&conn, id, &customer_name, &content).map_err(AppError::from)
}

#[tauri::command]
pub async fn delete_order(
    state: tauri::State<'_, DbConn>,
    id: i64,
) -> Result<(), AppError> {
    let conn = state.lock().map_err(|_| AppError::Database("lock poisoned".into()))?;
    orders::delete_order(&conn, id).map_err(AppError::from)
}

#[tauri::command]
pub async fn purge_old_orders(state: tauri::State<'_, DbConn>) -> Result<usize, AppError> {
    let conn = state.lock().map_err(|_| AppError::Database("lock poisoned".into()))?;
    orders::delete_orders_older_than(&conn, 365).map_err(AppError::from)
}

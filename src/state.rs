use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, PartialEq)]
pub struct CalendarCache {
    pub is_busy: bool,
    pub timestamp: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub status: Arc<RwLock<CalendarCache>>,
}

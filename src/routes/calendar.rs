use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
struct CalendarResponse {
    is_busy: bool,
    timestamp: String,
}

pub async fn handle_calendar_status_query(State(state): State<AppState>) -> impl IntoResponse {
    let calendar_cache = state.status.read().await;

    Json(CalendarResponse {
        is_busy: calendar_cache.is_busy,
        timestamp: calendar_cache.timestamp.clone(),
    })
    .into_response()
}

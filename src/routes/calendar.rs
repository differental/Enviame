// Enviame - Full-stack Priority Messenger with a Rust backend that respects priority settings and delivers messages.
// Copyright (C) 2025 Brian Chen (differental)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

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

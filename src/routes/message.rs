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

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::constants::MID_HASH_KEY;
use crate::state::AppState;
use crate::utils::check_hash;

#[derive(Deserialize)]
pub struct MessageStatusRequest {
    mid: i32,
    mid_hash: String,
}

#[derive(Serialize)]
struct MessageStatusResponse {
    mid: i32,
    status: String,
}

pub async fn handle_message_query(
    Query(params): Query<MessageStatusRequest>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if !check_hash(
        &params.mid.to_string(),
        &params.mid_hash,
        MID_HASH_KEY.as_str(),
    ) {
        return (StatusCode::BAD_REQUEST, "Hash validation failed.").into_response();
    }

    match sqlx::query!("SELECT status FROM messages WHERE id = $1", params.mid)
        .fetch_optional(&state.db)
        .await
        .unwrap()
    {
        Some(rec) => (
            StatusCode::OK,
            Json(MessageStatusResponse {
                mid: params.mid,
                status: rec.status,
            }),
        )
            .into_response(),
        None => (StatusCode::BAD_REQUEST, "Requested message does not exist.").into_response(),
    }
}

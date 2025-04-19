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

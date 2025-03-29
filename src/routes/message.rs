use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::{state::AppState, utils::check_hash};

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
    if !check_hash(&params.mid.to_string(), &params.mid_hash) {
        return (StatusCode::BAD_REQUEST, "Hash validation failed.").into_response();
    }

    let status = sqlx::query!("SELECT status FROM messages WHERE id = $1", params.mid)
        .fetch_one(&state.db)
        .await
        .unwrap()
        .status;

    (
        StatusCode::OK,
        Json(MessageStatusResponse {
            mid: params.mid,
            status,
        }),
    )
        .into_response()
}

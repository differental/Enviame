use axum::{response::IntoResponse, Json};
use serde::Serialize;

#[derive(Serialize)]
struct VersionResponse {
    version: String,
}

pub async fn handle_version() -> impl IntoResponse {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
    .into_response()
}
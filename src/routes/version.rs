use axum::{Json, response::IntoResponse};
use serde::Serialize;

#[derive(Serialize)]
struct VersionResponse {
    version: String,
    deployment: String,
}

pub async fn handle_version() -> impl IntoResponse {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        deployment: env!("DEPLOY_ENV").to_string(),
    })
    .into_response()
}

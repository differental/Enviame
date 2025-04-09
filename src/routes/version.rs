use axum::{Json, response::IntoResponse};
use serde::Serialize;
use std::env;

#[derive(Serialize)]
struct VersionResponse {
    version: String,
    deployment: String,
}

pub async fn handle_version() -> impl IntoResponse {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
        deployment: env::var("DEPLOY_ENV").expect("DEPLOY_ENV must be set"),
    })
    .into_response()
}

use axum::{Json, response::IntoResponse};
use serde::Serialize;

use crate::constants::{CARGO_PKG_VERSION, DEPLOY_ENV};

#[derive(Serialize)]
struct VersionResponse<'a> {
    version: &'a str,
    deployment: &'a str,
}

pub async fn handle_version() -> impl IntoResponse {
    Json(VersionResponse {
        version: CARGO_PKG_VERSION,
        deployment: DEPLOY_ENV.as_str(),
    })
    .into_response()
}

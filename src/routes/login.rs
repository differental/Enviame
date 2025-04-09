use axum::{
    Json,
    extract::{Query, State},
    http::header,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    token: String,
}

#[derive(Serialize)]
struct LoginResponse {
    email: Option<String>,
    name: Option<String>,
    verified: Option<bool>,
}

pub async fn handle_login(
    Query(params): Query<LoginRequest>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "SELECT email, name, verified FROM users WHERE token = $1",
        params.token
    )
    .fetch_optional(&state.db)
    .await
    .unwrap();

    match result {
        Some(user) => (
            [(
                header::SET_COOKIE,
                format!("token={}; Path=/; Secure; SameSite=Strict", params.token),
            )],
            Json(LoginResponse {
                email: Some(user.email),
                name: Some(user.name),
                verified: Some(user.verified),
            }),
        )
            .into_response(),
        None => Json(LoginResponse {
            email: None,
            name: None,
            verified: None,
        })
        .into_response(),
    }
}

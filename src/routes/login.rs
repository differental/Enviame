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
    role: Option<i32>,
}

pub async fn handle_login(
    Query(params): Query<LoginRequest>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "SELECT email, name, verified, role FROM users WHERE token = $1",
        params.token
    )
    .fetch_optional(&state.db)
    .await
    .unwrap();

    match result {
        Some(user) => {
            if !user.verified {
                sqlx::query!(
                    "UPDATE users SET verified = $1 WHERE token = $2",
                    true,
                    params.token
                )
                .execute(&state.db)
                .await
                .unwrap();
            }

            (
                [(
                    header::SET_COOKIE,
                    format!("token={}; Path=/; Secure; SameSite=Strict", params.token),
                )],
                Json(LoginResponse {
                    email: Some(user.email),
                    name: Some(user.name),
                    verified: Some(user.verified),
                    role: Some(user.role),
                }),
            )
                .into_response()
        }
        None => Json(LoginResponse {
            email: None,
            name: None,
            verified: None,
            role: None,
        })
        .into_response(),
    }
}

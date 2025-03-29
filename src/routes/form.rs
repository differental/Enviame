use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_csrf::CsrfToken;
use serde::{Deserialize, Serialize};

use crate::{state::AppState, utils::generate_hash};

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum MessagePriority {
    Standard,
    Urgent,
    Immediate,
}

impl ToString for MessagePriority {
    fn to_string(&self) -> String {
        match self {
            MessagePriority::Standard => "standard".to_string(),
            MessagePriority::Urgent => "urgent".to_string(),
            MessagePriority::Immediate => "immediate".to_string(),
        }
    }
}

#[derive(Deserialize)]
pub struct FormData {
    csrf_token: String,
    token: Option<String>,
    email: String,
    name: String,
    message: String,
    priority: MessagePriority,
}

#[derive(Serialize)]
struct SubmissionResponse {
    mid: i32,
    mid_hash: String,
}

pub async fn handle_form_submission(
    State(state): State<AppState>,
    token: CsrfToken,
    Json(payload): Json<FormData>,
) -> impl IntoResponse {
    // Validate csrf token
    if token.verify(&payload.csrf_token).is_err() {
        return (StatusCode::BAD_REQUEST, "CSRF token invalid.").into_response();
    }

    // If not prod or beta, do not modify database or send email
    if std::env::var("DEPLOY_ENV").unwrap_or_default() != "prod"
        && std::env::var("DEPLOY_ENV").unwrap_or_default() != "beta"
    {
        return (
            StatusCode::IM_A_TEAPOT,
            "Form submission ignored. This is not a production build.",
        )
            .into_response();
    }

    let user = match payload.token {
        Some(ref token) => sqlx::query!("SELECT uid, verified FROM users WHERE token = $1", token)
            .fetch_optional(&state.db)
            .await
            .unwrap(),
        None => None,
    };

    let sender_status = match user {
        Some(ref u) if u.verified => "verified",
        Some(_) => "unverified",
        None => "guest",
    };

    let message_id = sqlx::query!(
        "INSERT INTO messages (status, user_uid, sender, name, email, message, priority) VALUES ('pending', $1, $2, $3, $4, $5, $6) RETURNING id",
        user.as_ref().map(|u| u.uid),
        sender_status,
        payload.name,
        payload.email,
        payload.message,
        payload.priority.to_string()
    )
    .fetch_one(&state.db)
    .await
    .expect("Failed to insert data")
    .id;
    let mid_hash = generate_hash(&message_id.to_string());

    (
        StatusCode::OK,
        Json(SubmissionResponse {
            mid: message_id,
            mid_hash,
        }),
    )
        .into_response()
}

use axum::{
    extract::State,
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use axum_csrf::CsrfToken;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use serde::Deserialize;
use std::env;
use tokio::task;

use crate::{state::AppState, utils::capitalize_first};

#[derive(Deserialize, Clone)]
pub struct FormData {
    csrf_token: String,
    token: Option<String>,
    email: String,
    name: String,
    message: String,
    priority: String,
}

async fn send_email(to: &str, subject: &str, body: &str) -> Result<(), Box<dyn std::error::Error>> {
    let smtp_server = env::var("SMTP_SERVER")?;
    let smtp_username = env::var("SMTP_USERNAME")?;
    let smtp_password = env::var("SMTP_PASSWORD")?;
    let smtp_from = env::var("SMTP_FROM")?;

    let email = Message::builder()
        .from(smtp_from.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .body(body.to_string())?;

    let creds = Credentials::new(smtp_username, smtp_password);

    let mailer = SmtpTransport::relay(&smtp_server)?
        .credentials(creds)
        .build();

    mailer.send(&email)?;

    Ok(())
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
     && std::env::var("DEPLOY_ENV").unwrap_or_default() != "beta" {
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

    sqlx::query!(
        "INSERT INTO messages (user_uid, sender, name, email, message, priority) VALUES ($1, $2, $3, $4, $5, $6)",
        user.as_ref().map(|u| u.uid),
        sender_status,
        payload.name,
        payload.email,
        payload.message,
        payload.priority
    )
    .execute(&state.db)
    .await
    .expect("Failed to insert data");

    let master_email = env!("MASTER_EMAIL");
    let user_email = payload.email.clone();
    let payload_clone = payload.clone();
    let priority_capitalised = capitalize_first(payload_clone.priority);

    let utc_now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
    let cargo_version = env!("CARGO_PKG_VERSION");

    task::spawn(async move {
        let subject = format!(
            "[Enviame] {} Message from {}({})",
            priority_capitalised, payload_clone.name, sender_status
        );
        let body = format!(
            "Message details:\n\n---Start of Message---\n{}\n---End of Message---\n\nPriority: {}\nName: {}\nEmail: {}\nStatus: {}\nMessage delivered by Enviame {}, {}",
            payload_clone.message, priority_capitalised, payload_clone.name, payload_clone.email, sender_status, cargo_version, utc_now
        );
        let _ = send_email(master_email, &subject, &body).await;

        let subject_user = format!("[Enviame] {} Message Delivered", priority_capitalised);
        let body_user = format!(
            "Your following message has been delivered:\n\n---Start of Message---\n{}\n---End of Message---\n\nMessage written by {} ({}) and delivered by Enviame {}, {}",
            payload_clone.message, payload_clone.name, payload_clone.email, cargo_version, utc_now
        );
        let _ = send_email(&user_email, &subject_user, &body_user).await;
    });

    (StatusCode::OK, "Message submitted successfully!").into_response()
}

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_csrf::CsrfToken;
use reqwest::Client;
use serde::Deserialize;

use crate::constants::RECAPTCHA_SECRET_KEY;
use crate::routes::apply::send_login_link;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ResendLinkRequest {
    csrf_token: String,
    email: String,
    name: String,
    recaptcha: String,
}

#[derive(Deserialize)]
struct RecaptchaResponse {
    success: bool,
}

pub async fn handle_resend_link(
    State(state): State<AppState>,
    token: CsrfToken,
    Json(payload): Json<ResendLinkRequest>,
) -> impl IntoResponse {
    // Validate csrf token
    if token.verify(&payload.csrf_token).is_err() {
        return (StatusCode::BAD_REQUEST, "CSRF token invalid.").into_response();
    }

    if payload.recaptcha.is_empty() {
        return (StatusCode::BAD_REQUEST, "reCAPTCHA verification failed").into_response();
    }

    let client = Client::new();
    let params = [
        ("secret", RECAPTCHA_SECRET_KEY.as_str()),
        ("response", &payload.recaptcha),
    ];

    let res = client
        .post("https://www.google.com/recaptcha/api/siteverify")
        .form(&params)
        .send()
        .await;

    match res {
        Ok(response) => {
            if let Ok(recaptcha_response) = response.json::<RecaptchaResponse>().await {
                if recaptcha_response.success {
                    if let Some(rec) = sqlx::query!(
                        "SELECT token FROM users WHERE (email, name) = ($1, $2)",
                        payload.email,
                        payload.name
                    )
                    .fetch_optional(&state.db)
                    .await
                    .unwrap()
                    {
                        tokio::spawn(async move {
                            let _ =
                                send_login_link(&payload.name, &payload.email, &rec.token).await;
                        });

                        /* if let Err(ref err) = link_result {
                            eprintln!("Login link resender failed to resend login link: {:?}", err);
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Login link email failed to send.",
                            )
                                .into_response();
                        } */
                    }

                    return (
                        StatusCode::ACCEPTED,
                        "If your details are correct, please check your email for your permanent login link.",
                    )
                        .into_response();
                }
            }
            (StatusCode::BAD_REQUEST, "reCAPTCHA verification failed").into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error verifying reCAPTCHA",
        )
            .into_response(),
    }
}

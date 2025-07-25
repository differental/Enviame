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

use askama::Template;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_csrf::CsrfToken;
use reqwest::Client;
use serde::Deserialize;

use crate::constants::{
    ALLOW_MODIFY_DB, CARGO_PKG_VERSION, FROM_STANDARD, HOMEPAGE_URL, NOTIFICATION_EMAIL,
    RECAPTCHA_SECRET_KEY,
};
use crate::state::AppState;
use crate::utils::{generate_random_token, send_email};

#[derive(Template)]
#[template(path = "email_link.html")]
struct LinkEmailTemplate<'a> {
    link: &'a str,
    version: &'a str,
}

#[derive(Deserialize)]
pub struct ApplyRequest {
    csrf_token: String,
    email: String,
    name: String,
    recaptcha: String,
}

#[derive(Deserialize)]
struct RecaptchaResponse {
    success: bool,
}

pub async fn send_login_link(name: &str, email: &str, token: &str) -> anyhow::Result<()> {
    // It would be more reasonable to move this to the worker
    //    if there were significant registration/resend-link traffic
    let subject = format!("[Enviame] Login link for {name}");
    let link = format!("{}?token={}", *HOMEPAGE_URL, token);
    let link_template = LinkEmailTemplate {
        link: &link,
        version: CARGO_PKG_VERSION,
    };
    let link_body = link_template
        .render()
        .expect("Login link email failed to render");

    send_email(
        &FROM_STANDARD,
        email,
        &NOTIFICATION_EMAIL,
        &subject,
        &link_body,
    )
    .await
}

pub async fn handle_apply(
    State(state): State<AppState>,
    token: CsrfToken,
    Json(payload): Json<ApplyRequest>,
) -> impl IntoResponse {
    // If not prod or beta, do not modify database. See constants
    if !*ALLOW_MODIFY_DB {
        return (
            StatusCode::IM_A_TEAPOT,
            "Account application ignored. This is not a production build.",
        )
            .into_response();
    }

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
                    let token = generate_random_token();

                    match sqlx::query!(
                        "INSERT INTO users (email, name, token, verified, role) VALUES ($1, $2, $3, $4, $5)",
                        payload.email.trim(),
                        payload.name.trim(),
                        token,
                        false,
                        0
                    )
                    .execute(&state.db)
                    .await
                    {
                        Ok(_) => (),
                        Err(_) => {
                            return (StatusCode::BAD_REQUEST, "Duplicate Email").into_response();
                        }
                    }

                    tokio::spawn(async move {
                        let _ = send_login_link(payload.name.trim(), payload.email.trim(), &token)
                            .await;
                    });

                    /* if let Err(ref err) = link_result {
                        eprintln!("Application handler failed to send login link: {:?}", err);
                        return (
                            StatusCode::CREATED,
                            "Registration successful, but login link email failed to send. Please try again later.",
                        )
                            .into_response();
                    } */

                    return (
                        StatusCode::CREATED,
                        "Please check your email for your permanent login link.",
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

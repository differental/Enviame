use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

use crate::{state::AppState, utils::generate_token};

#[derive(Deserialize)]
pub struct ApplyRequest {
    email: String,
    name: String,
    recaptcha: String,
}

#[derive(Serialize, Deserialize)]
struct RecaptchaResponse {
    success: bool,
    challenge_ts: Option<String>,
    hostname: Option<String>,
}

pub async fn handle_apply(
    State(state): State<AppState>,
    Json(payload): Json<ApplyRequest>,
) -> impl IntoResponse {
    if payload.recaptcha.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("reCAPTCHA verification failed".into())
            .unwrap();
    }

    let recaptcha_key = env::var("RECAPTCHA_SECRET_KEY").expect("SECRET_KEY not found");

    let client = Client::new();
    let params = [("secret", recaptcha_key), ("response", payload.recaptcha)];

    let res = client
        .post("https://www.google.com/recaptcha/api/siteverify")
        .form(&params)
        .send()
        .await;

    match res {
        Ok(response) => {
            if let Ok(recaptcha_response) = response.json::<RecaptchaResponse>().await {
                if recaptcha_response.success {
                    let token = generate_token();

                    match sqlx::query!(
                        "INSERT INTO users (email, name, token, verified) VALUES ($1, $2, $3, $4)",
                        payload.email,
                        payload.name,
                        token,
                        false
                    )
                    .execute(&state.db)
                    .await
                    {
                        Ok(_) => (),
                        Err(_) => {
                            return Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body("Duplicate Email".into())
                                .unwrap();
                        }
                    }

                    let cookie_header = format!("token={}; Path=/; Secure; SameSite=Strict", token);

                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(header::SET_COOKIE, cookie_header)
                        .body(axum::body::Body::empty())
                        .unwrap();
                }
            }
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("reCAPTCHA verification failed".into())
                .unwrap()
        }
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Error verifying reCAPTCHA".into())
            .unwrap(),
    }
}

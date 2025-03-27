use axum::{extract::State, response::{Html, IntoResponse}};
use axum_csrf::CsrfToken;
use std::fs;

use crate::state::AppState;

pub async fn serve_index(State(_state): State<AppState>, token: CsrfToken) -> impl IntoResponse {
    let csrf_token = token.authenticity_token().unwrap();

    let html = fs::read_to_string("static/index.html")
        .unwrap_or_else(|_| "Error loading page".to_string())
        .replace("{{ csrf_token }}", &csrf_token);
    
    (token, Html(html)).into_response()
}

pub async fn serve_apply_form(State(_state): State<AppState>, token: CsrfToken) -> impl IntoResponse {
    let csrf_token = token.authenticity_token().unwrap();

    let html = fs::read_to_string("static/apply.html")
        .unwrap_or_else(|_| "Error loading application page".to_string())
        .replace("{{ csrf_token }}", &csrf_token)
        .replace("{{ recaptcha_site_token }}", env!("RECAPTCHA_SITE_KEY"));

    (token, Html(html)).into_response()
}

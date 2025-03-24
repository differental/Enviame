use axum::response::{Html, IntoResponse};
use std::fs;

pub async fn serve_index() -> impl IntoResponse {
    let html = fs::read_to_string("static/index.html")
        .unwrap_or_else(|_| "Error loading page".to_string());
    Html(html)
}

pub async fn serve_apply_form() -> impl IntoResponse {
    let html = fs::read_to_string("static/apply.html")
        .unwrap_or_else(|_| "Error loading application page".to_string());
    Html(html)
}

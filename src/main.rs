use axum::{
    extract::{Query, State},
    http::header,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use rand::{distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::fs;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

fn generate_token() -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(32) // 32-character token
        .map(char::from)
        .collect()
}

#[derive(Deserialize)]
struct LoginRequest {
    token: String,
}

#[derive(Serialize)]
struct LoginResponse {
    email: Option<String>,
    name: Option<String>,
}

async fn login(
    Query(params): Query<LoginRequest>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        "SELECT email, name FROM users WHERE token = $1",
        params.token
    )
    .fetch_optional(&state.db)
    .await
    .unwrap();

    match result {
        Some(user) => (
            [(
                header::SET_COOKIE,
                format!(
                    "token={}; Path=/;",
                    params.token
                ),
            )],
            Json(LoginResponse {
                email: Some(user.email),
                name: Some(user.name),
            }),
        )
            .into_response(),
        None => Json(LoginResponse {
            email: None,
            name: None,
        })
        .into_response(),
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPool::connect(&database_url).await?;

    let state = AppState { db: db_pool };

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/login", get(login))
        .route("/submit", post(submit_form))
        .layer(CorsLayer::new().allow_origin(Any))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

// Serve the static index.html file
async fn serve_index() -> impl IntoResponse {
    let html = fs::read_to_string("static/index.html")
        .unwrap_or_else(|_| "Error loading page".to_string());
    Html(html)
}

#[derive(Deserialize)]
struct FormData {
    token: Option<String>,
    email: String,
    name: String,
    message: String,
}

async fn submit_form(State(state): State<AppState>, Json(payload): Json<FormData>) -> &'static str {
    let user = match payload.token {
        Some(token) => sqlx::query!("SELECT uid FROM users WHERE token = $1", token)
            .fetch_optional(&state.db)
            .await
            .unwrap(),
        None => None,
    };

    sqlx::query!(
        "INSERT INTO messages (user_uid, sender, name, email, message) VALUES ($1, $2, $3, $4, $5)",
        user.as_ref().map(|u| u.uid),
        if user.is_some() { "user" } else { "guest" },
        payload.name,
        payload.email,
        payload.message
    )
    .execute(&state.db)
    .await
    .expect("Failed to insert data");

    "Message submitted successfully!"
}

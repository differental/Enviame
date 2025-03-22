use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rand::{distr::Alphanumeric, Rng};
use reqwest::Client;
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
                format!("token={}; Path=/; Secure; SameSite=Strict", params.token),
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

const RECAPTCHA_SECRET_KEY: &str = "6LdpEv0qAAAAADUCDqOi1q73tuR8ys0vADBtKMEe";

#[derive(Deserialize)]
struct ApplyRequest {
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

async fn apply(
    State(state): State<AppState>,
    Json(payload): Json<ApplyRequest>,
) -> impl IntoResponse {
    if payload.recaptcha.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("reCAPTCHA verification failed".into())
            .unwrap();
    }

    let client = Client::new();
    let params = [
        ("secret", RECAPTCHA_SECRET_KEY),
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

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPool::connect(&database_url).await?;

    let state = AppState { db: db_pool };

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/apply", get(serve_apply_form))
        .route("/api/login", get(login))
        .route("/api/submit", post(submit_form))
        .route("/api/apply", post(apply))
        .layer(CorsLayer::new().allow_origin(Any))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn serve_index() -> impl IntoResponse {
    let html = fs::read_to_string("static/index.html")
        .unwrap_or_else(|_| "Error loading page".to_string());
    Html(html)
}

async fn serve_apply_form() -> impl IntoResponse {
    let html = fs::read_to_string("static/apply.html")
        .unwrap_or_else(|_| "Error loading application page".to_string());
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
        Some(token) => sqlx::query!("SELECT uid, verified FROM users WHERE token = $1", token)
            .fetch_optional(&state.db)
            .await
            .unwrap(),
        None => None,
    };

    let sender_status = match user {
        Some(ref u) if u.verified.unwrap() => "verified",
        Some(_) => "unverified",
        None => "guest",
    };

    sqlx::query!(
        "INSERT INTO messages (user_uid, sender, name, email, message) VALUES ($1, $2, $3, $4, $5)",
        user.as_ref().map(|u| u.uid),
        sender_status,
        payload.name,
        payload.email,
        payload.message
    )
    .execute(&state.db)
    .await
    .expect("Failed to insert data");

    "Message submitted successfully!"
}

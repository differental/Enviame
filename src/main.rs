use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dotenvy::dotenv;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rand::{distr::Alphanumeric, Rng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::env;
use std::fs;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::task;
use tower_http::cors::{Any, CorsLayer};

const MASTER_EMAIL: &str = "brian@brianc.tech";

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

#[derive(Serialize)]
struct VersionResponse {
    version: String,
}

async fn get_version() -> impl IntoResponse {
    Json(VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
    .into_response()
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPool::connect(&database_url).await?;

    let state = AppState { db: db_pool };

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/apply", get(serve_apply_form))
        .route("/api/login", get(login))
        .route("/api/submit", post(submit_form))
        .route("/api/apply", post(apply))
        .route("/api/version", get(get_version))
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

#[derive(Deserialize, Clone)]
struct FormData {
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

async fn submit_form(State(state): State<AppState>, Json(payload): Json<FormData>) -> &'static str {
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

    let master_email = MASTER_EMAIL.to_string();
    let user_email = payload.email.clone();
    let payload_clone = payload.clone();

    task::spawn(async move {
        let subject = format!(
            "[Enviame] {} Message from {}({})",
            payload_clone.priority, payload_clone.name, sender_status
        );
        let body = format!(
            "Message details:\n\n---Start of Message---\n{}\n---End of Message---\n\nPriority: {}\nName: {}\nEmail: {}\nStatus: {}\nMessage delivered by Enviame {}, {}",
            payload_clone.message, payload_clone.priority, payload_clone.name, payload_clone.email, sender_status, env!("CARGO_PKG_VERSION"), chrono::Utc::now()
        );
        let _ = send_email(&master_email, &subject, &body).await;

        let subject_user = format!("[Enviame] {} Message Delivered", payload_clone.priority);
        let body_user = format!(
            "Your following message has been delivered:\n\n---Start of Message---\n{}\n---End of Message---\n\nMessage written by {} ({}) and delivered by Enviame {}, {}",
            payload_clone.message, payload_clone.name, payload_clone.email, env!("CARGO_PKG_VERSION"), chrono::Utc::now()
        );
        let _ = send_email(&user_email, &subject_user, &body_user).await;
    });

    "Message submitted successfully!"
}

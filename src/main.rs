use axum::{
    Router,
    routing::{get, post},
};
use axum_csrf::{CsrfConfig, CsrfLayer};
use dotenvy::dotenv;
use sqlx::PgPool;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

mod routes;
use routes::{
    apply::handle_apply,
    form::handle_form_submission,
    login::handle_login,
    message::handle_message_query,
    pages::{serve_about_page, serve_apply_form, serve_index},
    version::handle_version,
};

mod worker;
use worker::email_worker;

mod utils;

mod state;
use state::AppState;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPool::connect(&database_url).await?;

    let state = AppState { db: db_pool };

    let port: u16 = std::env::var("APP_PORT")
        .expect("APP_PORT must be set")
        .parse()
        .expect("APP_PORT must be a valid number");

    let csrf_config = CsrfConfig::default();

    let state_clone = state.clone();
    tokio::spawn(async move {
        email_worker(state_clone).await;
    });

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/apply", get(serve_apply_form))
        .route("/about", get(serve_about_page))
        .route("/api/login", get(handle_login))
        .route("/api/submit", post(handle_form_submission))
        .route("/api/apply", post(handle_apply))
        .route("/api/version", get(handle_version))
        .route("/api/message", get(handle_message_query))
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(CorsLayer::new().allow_origin(Any))
        .layer(CsrfLayer::new(csrf_config))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server running on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

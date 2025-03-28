use axum::{
    routing::{get, post},
    Router,
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
    pages::{serve_about_page, serve_apply_form, serve_index},
    version::handle_version,
};

mod utils;

mod state;
use state::AppState;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPool::connect(&database_url).await?;

    let state = AppState { db: db_pool };

    let port: u16 = std::env::var("APP_PORT")
        .expect("APP_PORT must be set")
        .parse()
        .expect("APP_PORT must be a valid number");

    let csrf_config = CsrfConfig::default();

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/apply", get(serve_apply_form))
        .route("/about", get(serve_about_page))
        .route("/api/login", get(handle_login))
        .route("/api/submit", post(handle_form_submission))
        .route("/api/apply", post(handle_apply))
        .route("/api/version", get(handle_version))
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

use axum::{
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use sqlx::PgPool;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod routes;
use routes::{
    apply::handle_apply,
    form::handle_form_submission,
    login::handle_login,
    pages::{serve_apply_form, serve_index},
    version::handle_version,
};

mod utils;

mod state;
use state::AppState;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPool::connect(&database_url).await?;

    let state = AppState { db: db_pool };

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/apply", get(serve_apply_form))
        .route("/api/login", get(handle_login))
        .route("/api/submit", post(handle_form_submission))
        .route("/api/apply", post(handle_apply))
        .route("/api/version", get(handle_version))
        .layer(CorsLayer::new().allow_origin(Any))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

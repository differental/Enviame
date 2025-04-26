use axum::{
    Router,
    routing::{get, post},
};
use axum_csrf::{CsrfConfig, CsrfLayer};
use dotenvy::dotenv;
use sqlx::PgPool;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};

mod routes;
use routes::{
    apply::handle_apply,
    assets::serve_embedded_assets,
    calendar::handle_calendar_status_query,
    form::handle_form_submission,
    login::handle_login,
    message::handle_message_query,
    pages::{serve_about_page, serve_apply_form, serve_index, serve_resend_link_form},
    resend_link::handle_resend_link,
    version::handle_version,
};

mod workers;
use workers::{calendar::calendar_worker, email::email_worker};

mod constants;

mod utils;

mod state;
use state::{AppState, CalendarCache};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPool::connect(&database_url).await?;

    let initial_cache = CalendarCache {
        is_busy: false,
        timestamp: "2099-12-31 23:59".to_owned(),
    };
    let initial_cache = Arc::new(RwLock::new(initial_cache));

    let state = AppState {
        db: db_pool,
        status: initial_cache,
    };

    let port: u16 = env::var("APP_PORT")
        .expect("APP_PORT must be set")
        .parse()
        .expect("APP_PORT must be a valid number");

    let csrf_config = CsrfConfig::default();

    let state_clone = state.clone();
    tokio::spawn(async move {
        email_worker(state_clone).await;
    });
    let state_clone = state.clone();
    tokio::spawn(async move {
        calendar_worker(state_clone).await;
    });

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/apply", get(serve_apply_form))
        .route("/about", get(serve_about_page))
        .route("/resendlink", get(serve_resend_link_form))
        .route("/api/login", get(handle_login))
        .route("/api/submit", post(handle_form_submission))
        .route("/api/apply", post(handle_apply))
        .route("/api/resendlink", post(handle_resend_link))
        .route("/api/version", get(handle_version))
        .route("/api/message", get(handle_message_query))
        .route("/api/calendar", get(handle_calendar_status_query))
        .route("/assets/{*file}", get(serve_embedded_assets))
        .layer(CsrfLayer::new(csrf_config))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Server running on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

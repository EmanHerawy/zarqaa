mod routes;
mod state;

use axum::{routing::post, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "zarqaa_gateway=debug,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = state::AppState::from_env();

    let app = Router::new()
        .route("/analyze", post(routes::analyze))
        .route("/analyze-intent", post(routes::analyze_intent))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await
        .expect("Failed to bind port 8080");

    tracing::info!("Zarqaa Gateway listening on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}

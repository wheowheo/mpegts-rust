use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

mod api;
mod ws;
mod ingest;
mod output;
mod state;

use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let (tx, _) = broadcast::channel::<String>(256);

    let state = Arc::new(AppState {
        analyzer: RwLock::new(ts_analyzer::StreamAnalyzer::new()),
        ws_tx: tx,
        output_session: RwLock::new(output::session::OutputSession::new()),
        system_stats: RwLock::new(ts_analyzer::system_stats::SystemStatsCollector::new()),
    });

    let api_routes = Router::new()
        .route("/stream", get(api::stream::get_stream_info))
        .route("/pids", get(api::pid::get_pid_map))
        .route("/pids/{pid}", get(api::pid::get_pid_detail))
        .route("/analyze", post(api::analyze::start_analysis))
        .route("/output/start", post(api::output::start_output))
        .route("/output/stop", post(api::output::stop_output))
        .route("/output/status", get(api::output::get_output_status))
        .route("/system", get(api::system::get_system_stats));

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/ws", get(ws::realtime::ws_handler))
        .fallback_service(ServeDir::new("dashboard/build"))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = "0.0.0.0:3200";
    tracing::info!("server listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

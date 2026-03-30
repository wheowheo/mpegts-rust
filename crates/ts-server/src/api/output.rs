use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use crate::state::AppState;
use crate::output::session::{OutputConfig, OutputStatus};

pub async fn start_output(
    State(state): State<Arc<AppState>>,
    Json(config): Json<OutputConfig>,
) -> Result<Json<OutputStatus>, (StatusCode, String)> {
    let ws_tx = state.ws_tx.clone();
    let mut session = state.output_session.write().await;

    session.start(config, ws_tx).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let status = session.status.read().await.clone();
    Ok(Json(status))
}

pub async fn stop_output(
    State(state): State<Arc<AppState>>,
) -> Json<OutputStatus> {
    let mut session = state.output_session.write().await;
    session.stop().await;
    let status = session.status.read().await.clone();
    Json(status)
}

pub async fn get_output_status(
    State(state): State<Arc<AppState>>,
) -> Json<OutputStatus> {
    let session = state.output_session.read().await;
    let status = session.status.read().await.clone();
    Json(status)
}

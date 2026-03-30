use axum::{
    extract::{Path, State},
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
    let mut manager = state.output_manager.write().await;

    let status = manager.start(config, ws_tx).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    Ok(Json(status))
}

pub async fn stop_output(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Json<serde_json::Value> {
    let mut manager = state.output_manager.write().await;
    manager.stop(&session_id).await;
    Json(serde_json::json!({ "stopped": session_id }))
}

pub async fn stop_all_outputs(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let mut manager = state.output_manager.write().await;
    manager.stop_all().await;
    Json(serde_json::json!({ "stopped": "all" }))
}

pub async fn get_output_status(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> Result<Json<OutputStatus>, StatusCode> {
    let manager = state.output_manager.read().await;
    manager.get_status(&session_id).await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn list_outputs(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<OutputStatus>> {
    let manager = state.output_manager.read().await;
    Json(manager.list_all().await)
}

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::state::AppState;
use crate::db::repository::{SessionRecord, PidSnapshotRecord, ErrorRecord, BitrateRecord, HistoryStats};

#[derive(Deserialize)]
pub struct ListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<SessionRecord>>, StatusCode> {
    let limit = q.limit.unwrap_or(50).min(200);
    let offset = q.offset.unwrap_or(0);
    state.db.list_sessions(limit, offset)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Serialize)]
pub struct SessionDetail {
    pub session: SessionRecord,
    pub pids: Vec<PidSnapshotRecord>,
    pub errors: Vec<ErrorRecord>,
}

pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SessionDetail>, StatusCode> {
    let session = state.db.get_session(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let pids = state.db.get_session_pids(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let errors = state.db.get_session_errors(&id, 500)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SessionDetail { session, pids, errors }))
}

pub async fn get_session_bitrate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<BitrateRecord>>, StatusCode> {
    state.db.get_session_bitrate(&id)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn get_session_errors(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<ErrorRecord>>, StatusCode> {
    let limit = q.limit.unwrap_or(200);
    state.db.get_session_errors(&id, limit)
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn delete_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let deleted = state.db.delete_session(&id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted { Ok(StatusCode::NO_CONTENT) } else { Err(StatusCode::NOT_FOUND) }
}

pub async fn get_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<HistoryStats>, StatusCode> {
    state.db.stats()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

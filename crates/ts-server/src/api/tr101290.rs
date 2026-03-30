use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use ts_analyzer::tr101290::Tr101290Summary;
use crate::state::AppState;

pub async fn get_tr101290(
    State(state): State<Arc<AppState>>,
) -> Json<Tr101290Summary> {
    let analyzer = state.analyzer.read().await;
    Json(analyzer.tr101290.summary())
}

#[derive(Deserialize)]
pub struct ErrorsQuery {
    pub limit: Option<usize>,
    pub priority: Option<String>,
}

pub async fn get_tr101290_errors(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ErrorsQuery>,
) -> Json<Vec<ts_analyzer::tr101290::Tr101290Error>> {
    let analyzer = state.analyzer.read().await;
    let limit = q.limit.unwrap_or(200);
    let mut errors = analyzer.tr101290.recent_errors(limit);

    if let Some(ref prio) = q.priority {
        let p = match prio.as_str() {
            "p1" | "P1" => Some(ts_analyzer::tr101290::Priority::P1),
            "p2" | "P2" => Some(ts_analyzer::tr101290::Priority::P2),
            "p3" | "P3" => Some(ts_analyzer::tr101290::Priority::P3),
            _ => None,
        };
        if let Some(p) = p {
            errors.retain(|e| e.priority == p);
        }
    }

    Json(errors)
}

use serde::Serialize;
use tokio::sync::{broadcast, RwLock, watch};
use crate::output::session::OutputSessionManager;
use ts_analyzer::system_stats::SystemStatsCollector;

#[derive(Debug, Clone, Serialize)]
pub struct IngestStatus {
    pub running: bool,
    pub url: String,
    pub protocol: String,
    pub packets_received: u64,
}

pub struct IngestSession {
    pub stop_tx: watch::Sender<bool>,
    pub url: String,
    pub protocol: String,
}

pub struct AppState {
    pub analyzer: RwLock<ts_analyzer::StreamAnalyzer>,
    pub ws_tx: broadcast::Sender<String>,
    pub output_manager: RwLock<OutputSessionManager>,
    pub system_stats: RwLock<SystemStatsCollector>,
    pub ingest: RwLock<Option<IngestSession>>,
}

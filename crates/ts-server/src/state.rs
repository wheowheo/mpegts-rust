use tokio::sync::{broadcast, RwLock};
use crate::output::session::OutputSessionManager;
use ts_analyzer::system_stats::SystemStatsCollector;

pub struct AppState {
    pub analyzer: RwLock<ts_analyzer::StreamAnalyzer>,
    pub ws_tx: broadcast::Sender<String>,
    pub output_manager: RwLock<OutputSessionManager>,
    pub system_stats: RwLock<SystemStatsCollector>,
}

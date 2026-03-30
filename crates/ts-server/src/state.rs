use tokio::sync::{broadcast, RwLock};
use crate::output::session::OutputSession;
use ts_analyzer::system_stats::SystemStatsCollector;

pub struct AppState {
    pub analyzer: RwLock<ts_analyzer::StreamAnalyzer>,
    pub ws_tx: broadcast::Sender<String>,
    pub output_session: RwLock<OutputSession>,
    pub system_stats: RwLock<SystemStatsCollector>,
}

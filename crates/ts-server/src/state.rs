use tokio::sync::{broadcast, RwLock};
use crate::output::session::OutputSession;

pub struct AppState {
    pub analyzer: RwLock<ts_analyzer::StreamAnalyzer>,
    pub ws_tx: broadcast::Sender<String>,
    pub output_session: RwLock<OutputSession>,
}

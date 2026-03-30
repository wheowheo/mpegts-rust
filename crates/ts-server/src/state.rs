use tokio::sync::{broadcast, RwLock};

pub struct AppState {
    pub analyzer: RwLock<ts_analyzer::StreamAnalyzer>,
    pub ws_tx: broadcast::Sender<String>,
}

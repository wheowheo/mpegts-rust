use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing_subscriber::EnvFilter;

mod state;

use state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let (tx, _) = broadcast::channel::<String>(256);

    let _state = Arc::new(AppState {
        analyzer: RwLock::new(ts_analyzer::StreamAnalyzer::new()),
        ws_tx: tx,
    });

    tracing::info!("ts-server scaffold ready");
}

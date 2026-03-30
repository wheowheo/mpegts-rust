use std::sync::Arc;
use tokio::sync::watch;
use ts_core::packet::TS_PACKET_SIZE;
use crate::state::AppState;

pub async fn start_http_ingest(
    url: String,
    state: Arc<AppState>,
    mut stop_rx: watch::Receiver<bool>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("HTTP ingest started: {}", url);

    loop {
        tokio::select! {
            result = fetch_and_parse(&url, &state) => {
                if let Err(e) = result {
                    tracing::warn!("HTTP ingest fetch error: {}", e);
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            _ = stop_rx.changed() => {
                if *stop_rx.borrow() {
                    tracing::info!("HTTP ingest stopped");
                    break;
                }
            }
        }
    }

    Ok(())
}

async fn fetch_and_parse(
    url: &str,
    state: &Arc<AppState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let bytes = client.get(url).send().await?.bytes().await?;

    let mut offset = 0;
    let data = bytes.as_ref();

    // find sync
    while offset < data.len() && data[offset] != 0x47 {
        offset += 1;
    }

    let mut analyzer = state.analyzer.write().await;
    while offset + TS_PACKET_SIZE <= data.len() {
        analyzer.feed_packet(&data[offset..offset + TS_PACKET_SIZE]);
        offset += TS_PACKET_SIZE;
    }

    analyzer.sync_pid_bitrates();
    if let Ok(json) = serde_json::to_string(&serde_json::json!({
        "total_bitrate_bps": analyzer.bitrate_stats.latest_total_bitrate(),
        "pcr_jitter_ms": analyzer.pcr_jitter.average_jitter_ms(),
        "total_packets": analyzer.total_packets(),
    })) {
        let _ = state.ws_tx.send(json);
    }

    Ok(())
}

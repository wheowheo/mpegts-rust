use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::sync::watch;
use crate::state::{AppState, IngestSession, IngestStatus};

#[derive(Deserialize)]
pub struct IngestRequest {
    pub url: String,
    pub protocol: Option<String>,
}

pub async fn start_ingest(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IngestRequest>,
) -> Result<Json<IngestStatus>, (StatusCode, String)> {
    // stop existing ingest if any
    {
        let mut ingest = state.ingest.write().await;
        if let Some(session) = ingest.take() {
            let _ = session.stop_tx.send(true);
        }
    }

    // reset analyzer
    {
        let mut analyzer = state.analyzer.write().await;
        *analyzer = ts_analyzer::StreamAnalyzer::new();
        analyzer.set_filename(&req.url);
    }

    let protocol = req.protocol.clone().unwrap_or_else(|| detect_protocol(&req.url));
    let (stop_tx, stop_rx) = watch::channel(false);

    {
        let mut ingest = state.ingest.write().await;
        *ingest = Some(IngestSession {
            stop_tx,
            url: req.url.clone(),
            protocol: protocol.clone(),
        });
    }

    let state_clone = Arc::clone(&state);
    let url = req.url.clone();

    match protocol.as_str() {
        "udp" | "rtp" => {
            let rtp = protocol == "rtp";
            let (addr, port) = parse_udp_url(&url)
                .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
            tokio::spawn(async move {
                if let Err(e) = crate::ingest::udp::start_udp_ingest(addr, port, rtp, state_clone, stop_rx).await {
                    tracing::error!("UDP ingest error: {}", e);
                }
            });
        }
        "http" | "https" => {
            tokio::spawn(async move {
                if let Err(e) = crate::ingest::http::start_http_ingest(url, state_clone, stop_rx).await {
                    tracing::error!("HTTP ingest error: {}", e);
                }
            });
        }
        _ => {
            return Err((StatusCode::BAD_REQUEST, format!("unsupported protocol: {}", protocol)));
        }
    }

    Ok(Json(IngestStatus {
        running: true,
        url: req.url,
        protocol,
        packets_received: 0,
    }))
}

pub async fn stop_ingest(
    State(state): State<Arc<AppState>>,
) -> Json<IngestStatus> {
    let mut ingest = state.ingest.write().await;
    if let Some(session) = ingest.take() {
        let _ = session.stop_tx.send(true);
        let analyzer = state.analyzer.read().await;
        return Json(IngestStatus {
            running: false,
            url: session.url,
            protocol: session.protocol,
            packets_received: analyzer.total_packets(),
        });
    }
    Json(IngestStatus {
        running: false,
        url: String::new(),
        protocol: String::new(),
        packets_received: 0,
    })
}

pub async fn get_ingest_status(
    State(state): State<Arc<AppState>>,
) -> Json<IngestStatus> {
    let ingest = state.ingest.read().await;
    let analyzer = state.analyzer.read().await;

    if let Some(ref session) = *ingest {
        Json(IngestStatus {
            running: true,
            url: session.url.clone(),
            protocol: session.protocol.clone(),
            packets_received: analyzer.total_packets(),
        })
    } else {
        Json(IngestStatus {
            running: false,
            url: String::new(),
            protocol: String::new(),
            packets_received: 0,
        })
    }
}

fn detect_protocol(url: &str) -> String {
    if url.starts_with("rtp://") { "rtp".into() }
    else if url.starts_with("udp://") { "udp".into() }
    else if url.starts_with("http://") || url.starts_with("https://") { "http".into() }
    else { "udp".into() }
}

fn parse_udp_url(url: &str) -> Result<(Ipv4Addr, u16), String> {
    let stripped = url.trim_start_matches("udp://").trim_start_matches("rtp://");
    let parts: Vec<&str> = stripped.split(':').collect();
    if parts.len() != 2 {
        return Err("expected format: udp://addr:port or rtp://addr:port".into());
    }
    let addr: Ipv4Addr = parts[0].parse()
        .map_err(|_| format!("invalid address: {}", parts[0]))?;
    let port: u16 = parts[1].parse()
        .map_err(|_| format!("invalid port: {}", parts[1]))?;
    Ok((addr, port))
}

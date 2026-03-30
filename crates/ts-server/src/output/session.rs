use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::task::JoinHandle;
use ts_core::packet::TS_PACKET_SIZE;

use super::udp::UdpSender;
use super::rtp::RtpSender;
use super::pacer::Pacer;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Udp,
    Rtp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    File,
    Udp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub session_id: Option<String>,
    pub source_type: SourceType,
    pub source_path: Option<String>,
    pub source_addr: Option<String>,
    pub dest_addr: String,
    pub dest_port: u16,
    pub protocol: Protocol,
    pub bitrate_bps: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub level: AlertLevel,
    pub message: String,
    pub timestamp_sec: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertLevel {
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutputStatus {
    pub session_id: String,
    pub running: bool,
    pub config: Option<OutputConfig>,
    pub packets_sent: u64,
    pub bytes_sent: u64,
    pub elapsed_sec: f64,
    pub actual_bitrate_bps: f64,
    pub alerts: Vec<Alert>,
}

const BITRATE_DEVIATION_WARN: f64 = 0.05; // 5%
const BITRATE_DEVIATION_CRIT: f64 = 0.15; // 15%

fn check_alerts(status: &mut OutputStatus) {
    if !status.running {
        return;
    }
    let elapsed = status.elapsed_sec;
    if elapsed < 2.0 {
        return;
    }

    status.alerts.clear();

    if let Some(cfg) = &status.config {
        let target = cfg.bitrate_bps as f64;
        if target > 0.0 {
            let deviation = (status.actual_bitrate_bps - target).abs() / target;
            if deviation > BITRATE_DEVIATION_CRIT {
                status.alerts.push(Alert {
                    level: AlertLevel::Critical,
                    message: format!(
                        "bitrate deviation {:.1}% (target: {:.1} Mbps, actual: {:.1} Mbps)",
                        deviation * 100.0,
                        target / 1_000_000.0,
                        status.actual_bitrate_bps / 1_000_000.0
                    ),
                    timestamp_sec: elapsed,
                });
            } else if deviation > BITRATE_DEVIATION_WARN {
                status.alerts.push(Alert {
                    level: AlertLevel::Warning,
                    message: format!(
                        "bitrate deviation {:.1}%",
                        deviation * 100.0
                    ),
                    timestamp_sec: elapsed,
                });
            }
        }
    }
}

struct SessionInner {
    status: Arc<RwLock<OutputStatus>>,
    task: Option<JoinHandle<()>>,
    stop_tx: Option<broadcast::Sender<()>>,
}

pub struct OutputSessionManager {
    sessions: HashMap<String, SessionInner>,
    next_id: u32,
}

impl OutputSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            next_id: 1,
        }
    }

    pub async fn start(
        &mut self,
        config: OutputConfig,
        _ws_tx: broadcast::Sender<String>,
    ) -> Result<OutputStatus, String> {
        let session_id = config.session_id.clone().unwrap_or_else(|| {
            let id = format!("s{}", self.next_id);
            self.next_id += 1;
            id
        });

        // stop existing session with same id
        if self.sessions.contains_key(&session_id) {
            self.stop(&session_id).await;
        }

        let dest: SocketAddr = format!("{}:{}", config.dest_addr, config.dest_port)
            .parse()
            .map_err(|e| format!("invalid dest address: {e}"))?;

        let ts_data = match config.source_type {
            SourceType::File => {
                let path = config.source_path.as_ref()
                    .ok_or("source_path required for file source")?;
                std::fs::read(path).map_err(|e| format!("failed to read file: {e}"))?
            }
            SourceType::Udp => {
                return Err("UDP relay not yet implemented".into());
            }
        };

        let mut packets: Vec<[u8; TS_PACKET_SIZE]> = Vec::new();
        let mut offset = 0;
        while offset < ts_data.len() {
            if ts_data[offset] == 0x47 { break; }
            offset += 1;
        }
        while offset + TS_PACKET_SIZE <= ts_data.len() {
            let mut pkt = [0u8; TS_PACKET_SIZE];
            pkt.copy_from_slice(&ts_data[offset..offset + TS_PACKET_SIZE]);
            packets.push(pkt);
            offset += TS_PACKET_SIZE;
        }

        if packets.is_empty() {
            return Err("no valid TS packets in source".into());
        }

        let (stop_tx, _) = broadcast::channel(1);
        let mut stop_rx = stop_tx.subscribe();

        let status = Arc::new(RwLock::new(OutputStatus {
            session_id: session_id.clone(),
            running: true,
            config: Some(config),
            packets_sent: 0,
            bytes_sent: 0,
            elapsed_sec: 0.0,
            actual_bitrate_bps: 0.0,
            alerts: Vec::new(),
        }));

        let task_status = status.clone();
        let sid = session_id.clone();
        let bitrate = {
            let s = status.read().await;
            s.config.as_ref().unwrap().bitrate_bps
        };
        let protocol = {
            let s = status.read().await;
            s.config.as_ref().unwrap().protocol.clone()
        };

        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            let mut pacer = Pacer::new(bitrate);
            let burst_size = pacer.packets_per_burst();
            let mut total_sent = 0u64;

            match protocol {
                Protocol::Udp => {
                    let sender = match UdpSender::new(dest).await {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("[{sid}] failed to create UDP sender: {e}");
                            let mut s = task_status.write().await;
                            s.running = false;
                            return;
                        }
                    };

                    loop {
                        for chunk_start in (0..packets.len()).step_by(burst_size) {
                            if stop_rx.try_recv().is_ok() {
                                let mut s = task_status.write().await;
                                s.running = false;
                                return;
                            }

                            pacer.wait_next().await;

                            let chunk_end = (chunk_start + burst_size).min(packets.len());
                            if let Ok(n) = sender.send_packets(&packets[chunk_start..chunk_end]).await {
                                total_sent += n;
                                let elapsed = start.elapsed().as_secs_f64();
                                let bytes = total_sent * TS_PACKET_SIZE as u64;
                                let actual_bps = if elapsed > 0.0 {
                                    bytes as f64 * 8.0 / elapsed
                                } else { 0.0 };

                                let mut s = task_status.write().await;
                                s.packets_sent = total_sent;
                                s.bytes_sent = bytes;
                                s.elapsed_sec = elapsed;
                                s.actual_bitrate_bps = actual_bps;
                                check_alerts(&mut s);
                            }
                        }
                    }
                }
                Protocol::Rtp => {
                    let mut sender = match RtpSender::new(dest).await {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("[{sid}] failed to create RTP sender: {e}");
                            let mut s = task_status.write().await;
                            s.running = false;
                            return;
                        }
                    };

                    loop {
                        for chunk_start in (0..packets.len()).step_by(burst_size) {
                            if stop_rx.try_recv().is_ok() {
                                let mut s = task_status.write().await;
                                s.running = false;
                                return;
                            }

                            pacer.wait_next().await;

                            let chunk_end = (chunk_start + burst_size).min(packets.len());
                            if let Ok(n) = sender.send_packets(&packets[chunk_start..chunk_end]).await {
                                total_sent += n;
                                let elapsed = start.elapsed().as_secs_f64();
                                let bytes = total_sent * TS_PACKET_SIZE as u64;
                                let actual_bps = if elapsed > 0.0 {
                                    bytes as f64 * 8.0 / elapsed
                                } else { 0.0 };

                                let mut s = task_status.write().await;
                                s.packets_sent = total_sent;
                                s.bytes_sent = bytes;
                                s.elapsed_sec = elapsed;
                                s.actual_bitrate_bps = actual_bps;
                                check_alerts(&mut s);
                            }

                            sender.advance_timestamp(pacer.burst_duration_sec());
                        }
                    }
                }
            }
        });

        self.sessions.insert(session_id.clone(), SessionInner {
            status: status.clone(),
            task: Some(handle),
            stop_tx: Some(stop_tx),
        });

        let s = status.read().await;
        Ok(s.clone())
    }

    pub async fn stop(&mut self, session_id: &str) {
        if let Some(mut inner) = self.sessions.remove(session_id) {
            if let Some(tx) = inner.stop_tx.take() {
                let _ = tx.send(());
            }
            if let Some(handle) = inner.task.take() {
                let _ = handle.await;
            }
            let mut s = inner.status.write().await;
            s.running = false;
        }
    }

    pub async fn stop_all(&mut self) {
        let ids: Vec<String> = self.sessions.keys().cloned().collect();
        for id in ids {
            self.stop(&id).await;
        }
    }

    pub async fn get_status(&self, session_id: &str) -> Option<OutputStatus> {
        self.sessions.get(session_id)
            .map(|inner| async { inner.status.read().await.clone() })
            .map(|fut| {
                // blocking: ok since we're in async context
                tokio::runtime::Handle::current().block_on(fut)
            })
    }

    pub async fn list_all(&self) -> Vec<OutputStatus> {
        let mut result = Vec::new();
        for inner in self.sessions.values() {
            result.push(inner.status.read().await.clone());
        }
        result
    }

    #[allow(dead_code)]
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

use serde::{Deserialize, Serialize};
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
    pub source_type: SourceType,
    pub source_path: Option<String>,
    pub source_addr: Option<String>,
    pub dest_addr: String,
    pub dest_port: u16,
    pub protocol: Protocol,
    pub bitrate_bps: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutputStatus {
    pub running: bool,
    pub config: Option<OutputConfig>,
    pub packets_sent: u64,
    pub bytes_sent: u64,
    pub elapsed_sec: f64,
    pub actual_bitrate_bps: f64,
}

pub struct OutputSession {
    pub status: Arc<RwLock<OutputStatus>>,
    task: Option<JoinHandle<()>>,
    stop_tx: Option<broadcast::Sender<()>>,
}

impl OutputSession {
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(OutputStatus {
                running: false,
                config: None,
                packets_sent: 0,
                bytes_sent: 0,
                elapsed_sec: 0.0,
                actual_bitrate_bps: 0.0,
            })),
            task: None,
            stop_tx: None,
        }
    }

    pub async fn start(
        &mut self,
        config: OutputConfig,
        _ws_tx: broadcast::Sender<String>,
    ) -> Result<(), String> {
        if self.task.is_some() {
            self.stop().await;
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

        // extract TS packets from file
        let mut packets: Vec<[u8; TS_PACKET_SIZE]> = Vec::new();
        let mut offset = 0;
        while offset < ts_data.len() {
            if ts_data[offset] == 0x47 {
                break;
            }
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
        self.stop_tx = Some(stop_tx);

        let status = self.status.clone();
        let bitrate = config.bitrate_bps;
        let protocol = config.protocol.clone();

        {
            let mut s = status.write().await;
            s.running = true;
            s.config = Some(config);
            s.packets_sent = 0;
            s.bytes_sent = 0;
            s.elapsed_sec = 0.0;
            s.actual_bitrate_bps = 0.0;
        }

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
                            tracing::error!("failed to create UDP sender: {e}");
                            return;
                        }
                    };

                    loop {
                        for chunk_start in (0..packets.len()).step_by(burst_size) {
                            if stop_rx.try_recv().is_ok() {
                                let mut s = status.write().await;
                                s.running = false;
                                return;
                            }

                            pacer.wait_next().await;

                            let chunk_end = (chunk_start + burst_size).min(packets.len());
                            let chunk = &packets[chunk_start..chunk_end];

                            match sender.send_packets(chunk).await {
                                Ok(n) => {
                                    total_sent += n;
                                    let elapsed = start.elapsed().as_secs_f64();
                                    let bytes = total_sent * TS_PACKET_SIZE as u64;
                                    let actual_bps = if elapsed > 0.0 {
                                        bytes as f64 * 8.0 / elapsed
                                    } else {
                                        0.0
                                    };

                                    let mut s = status.write().await;
                                    s.packets_sent = total_sent;
                                    s.bytes_sent = bytes;
                                    s.elapsed_sec = elapsed;
                                    s.actual_bitrate_bps = actual_bps;
                                }
                                Err(e) => {
                                    tracing::error!("UDP send error: {e}");
                                }
                            }
                        }
                        // loop file
                    }
                }
                Protocol::Rtp => {
                    let mut sender = match RtpSender::new(dest).await {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!("failed to create RTP sender: {e}");
                            return;
                        }
                    };

                    loop {
                        for chunk_start in (0..packets.len()).step_by(burst_size) {
                            if stop_rx.try_recv().is_ok() {
                                let mut s = status.write().await;
                                s.running = false;
                                return;
                            }

                            pacer.wait_next().await;

                            let chunk_end = (chunk_start + burst_size).min(packets.len());
                            let chunk = &packets[chunk_start..chunk_end];

                            match sender.send_packets(chunk).await {
                                Ok(n) => {
                                    total_sent += n;
                                    let elapsed = start.elapsed().as_secs_f64();
                                    let bytes = total_sent * TS_PACKET_SIZE as u64;
                                    let actual_bps = if elapsed > 0.0 {
                                        bytes as f64 * 8.0 / elapsed
                                    } else {
                                        0.0
                                    };

                                    let mut s = status.write().await;
                                    s.packets_sent = total_sent;
                                    s.bytes_sent = bytes;
                                    s.elapsed_sec = elapsed;
                                    s.actual_bitrate_bps = actual_bps;
                                }
                                Err(e) => {
                                    tracing::error!("RTP send error: {e}");
                                }
                            }

                            sender.advance_timestamp(pacer.burst_duration_sec());
                        }
                    }
                }
            }
        });

        self.task = Some(handle);
        Ok(())
    }

    pub async fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.task.take() {
            let _ = handle.await;
        }
        let mut s = self.status.write().await;
        s.running = false;
    }
}

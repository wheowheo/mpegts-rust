use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::watch;
use ts_core::packet::TS_PACKET_SIZE;
use crate::state::AppState;

const UDP_BUF_SIZE: usize = 2048;
const RTP_HEADER_SIZE: usize = 12;

pub async fn start_udp_ingest(
    multicast_addr: Ipv4Addr,
    port: u16,
    rtp: bool,
    state: Arc<AppState>,
    mut stop_rx: watch::Receiver<bool>,
) -> std::io::Result<()> {
    let bind_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
    let socket = UdpSocket::bind(SocketAddr::V4(bind_addr)).await?;

    if multicast_addr.is_multicast() {
        socket.join_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED)?;
        tracing::info!("joined multicast group {}", multicast_addr);
    }

    tracing::info!("UDP ingest started on port {} (rtp={})", port, rtp);

    let mut buf = [0u8; UDP_BUF_SIZE];
    let mut packet_count = 0u64;

    loop {
        tokio::select! {
            result = socket.recv_from(&mut buf) => {
                let (len, _src) = result?;
                let data = if rtp && len > RTP_HEADER_SIZE {
                    strip_rtp_header(&buf[..len])
                } else {
                    &buf[..len]
                };

                // collect packet offsets first without lock
                let mut offsets = Vec::new();
                let mut offset = 0;
                while offset + TS_PACKET_SIZE <= data.len() {
                    if data[offset] == 0x47 {
                        offsets.push(offset);
                        offset += TS_PACKET_SIZE;
                    } else {
                        offset += 1;
                    }
                }

                // acquire lock only for feeding packets
                if !offsets.is_empty() {
                    let mut analyzer = state.analyzer.write().await;
                    for &off in &offsets {
                        analyzer.feed_packet(&data[off..off + TS_PACKET_SIZE]);
                    }
                    packet_count += offsets.len() as u64;

                    if packet_count % 500 == 0 {
                        analyzer.sync_pid_bitrates();
                        let json_data = serde_json::json!({
                            "total_bitrate_bps": analyzer.bitrate_stats.latest_total_bitrate(),
                            "pcr_jitter_ms": analyzer.pcr_jitter.average_jitter_ms(),
                            "total_packets": analyzer.total_packets(),
                        });
                        drop(analyzer); // release lock before send
                        if let Ok(json) = serde_json::to_string(&json_data) {
                            let _ = state.ws_tx.send(json);
                        }
                    }
                }
            }
            _ = stop_rx.changed() => {
                if *stop_rx.borrow() {
                    tracing::info!("UDP ingest stopped");
                    if multicast_addr.is_multicast() {
                        let _ = socket.leave_multicast_v4(multicast_addr, Ipv4Addr::UNSPECIFIED);
                    }
                    break;
                }
            }
        }
    }

    Ok(())
}

fn strip_rtp_header(data: &[u8]) -> &[u8] {
    if data.len() < RTP_HEADER_SIZE { return data; }
    let version = (data[0] >> 6) & 0x03;
    if version != 2 { return data; }
    let cc = (data[0] & 0x0F) as usize;
    let header_len = RTP_HEADER_SIZE + cc * 4;
    let extension = (data[0] >> 4) & 0x01;
    let mut offset = header_len;
    if extension != 0 && offset + 4 <= data.len() {
        let ext_len = ((data[offset + 2] as usize) << 8 | data[offset + 3] as usize) * 4;
        offset += 4 + ext_len;
    }
    if offset < data.len() { &data[offset..] } else { &[] }
}

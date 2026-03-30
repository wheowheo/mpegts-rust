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

    loop {
        tokio::select! {
            result = socket.recv_from(&mut buf) => {
                let (len, _src) = result?;
                let data = if rtp && len > RTP_HEADER_SIZE {
                    strip_rtp_header(&buf[..len])
                } else {
                    &buf[..len]
                };

                let mut offset = 0;
                let mut analyzer = state.analyzer.write().await;

                while offset + TS_PACKET_SIZE <= data.len() {
                    if data[offset] == 0x47 {
                        analyzer.feed_packet(&data[offset..offset + TS_PACKET_SIZE]);
                        offset += TS_PACKET_SIZE;
                    } else {
                        offset += 1;
                    }
                }

                if analyzer.total_packets() % 500 == 0 {
                    analyzer.sync_pid_bitrates();
                    if let Ok(json) = serde_json::to_string(&serde_json::json!({
                        "total_bitrate_bps": analyzer.bitrate_stats.latest_total_bitrate(),
                        "pcr_jitter_ms": analyzer.pcr_jitter.average_jitter_ms(),
                        "total_packets": analyzer.total_packets(),
                    })) {
                        let _ = state.ws_tx.send(json);
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

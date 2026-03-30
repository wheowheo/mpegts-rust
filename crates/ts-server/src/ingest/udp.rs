use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use ts_core::packet::TS_PACKET_SIZE;
use crate::state::AppState;

const UDP_BUF_SIZE: usize = 1500; // typical MTU

pub async fn start_udp_listener(
    addr: SocketAddr,
    state: Arc<AppState>,
) -> std::io::Result<()> {
    let socket = UdpSocket::bind(addr).await?;
    tracing::info!("UDP listener started on {}", addr);

    let mut buf = [0u8; UDP_BUF_SIZE];

    loop {
        let (len, _src) = socket.recv_from(&mut buf).await?;

        // TS 패킷은 보통 7개씩 UDP 패킷에 담김 (7 * 188 = 1316)
        let mut offset = 0;
        let mut analyzer = state.analyzer.write().await;

        while offset + TS_PACKET_SIZE <= len {
            if buf[offset] == 0x47 {
                analyzer.feed_packet(&buf[offset..offset + TS_PACKET_SIZE]);
                offset += TS_PACKET_SIZE;
            } else {
                offset += 1;
            }
        }

        // 주기적으로 WebSocket push
        if analyzer.total_packets() % 1000 == 0 {
            if let Ok(json) = serde_json::to_string(&serde_json::json!({
                "total_bitrate_bps": analyzer.bitrate_stats.latest_total_bitrate(),
                "pcr_jitter_ms": analyzer.pcr_jitter.average_jitter_ms(),
                "total_packets": analyzer.total_packets(),
            })) {
                let _ = state.ws_tx.send(json);
            }
        }
    }
}

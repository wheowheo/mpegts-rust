use std::net::SocketAddr;
use tokio::net::UdpSocket;
use ts_core::packet::TS_PACKET_SIZE;

const TS_PER_DATAGRAM: usize = 7; // 7 * 188 = 1316 bytes

pub struct UdpSender {
    socket: UdpSocket,
    dest: SocketAddr,
}

impl UdpSender {
    pub async fn new(dest: SocketAddr) -> std::io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;

        // multicast인 경우 TTL 설정
        if dest.ip().is_multicast() {
            socket.set_multicast_loop_v4(false)?;
            socket.set_ttl(16)?;
        }

        Ok(Self { socket, dest })
    }

    pub async fn send_packets(&self, packets: &[[u8; TS_PACKET_SIZE]]) -> std::io::Result<u64> {
        let mut sent = 0u64;
        let mut i = 0;

        while i < packets.len() {
            let end = (i + TS_PER_DATAGRAM).min(packets.len());
            let mut buf = Vec::with_capacity((end - i) * TS_PACKET_SIZE);
            for pkt in &packets[i..end] {
                buf.extend_from_slice(pkt);
            }
            self.socket.send_to(&buf, self.dest).await?;
            sent += (end - i) as u64;
            i = end;
        }

        Ok(sent)
    }
}

use std::net::SocketAddr;
use tokio::net::UdpSocket;
use ts_core::packet::TS_PACKET_SIZE;

const TS_PER_RTP: usize = 7;
const RTP_HEADER_SIZE: usize = 12;
const RTP_PAYLOAD_TYPE: u8 = 33; // MPEG-TS over RTP (RFC 2250)
const RTP_CLOCK_RATE: u64 = 90_000;

pub struct RtpSender {
    socket: UdpSocket,
    dest: SocketAddr,
    ssrc: u32,
    sequence: u16,
    timestamp: u32,
}

impl RtpSender {
    pub async fn new(dest: SocketAddr) -> std::io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;

        if dest.ip().is_multicast() {
            socket.set_multicast_loop_v4(false)?;
            socket.set_ttl(16)?;
        }

        let ssrc = rand::random::<u32>();

        Ok(Self {
            socket,
            dest,
            ssrc,
            sequence: 0,
            timestamp: 0,
        })
    }

    pub async fn send_packets(&mut self, packets: &[[u8; TS_PACKET_SIZE]]) -> std::io::Result<u64> {
        let mut sent = 0u64;
        let mut i = 0;

        while i < packets.len() {
            let end = (i + TS_PER_RTP).min(packets.len());
            let ts_count = end - i;
            let payload_size = ts_count * TS_PACKET_SIZE;

            let mut buf = Vec::with_capacity(RTP_HEADER_SIZE + payload_size);

            // RTP header (12 bytes)
            buf.push(0x80); // V=2, P=0, X=0, CC=0
            buf.push(RTP_PAYLOAD_TYPE); // M=0, PT=33
            buf.push((self.sequence >> 8) as u8);
            buf.push((self.sequence & 0xFF) as u8);
            buf.push((self.timestamp >> 24) as u8);
            buf.push((self.timestamp >> 16) as u8);
            buf.push((self.timestamp >> 8) as u8);
            buf.push((self.timestamp & 0xFF) as u8);
            buf.push((self.ssrc >> 24) as u8);
            buf.push((self.ssrc >> 16) as u8);
            buf.push((self.ssrc >> 8) as u8);
            buf.push((self.ssrc & 0xFF) as u8);

            for pkt in &packets[i..end] {
                buf.extend_from_slice(pkt);
            }

            self.socket.send_to(&buf, self.dest).await?;
            self.sequence = self.sequence.wrapping_add(1);
            sent += ts_count as u64;
            i = end;
        }

        Ok(sent)
    }

    pub fn advance_timestamp(&mut self, duration_sec: f64) {
        let ticks = (duration_sec * RTP_CLOCK_RATE as f64) as u32;
        self.timestamp = self.timestamp.wrapping_add(ticks);
    }
}

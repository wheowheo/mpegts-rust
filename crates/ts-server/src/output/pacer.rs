use std::time::{Duration, Instant};
use ts_core::packet::TS_PACKET_SIZE;

pub struct Pacer {
    interval: Duration,
    packets_per_burst: usize,
    last_send: Option<Instant>,
}

impl Pacer {
    pub fn new(target_bitrate_bps: u64) -> Self {
        // 7 packets per UDP datagram, send at intervals to match CBR
        let packets_per_burst = 7usize;
        let bits_per_burst = packets_per_burst as u64 * TS_PACKET_SIZE as u64 * 8;
        let interval_ns = bits_per_burst as f64 / target_bitrate_bps as f64 * 1_000_000_000.0;

        Self {
            interval: Duration::from_nanos(interval_ns as u64),
            packets_per_burst,
            last_send: None,
        }
    }

    pub fn packets_per_burst(&self) -> usize {
        self.packets_per_burst
    }

    pub async fn wait_next(&mut self) {
        match self.last_send {
            Some(last) => {
                let elapsed = last.elapsed();
                if elapsed < self.interval {
                    tokio::time::sleep(self.interval - elapsed).await;
                }
            }
            None => {}
        }
        self.last_send = Some(Instant::now());
    }

    pub fn burst_duration_sec(&self) -> f64 {
        self.interval.as_secs_f64()
    }
}

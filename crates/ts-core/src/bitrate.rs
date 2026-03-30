use crate::timing::PCR_CLOCK_RATE;

/// PCR 기반 비트레이트 계산기
#[derive(Debug)]
pub struct BitrateCalculator {
    first_pcr: Option<u64>,
    last_pcr: Option<u64>,
    first_packet_index: u64,
    last_packet_index: u64,
    total_bytes: u64,
}

impl BitrateCalculator {
    pub fn new() -> Self {
        Self {
            first_pcr: None,
            last_pcr: None,
            first_packet_index: 0,
            last_packet_index: 0,
            total_bytes: 0,
        }
    }

    pub fn update_pcr(&mut self, pcr: u64, packet_index: u64) {
        if self.first_pcr.is_none() {
            self.first_pcr = Some(pcr);
            self.first_packet_index = packet_index;
        }
        self.last_pcr = Some(pcr);
        self.last_packet_index = packet_index;
    }

    pub fn add_bytes(&mut self, bytes: u64) {
        self.total_bytes += bytes;
    }

    /// 전체 비트레이트 (bps) 계산
    pub fn bitrate_bps(&self) -> Option<f64> {
        let first = self.first_pcr?;
        let last = self.last_pcr?;
        if last <= first {
            return None;
        }

        let duration_sec = (last - first) as f64 / PCR_CLOCK_RATE as f64;
        if duration_sec <= 0.0 {
            return None;
        }

        Some(self.total_bytes as f64 * 8.0 / duration_sec)
    }

    /// 패킷 수 기반 비트레이트 추정 (PCR 없을 때 fallback)
    pub fn estimated_bitrate_bps(&self, packet_size: u64) -> Option<f64> {
        let first = self.first_pcr?;
        let last = self.last_pcr?;
        if last <= first || self.last_packet_index <= self.first_packet_index {
            return None;
        }

        let duration_sec = (last - first) as f64 / PCR_CLOCK_RATE as f64;
        let packets = self.last_packet_index - self.first_packet_index;
        Some(packets as f64 * packet_size as f64 * 8.0 / duration_sec)
    }
}

impl Default for BitrateCalculator {
    fn default() -> Self {
        Self::new()
    }
}

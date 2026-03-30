use serde::Serialize;

pub const PCR_CLOCK_RATE: u64 = 27_000_000; // 27 MHz
pub const PTS_CLOCK_RATE: u64 = 90_000; // 90 kHz

#[derive(Debug, Clone, Serialize)]
pub struct PcrInfo {
    pub pid: u16,
    pub pcr_base: u64,
    pub pcr_ext: u16,
    pub pcr_value: u64, // 27MHz 단위 전체 값
    pub packet_index: u64,
}

impl PcrInfo {
    pub fn from_raw(pid: u16, raw_pcr: u64, packet_index: u64) -> Self {
        let pcr_base = raw_pcr / 300;
        let pcr_ext = (raw_pcr % 300) as u16;
        Self {
            pid,
            pcr_base,
            pcr_ext,
            pcr_value: raw_pcr,
            packet_index,
        }
    }

    pub fn to_seconds(&self) -> f64 {
        self.pcr_value as f64 / PCR_CLOCK_RATE as f64
    }
}

/// PTS/DTS를 초 단위로 변환
pub fn pts_to_seconds(pts: u64) -> f64 {
    pts as f64 / PTS_CLOCK_RATE as f64
}

/// 두 PCR 값의 차이를 밀리초로 계산 (wrap-around 고려)
pub fn pcr_diff_ms(pcr1: u64, pcr2: u64) -> f64 {
    let max_pcr: u64 = (1 << 33) * 300 + 299; // 최대 PCR 값
    let diff = if pcr2 >= pcr1 {
        pcr2 - pcr1
    } else {
        max_pcr - pcr1 + pcr2
    };
    (diff as f64 / PCR_CLOCK_RATE as f64) * 1000.0
}

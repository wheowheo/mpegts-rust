use serde::Serialize;
use ts_core::timing::{PcrInfo, PCR_CLOCK_RATE};

#[derive(Debug, Clone, Serialize)]
pub struct PcrJitterSample {
    pub packet_index: u64,
    pub pcr_seconds: f64,
    pub jitter_ms: f64,
    pub bitrate_bps: f64,
}

#[derive(Debug)]
pub struct PcrJitterAnalyzer {
    samples: Vec<PcrJitterSample>,
    last_pcr: Option<PcrInfo>,
    last_bytes: u64,
    current_bytes: u64,
}

impl PcrJitterAnalyzer {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            last_pcr: None,
            last_bytes: 0,
            current_bytes: 0,
        }
    }

    pub fn add_bytes(&mut self, bytes: u64) {
        self.current_bytes += bytes;
    }

    pub fn update(&mut self, pcr: PcrInfo) {
        if let Some(ref prev) = self.last_pcr {
            if pcr.pcr_value > prev.pcr_value {
                let pcr_diff = pcr.pcr_value - prev.pcr_value;
                let expected_interval_ms =
                    (pcr_diff as f64 / PCR_CLOCK_RATE as f64) * 1000.0;

                // 패킷 간격 기반 예상 PCR 간격
                let byte_diff = self.current_bytes - self.last_bytes;

                // jitter = 실제 PCR 간격과 이전 샘플 대비 편차
                let jitter_ms = if self.samples.len() >= 2 {
                    let prev_interval = {
                        let s = &self.samples;
                        let len = s.len();
                        (s[len - 1].pcr_seconds - s[len - 2].pcr_seconds) * 1000.0
                    };
                    (expected_interval_ms - prev_interval).abs()
                } else {
                    0.0
                };

                let bitrate_bps = if pcr_diff > 0 {
                    byte_diff as f64 * 8.0 / (pcr_diff as f64 / PCR_CLOCK_RATE as f64)
                } else {
                    0.0
                };

                self.samples.push(PcrJitterSample {
                    packet_index: pcr.packet_index,
                    pcr_seconds: pcr.to_seconds(),
                    jitter_ms,
                    bitrate_bps,
                });
            }
        }

        self.last_bytes = self.current_bytes;
        self.last_pcr = Some(pcr);
    }

    pub fn samples(&self) -> &[PcrJitterSample] {
        &self.samples
    }

    pub fn average_jitter_ms(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.samples.iter().map(|s| s.jitter_ms).sum();
        sum / self.samples.len() as f64
    }

    pub fn max_jitter_ms(&self) -> f64 {
        self.samples
            .iter()
            .map(|s| s.jitter_ms)
            .fold(0.0, f64::max)
    }

    pub fn recent_samples(&self, count: usize) -> &[PcrJitterSample] {
        let start = self.samples.len().saturating_sub(count);
        &self.samples[start..]
    }
}

impl Default for PcrJitterAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

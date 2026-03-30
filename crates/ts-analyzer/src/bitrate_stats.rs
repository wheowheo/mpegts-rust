use serde::Serialize;
use std::collections::HashMap;
use ts_core::timing::PCR_CLOCK_RATE;

#[derive(Debug, Clone, Serialize)]
pub struct BitrateSample {
    pub timestamp_sec: f64,
    pub bitrate_bps: f64,
}

#[derive(Debug)]
struct PidAccumulator {
    packet_count: u64,
    last_pcr: Option<u64>,
    last_count_at_pcr: u64,
}

#[derive(Debug)]
pub struct BitrateStats {
    per_pid: HashMap<u16, Vec<BitrateSample>>,
    total_samples: Vec<BitrateSample>,
    accumulators: HashMap<u16, PidAccumulator>,
    global_packet_count: u64,
    global_last_pcr: Option<u64>,
    global_last_count: u64,
    packet_size: u64,
}

impl BitrateStats {
    pub fn new(packet_size: u64) -> Self {
        Self {
            per_pid: HashMap::new(),
            total_samples: Vec::new(),
            accumulators: HashMap::new(),
            global_packet_count: 0,
            global_last_pcr: None,
            global_last_count: 0,
            packet_size,
        }
    }

    pub fn count_packet(&mut self, pid: u16) {
        self.global_packet_count += 1;

        let acc = self.accumulators.entry(pid).or_insert(PidAccumulator {
            packet_count: 0,
            last_pcr: None,
            last_count_at_pcr: 0,
        });
        acc.packet_count += 1;
    }

    pub fn update_pcr(&mut self, pcr: u64, pcr_pid: u16, timestamp_sec: f64) {
        // 전체 비트레이트 계산
        if let Some(last_pcr) = self.global_last_pcr {
            if pcr > last_pcr {
                let duration = (pcr - last_pcr) as f64 / PCR_CLOCK_RATE as f64;
                if duration > 0.0 {
                    let packets = self.global_packet_count - self.global_last_count;
                    let bps = packets as f64 * self.packet_size as f64 * 8.0 / duration;
                    self.total_samples.push(BitrateSample {
                        timestamp_sec,
                        bitrate_bps: bps,
                    });
                }
            }
        }
        self.global_last_pcr = Some(pcr);
        self.global_last_count = self.global_packet_count;

        // PID별 비트레이트 계산
        for (&pid, acc) in &mut self.accumulators {
            if let Some(last_pcr) = acc.last_pcr {
                if pcr > last_pcr {
                    let duration = (pcr - last_pcr) as f64 / PCR_CLOCK_RATE as f64;
                    if duration > 0.0 {
                        let packets = acc.packet_count - acc.last_count_at_pcr;
                        let bps = packets as f64 * self.packet_size as f64 * 8.0 / duration;
                        self.per_pid.entry(pid).or_default().push(BitrateSample {
                            timestamp_sec,
                            bitrate_bps: bps,
                        });
                    }
                }
            }
            acc.last_pcr = Some(pcr);
            acc.last_count_at_pcr = acc.packet_count;
        }

        // PCR PID 자체도 업데이트
        if let Some(acc) = self.accumulators.get_mut(&pcr_pid) {
            acc.last_pcr = Some(pcr);
            acc.last_count_at_pcr = acc.packet_count;
        }
    }

    pub fn total_bitrate_samples(&self) -> &[BitrateSample] {
        &self.total_samples
    }

    pub fn pid_bitrate_samples(&self, pid: u16) -> Option<&[BitrateSample]> {
        self.per_pid.get(&pid).map(|v| v.as_slice())
    }

    pub fn latest_total_bitrate(&self) -> Option<f64> {
        self.total_samples.last().map(|s| s.bitrate_bps)
    }
}

impl Default for BitrateStats {
    fn default() -> Self {
        Self::new(188)
    }
}

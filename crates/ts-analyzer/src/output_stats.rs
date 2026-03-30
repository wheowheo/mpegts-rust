use serde::Serialize;
use std::collections::VecDeque;
use std::time::Instant;

const MAX_SAMPLES: usize = 600; // 10분 @ 1sample/sec

#[derive(Debug, Clone, Serialize)]
pub struct OutputSample {
    pub timestamp_sec: f64,
    pub actual_bitrate_bps: f64,
    pub target_bitrate_bps: f64,
    pub inter_packet_jitter_us: f64,
    pub pcr_drift_us: f64,
    pub buffer_occupancy_pct: f64,
}

#[derive(Debug)]
pub struct OutputStatsCollector {
    samples: VecDeque<OutputSample>,
    start: Option<Instant>,

    // jitter tracking
    last_send_time: Option<Instant>,
    expected_interval: Option<std::time::Duration>,
    jitter_acc: f64,
    jitter_count: u64,

    // PCR drift
    first_pcr: Option<(u64, Instant)>,
    last_pcr_drift_us: f64,

    // bitrate
    target_bitrate: u64,
    bytes_in_window: u64,
    window_start: Option<Instant>,
}

impl OutputStatsCollector {
    pub fn new(target_bitrate: u64) -> Self {
        Self {
            samples: VecDeque::with_capacity(MAX_SAMPLES),
            start: None,
            last_send_time: None,
            expected_interval: None,
            jitter_acc: 0.0,
            jitter_count: 0,
            first_pcr: None,
            last_pcr_drift_us: 0.0,
            target_bitrate,
            bytes_in_window: 0,
            window_start: None,
        }
    }

    pub fn set_expected_interval(&mut self, interval: std::time::Duration) {
        self.expected_interval = Some(interval);
    }

    pub fn record_send(&mut self, bytes: u64) {
        let now = Instant::now();

        if self.start.is_none() {
            self.start = Some(now);
        }

        // jitter calculation
        if let (Some(last), Some(expected)) = (self.last_send_time, self.expected_interval) {
            let actual = now.duration_since(last);
            let diff = if actual > expected {
                actual - expected
            } else {
                expected - actual
            };
            self.jitter_acc += diff.as_micros() as f64;
            self.jitter_count += 1;
        }
        self.last_send_time = Some(now);

        // bitrate window
        if self.window_start.is_none() {
            self.window_start = Some(now);
        }
        self.bytes_in_window += bytes;
    }

    pub fn record_pcr(&mut self, pcr_value: u64) {
        let now = Instant::now();
        match self.first_pcr {
            None => {
                self.first_pcr = Some((pcr_value, now));
            }
            Some((first_pcr, first_time)) => {
                let pcr_elapsed_us = (pcr_value - first_pcr) as f64 / 27.0; // 27MHz → us
                let wall_elapsed_us = now.duration_since(first_time).as_micros() as f64;
                self.last_pcr_drift_us = pcr_elapsed_us - wall_elapsed_us;
            }
        }
    }

    pub fn snapshot(&mut self, buffer_occupancy_pct: f64) -> Option<OutputSample> {
        let now = Instant::now();
        let start = self.start?;
        let window_start = self.window_start?;

        let window_dur = now.duration_since(window_start).as_secs_f64();
        if window_dur < 0.5 {
            return None;
        }

        let actual_bitrate = self.bytes_in_window as f64 * 8.0 / window_dur;
        let avg_jitter = if self.jitter_count > 0 {
            self.jitter_acc / self.jitter_count as f64
        } else {
            0.0
        };

        let sample = OutputSample {
            timestamp_sec: now.duration_since(start).as_secs_f64(),
            actual_bitrate_bps: actual_bitrate,
            target_bitrate_bps: self.target_bitrate as f64,
            inter_packet_jitter_us: avg_jitter,
            pcr_drift_us: self.last_pcr_drift_us,
            buffer_occupancy_pct,
        };

        // reset window
        self.bytes_in_window = 0;
        self.window_start = Some(now);
        self.jitter_acc = 0.0;
        self.jitter_count = 0;

        if self.samples.len() >= MAX_SAMPLES {
            self.samples.pop_front();
        }
        self.samples.push_back(sample.clone());

        Some(sample)
    }

    pub fn samples(&self) -> &VecDeque<OutputSample> {
        &self.samples
    }

    pub fn latest(&self) -> Option<&OutputSample> {
        self.samples.back()
    }
}

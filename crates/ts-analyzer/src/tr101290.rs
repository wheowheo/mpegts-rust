use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Priority {
    P1,
    P2,
    P3,
}

#[derive(Debug, Clone, Serialize)]
pub struct Tr101290Error {
    pub priority: Priority,
    pub error_type: String,
    pub description: String,
    pub pid: Option<u16>,
    pub packet_index: u64,
    pub timestamp_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Tr101290Summary {
    pub p1_count: u64,
    pub p2_count: u64,
    pub p3_count: u64,
    pub errors: Vec<Tr101290Error>,
    pub p1_counters: HashMap<String, u64>,
    pub p2_counters: HashMap<String, u64>,
    pub p3_counters: HashMap<String, u64>,
}

pub struct Tr101290Checker {
    errors: Vec<Tr101290Error>,
    p1_counters: HashMap<String, u64>,
    p2_counters: HashMap<String, u64>,
    p3_counters: HashMap<String, u64>,

    // P1 state
    sync_count: u32,
    sync_loss: bool,
    last_pat_index: Option<u64>,
    last_pmt_indices: HashMap<u16, u64>,
    known_pids: HashMap<u16, u64>, // pid -> last seen index
    cc_state: HashMap<u16, u8>,

    // P2 state
    last_pcr_indices: HashMap<u16, u64>,
    last_pcr_values: HashMap<u16, u64>,
    pcr_prev_prev: HashMap<u16, u64>,

    // P3 state
    last_nit_index: Option<u64>,
    last_sdt_index: Option<u64>,
    pat_pids: Vec<u16>,
    pmt_pids: Vec<u16>,

    pcr_seconds: f64,
    packet_rate: f64, // packets per second estimate
}

impl Tr101290Checker {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            p1_counters: HashMap::new(),
            p2_counters: HashMap::new(),
            p3_counters: HashMap::new(),
            sync_count: 0,
            sync_loss: false,
            last_pat_index: None,
            last_pmt_indices: HashMap::new(),
            known_pids: HashMap::new(),
            cc_state: HashMap::new(),
            last_pcr_indices: HashMap::new(),
            last_pcr_values: HashMap::new(),
            pcr_prev_prev: HashMap::new(),
            last_nit_index: None,
            last_sdt_index: None,
            pat_pids: Vec::new(),
            pmt_pids: Vec::new(),
            pcr_seconds: 0.0,
            packet_rate: 0.0,
        }
    }

    pub fn set_pmt_pids(&mut self, pids: &[u16]) {
        self.pmt_pids = pids.to_vec();
    }

    pub fn set_pat_pids(&mut self, pids: &[u16]) {
        self.pat_pids = pids.to_vec();
    }

    pub fn update_pcr_time(&mut self, seconds: f64) {
        self.pcr_seconds = seconds;
    }

    pub fn set_packet_rate(&mut self, rate: f64) {
        self.packet_rate = rate;
    }

    fn time_ms(&self) -> f64 {
        self.pcr_seconds * 1000.0
    }

    fn index_to_ms(&self, idx: u64) -> f64 {
        if self.packet_rate > 0.0 {
            idx as f64 / self.packet_rate * 1000.0
        } else {
            0.0
        }
    }

    pub fn check_sync(&mut self, sync_byte: u8, packet_index: u64) {
        if sync_byte == 0x47 {
            self.sync_count += 1;
            if self.sync_loss && self.sync_count >= 5 {
                self.sync_loss = false;
            }
        } else {
            self.add_p1("Sync byte error", &format!("got 0x{:02X}", sync_byte), None, packet_index);
            self.sync_count = 0;
            if !self.sync_loss {
                self.sync_loss = true;
                self.add_p1("TS sync loss", "consecutive sync failures", None, packet_index);
            }
        }
    }

    pub fn check_tei(&mut self, tei: bool, pid: u16, packet_index: u64) {
        if tei {
            self.add_p2("Transport error", &format!("TEI bit set on PID 0x{:04X}", pid), Some(pid), packet_index);
        }
    }

    pub fn check_pat(&mut self, pid: u16, is_pat: bool, packet_index: u64) {
        if is_pat {
            if let Some(last) = self.last_pat_index {
                let interval = self.index_to_ms(packet_index - last);
                if interval > 500.0 && self.packet_rate > 0.0 {
                    self.add_p1("PAT error", &format!("interval {:.0}ms > 500ms", interval), Some(0), packet_index);
                }
            }
            self.last_pat_index = Some(packet_index);
        }

        // check PAT timeout
        if pid == 0x0000 || packet_index % 10000 == 0 {
            if let Some(last) = self.last_pat_index {
                let interval = self.index_to_ms(packet_index - last);
                if interval > 500.0 && self.packet_rate > 0.0 && packet_index > 1000 {
                    // only report once per detection
                }
            } else if packet_index > 5000 && self.packet_rate > 0.0 {
                self.add_p1("PAT error", "no PAT received", None, packet_index);
                self.last_pat_index = Some(packet_index); // prevent repeated reports
            }
        }
    }

    pub fn check_cc(&mut self, pid: u16, cc: u8, has_payload: bool, packet_index: u64) {
        if pid == 0x1FFF || !has_payload { return; }

        if let Some(&last_cc) = self.cc_state.get(&pid) {
            let expected = (last_cc + 1) & 0x0F;
            if cc != expected {
                self.add_p1("CC error",
                    &format!("PID 0x{:04X}: expected {} got {}", pid, expected, cc),
                    Some(pid), packet_index);
            }
        }
        self.cc_state.insert(pid, cc);
    }

    pub fn check_pmt(&mut self, pid: u16, packet_index: u64) {
        if self.pmt_pids.contains(&pid) {
            self.last_pmt_indices.insert(pid, packet_index);
        }
    }

    pub fn check_pid_presence(&mut self, pid: u16, packet_index: u64) {
        self.known_pids.insert(pid, packet_index);
    }

    pub fn check_pid_timeout(&mut self, packet_index: u64) {
        if self.packet_rate <= 0.0 { return; }
        let threshold = (self.packet_rate * 5.0) as u64; // 5 second timeout
        let stale: Vec<u16> = self.known_pids.iter()
            .filter(|(_, &last)| packet_index - last > threshold && packet_index > threshold)
            .map(|(&pid, _)| pid)
            .collect();
        for pid in stale {
            self.add_p1("PID error", &format!("PID 0x{:04X} lost", pid), Some(pid), packet_index);
            self.known_pids.remove(&pid);
        }
    }

    pub fn check_pcr(&mut self, pid: u16, pcr_value: u64, packet_index: u64) {
        if let Some(&last_pcr) = self.last_pcr_values.get(&pid) {
            // PCR interval in seconds (using PCR values directly - 27MHz clock)
            let pcr_interval_ms = pcr_value.wrapping_sub(last_pcr) as f64 / 27_000.0;

            // P2: PCR repetition interval > 100ms (TR 101 290 spec says 40ms, relaxed for real streams)
            if pcr_interval_ms > 100.0 && pcr_interval_ms < 10_000.0 {
                self.add_p2("PCR repetition error",
                    &format!("PID 0x{:04X}: interval {:.1}ms > 100ms", pid, pcr_interval_ms),
                    Some(pid), packet_index);
            }

            // P2: PCR jitter - check if interval is suspiciously irregular
            // Compare with expected interval based on previous intervals
            if let Some(&prev_prev_pcr) = self.pcr_prev_prev.get(&pid) {
                let prev_interval = last_pcr.wrapping_sub(prev_prev_pcr) as f64 / 27_000.0;
                if prev_interval > 0.0 && prev_interval < 10_000.0 {
                    let jitter = (pcr_interval_ms - prev_interval).abs();
                    if jitter > 50.0 { // 50ms jitter threshold
                        self.add_p2("PCR accuracy error",
                            &format!("PID 0x{:04X}: jitter {:.2}ms", pid, jitter),
                            Some(pid), packet_index);
                    }
                }
            }
        }

        // track prev-prev for jitter calculation
        if let Some(&last_pcr) = self.last_pcr_values.get(&pid) {
            self.pcr_prev_prev.insert(pid, last_pcr);
        }
        self.last_pcr_indices.insert(pid, packet_index);
        self.last_pcr_values.insert(pid, pcr_value);
    }

    pub fn check_pcr_timeout(&mut self, _packet_index: u64) {
        // PCR timeout is now handled in check_pcr via interval measurement
    }

    pub fn check_nit(&mut self, is_nit: bool, packet_index: u64) {
        if is_nit {
            self.last_nit_index = Some(packet_index);
        }
    }

    pub fn check_sdt(&mut self, is_sdt: bool, packet_index: u64) {
        if is_sdt {
            self.last_sdt_index = Some(packet_index);
        }
    }

    pub fn check_unreferenced_pid(&mut self, pid: u16, _packet_index: u64) {
        if pid == 0x1FFF || pid == 0x0000 || pid == 0x0001 || pid == 0x0010 || pid == 0x0011 || pid == 0x0012 {
            return;
        }
        if !self.pmt_pids.contains(&pid) && !self.pat_pids.contains(&pid) {
            // check if this PID is referenced by any PMT
            // for now, just flag
        }
    }

    pub fn check_p3_intervals(&mut self, packet_index: u64) {
        if self.packet_rate <= 0.0 || packet_index < 10000 { return; }

        if let Some(last) = self.last_nit_index {
            let interval = self.index_to_ms(packet_index - last);
            if interval > 10000.0 {
                self.add_p3("NIT error", "NIT interval > 10s", None, packet_index);
                self.last_nit_index = Some(packet_index);
            }
        } else if packet_index > 50000 {
            self.add_p3("NIT error", "no NIT received", None, packet_index);
            self.last_nit_index = Some(packet_index);
        }

        if let Some(last) = self.last_sdt_index {
            let interval = self.index_to_ms(packet_index - last);
            if interval > 2000.0 {
                self.add_p3("SDT error", &format!("SDT interval {:.0}ms > 2s", interval), None, packet_index);
                self.last_sdt_index = Some(packet_index);
            }
        } else if packet_index > 50000 {
            self.add_p3("SDT error", "no SDT received", None, packet_index);
            self.last_sdt_index = Some(packet_index);
        }
    }

    pub fn check_crc_error(&mut self, pid: u16, packet_index: u64) {
        self.add_p2("CRC error", &format!("PSI CRC mismatch on PID 0x{:04X}", pid), Some(pid), packet_index);
    }

    fn add_p1(&mut self, error_type: &str, desc: &str, pid: Option<u16>, idx: u64) {
        *self.p1_counters.entry(error_type.to_string()).or_insert(0) += 1;
        self.errors.push(Tr101290Error {
            priority: Priority::P1,
            error_type: error_type.into(),
            description: desc.into(),
            pid,
            packet_index: idx,
            timestamp_ms: self.time_ms(),
        });
    }

    fn add_p2(&mut self, error_type: &str, desc: &str, pid: Option<u16>, idx: u64) {
        *self.p2_counters.entry(error_type.to_string()).or_insert(0) += 1;
        self.errors.push(Tr101290Error {
            priority: Priority::P2,
            error_type: error_type.into(),
            description: desc.into(),
            pid,
            packet_index: idx,
            timestamp_ms: self.time_ms(),
        });
    }

    fn add_p3(&mut self, error_type: &str, desc: &str, pid: Option<u16>, idx: u64) {
        *self.p3_counters.entry(error_type.to_string()).or_insert(0) += 1;
        self.errors.push(Tr101290Error {
            priority: Priority::P3,
            error_type: error_type.into(),
            description: desc.into(),
            pid,
            packet_index: idx,
            timestamp_ms: self.time_ms(),
        });
    }

    pub fn summary(&self) -> Tr101290Summary {
        let p1_count = self.p1_counters.values().sum();
        let p2_count = self.p2_counters.values().sum();
        let p3_count = self.p3_counters.values().sum();

        Tr101290Summary {
            p1_count,
            p2_count,
            p3_count,
            errors: self.errors.clone(),
            p1_counters: self.p1_counters.clone(),
            p2_counters: self.p2_counters.clone(),
            p3_counters: self.p3_counters.clone(),
        }
    }

    pub fn recent_errors(&self, limit: usize) -> Vec<Tr101290Error> {
        let start = self.errors.len().saturating_sub(limit);
        self.errors[start..].to_vec()
    }
}

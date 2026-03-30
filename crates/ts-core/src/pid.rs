use serde::Serialize;
use std::collections::HashMap;

pub const PID_PAT: u16 = 0x0000;
pub const PID_CAT: u16 = 0x0001;
pub const PID_TSDT: u16 = 0x0002;
pub const PID_NIT: u16 = 0x0010;
pub const PID_SDT: u16 = 0x0011;
pub const PID_EIT: u16 = 0x0012;
pub const PID_NULL: u16 = 0x1FFF;

#[derive(Debug, Clone, Serialize)]
pub struct PidInfo {
    pub pid: u16,
    pub label: String,
    pub stream_type: Option<u8>,
    pub packet_count: u64,
    pub cc_errors: u64,
    pub last_cc: u8,
    pub bitrate_bps: f64,
    pub has_pcr: bool,
    pub scrambled: bool,
}

#[derive(Debug, Default)]
pub struct PidMap {
    pub pids: HashMap<u16, PidInfo>,
    pub total_packets: u64,
}

impl PidMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, pid: u16, cc: u8, scrambled: bool, has_pcr: bool) {
        self.total_packets += 1;

        let info = self.pids.entry(pid).or_insert_with(|| PidInfo {
            pid,
            label: Self::default_label(pid),
            stream_type: None,
            packet_count: 0,
            cc_errors: 0,
            last_cc: cc.wrapping_sub(1) & 0x0F,
            bitrate_bps: 0.0,
            has_pcr: false,
            scrambled: false,
        });

        info.packet_count += 1;
        info.scrambled = scrambled;
        if has_pcr {
            info.has_pcr = true;
        }

        // Null packets (0x1FFF) don't carry meaningful CC
        if pid != PID_NULL {
            let expected = (info.last_cc + 1) & 0x0F;
            if cc != expected && info.packet_count > 1 {
                info.cc_errors += 1;
            }
        }
        info.last_cc = cc;
    }

    fn default_label(pid: u16) -> String {
        match pid {
            PID_PAT => "PAT".into(),
            PID_CAT => "CAT".into(),
            PID_TSDT => "TSDT".into(),
            PID_NIT => "NIT/ST".into(),
            PID_SDT => "SDT/BAT/ST".into(),
            PID_EIT => "EIT/ST".into(),
            PID_NULL => "Null".into(),
            0x0003..=0x000F => "Reserved".into(),
            _ => format!("PID 0x{:04X}", pid),
        }
    }
}

use serde::Serialize;
use ts_core::psi::pat::Pat;
use ts_core::psi::pmt::Pmt;

#[derive(Debug, Clone, Serialize)]
pub struct StreamInfo {
    pub filename: String,
    pub duration_ms: Option<f64>,
    pub total_packets: u64,
    pub total_bitrate_bps: f64,
    pub programs: Vec<ProgramInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgramInfo {
    pub program_number: u16,
    pub pmt_pid: u16,
    pub streams: Vec<ElementaryStreamInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ElementaryStreamInfo {
    pub pid: u16,
    pub stream_type: u8,
    pub stream_type_name: String,
    pub codec: Option<String>,
}

impl StreamInfo {
    pub fn from_pat_pmt(
        filename: &str,
        pat: &Pat,
        pmts: &[(u16, Pmt)],
        total_packets: u64,
        bitrate_bps: f64,
        duration_ms: Option<f64>,
    ) -> Self {
        let programs = pat
            .entries
            .iter()
            .filter(|e| e.program_number != 0)
            .map(|entry| {
                let streams = pmts
                    .iter()
                    .find(|(pid, _)| *pid == entry.pid)
                    .map(|(_, pmt)| {
                        pmt.streams
                            .iter()
                            .map(|s| ElementaryStreamInfo {
                                pid: s.elementary_pid,
                                stream_type: s.stream_type,
                                stream_type_name: Pmt::stream_type_name(s.stream_type).to_string(),
                                codec: Self::guess_codec(s.stream_type),
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                ProgramInfo {
                    program_number: entry.program_number,
                    pmt_pid: entry.pid,
                    streams,
                }
            })
            .collect();

        StreamInfo {
            filename: filename.to_string(),
            duration_ms,
            total_packets,
            total_bitrate_bps: bitrate_bps,
            programs,
        }
    }

    fn guess_codec(stream_type: u8) -> Option<String> {
        match stream_type {
            0x01 | 0x02 => Some("MPEG-2".into()),
            0x03 | 0x04 => Some("MPEG Audio".into()),
            0x0F => Some("AAC".into()),
            0x11 => Some("AAC-LATM".into()),
            0x1B => Some("H.264".into()),
            0x24 => Some("H.265".into()),
            0x81 => Some("AC-3".into()),
            0x87 => Some("E-AC-3".into()),
            _ => None,
        }
    }
}

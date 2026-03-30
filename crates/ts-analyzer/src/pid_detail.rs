use serde::Serialize;
use ts_core::descriptors::{parse_descriptors, parse_descriptor_detail, ParsedDescriptor};
use ts_core::pes::PesHeader;
use ts_core::timing::PCR_CLOCK_RATE;

const MAX_PCR_SAMPLES: usize = 200;
const MAX_PES_SAMPLES: usize = 50;
const MAX_CC_ERRORS: usize = 100;

#[derive(Debug, Clone, Serialize)]
pub struct PcrSample {
    pub packet_index: u64,
    pub pcr_base: u64,
    pub pcr_ext: u16,
    pub pcr_value: u64,
    pub pcr_seconds: f64,
    pub interval_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PesSample {
    pub packet_index: u64,
    pub stream_id: u8,
    pub stream_id_name: String,
    pub packet_length: u16,
    pub pts: Option<f64>,
    pub dts: Option<f64>,
    pub pts_raw: Option<u64>,
    pub dts_raw: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdaptationStats {
    pub total_with_af: u64,
    pub discontinuity_count: u64,
    pub random_access_count: u64,
    pub pcr_count: u64,
    pub opcr_count: u64,
    pub splice_countdown_count: u64,
    pub af_only_count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransportStats {
    pub total_packets: u64,
    pub payload_unit_start_count: u64,
    pub transport_error_count: u64,
    pub transport_priority_count: u64,
    pub scrambled_even_count: u64,
    pub scrambled_odd_count: u64,
    pub adaptation_field_control: [u64; 4],
}

#[derive(Debug, Clone, Serialize)]
pub struct CcErrorDetail {
    pub packet_index: u64,
    pub expected: u8,
    pub got: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct PidDetailData {
    pub pid: u16,
    pub label: String,
    pub stream_type: Option<u8>,
    pub stream_type_name: Option<String>,

    pub transport: TransportStats,
    pub adaptation: AdaptationStats,

    pub pcr_samples: Vec<PcrSample>,
    pub pcr_bitrate_bps: Option<f64>,
    pub pcr_jitter_avg_ms: Option<f64>,
    pub pcr_jitter_max_ms: Option<f64>,

    pub pes_samples: Vec<PesSample>,

    pub descriptors: Vec<ParsedDescriptor>,

    pub cc_errors: Vec<CcErrorDetail>,
    pub cc_error_rate: f64,

    pub bitrate_bps: f64,
    pub percentage: f64,

    pub first_packet_index: u64,
    pub last_packet_index: u64,
}

#[derive(Debug)]
pub struct PidDetailCollector {
    pub pid: u16,

    pub transport: TransportStats,
    pub adaptation: AdaptationStats,

    pcr_samples: Vec<PcrSample>,
    pes_samples: Vec<PesSample>,
    cc_errors: Vec<CcErrorDetail>,
    raw_descriptors: Vec<u8>,
    descriptors_parsed: bool,

    first_packet: Option<u64>,
    last_packet: Option<u64>,
    last_pcr: Option<u64>,
}

impl PidDetailCollector {
    pub fn new(pid: u16) -> Self {
        Self {
            pid,
            transport: TransportStats {
                total_packets: 0,
                payload_unit_start_count: 0,
                transport_error_count: 0,
                transport_priority_count: 0,
                scrambled_even_count: 0,
                scrambled_odd_count: 0,
                adaptation_field_control: [0; 4],
            },
            adaptation: AdaptationStats {
                total_with_af: 0,
                discontinuity_count: 0,
                random_access_count: 0,
                pcr_count: 0,
                opcr_count: 0,
                splice_countdown_count: 0,
                af_only_count: 0,
            },
            pcr_samples: Vec::new(),
            pes_samples: Vec::new(),
            cc_errors: Vec::new(),
            raw_descriptors: Vec::new(),
            descriptors_parsed: false,
            first_packet: None,
            last_packet: None,
            last_pcr: None,
        }
    }

    pub fn feed_header(&mut self, pkt: &ts_core::packet::TsPacket, packet_index: u64) {
        let h = &pkt.header;
        self.transport.total_packets += 1;
        if h.payload_unit_start { self.transport.payload_unit_start_count += 1; }
        if h.transport_error { self.transport.transport_error_count += 1; }
        if h.transport_priority { self.transport.transport_priority_count += 1; }
        match h.scrambling_control {
            2 => self.transport.scrambled_even_count += 1,
            3 => self.transport.scrambled_odd_count += 1,
            _ => {}
        }
        let afc = (h.adaptation_field_control & 0x03) as usize;
        self.transport.adaptation_field_control[afc] += 1;

        if self.first_packet.is_none() { self.first_packet = Some(packet_index); }
        self.last_packet = Some(packet_index);

        if let Some(ref af) = pkt.adaptation {
            self.adaptation.total_with_af += 1;
            if af.discontinuity { self.adaptation.discontinuity_count += 1; }
            if af.random_access { self.adaptation.random_access_count += 1; }
            if af.splice_countdown.is_some() { self.adaptation.splice_countdown_count += 1; }
            if af.opcr.is_some() { self.adaptation.opcr_count += 1; }
            if h.adaptation_field_control == 2 { self.adaptation.af_only_count += 1; }

            if let Some(pcr) = af.pcr {
                self.adaptation.pcr_count += 1;
                let pcr_base = pcr / 300;
                let pcr_ext = (pcr % 300) as u16;
                let pcr_seconds = pcr as f64 / PCR_CLOCK_RATE as f64;

                let interval_ms = self.last_pcr.map(|last| {
                    if pcr > last {
                        (pcr - last) as f64 / PCR_CLOCK_RATE as f64 * 1000.0
                    } else { 0.0 }
                });

                if self.pcr_samples.len() < MAX_PCR_SAMPLES {
                    self.pcr_samples.push(PcrSample {
                        packet_index,
                        pcr_base,
                        pcr_ext,
                        pcr_value: pcr,
                        pcr_seconds,
                        interval_ms,
                    });
                }
                self.last_pcr = Some(pcr);
            }
        }
    }

    pub fn feed_pes_header(&mut self, header: &PesHeader, packet_index: u64) {
        if self.pes_samples.len() >= MAX_PES_SAMPLES { return; }
        let name = stream_id_name(header.stream_id);
        self.pes_samples.push(PesSample {
            packet_index,
            stream_id: header.stream_id,
            stream_id_name: name.to_string(),
            packet_length: header.packet_length,
            pts: header.pts.map(|v| v as f64 / 90_000.0),
            dts: header.dts.map(|v| v as f64 / 90_000.0),
            pts_raw: header.pts,
            dts_raw: header.dts,
        });
    }

    pub fn feed_cc_error(&mut self, packet_index: u64, expected: u8, got: u8) {
        if self.cc_errors.len() < MAX_CC_ERRORS {
            self.cc_errors.push(CcErrorDetail { packet_index, expected, got });
        }
    }

    pub fn set_descriptors(&mut self, raw: &[u8]) {
        if !self.descriptors_parsed {
            self.raw_descriptors = raw.to_vec();
            self.descriptors_parsed = true;
        }
    }

    pub fn build_detail(&self, bitrate_bps: f64, percentage: f64, label: &str, stream_type: Option<u8>) -> PidDetailData {
        let descriptors = parse_descriptors(&self.raw_descriptors)
            .iter().map(|d| parse_descriptor_detail(d)).collect();

        let pcr_jitter = self.compute_pcr_jitter();

        PidDetailData {
            pid: self.pid,
            label: label.to_string(),
            stream_type,
            stream_type_name: stream_type.map(|st| ts_core::psi::pmt::Pmt::stream_type_name(st).to_string()),
            transport: self.transport.clone(),
            adaptation: self.adaptation.clone(),
            pcr_samples: self.pcr_samples.clone(),
            pcr_bitrate_bps: self.compute_pcr_bitrate(),
            pcr_jitter_avg_ms: pcr_jitter.0,
            pcr_jitter_max_ms: pcr_jitter.1,
            pes_samples: self.pes_samples.clone(),
            descriptors,
            cc_errors: self.cc_errors.clone(),
            cc_error_rate: if self.transport.total_packets > 0 {
                self.cc_errors.len() as f64 / self.transport.total_packets as f64
            } else { 0.0 },
            bitrate_bps,
            percentage,
            first_packet_index: self.first_packet.unwrap_or(0),
            last_packet_index: self.last_packet.unwrap_or(0),
        }
    }

    fn compute_pcr_bitrate(&self) -> Option<f64> {
        if self.pcr_samples.len() < 2 { return None; }
        let first = &self.pcr_samples[0];
        let last = self.pcr_samples.last().unwrap();
        let duration = last.pcr_seconds - first.pcr_seconds;
        if duration <= 0.0 { return None; }
        let packets = last.packet_index - first.packet_index;
        Some(packets as f64 * 188.0 * 8.0 / duration)
    }

    fn compute_pcr_jitter(&self) -> (Option<f64>, Option<f64>) {
        let intervals: Vec<f64> = self.pcr_samples.iter()
            .filter_map(|s| s.interval_ms)
            .collect();
        if intervals.len() < 2 { return (None, None); }
        let avg = intervals.iter().sum::<f64>() / intervals.len() as f64;
        let jitters: Vec<f64> = intervals.iter().map(|&v| (v - avg).abs()).collect();
        let avg_jitter = jitters.iter().sum::<f64>() / jitters.len() as f64;
        let max_jitter = jitters.iter().cloned().fold(0.0f64, f64::max);
        (Some(avg_jitter), Some(max_jitter))
    }
}

fn stream_id_name(id: u8) -> &'static str {
    match id {
        0xBC => "program_stream_map",
        0xBD => "private_stream_1",
        0xBE => "padding_stream",
        0xBF => "private_stream_2",
        0xC0..=0xDF => "audio_stream",
        0xE0..=0xEF => "video_stream",
        0xF0 => "ECM_stream",
        0xF1 => "EMM_stream",
        0xF2 => "DSMCC_stream",
        0xF3 => "ISO_13522_stream",
        0xF4 => "ITU-T_Rec_H.222.1_A",
        0xF5 => "ITU-T_Rec_H.222.1_B",
        0xF6 => "ITU-T_Rec_H.222.1_C",
        0xF7 => "ITU-T_Rec_H.222.1_D",
        0xF8 => "ITU-T_Rec_H.222.1_E",
        0xF9 => "ancillary_stream",
        0xFA => "SL_packetized_stream",
        0xFB => "FlexMux_stream",
        0xFC => "metadata_stream",
        0xFD => "extended_stream_id",
        0xFE => "reserved",
        0xFF => "program_stream_directory",
        _ => "unknown",
    }
}

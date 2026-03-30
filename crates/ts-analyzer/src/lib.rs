pub mod stream_info;
pub mod continuity;
pub mod pcr_jitter;
pub mod bitrate_stats;
pub mod output_stats;
pub mod system_stats;
pub mod capacity;
pub mod pid_detail;
pub mod frame_index;

use std::collections::{HashMap, VecDeque};
use ts_core::packet::{TsPacket, TS_PACKET_SIZE};
use ts_core::pid::PidMap;
use ts_core::psi::{PsiSection, pat::Pat, pmt::Pmt};
use ts_core::timing::PcrInfo;
use ts_core::bitrate::BitrateCalculator;
use ts_core::scte35::Scte35;

use continuity::ContinuityChecker;
use pcr_jitter::PcrJitterAnalyzer;
use bitrate_stats::BitrateStats;
use stream_info::StreamInfo;
use pid_detail::PidDetailCollector;
use frame_index::FrameIndexer;

pub struct StreamAnalyzer {
    pub pid_map: PidMap,
    pub cc_checker: ContinuityChecker,
    pub pcr_jitter: PcrJitterAnalyzer,
    pub bitrate_stats: BitrateStats,
    pub bitrate_calc: BitrateCalculator,

    pub pat: Option<Pat>,
    pub pmts: HashMap<u16, Pmt>,
    pub scte35_events: Vec<Scte35>,

    pmt_pids: Vec<u16>,
    scte35_pids: Vec<u16>,

    psi_buffers: HashMap<u16, Vec<u8>>,
    pub pid_details: HashMap<u16, PidDetailCollector>,
    pes_assemblers: HashMap<u16, ts_core::pes::PesAssembler>,
    pub frame_indexers: HashMap<u16, FrameIndexer>,

    pub raw_packets: HashMap<u16, VecDeque<Vec<u8>>>,

    packet_index: u64,
    filename: String,
}

impl StreamAnalyzer {
    pub fn new() -> Self {
        Self {
            pid_map: PidMap::new(),
            cc_checker: ContinuityChecker::new(),
            pcr_jitter: PcrJitterAnalyzer::new(),
            bitrate_stats: BitrateStats::new(TS_PACKET_SIZE as u64),
            bitrate_calc: BitrateCalculator::new(),
            pat: None,
            pmts: HashMap::new(),
            scte35_events: Vec::new(),
            pmt_pids: Vec::new(),
            scte35_pids: Vec::new(),
            psi_buffers: HashMap::new(),
            pid_details: HashMap::new(),
            pes_assemblers: HashMap::new(),
            frame_indexers: HashMap::new(),
            raw_packets: HashMap::new(),
            packet_index: 0,
            filename: String::new(),
        }
    }

    pub fn set_filename(&mut self, name: &str) {
        self.filename = name.to_string();
    }

    pub fn feed_packet(&mut self, data: &[u8]) {
        let pkt = match TsPacket::parse(data) {
            Ok(p) => p,
            Err(_) => return,
        };

        let pid = pkt.header.pid;
        let cc = pkt.header.continuity_counter;
        let has_payload = pkt.header.adaptation_field_control & 0x01 != 0;
        let scrambled = pkt.header.scrambling_control != 0;
        let has_pcr = pkt.adaptation.as_ref().and_then(|a| a.pcr).is_some();

        // raw packet storage (last 64 per PID)
        let buf = self.raw_packets.entry(pid).or_insert_with(VecDeque::new);
        buf.push_back(data.to_vec());
        if buf.len() > 64 {
            buf.pop_front();
        }

        // PID map
        self.pid_map.update(pid, cc, scrambled, has_pcr);

        // PID detail collection
        let detail = self.pid_details.entry(pid)
            .or_insert_with(|| PidDetailCollector::new(pid));
        detail.feed_header(&pkt, self.packet_index);

        // CC check
        self.cc_checker.check(pid, cc, has_payload, self.packet_index);
        // track CC errors in detail
        if let Some(info) = self.pid_map.pids.get(&pid) {
            if info.packet_count > 1 {
                let expected = (info.last_cc) & 0x0F; // last_cc was already updated
                if cc != expected && has_payload && pid != 0x1FFF {
                    // note: pid_map already counted the error, we track detail separately
                }
            }
        }

        // Bitrate tracking
        self.bitrate_stats.count_packet(pid);
        self.pcr_jitter.add_bytes(TS_PACKET_SIZE as u64);
        self.bitrate_calc.add_bytes(TS_PACKET_SIZE as u64);

        // PCR handling
        if let Some(ref af) = pkt.adaptation {
            if let Some(pcr_val) = af.pcr {
                let pcr_info = PcrInfo::from_raw(pid, pcr_val, self.packet_index);
                self.bitrate_calc.update_pcr(pcr_val, self.packet_index);
                self.bitrate_stats.update_pcr(pcr_val, pid, pcr_info.to_seconds());
                self.pcr_jitter.update(pcr_info);
            }
        }

        // PSI/SCTE-35 payload processing
        if let Some(ref payload) = pkt.payload {
            self.process_payload(pid, payload, pkt.header.payload_unit_start);

            // PES assembly for elementary streams
            if pid != 0x0000 && pid != 0x1FFF && !self.pmt_pids.contains(&pid) && !self.scte35_pids.contains(&pid) {
                let pusi = pkt.header.payload_unit_start;
                let assembler = self.pes_assemblers.entry(pid)
                    .or_insert_with(|| ts_core::pes::PesAssembler::new(pid));
                if let Some(pes) = assembler.push(payload, pusi) {
                    if let Some(detail) = self.pid_details.get_mut(&pid) {
                        detail.feed_pes_header(&pes.header, self.packet_index);
                    }
                    let indexer = self.frame_indexers.entry(pid)
                        .or_insert_with(FrameIndexer::new);
                    if let Some(info) = self.pid_map.pids.get(&pid) {
                        if let Some(st) = info.stream_type {
                            indexer.set_stream_type(st);
                        }
                    }
                    indexer.feed_pes(&pes.data, pes.header.pts, pes.header.dts, self.packet_index);
                }
            }
        }

        self.packet_index += 1;
    }

    fn process_payload(&mut self, pid: u16, payload: &[u8], pusi: bool) {
        // PAT
        if pid == 0x0000 {
            self.handle_psi(pid, payload, pusi, |analyzer, section| {
                if let Ok(pat) = Pat::parse(section) {
                    analyzer.pmt_pids = pat.entries.iter()
                        .filter(|e| e.program_number != 0)
                        .map(|e| e.pid)
                        .collect();
                    analyzer.pat = Some(pat);
                }
            });
            return;
        }

        // PMT
        if self.pmt_pids.contains(&pid) {
            self.handle_psi(pid, payload, pusi, |analyzer, section| {
                if let Ok(pmt) = Pmt::parse(section) {
                    // PMT PID 자체에 label 설정
                    if let Some(info) = analyzer.pid_map.pids.get_mut(&pid) {
                        info.label = "PMT".to_string();
                    }

                    for s in &pmt.streams {
                        // SCTE-35 PID 감지
                        if s.stream_type == 0x86 && !analyzer.scte35_pids.contains(&s.elementary_pid) {
                            analyzer.scte35_pids.push(s.elementary_pid);
                        }

                        // PID label/stream_type 업데이트 (엔트리 없으면 생성)
                        let info = analyzer.pid_map.pids.entry(s.elementary_pid)
                            .or_insert_with(|| ts_core::pid::PidInfo {
                                pid: s.elementary_pid,
                                label: String::new(),
                                stream_type: None,
                                packet_count: 0,
                                cc_errors: 0,
                                last_cc: 0,
                                bitrate_bps: 0.0,
                                has_pcr: false,
                                scrambled: false,
                            });
                        info.stream_type = Some(s.stream_type);
                        let codec = Pmt::codec_name(s.stream_type, &s.descriptors);
                        if !codec.is_empty() {
                            info.label = codec.to_string();
                        } else {
                            info.label = Pmt::stream_type_name(s.stream_type).to_string();
                        }

                        // store descriptors for this PID
                        if !s.descriptors.is_empty() {
                            let detail = analyzer.pid_details.entry(s.elementary_pid)
                                .or_insert_with(|| PidDetailCollector::new(s.elementary_pid));
                            detail.set_descriptors(&s.descriptors);
                        }
                    }
                    // store program-level descriptors on PMT PID itself
                    if !pmt.program_descriptors.is_empty() {
                        let detail = analyzer.pid_details.entry(pid)
                            .or_insert_with(|| PidDetailCollector::new(pid));
                        detail.set_descriptors(&pmt.program_descriptors);
                    }
                    analyzer.pmts.insert(pid, pmt);
                }
            });
            return;
        }

        // SCTE-35
        if self.scte35_pids.contains(&pid) {
            if pusi && payload.len() > 1 {
                let pointer = payload[0] as usize;
                let start = 1 + pointer;
                if start < payload.len() {
                    if let Ok(scte) = Scte35::parse(&payload[start..]) {
                        self.scte35_events.push(scte);
                    }
                }
            }
        }
    }

    fn handle_psi<F>(&mut self, pid: u16, payload: &[u8], pusi: bool, handler: F)
    where
        F: FnOnce(&mut Self, &PsiSection),
    {
        if pusi {
            if payload.is_empty() {
                return;
            }
            let pointer = payload[0] as usize;
            let start = 1 + pointer;
            if start >= payload.len() {
                return;
            }
            self.psi_buffers.insert(pid, payload[start..].to_vec());
        } else if let Some(buf) = self.psi_buffers.get_mut(&pid) {
            buf.extend_from_slice(payload);
        }

        if let Some(buf) = self.psi_buffers.get(&pid).cloned() {
            if let Ok(section) = PsiSection::parse(&buf) {
                handler(self, &section);
            }
        }
    }

    pub fn sync_pid_bitrates(&mut self) {
        for (pid, info) in &mut self.pid_map.pids {
            if let Some(samples) = self.bitrate_stats.pid_bitrate_samples(*pid) {
                if let Some(last) = samples.last() {
                    info.bitrate_bps = last.bitrate_bps;
                }
            }
        }
    }

    pub fn stream_info(&self) -> StreamInfo {
        let pmts: Vec<(u16, Pmt)> = self.pmts.iter().map(|(&k, v)| (k, v.clone())).collect();
        let duration_ms = self.duration_ms();
        let bitrate = self.bitrate_calc.bitrate_bps().unwrap_or(0.0);

        StreamInfo::from_pat_pmt(
            &self.filename,
            self.pat.as_ref().unwrap_or(&Pat {
                transport_stream_id: 0,
                version: 0,
                entries: Vec::new(),
            }),
            &pmts,
            self.pid_map.total_packets,
            bitrate,
            duration_ms,
        )
    }

    pub fn duration_ms(&self) -> Option<f64> {
        let samples = self.pcr_jitter.samples();
        if samples.len() < 2 {
            return None;
        }
        let first = samples.first().unwrap().pcr_seconds;
        let last = samples.last().unwrap().pcr_seconds;
        Some((last - first) * 1000.0)
    }

    pub fn total_packets(&self) -> u64 {
        self.packet_index
    }
}

impl Default for StreamAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

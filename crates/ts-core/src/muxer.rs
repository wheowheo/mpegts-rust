use crate::packet::TS_PACKET_SIZE;
use crate::packet_builder::{TsPacketBuilder, build_null_packet};
use crate::psi_builder::{PatBuilder, PmtBuilder};
use crate::timing::PCR_CLOCK_RATE;

const PCR_INTERVAL_MS: u64 = 40; // PCR every 40ms (spec: max 100ms)
const PAT_INTERVAL_MS: u64 = 500;

pub struct MuxerStream {
    pub pid: u16,
    pub stream_type: u8,
    pub is_pcr: bool,
    data: Vec<u8>,
    read_pos: usize,
    cc: u8,
}

impl MuxerStream {
    pub fn new(pid: u16, stream_type: u8, is_pcr: bool) -> Self {
        Self {
            pid,
            stream_type,
            is_pcr,
            data: Vec::new(),
            read_pos: 0,
            cc: 0,
        }
    }

    pub fn push_data(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    pub fn has_data(&self) -> bool {
        self.read_pos < self.data.len()
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.read_pos
    }

    fn next_cc(&mut self) -> u8 {
        let cc = self.cc;
        self.cc = (self.cc + 1) & 0x0F;
        cc
    }

    fn consume(&mut self, n: usize) -> &[u8] {
        let end = (self.read_pos + n).min(self.data.len());
        let slice = &self.data[self.read_pos..end];
        self.read_pos = end;
        slice
    }
}

pub struct TsMuxer {
    streams: Vec<MuxerStream>,
    pat_builder: PatBuilder,
    pmt_builder: PmtBuilder,
    pmt_pid: u16,
    program_number: u16,
    pcr_pid: u16,

    target_bitrate_bps: u64,
    pcr_base: u64,
    packet_count: u64,
    pat_cc: u8,
    pmt_cc: u8,

    last_pcr_packet: u64,
    last_pat_packet: u64,
}

impl TsMuxer {
    pub fn new(program_number: u16, pmt_pid: u16, pcr_pid: u16, target_bitrate_bps: u64) -> Self {
        Self {
            streams: Vec::new(),
            pat_builder: PatBuilder::new(1).add_program(program_number, pmt_pid),
            pmt_builder: PmtBuilder::new(program_number, pcr_pid),
            pmt_pid,
            program_number,
            pcr_pid,
            target_bitrate_bps,
            pcr_base: 0,
            packet_count: 0,
            pat_cc: 0,
            pmt_cc: 0,
            last_pcr_packet: 0,
            last_pat_packet: 0,
        }
    }

    pub fn add_stream(&mut self, pid: u16, stream_type: u8) {
        let is_pcr = pid == self.pcr_pid;
        self.streams.push(MuxerStream::new(pid, stream_type, is_pcr));
        self.pmt_builder = PmtBuilder::new(self.program_number, self.pcr_pid);
        for s in &self.streams {
            self.pmt_builder.streams.push((s.stream_type, s.pid));
        }
    }

    pub fn push_data(&mut self, pid: u16, data: &[u8]) {
        for stream in &mut self.streams {
            if stream.pid == pid {
                stream.push_data(data);
                return;
            }
        }
    }

    pub fn mux_packets(&mut self, output: &mut Vec<[u8; TS_PACKET_SIZE]>) {
        let packets_per_sec = self.target_bitrate_bps / (TS_PACKET_SIZE as u64 * 8);
        let pcr_interval_pkts = (packets_per_sec * PCR_INTERVAL_MS / 1000).max(1);
        let pat_interval_pkts = (packets_per_sec * PAT_INTERVAL_MS / 1000).max(1);

        // PAT/PMT insertion
        if self.packet_count == 0 || self.packet_count - self.last_pat_packet >= pat_interval_pkts {
            output.push(self.pat_builder.build_packet(self.pat_cc));
            self.pat_cc = (self.pat_cc + 1) & 0x0F;
            self.packet_count += 1;

            output.push(self.pmt_builder.build_packet(self.pmt_pid, self.pmt_cc));
            self.pmt_cc = (self.pmt_cc + 1) & 0x0F;
            self.packet_count += 1;

            self.last_pat_packet = self.packet_count;
        }

        // data packets
        for i in 0..self.streams.len() {
            while self.streams[i].has_data() {
                let need_pcr = self.streams[i].is_pcr
                    && (self.packet_count == 0
                        || self.packet_count - self.last_pcr_packet >= pcr_interval_pkts);

                let cc = self.streams[i].next_cc();
                let pid = self.streams[i].pid;

                if need_pcr {
                    let pcr = self.current_pcr();
                    let builder = TsPacketBuilder::new(pid, cc).with_pcr(pcr);
                    let cap = builder.payload_capacity();
                    let data = self.streams[i].consume(cap).to_vec();
                    output.push(builder.payload(&data).build());
                    self.last_pcr_packet = self.packet_count;
                } else {
                    let cap = TS_PACKET_SIZE - 4; // 184 bytes payload
                    let data = self.streams[i].consume(cap).to_vec();
                    output.push(TsPacketBuilder::new(pid, cc).payload(&data).build());
                }

                self.packet_count += 1;
                self.pcr_base += PCR_CLOCK_RATE * TS_PACKET_SIZE as u64 * 8
                    / self.target_bitrate_bps;
            }
        }
    }

    pub fn stuff_to_cbr(&mut self, output: &mut Vec<[u8; TS_PACKET_SIZE]>, target_packets: usize) {
        while output.len() < target_packets {
            output.push(build_null_packet());
            self.packet_count += 1;
            self.pcr_base +=
                PCR_CLOCK_RATE * TS_PACKET_SIZE as u64 * 8 / self.target_bitrate_bps;
        }
    }

    fn current_pcr(&self) -> u64 {
        self.pcr_base
    }

    pub fn packet_count(&self) -> u64 {
        self.packet_count
    }
}

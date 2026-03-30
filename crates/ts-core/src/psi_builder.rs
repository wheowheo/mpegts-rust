use crate::crc32::crc32_mpeg2;
use crate::packet_builder::TsPacketBuilder;
use crate::packet::TS_PACKET_SIZE;

pub struct PatBuilder {
    pub transport_stream_id: u16,
    pub version: u8,
    pub entries: Vec<(u16, u16)>, // (program_number, pmt_pid)
}

impl PatBuilder {
    pub fn new(tsid: u16) -> Self {
        Self {
            transport_stream_id: tsid,
            version: 0,
            entries: Vec::new(),
        }
    }

    pub fn add_program(mut self, program_number: u16, pmt_pid: u16) -> Self {
        self.entries.push((program_number, pmt_pid));
        self
    }

    pub fn build_section(&self) -> Vec<u8> {
        let data_len = self.entries.len() * 4;
        let section_length = (5 + data_len + 4) as u16; // header(5) + data + crc(4)

        let mut section = Vec::with_capacity(3 + section_length as usize);
        section.push(0x00); // table_id = PAT
        section.push(0xB0 | ((section_length >> 8) & 0x0F) as u8);
        section.push((section_length & 0xFF) as u8);
        section.push((self.transport_stream_id >> 8) as u8);
        section.push((self.transport_stream_id & 0xFF) as u8);
        section.push(0xC1 | ((self.version & 0x1F) << 1));
        section.push(0x00); // section_number
        section.push(0x00); // last_section_number

        for &(pn, pid) in &self.entries {
            section.push((pn >> 8) as u8);
            section.push((pn & 0xFF) as u8);
            section.push(0xE0 | ((pid >> 8) & 0x1F) as u8);
            section.push((pid & 0xFF) as u8);
        }

        let crc = crc32_mpeg2(&section);
        section.push((crc >> 24) as u8);
        section.push((crc >> 16) as u8);
        section.push((crc >> 8) as u8);
        section.push((crc & 0xFF) as u8);

        section
    }

    pub fn build_packet(&self, cc: u8) -> [u8; TS_PACKET_SIZE] {
        let section = self.build_section();
        let mut payload = vec![0x00]; // pointer_field
        payload.extend_from_slice(&section);
        TsPacketBuilder::new(0x0000, cc)
            .pusi()
            .payload(&payload)
            .build()
    }
}

pub struct PmtBuilder {
    pub program_number: u16,
    pub version: u8,
    pub pcr_pid: u16,
    pub streams: Vec<(u8, u16)>, // (stream_type, elementary_pid)
}

impl PmtBuilder {
    pub fn new(program_number: u16, pcr_pid: u16) -> Self {
        Self {
            program_number,
            version: 0,
            pcr_pid,
            streams: Vec::new(),
        }
    }

    pub fn add_stream(mut self, stream_type: u8, pid: u16) -> Self {
        self.streams.push((stream_type, pid));
        self
    }

    pub fn build_section(&self) -> Vec<u8> {
        let data_len = 4 + self.streams.len() * 5; // pcr_pid(2) + prog_info_len(2) + streams
        let section_length = (5 + data_len + 4) as u16;

        let mut section = Vec::with_capacity(3 + section_length as usize);
        section.push(0x02); // table_id = PMT
        section.push(0xB0 | ((section_length >> 8) & 0x0F) as u8);
        section.push((section_length & 0xFF) as u8);
        section.push((self.program_number >> 8) as u8);
        section.push((self.program_number & 0xFF) as u8);
        section.push(0xC1 | ((self.version & 0x1F) << 1));
        section.push(0x00);
        section.push(0x00);

        section.push(0xE0 | ((self.pcr_pid >> 8) & 0x1F) as u8);
        section.push((self.pcr_pid & 0xFF) as u8);
        section.push(0xF0);
        section.push(0x00); // program_info_length = 0

        for &(st, pid) in &self.streams {
            section.push(st);
            section.push(0xE0 | ((pid >> 8) & 0x1F) as u8);
            section.push((pid & 0xFF) as u8);
            section.push(0xF0);
            section.push(0x00); // ES_info_length = 0
        }

        let crc = crc32_mpeg2(&section);
        section.push((crc >> 24) as u8);
        section.push((crc >> 16) as u8);
        section.push((crc >> 8) as u8);
        section.push((crc & 0xFF) as u8);

        section
    }

    pub fn build_packet(&self, pmt_pid: u16, cc: u8) -> [u8; TS_PACKET_SIZE] {
        let section = self.build_section();
        let mut payload = vec![0x00];
        payload.extend_from_slice(&section);
        TsPacketBuilder::new(pmt_pid, cc)
            .pusi()
            .payload(&payload)
            .build()
    }
}

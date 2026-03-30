use crate::packet::{TS_PACKET_SIZE, SYNC_BYTE};

pub struct TsPacketBuilder {
    buf: [u8; TS_PACKET_SIZE],
    payload_offset: usize,
}

impl TsPacketBuilder {
    pub fn new(pid: u16, cc: u8) -> Self {
        let mut buf = [0xFF; TS_PACKET_SIZE];
        buf[0] = SYNC_BYTE;
        buf[1] = ((pid >> 8) & 0x1F) as u8;
        buf[2] = (pid & 0xFF) as u8;
        buf[3] = 0x10 | (cc & 0x0F); // payload only, no adaptation
        Self {
            buf,
            payload_offset: 4,
        }
    }

    pub fn pusi(mut self) -> Self {
        self.buf[1] |= 0x40;
        self
    }

    pub fn with_pcr(mut self, pcr: u64) -> Self {
        // adaptation(0x20) + payload(0x10) = 0x30
        self.buf[3] = (self.buf[3] & 0x0F) | 0x30;
        let af_len = 7; // 1 flags + 6 PCR
        self.buf[4] = af_len;
        self.buf[5] = 0x10; // PCR flag
        let base = pcr / 300;
        let ext = (pcr % 300) as u16;
        self.buf[6] = (base >> 25) as u8;
        self.buf[7] = (base >> 17) as u8;
        self.buf[8] = (base >> 9) as u8;
        self.buf[9] = (base >> 1) as u8;
        self.buf[10] = ((base & 1) << 7) as u8 | 0x7E | ((ext >> 8) & 0x01) as u8;
        self.buf[11] = (ext & 0xFF) as u8;
        self.payload_offset = 4 + 1 + af_len as usize;
        self
    }

    pub fn with_adaptation_stuffing(mut self, stuff_bytes: usize) -> Self {
        if stuff_bytes == 0 {
            return self;
        }
        // adaptation(0x20) + payload(0x10) = 0x30
        self.buf[3] = (self.buf[3] & 0x0F) | 0x30;
        let af_len = stuff_bytes.max(1);
        self.buf[4] = af_len as u8;
        self.buf[5] = 0x00; // no flags
        for i in 6..4 + 1 + af_len {
            self.buf[i] = 0xFF; // stuffing
        }
        self.payload_offset = 4 + 1 + af_len;
        self
    }

    pub fn payload(mut self, data: &[u8]) -> Self {
        let avail = TS_PACKET_SIZE - self.payload_offset;
        let len = data.len().min(avail);
        self.buf[self.payload_offset..self.payload_offset + len].copy_from_slice(&data[..len]);
        // stuff remaining with 0xFF
        for i in self.payload_offset + len..TS_PACKET_SIZE {
            self.buf[i] = 0xFF;
        }
        self
    }

    pub fn build(self) -> [u8; TS_PACKET_SIZE] {
        self.buf
    }

    pub fn payload_capacity(&self) -> usize {
        TS_PACKET_SIZE - self.payload_offset
    }
}

pub fn build_null_packet() -> [u8; TS_PACKET_SIZE] {
    TsPacketBuilder::new(0x1FFF, 0).build()
}

use serde::Serialize;
use crate::packet::TsError;

#[derive(Debug, Clone, Serialize)]
pub struct PesHeader {
    pub stream_id: u8,
    pub packet_length: u16,
    pub pts: Option<u64>,
    pub dts: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct PesPacket {
    pub header: PesHeader,
    pub data: Vec<u8>,
}

impl PesPacket {
    pub fn parse(data: &[u8]) -> Result<Self, TsError> {
        if data.len() < 9 {
            return Err(TsError::TooShort(data.len()));
        }

        // PES start code prefix: 0x000001
        if data[0] != 0x00 || data[1] != 0x00 || data[2] != 0x01 {
            return Err(TsError::SyncError(data[0]));
        }

        let stream_id = data[3];
        let packet_length = ((data[4] as u16) << 8) | data[5] as u16;

        let mut pts = None;
        let mut dts = None;
        let mut header_data_offset = 6;

        // stream_id에 따라 optional PES header 존재 여부 결정
        let has_optional_header = !matches!(
            stream_id,
            0xBC | 0xBE | 0xBF | 0xF0 | 0xF1 | 0xF2 | 0xF8 | 0xFF
        );

        if has_optional_header && data.len() >= 9 {
            let pes_header_data_length = data[8] as usize;
            let pts_dts_flags = (data[7] >> 6) & 0x03;

            if pts_dts_flags >= 2 && data.len() >= 14 {
                pts = Some(Self::parse_timestamp(&data[9..14]));
            }
            if pts_dts_flags == 3 && data.len() >= 19 {
                dts = Some(Self::parse_timestamp(&data[14..19]));
            }

            header_data_offset = 9 + pes_header_data_length;
        }

        let payload = if header_data_offset < data.len() {
            data[header_data_offset..].to_vec()
        } else {
            Vec::new()
        };

        Ok(PesPacket {
            header: PesHeader {
                stream_id,
                packet_length,
                pts,
                dts,
            },
            data: payload,
        })
    }

    fn parse_timestamp(data: &[u8]) -> u64 {
        let b0 = data[0] as u64;
        let b1 = data[1] as u64;
        let b2 = data[2] as u64;
        let b3 = data[3] as u64;
        let b4 = data[4] as u64;

        ((b0 >> 1) & 0x07) << 30
            | (b1 << 22)
            | ((b2 >> 1) << 15)
            | (b3 << 7)
            | (b4 >> 1)
    }
}

/// PES 패킷 조립기: 여러 TS 패킷의 payload를 모아서 하나의 PES 패킷으로 조립
#[derive(Debug)]
pub struct PesAssembler {
    buffer: Vec<u8>,
    pid: u16,
    active: bool,
}

impl PesAssembler {
    pub fn new(pid: u16) -> Self {
        Self {
            buffer: Vec::with_capacity(65536),
            pid,
            active: false,
        }
    }

    pub fn pid(&self) -> u16 {
        self.pid
    }

    /// TS 패킷 payload를 추가. payload_unit_start가 true이면 새 PES 시작.
    /// 완성된 PES가 있으면 반환.
    pub fn push(&mut self, payload: &[u8], payload_unit_start: bool) -> Option<PesPacket> {
        let mut completed = None;

        if payload_unit_start {
            if self.active && self.buffer.len() >= 6 {
                completed = PesPacket::parse(&self.buffer).ok();
            }
            self.buffer.clear();
            self.active = true;
        }

        if self.active {
            self.buffer.extend_from_slice(payload);
        }

        completed
    }
}

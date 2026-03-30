use serde::Serialize;
use thiserror::Error;

pub const TS_PACKET_SIZE: usize = 188;
pub const SYNC_BYTE: u8 = 0x47;

#[derive(Error, Debug)]
pub enum TsError {
    #[error("sync byte mismatch: expected 0x47, got 0x{0:02X}")]
    SyncError(u8),
    #[error("packet too short: {0} bytes")]
    TooShort(usize),
    #[error("invalid adaptation field")]
    InvalidAdaptation,
}

#[derive(Debug, Clone, Serialize)]
pub struct TsHeader {
    pub sync_byte: u8,
    pub transport_error: bool,
    pub payload_unit_start: bool,
    pub transport_priority: bool,
    pub pid: u16,
    pub scrambling_control: u8,
    pub adaptation_field_control: u8,
    pub continuity_counter: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdaptationField {
    pub length: u8,
    pub discontinuity: bool,
    pub random_access: bool,
    pub pcr: Option<u64>,
    pub opcr: Option<u64>,
    pub splice_countdown: Option<i8>,
}

#[derive(Debug, Clone)]
pub struct TsPacket {
    pub header: TsHeader,
    pub adaptation: Option<AdaptationField>,
    pub payload: Option<Vec<u8>>,
}

impl TsPacket {
    pub fn parse(data: &[u8]) -> Result<Self, TsError> {
        if data.len() < TS_PACKET_SIZE {
            return Err(TsError::TooShort(data.len()));
        }
        if data[0] != SYNC_BYTE {
            return Err(TsError::SyncError(data[0]));
        }

        let header = TsHeader {
            sync_byte: data[0],
            transport_error: (data[1] & 0x80) != 0,
            payload_unit_start: (data[1] & 0x40) != 0,
            transport_priority: (data[1] & 0x20) != 0,
            pid: ((data[1] as u16 & 0x1F) << 8) | data[2] as u16,
            scrambling_control: (data[3] >> 6) & 0x03,
            adaptation_field_control: (data[3] >> 4) & 0x03,
            continuity_counter: data[3] & 0x0F,
        };

        let mut offset = 4usize;
        let mut adaptation = None;

        if header.adaptation_field_control >= 2 {
            let af_length = data[offset] as usize;
            offset += 1;

            if af_length > 0 && offset + af_length <= TS_PACKET_SIZE {
                let flags = data[offset];
                let mut af = AdaptationField {
                    length: af_length as u8,
                    discontinuity: (flags & 0x80) != 0,
                    random_access: (flags & 0x40) != 0,
                    pcr: None,
                    opcr: None,
                    splice_countdown: None,
                };

                let mut af_offset = offset + 1;

                // PCR (6 bytes)
                if (flags & 0x10) != 0 && af_offset + 6 <= offset + af_length {
                    let base = ((data[af_offset] as u64) << 25)
                        | ((data[af_offset + 1] as u64) << 17)
                        | ((data[af_offset + 2] as u64) << 9)
                        | ((data[af_offset + 3] as u64) << 1)
                        | ((data[af_offset + 4] as u64) >> 7);
                    let ext = (((data[af_offset + 4] & 0x01) as u64) << 8)
                        | data[af_offset + 5] as u64;
                    af.pcr = Some(base * 300 + ext);
                    af_offset += 6;
                }

                // OPCR (6 bytes)
                if (flags & 0x08) != 0 && af_offset + 6 <= offset + af_length {
                    let base = ((data[af_offset] as u64) << 25)
                        | ((data[af_offset + 1] as u64) << 17)
                        | ((data[af_offset + 2] as u64) << 9)
                        | ((data[af_offset + 3] as u64) << 1)
                        | ((data[af_offset + 4] as u64) >> 7);
                    let ext = (((data[af_offset + 4] & 0x01) as u64) << 8)
                        | data[af_offset + 5] as u64;
                    af.opcr = Some(base * 300 + ext);
                    af_offset += 6;
                }

                // Splice countdown (1 byte)
                if (flags & 0x04) != 0 && af_offset < offset + af_length {
                    af.splice_countdown = Some(data[af_offset] as i8);
                }

                adaptation = Some(af);
            }
            offset = 4 + 1 + af_length;
        }

        let payload = if header.adaptation_field_control & 0x01 != 0
            && offset < TS_PACKET_SIZE
        {
            Some(data[offset..TS_PACKET_SIZE].to_vec())
        } else {
            None
        };

        Ok(TsPacket {
            header,
            adaptation,
            payload,
        })
    }
}

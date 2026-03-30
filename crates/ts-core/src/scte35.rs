use serde::Serialize;
use crate::packet::TsError;

#[derive(Debug, Clone, Serialize)]
pub enum SpliceCommandType {
    Null,
    SpliceInsert(SpliceInsert),
    TimeSignal(TimeSignal),
    Unknown(u8),
}

#[derive(Debug, Clone, Serialize)]
pub struct SpliceInsert {
    pub splice_event_id: u32,
    pub splice_event_cancel: bool,
    pub out_of_network: bool,
    pub splice_immediate: bool,
    pub pts_time: Option<u64>,
    pub duration: Option<u64>,
    pub unique_program_id: u16,
    pub avail_num: u8,
    pub avails_expected: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimeSignal {
    pub pts_time: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Scte35 {
    pub table_id: u8,
    pub section_length: u16,
    pub pts_adjustment: u64,
    pub tier: u16,
    pub command: SpliceCommandType,
}

impl Scte35 {
    pub fn parse(data: &[u8]) -> Result<Self, TsError> {
        if data.len() < 15 {
            return Err(TsError::TooShort(data.len()));
        }

        let table_id = data[0];
        if table_id != 0xFC {
            return Err(TsError::SyncError(table_id));
        }

        let section_length = ((data[1] as u16 & 0x0F) << 8) | data[2] as u16;
        // protocol_version = data[3]

        // encrypted_packet + pts_adjustment (33 bits)
        let pts_adjustment = ((data[4] as u64 & 0x01) << 32)
            | ((data[5] as u64) << 24)
            | ((data[6] as u64) << 16)
            | ((data[7] as u64) << 8)
            | data[8] as u64;

        // cw_index = data[9]
        let tier = ((data[10] as u16) << 4) | ((data[11] as u16) >> 4);

        let splice_command_length =
            ((data[11] as u16 & 0x0F) << 8) | data[12] as u16;
        let splice_command_type = data[13];

        let cmd_start = 14;
        let cmd_end = if splice_command_length == 0xFFF {
            data.len()
        } else {
            std::cmp::min(cmd_start + splice_command_length as usize, data.len())
        };
        let cmd_data = &data[cmd_start..cmd_end];

        let command = match splice_command_type {
            0x00 => SpliceCommandType::Null,
            0x05 => SpliceCommandType::SpliceInsert(Self::parse_splice_insert(cmd_data)?),
            0x06 => SpliceCommandType::TimeSignal(Self::parse_time_signal(cmd_data)?),
            other => SpliceCommandType::Unknown(other),
        };

        Ok(Scte35 {
            table_id,
            section_length,
            pts_adjustment,
            tier,
            command,
        })
    }

    fn parse_splice_insert(data: &[u8]) -> Result<SpliceInsert, TsError> {
        if data.len() < 5 {
            return Err(TsError::TooShort(data.len()));
        }

        let splice_event_id = ((data[0] as u32) << 24)
            | ((data[1] as u32) << 16)
            | ((data[2] as u32) << 8)
            | data[3] as u32;
        let splice_event_cancel = (data[4] & 0x80) != 0;

        if splice_event_cancel {
            return Ok(SpliceInsert {
                splice_event_id,
                splice_event_cancel: true,
                out_of_network: false,
                splice_immediate: false,
                pts_time: None,
                duration: None,
                unique_program_id: 0,
                avail_num: 0,
                avails_expected: 0,
            });
        }

        if data.len() < 10 {
            return Err(TsError::TooShort(data.len()));
        }

        let out_of_network = (data[5] & 0x80) != 0;
        let program_splice = (data[5] & 0x40) != 0;
        let _duration_flag = (data[5] & 0x20) != 0;
        let splice_immediate = (data[5] & 0x10) != 0;

        let mut offset = 6;
        let mut pts_time = None;

        if program_splice && !splice_immediate {
            if offset + 5 <= data.len() && (data[offset] & 0x80) != 0 {
                pts_time = Some(Self::read_pts(&data[offset..offset + 5]));
                offset += 5;
            }
        }

        let duration = if _duration_flag && offset + 5 <= data.len() {
            let dur = ((data[offset] as u64 & 0x01) << 32)
                | ((data[offset + 1] as u64) << 24)
                | ((data[offset + 2] as u64) << 16)
                | ((data[offset + 3] as u64) << 8)
                | data[offset + 4] as u64;
            offset += 5;
            Some(dur)
        } else {
            None
        };

        let (unique_program_id, avail_num, avails_expected) =
            if offset + 4 <= data.len() {
                (
                    ((data[offset] as u16) << 8) | data[offset + 1] as u16,
                    data[offset + 2],
                    data[offset + 3],
                )
            } else {
                (0, 0, 0)
            };

        Ok(SpliceInsert {
            splice_event_id,
            splice_event_cancel: false,
            out_of_network,
            splice_immediate,
            pts_time,
            duration,
            unique_program_id,
            avail_num,
            avails_expected,
        })
    }

    fn parse_time_signal(data: &[u8]) -> Result<TimeSignal, TsError> {
        if data.is_empty() {
            return Ok(TimeSignal { pts_time: None });
        }

        let pts_time = if (data[0] & 0x80) != 0 && data.len() >= 5 {
            Some(Self::read_pts(&data[0..5]))
        } else {
            None
        };

        Ok(TimeSignal { pts_time })
    }

    fn read_pts(data: &[u8]) -> u64 {
        ((data[0] as u64 & 0x0E) << 29)
            | ((data[1] as u64) << 22)
            | ((data[2] as u64 & 0xFE) << 14)
            | ((data[3] as u64) << 7)
            | ((data[4] as u64) >> 1)
    }
}

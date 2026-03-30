pub mod pat;
pub mod pmt;
pub mod sdt;
pub mod nit;

use crate::packet::TsError;

/// PSI 섹션 공통 헤더
#[derive(Debug, Clone)]
pub struct PsiSection {
    pub table_id: u8,
    pub section_length: u16,
    pub table_id_extension: u16,
    pub version_number: u8,
    pub current_next: bool,
    pub section_number: u8,
    pub last_section_number: u8,
    pub data: Vec<u8>,
}

impl PsiSection {
    pub fn parse(payload: &[u8]) -> Result<Self, TsError> {
        if payload.len() < 8 {
            return Err(TsError::TooShort(payload.len()));
        }

        let table_id = payload[0];
        let section_length = ((payload[1] as u16 & 0x0F) << 8) | payload[2] as u16;
        let table_id_extension = ((payload[3] as u16) << 8) | payload[4] as u16;
        let version_number = (payload[5] >> 1) & 0x1F;
        let current_next = (payload[5] & 0x01) != 0;
        let section_number = payload[6];
        let last_section_number = payload[7];

        let data_end = std::cmp::min(3 + section_length as usize, payload.len());
        let data = payload[8..data_end.saturating_sub(4)].to_vec(); // exclude CRC32

        Ok(PsiSection {
            table_id,
            section_length,
            table_id_extension,
            version_number,
            current_next,
            section_number,
            last_section_number,
            data,
        })
    }
}

use serde::Serialize;
use super::PsiSection;
use crate::packet::TsError;

#[derive(Debug, Clone, Serialize)]
pub struct PatEntry {
    pub program_number: u16,
    pub pid: u16, // PMT PID (or NIT PID if program_number == 0)
}

#[derive(Debug, Clone, Serialize)]
pub struct Pat {
    pub transport_stream_id: u16,
    pub version: u8,
    pub entries: Vec<PatEntry>,
}

impl Pat {
    pub fn parse(section: &PsiSection) -> Result<Self, TsError> {
        let mut entries = Vec::new();
        let data = &section.data;

        let mut i = 0;
        while i + 4 <= data.len() {
            let program_number = ((data[i] as u16) << 8) | data[i + 1] as u16;
            let pid = ((data[i + 2] as u16 & 0x1F) << 8) | data[i + 3] as u16;
            entries.push(PatEntry { program_number, pid });
            i += 4;
        }

        Ok(Pat {
            transport_stream_id: section.table_id_extension,
            version: section.version_number,
            entries,
        })
    }
}

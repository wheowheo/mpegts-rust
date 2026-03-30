use serde::Serialize;
use super::PsiSection;
use crate::packet::TsError;

#[derive(Debug, Clone, Serialize)]
pub struct PmtStream {
    pub stream_type: u8,
    pub elementary_pid: u16,
    pub descriptors: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Pmt {
    pub program_number: u16,
    pub version: u8,
    pub pcr_pid: u16,
    pub program_descriptors: Vec<u8>,
    pub streams: Vec<PmtStream>,
}

impl Pmt {
    pub fn parse(section: &PsiSection) -> Result<Self, TsError> {
        let data = &section.data;
        if data.len() < 4 {
            return Err(TsError::TooShort(data.len()));
        }

        let pcr_pid = ((data[0] as u16 & 0x1F) << 8) | data[1] as u16;
        let program_info_length = ((data[2] as u16 & 0x0F) << 8) | data[3] as u16;
        let pi_len = program_info_length as usize;

        let program_descriptors = if pi_len > 0 && 4 + pi_len <= data.len() {
            data[4..4 + pi_len].to_vec()
        } else {
            Vec::new()
        };

        let mut streams = Vec::new();
        let mut i = 4 + pi_len;

        while i + 5 <= data.len() {
            let stream_type = data[i];
            let elementary_pid = ((data[i + 1] as u16 & 0x1F) << 8) | data[i + 2] as u16;
            let es_info_length = ((data[i + 3] as u16 & 0x0F) << 8) | data[i + 4] as u16;
            let es_len = es_info_length as usize;

            let descriptors = if es_len > 0 && i + 5 + es_len <= data.len() {
                data[i + 5..i + 5 + es_len].to_vec()
            } else {
                Vec::new()
            };

            streams.push(PmtStream {
                stream_type,
                elementary_pid,
                descriptors,
            });

            i += 5 + es_len;
        }

        Ok(Pmt {
            program_number: section.table_id_extension,
            version: section.version_number,
            pcr_pid,
            program_descriptors,
            streams,
        })
    }

    pub fn stream_type_name(stream_type: u8) -> &'static str {
        match stream_type {
            0x01 => "MPEG-1 Video",
            0x02 => "MPEG-2 Video",
            0x03 => "MPEG-1 Audio",
            0x04 => "MPEG-2 Audio",
            0x06 => "PES private data",
            0x0F => "AAC Audio",
            0x10 => "MPEG-4 Video",
            0x11 => "AAC LATM Audio",
            0x15 => "Metadata",
            0x1B => "H.264/AVC",
            0x24 => "H.265/HEVC",
            0x81 => "AC-3 Audio",
            0x82 => "DTS Audio",
            0x83 => "Dolby TrueHD",
            0x84 => "Dolby Digital Plus (secondary)",
            0x85 => "DTS-HD Audio",
            0x86 => "SCTE-35",
            0x87 => "E-AC-3 Audio",
            0xA1 => "E-AC-3 Audio (secondary)",
            0xA2 => "DTS-HD Audio (secondary)",
            _ => "Unknown",
        }
    }

    pub fn codec_name(stream_type: u8, descriptors: &[u8]) -> &'static str {
        let descs = crate::descriptors::parse_descriptors(descriptors);
        for d in &descs {
            match d.tag {
                // Dolby Vision via registration descriptor "DOVI"
                0x05 if d.data.len() >= 4 && &d.data[0..4] == b"DOVI" => return "Dolby Vision",
                // AC-4 descriptor
                0xA6 => return "Dolby AC-4",
                // Enhanced AC-3 with JOC (Atmos)
                0x7A if !d.data.is_empty() && d.data[0] & 0x01 != 0 => return "Dolby Atmos (E-AC-3 JOC)",
                0xCC if d.data.len() >= 2 && d.data[1] & 0x01 != 0 => return "Dolby Atmos (E-AC-3 JOC)",
                // Standard AC-3 / E-AC-3
                0x6A | 0x81 => return "Dolby Digital (AC-3)",
                0x7A | 0xCC => return "Dolby Digital Plus (E-AC-3)",
                // DTS
                0x7B => return "DTS",
                // AAC
                0x7C => return "AAC",
                _ => {}
            }
        }
        match stream_type {
            0x81 => "AC-3",
            0x87 => "E-AC-3",
            0x83 => "Dolby TrueHD",
            0x1B => "H.264",
            0x24 => "H.265",
            0x01 | 0x02 => "MPEG Video",
            0x03 | 0x04 => "MPEG Audio",
            0x0F => "AAC",
            0x11 => "AAC-LATM",
            _ => "",
        }
    }
}

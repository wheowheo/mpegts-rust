use serde::Serialize;
use super::PsiSection;
use crate::packet::TsError;

#[derive(Debug, Clone, Serialize)]
pub struct NitTransportStream {
    pub transport_stream_id: u16,
    pub original_network_id: u16,
    pub descriptors: Vec<u8>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Nit {
    pub network_id: u16,
    pub version: u8,
    pub network_descriptors: Vec<u8>,
    pub transport_streams: Vec<NitTransportStream>,
}

impl Nit {
    pub fn parse(section: &PsiSection) -> Result<Self, TsError> {
        let data = &section.data;
        if data.len() < 2 {
            return Err(TsError::TooShort(data.len()));
        }

        let network_desc_length = ((data[0] as u16 & 0x0F) << 8) | data[1] as u16;
        let nd_len = network_desc_length as usize;

        let network_descriptors = if nd_len > 0 && 2 + nd_len <= data.len() {
            data[2..2 + nd_len].to_vec()
        } else {
            Vec::new()
        };

        let mut i = 2 + nd_len;
        let mut transport_streams = Vec::new();

        if i + 2 <= data.len() {
            let ts_loop_length = ((data[i] as u16 & 0x0F) << 8) | data[i + 1] as u16;
            let _ = ts_loop_length;
            i += 2;

            while i + 6 <= data.len() {
                let transport_stream_id = ((data[i] as u16) << 8) | data[i + 1] as u16;
                let original_network_id = ((data[i + 2] as u16) << 8) | data[i + 3] as u16;
                let ts_desc_length = ((data[i + 4] as u16 & 0x0F) << 8) | data[i + 5] as u16;
                let td_len = ts_desc_length as usize;

                let descriptors = if td_len > 0 && i + 6 + td_len <= data.len() {
                    data[i + 6..i + 6 + td_len].to_vec()
                } else {
                    Vec::new()
                };

                transport_streams.push(NitTransportStream {
                    transport_stream_id,
                    original_network_id,
                    descriptors,
                });

                i += 6 + td_len;
            }
        }

        Ok(Nit {
            network_id: section.table_id_extension,
            version: section.version_number,
            network_descriptors,
            transport_streams,
        })
    }
}

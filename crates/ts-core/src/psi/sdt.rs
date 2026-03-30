use serde::Serialize;
use super::PsiSection;
use crate::packet::TsError;

#[derive(Debug, Clone, Serialize)]
pub struct SdtService {
    pub service_id: u16,
    pub eit_schedule: bool,
    pub eit_present_following: bool,
    pub running_status: u8,
    pub free_ca_mode: bool,
    pub service_name: String,
    pub provider_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Sdt {
    pub transport_stream_id: u16,
    pub version: u8,
    pub original_network_id: u16,
    pub services: Vec<SdtService>,
}

impl Sdt {
    pub fn parse(section: &PsiSection) -> Result<Self, TsError> {
        let data = &section.data;
        if data.len() < 2 {
            return Err(TsError::TooShort(data.len()));
        }

        let original_network_id = ((data[0] as u16) << 8) | data[1] as u16;
        // skip reserved byte
        let mut services = Vec::new();
        let mut i = 3;

        while i + 5 <= data.len() {
            let service_id = ((data[i] as u16) << 8) | data[i + 1] as u16;
            let eit_schedule = (data[i + 2] & 0x02) != 0;
            let eit_present_following = (data[i + 2] & 0x01) != 0;
            let running_status = (data[i + 3] >> 5) & 0x07;
            let free_ca_mode = (data[i + 3] & 0x10) != 0;
            let desc_loop_length = ((data[i + 3] as u16 & 0x0F) << 8) | data[i + 4] as u16;
            let dl_len = desc_loop_length as usize;

            let (service_name, provider_name) = if dl_len > 0 && i + 5 + dl_len <= data.len() {
                Self::parse_service_descriptor(&data[i + 5..i + 5 + dl_len])
            } else {
                (String::new(), String::new())
            };

            services.push(SdtService {
                service_id,
                eit_schedule,
                eit_present_following,
                running_status,
                free_ca_mode,
                service_name,
                provider_name,
            });

            i += 5 + dl_len;
        }

        Ok(Sdt {
            transport_stream_id: section.table_id_extension,
            version: section.version_number,
            original_network_id,
            services,
        })
    }

    fn parse_service_descriptor(data: &[u8]) -> (String, String) {
        let mut i = 0;
        while i + 2 < data.len() {
            let tag = data[i];
            let len = data[i + 1] as usize;
            if tag == 0x48 && i + 2 + len <= data.len() {
                // service_descriptor
                let desc = &data[i + 2..i + 2 + len];
                if desc.len() >= 2 {
                    let _service_type = desc[0];
                    let provider_len = desc[1] as usize;
                    let provider = if 2 + provider_len <= desc.len() {
                        String::from_utf8_lossy(&desc[2..2 + provider_len]).to_string()
                    } else {
                        String::new()
                    };
                    let name_offset = 2 + provider_len;
                    let name = if name_offset < desc.len() {
                        let name_len = desc[name_offset] as usize;
                        if name_offset + 1 + name_len <= desc.len() {
                            String::from_utf8_lossy(
                                &desc[name_offset + 1..name_offset + 1 + name_len],
                            )
                            .to_string()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    return (name, provider);
                }
            }
            i += 2 + len;
        }
        (String::new(), String::new())
    }
}

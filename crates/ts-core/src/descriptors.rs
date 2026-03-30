use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Descriptor {
    pub tag: u8,
    pub length: u8,
    pub data: Vec<u8>,
}

pub fn parse_descriptors(data: &[u8]) -> Vec<Descriptor> {
    let mut descriptors = Vec::new();
    let mut i = 0;

    while i + 2 <= data.len() {
        let tag = data[i];
        let length = data[i + 1];
        let len = length as usize;

        let desc_data = if i + 2 + len <= data.len() {
            data[i + 2..i + 2 + len].to_vec()
        } else {
            break;
        };

        descriptors.push(Descriptor {
            tag,
            length,
            data: desc_data,
        });

        i += 2 + len;
    }

    descriptors
}

pub fn descriptor_tag_name(tag: u8) -> &'static str {
    match tag {
        0x00 => "reserved",
        0x01 => "forbidden",
        0x02 => "video_stream",
        0x03 => "audio_stream",
        0x04 => "hierarchy",
        0x05 => "registration",
        0x06 => "data_stream_alignment",
        0x07 => "target_background_grid",
        0x08 => "video_window",
        0x09 => "CA",
        0x0A => "ISO_639_language",
        0x0B => "system_clock",
        0x0C => "multiplex_buffer_utilization",
        0x0D => "copyright",
        0x0E => "maximum_bitrate",
        0x0F => "private_data_indicator",
        0x10 => "smoothing_buffer",
        0x11 => "STD",
        0x12 => "IBP",
        0x1B => "MPEG-4_video",
        0x1C => "MPEG-4_audio",
        0x1D => "IOD",
        0x1E => "SL",
        0x1F => "FMC",
        0x20 => "external_ES_ID",
        0x21 => "MuxCode",
        0x22 => "FmxBufferSize",
        0x23 => "multiplexBuffer",
        0x25 => "metadata_pointer",
        0x26 => "metadata",
        0x27 => "metadata_STD",
        0x28 => "AVC_video",
        0x2A => "AVC_timing_and_HRD",
        0x2B => "MPEG-2_AAC_audio",
        0x2C => "FlexMuxTiming",
        0x2D => "MPEG-4_text",
        0x2E => "MPEG-4_audio_extension",
        0x2F => "auxiliary_video_stream",
        0x30 => "SVC_extension",
        0x31 => "MVC_extension",
        0x33 => "J2K_video",
        0x34 => "MVC_operation_point",
        0x35 => "MPEG-2_stereoscopic_video_format",
        0x36 => "stereoscopic_program_info",
        0x37 => "stereoscopic_video_info",
        0x38 => "HEVC_video",
        0x3F => "extension",
        0x40 => "network_name",
        0x41 => "service_list",
        0x42 => "stuffing",
        0x43 => "satellite_delivery_system",
        0x44 => "cable_delivery_system",
        0x45 => "VBI_data",
        0x46 => "VBI_teletext",
        0x47 => "bouquet_name",
        0x48 => "service",
        0x49 => "country_availability",
        0x4A => "linkage",
        0x4B => "NVOD_reference",
        0x4C => "time_shifted_service",
        0x4D => "short_event",
        0x4E => "extended_event",
        0x4F => "time_shifted_event",
        0x50 => "component",
        0x51 => "mosaic",
        0x52 => "stream_identifier",
        0x53 => "CA_identifier",
        0x54 => "content",
        0x55 => "parental_rating",
        0x56 => "teletext",
        0x57 => "telephone",
        0x58 => "local_time_offset",
        0x59 => "subtitling",
        0x5A => "terrestrial_delivery_system",
        0x5B => "multilingual_network_name",
        0x5C => "multilingual_bouquet_name",
        0x5D => "multilingual_service_name",
        0x5E => "multilingual_component",
        0x5F => "private_data_specifier",
        0x60 => "service_move",
        0x61 => "short_smoothing_buffer",
        0x62 => "frequency_list",
        0x63 => "partial_transport_stream",
        0x64 => "data_broadcast",
        0x65 => "scrambling",
        0x66 => "data_broadcast_id",
        0x67 => "transport_stream",
        0x68 => "DSNG",
        0x69 => "PDC",
        0x6A => "AC-3",
        0x6B => "ancillary_data",
        0x6C => "cell_list",
        0x6D => "cell_frequency_link",
        0x6E => "announcement_support",
        0x6F => "application_signalling",
        0x70 => "adaptation_field_data",
        0x71 => "service_identifier",
        0x72 => "service_availability",
        0x73 => "default_authority",
        0x74 => "related_content",
        0x75 => "TVA_id",
        0x76 => "content_identifier",
        0x77 => "time_slice_fec_identifier",
        0x78 => "ECM_repetition_rate",
        0x79 => "S2_satellite_delivery_system",
        0x7A => "enhanced_AC-3",
        0x7B => "DTS_audio",
        0x7C => "AAC",
        0x7D => "XAIT_location",
        0x7E => "FTA_content_management",
        0x7F => "extension",
        0x81 => "AC-3_audio_stream (ATSC)",
        0x86 => "caption_service (ATSC)",
        0x87 => "content_advisory (ATSC)",
        0xA0 => "extended_channel_name (ATSC)",
        0xA1 => "service_location (ATSC)",
        0xA2 => "time_shifted_service (ATSC)",
        0xA3 => "component_name (ATSC)",
        0xCC => "E-AC-3_audio_stream (ATSC)",
        _ => "private/unknown",
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ParsedDescriptor {
    pub tag: u8,
    pub tag_name: String,
    pub length: u8,
    pub fields: Vec<(String, String)>,
    pub raw_hex: String,
}

pub fn parse_descriptor_detail(desc: &Descriptor) -> ParsedDescriptor {
    let tag_name = descriptor_tag_name(desc.tag).to_string();
    let raw_hex = desc.data.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
    let fields = match desc.tag {
        0x02 => parse_video_stream_desc(&desc.data),
        0x03 => parse_audio_stream_desc(&desc.data),
        0x05 => parse_registration_desc(&desc.data),
        0x06 => parse_data_stream_alignment_desc(&desc.data),
        0x09 => parse_ca_desc(&desc.data),
        0x0A => parse_language_desc(&desc.data),
        0x0E => parse_max_bitrate_desc(&desc.data),
        0x28 => parse_avc_video_desc(&desc.data),
        0x38 => parse_hevc_video_desc(&desc.data),
        0x48 => parse_service_desc(&desc.data),
        0x4D => parse_short_event_desc(&desc.data),
        0x52 => parse_stream_identifier_desc(&desc.data),
        0x56 => parse_teletext_desc(&desc.data),
        0x59 => parse_subtitling_desc(&desc.data),
        0x6A => parse_ac3_desc(&desc.data),
        0x7A => parse_eac3_desc(&desc.data),
        0x7C => parse_aac_desc(&desc.data),
        0x81 => parse_atsc_ac3_desc(&desc.data),
        0x86 => parse_caption_service_desc(&desc.data),
        _ => vec![],
    };

    ParsedDescriptor { tag: desc.tag, tag_name, length: desc.length, fields, raw_hex }
}

fn parse_video_stream_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.is_empty() { return vec![]; }
    let mut f = vec![];
    f.push(("multiple_frame_rate".into(), format!("{}", (d[0] >> 7) & 1)));
    f.push(("frame_rate_code".into(), format!("{}", (d[0] >> 3) & 0x0F)));
    f.push(("MPEG_1_only".into(), format!("{}", (d[0] >> 2) & 1)));
    f.push(("constrained_parameter".into(), format!("{}", (d[0] >> 1) & 1)));
    f.push(("still_picture".into(), format!("{}", d[0] & 1)));
    f
}

fn parse_audio_stream_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.is_empty() { return vec![]; }
    let mut f = vec![];
    f.push(("free_format".into(), format!("{}", (d[0] >> 7) & 1)));
    f.push(("ID".into(), format!("{}", (d[0] >> 6) & 1)));
    f.push(("layer".into(), format!("{}", (d[0] >> 4) & 3)));
    f.push(("variable_rate_audio".into(), format!("{}", (d[0] >> 3) & 1)));
    f
}

fn parse_registration_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.len() < 4 { return vec![]; }
    let id = String::from_utf8_lossy(&d[0..4]).to_string();
    let hex = format!("0x{:02X}{:02X}{:02X}{:02X}", d[0], d[1], d[2], d[3]);
    vec![
        ("format_identifier".into(), id),
        ("format_identifier_hex".into(), hex),
    ]
}

fn parse_data_stream_alignment_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.is_empty() { return vec![]; }
    let name = match d[0] {
        0x00 => "reserved",
        0x01 => "slice/video_access_unit",
        0x02 => "video_access_unit",
        0x03 => "GOP/SEQ",
        0x04 => "SEQ",
        _ => "reserved",
    };
    vec![("alignment_type".into(), format!("{} ({})", d[0], name))]
}

fn parse_ca_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.len() < 4 { return vec![]; }
    let ca_system_id = ((d[0] as u16) << 8) | d[1] as u16;
    let ca_pid = ((d[2] as u16 & 0x1F) << 8) | d[3] as u16;
    vec![
        ("CA_system_ID".into(), format!("0x{:04X}", ca_system_id)),
        ("CA_PID".into(), format!("0x{:04X} ({})", ca_pid, ca_pid)),
    ]
}

fn parse_language_desc(d: &[u8]) -> Vec<(String, String)> {
    let mut f = vec![];
    let mut i = 0;
    let mut idx = 0;
    while i + 4 <= d.len() {
        let lang = String::from_utf8_lossy(&d[i..i+3]).to_string();
        let audio_type = match d[i + 3] {
            0x00 => "undefined",
            0x01 => "clean effects",
            0x02 => "hearing impaired",
            0x03 => "visual impaired commentary",
            _ => "reserved",
        };
        f.push((format!("language[{}]", idx), lang));
        f.push((format!("audio_type[{}]", idx), format!("{} (0x{:02X})", audio_type, d[i+3])));
        i += 4;
        idx += 1;
    }
    f
}

fn parse_max_bitrate_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.len() < 3 { return vec![]; }
    let max_br = (((d[0] & 0x3F) as u32) << 16) | ((d[1] as u32) << 8) | d[2] as u32;
    let bps = max_br * 50 * 8;
    vec![
        ("maximum_bitrate".into(), format!("{} ({}x50 bytes/s = {} bps)", max_br, max_br, bps)),
    ]
}

fn parse_avc_video_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.len() < 4 { return vec![]; }
    vec![
        ("profile_idc".into(), format!("0x{:02X} ({})", d[0], d[0])),
        ("constraint_set_flags".into(), format!("0b{:08b}", d[1])),
        ("level_idc".into(), format!("{}.{}", d[2] / 10, d[2] % 10)),
        ("AVC_still_present".into(), format!("{}", (d[3] >> 7) & 1)),
        ("AVC_24_hour_picture".into(), format!("{}", (d[3] >> 6) & 1)),
    ]
}

fn parse_hevc_video_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.len() < 13 { return vec![]; }
    let mut f = vec![];
    f.push(("profile_space".into(), format!("{}", (d[0] >> 6) & 3)));
    f.push(("tier".into(), if d[0] & 0x20 != 0 { "High".into() } else { "Main".into() }));
    f.push(("profile_idc".into(), format!("{}", d[0] & 0x1F)));
    let compat = u32::from_be_bytes([d[1], d[2], d[3], d[4]]);
    f.push(("profile_compatibility".into(), format!("0x{:08X}", compat)));
    f.push(("progressive_source".into(), format!("{}", (d[5] >> 7) & 1)));
    f.push(("interlaced_source".into(), format!("{}", (d[5] >> 6) & 1)));
    f.push(("non_packed_constraint".into(), format!("{}", (d[5] >> 5) & 1)));
    f.push(("frame_only_constraint".into(), format!("{}", (d[5] >> 4) & 1)));
    f.push(("level_idc".into(), format!("{} (Level {}.{})", d[12], d[12] / 30, (d[12] % 30) / 3)));
    f
}

fn parse_service_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.is_empty() { return vec![]; }
    let mut f = vec![];
    let service_type = d[0];
    let st_name = match service_type {
        0x01 => "digital television",
        0x02 => "digital radio",
        0x03 => "teletext",
        0x04 => "NVOD reference",
        0x05 => "NVOD time-shifted",
        0x06 => "mosaic",
        0x0A => "advanced codec digital radio",
        0x0B => "advanced codec mosaic",
        0x0C => "data broadcast",
        0x11 => "MPEG-2 HD digital television",
        0x16 => "H.264/AVC SD digital television",
        0x19 => "H.264/AVC HD digital television",
        0x1F => "HEVC digital television",
        _ => "other",
    };
    f.push(("service_type".into(), format!("0x{:02X} ({})", service_type, st_name)));
    if d.len() > 1 {
        let provider_len = d[1] as usize;
        if 2 + provider_len <= d.len() {
            let provider = String::from_utf8_lossy(&d[2..2 + provider_len]).to_string();
            f.push(("provider_name".into(), provider));
        }
        let name_start = 2 + provider_len;
        if name_start < d.len() {
            let name_len = d[name_start] as usize;
            if name_start + 1 + name_len <= d.len() {
                let name = String::from_utf8_lossy(&d[name_start + 1..name_start + 1 + name_len]).to_string();
                f.push(("service_name".into(), name));
            }
        }
    }
    f
}

fn parse_short_event_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.len() < 4 { return vec![]; }
    let lang = String::from_utf8_lossy(&d[0..3]).to_string();
    let mut f = vec![("language".into(), lang)];
    let name_len = d[3] as usize;
    if 4 + name_len <= d.len() {
        f.push(("event_name".into(), String::from_utf8_lossy(&d[4..4 + name_len]).to_string()));
    }
    f
}

fn parse_stream_identifier_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.is_empty() { return vec![]; }
    vec![("component_tag".into(), format!("0x{:02X} ({})", d[0], d[0]))]
}

fn parse_teletext_desc(d: &[u8]) -> Vec<(String, String)> {
    let mut f = vec![];
    let mut i = 0;
    let mut idx = 0;
    while i + 5 <= d.len() {
        let lang = String::from_utf8_lossy(&d[i..i+3]).to_string();
        let tt_type = (d[i+3] >> 3) & 0x1F;
        let page = ((d[i+3] & 0x07) as u16) * 100 + (((d[i+4] >> 4) & 0x0F) as u16) * 10 + (d[i+4] & 0x0F) as u16;
        let type_name = match tt_type {
            0x01 => "initial teletext page",
            0x02 => "teletext subtitle page",
            0x03 => "additional information page",
            0x04 => "programme schedule page",
            0x05 => "hearing impaired page",
            _ => "reserved",
        };
        f.push((format!("language[{}]", idx), lang));
        f.push((format!("type[{}]", idx), format!("{} ({})", type_name, tt_type)));
        f.push((format!("page[{}]", idx), format!("{}", page)));
        i += 5;
        idx += 1;
    }
    f
}

fn parse_subtitling_desc(d: &[u8]) -> Vec<(String, String)> {
    let mut f = vec![];
    let mut i = 0;
    let mut idx = 0;
    while i + 8 <= d.len() {
        let lang = String::from_utf8_lossy(&d[i..i+3]).to_string();
        let sub_type = d[i+3];
        let comp_page = ((d[i+4] as u16) << 8) | d[i+5] as u16;
        let anc_page = ((d[i+6] as u16) << 8) | d[i+7] as u16;
        f.push((format!("language[{}]", idx), lang));
        f.push((format!("subtitling_type[{}]", idx), format!("0x{:02X}", sub_type)));
        f.push((format!("composition_page[{}]", idx), format!("{}", comp_page)));
        f.push((format!("ancillary_page[{}]", idx), format!("{}", anc_page)));
        i += 8;
        idx += 1;
    }
    f
}

fn parse_ac3_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.is_empty() { return vec![]; }
    let mut f = vec![];
    let flags = d[0];
    f.push(("component_type_flag".into(), format!("{}", (flags >> 7) & 1)));
    f.push(("bsid_flag".into(), format!("{}", (flags >> 6) & 1)));
    f.push(("mainid_flag".into(), format!("{}", (flags >> 5) & 1)));
    f.push(("asvc_flag".into(), format!("{}", (flags >> 4) & 1)));
    let mut i = 1;
    if flags & 0x80 != 0 && i < d.len() { f.push(("component_type".into(), format!("0x{:02X}", d[i]))); i += 1; }
    if flags & 0x40 != 0 && i < d.len() { f.push(("bsid".into(), format!("{}", d[i]))); i += 1; }
    if flags & 0x20 != 0 && i < d.len() { f.push(("mainid".into(), format!("{}", d[i]))); i += 1; }
    if flags & 0x10 != 0 && i < d.len() { f.push(("asvc".into(), format!("0x{:02X}", d[i]))); }
    f
}

fn parse_eac3_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.is_empty() { return vec![]; }
    let mut f = vec![];
    f.push(("component_type_flag".into(), format!("{}", (d[0] >> 7) & 1)));
    f.push(("bsid_flag".into(), format!("{}", (d[0] >> 6) & 1)));
    if d.len() > 1 && d[0] & 0x80 != 0 { f.push(("component_type".into(), format!("0x{:02X}", d[1]))); }
    f
}

fn parse_aac_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.len() < 2 { return vec![]; }
    let profile = match d[0] {
        0x50 => "AAC-LC",
        0x51 => "HE-AAC",
        0x52 => "HE-AAC v2",
        _ => "unknown",
    };
    vec![
        ("profile_and_level".into(), format!("0x{:02X} ({})", d[0], profile)),
        ("AAC_type_flag".into(), format!("{}", (d[1] >> 7) & 1)),
        ("SAOC_DE_flag".into(), format!("{}", (d[1] >> 6) & 1)),
    ]
}

fn parse_atsc_ac3_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.len() < 3 { return vec![]; }
    let sample_rate_code = (d[0] >> 5) & 0x07;
    let bsid = d[0] & 0x1F;
    let bit_rate_code = (d[1] >> 2) & 0x3F;
    let surround_mode = d[1] & 0x03;
    let bsmod = (d[2] >> 5) & 0x07;
    let num_channels = (d[2] >> 1) & 0x0F;
    let sr = match sample_rate_code { 0 => "48kHz", 1 => "44.1kHz", 2 => "32kHz", _ => "reserved" };
    let ch = match num_channels {
        0x01 => "1/0 (mono)", 0x02 => "2/0 (stereo)", 0x03 => "3/0",
        0x04 => "2/1", 0x05 => "3/1", 0x06 => "2/2",
        0x07 => "3/2", 0x08 => "1+1 (dual mono)",
        _ => "reserved",
    };
    vec![
        ("sample_rate".into(), format!("{} ({})", sample_rate_code, sr)),
        ("bsid".into(), format!("{}", bsid)),
        ("bit_rate_code".into(), format!("{}", bit_rate_code)),
        ("surround_mode".into(), format!("{}", surround_mode)),
        ("bsmod".into(), format!("{}", bsmod)),
        ("num_channels".into(), format!("{} ({})", num_channels, ch)),
    ]
}

fn parse_caption_service_desc(d: &[u8]) -> Vec<(String, String)> {
    if d.is_empty() { return vec![]; }
    let count = d[0] & 0x1F;
    let mut f = vec![("number_of_services".into(), format!("{}", count))];
    let mut i = 1;
    for idx in 0..count as usize {
        if i + 6 > d.len() { break; }
        let lang = String::from_utf8_lossy(&d[i..i+3]).to_string();
        let cc_type = (d[i+3] >> 7) & 1;
        let svc_num = if cc_type == 1 { d[i+3] & 0x3F } else { d[i+3] & 0x3F };
        f.push((format!("language[{}]", idx), lang));
        f.push((format!("digital_cc[{}]", idx), format!("{}", cc_type)));
        f.push((format!("service_number[{}]", idx), format!("{}", svc_num)));
        i += 6;
    }
    f
}

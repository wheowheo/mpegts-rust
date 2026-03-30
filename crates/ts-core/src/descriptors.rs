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

/// 잘 알려진 descriptor tag 이름
pub fn descriptor_tag_name(tag: u8) -> &'static str {
    match tag {
        0x02 => "video_stream",
        0x03 => "audio_stream",
        0x05 => "registration",
        0x06 => "data_stream_alignment",
        0x09 => "CA",
        0x0A => "ISO_639_language",
        0x0E => "maximum_bitrate",
        0x10 => "smoothing_buffer",
        0x28 => "AVC_video",
        0x2B => "MPEG-2_AAC_audio",
        0x38 => "HEVC_video",
        0x48 => "service",
        0x4D => "short_event",
        0x4E => "extended_event",
        0x52 => "stream_identifier",
        0x56 => "teletext",
        0x59 => "subtitling",
        0x6A => "AC-3",
        0x7A => "enhanced_AC-3",
        0x7C => "AAC",
        0x86 => "caption_service",
        _ => "unknown",
    }
}

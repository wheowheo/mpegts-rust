use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AacFrame {
    pub profile: u8,
    pub profile_name: String,
    pub sample_rate: u32,
    pub channel_configuration: u8,
    pub channels: u8,
    pub channel_layout: String,
    pub frame_length: u16,
    pub num_raw_data_blocks: u8,
    pub crc_present: bool,
}

const AAC_SAMPLE_RATES: [u32; 16] = [
    96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050,
    16000, 12000, 11025, 8000, 7350, 0, 0, 0,
];

fn profile_name(p: u8) -> &'static str {
    match p {
        0 => "AAC Main",
        1 => "AAC-LC",
        2 => "AAC SSR",
        3 => "AAC LTP",
        _ => "Unknown",
    }
}

fn channel_layout(cfg: u8) -> (u8, &'static str) {
    match cfg {
        1 => (1, "C"),
        2 => (2, "L+R"),
        3 => (3, "L+C+R"),
        4 => (4, "L+C+R+Cs"),
        5 => (5, "L+C+R+Ls+Rs"),
        6 => (6, "L+C+R+Ls+Rs+LFE (5.1)"),
        7 => (8, "L+C+R+Ls+Rs+Rls+Rrs+LFE (7.1)"),
        _ => (0, "Unknown"),
    }
}

pub fn parse_adts(data: &[u8]) -> Option<AacFrame> {
    if data.len() < 7 { return None; }

    // sync word: 0xFFF
    if data[0] != 0xFF || (data[1] & 0xF0) != 0xF0 { return None; }

    let _mpeg_version = (data[1] >> 3) & 0x01; // 0=MPEG-4, 1=MPEG-2
    let _layer = (data[1] >> 1) & 0x03;
    let crc_absent = data[1] & 0x01 != 0;

    let profile = (data[2] >> 6) & 0x03;
    let freq_idx = (data[2] >> 2) & 0x0F;
    let channel_cfg = ((data[2] & 0x01) << 2) | ((data[3] >> 6) & 0x03);

    let frame_length = (((data[3] & 0x03) as u16) << 11)
        | ((data[4] as u16) << 3)
        | ((data[5] >> 5) as u16);

    let num_raw_data_blocks = (data[6] & 0x03) + 1;

    let sample_rate = if (freq_idx as usize) < AAC_SAMPLE_RATES.len() {
        AAC_SAMPLE_RATES[freq_idx as usize]
    } else { 0 };

    let (channels, layout) = channel_layout(channel_cfg);

    Some(AacFrame {
        profile,
        profile_name: profile_name(profile).into(),
        sample_rate,
        channel_configuration: channel_cfg,
        channels,
        channel_layout: layout.into(),
        frame_length,
        num_raw_data_blocks,
        crc_present: !crc_absent,
    })
}

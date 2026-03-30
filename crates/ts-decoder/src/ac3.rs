use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Ac3Frame {
    pub codec: String,
    pub bsid: u8,
    pub bsmod: u8,
    pub acmod: u8,
    pub lfeon: bool,
    pub sample_rate: u32,
    pub bitrate_kbps: u32,
    pub frame_size: usize,
    pub channels: u8,
    pub channel_layout: String,
    pub dialnorm: i8,
    pub compre: bool,
    pub atmos_joc: bool,
}

const AC3_SAMPLE_RATES: [u32; 3] = [48000, 44100, 32000];
const AC3_BITRATES: [u32; 19] = [
    32, 40, 48, 56, 64, 80, 96, 112, 128, 160,
    192, 224, 256, 320, 384, 448, 512, 576, 640,
];
const AC3_FRAME_SIZES_48K: [u16; 19] = [
    64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
    384, 448, 512, 640, 768, 896, 1024, 1152, 1280,
];

fn channel_layout(acmod: u8, lfeon: bool) -> (u8, String) {
    let (ch, layout) = match acmod {
        0 => (2, "Ch1+Ch2 (dual mono)"),
        1 => (1, "C"),
        2 => (2, "L+R"),
        3 => (3, "L+C+R"),
        4 => (2, "L+R+S"),
        5 => (3, "L+C+R+S"),
        6 => (3, "L+R+SL+SR"),
        7 => (4, "L+C+R+SL+SR"),
        _ => (0, "Unknown"),
    };
    let total = ch + if lfeon { 1 } else { 0 };
    let name = if lfeon {
        format!("{}.1 ({}+LFE)", ch, layout)
    } else {
        format!("{}.0 ({})", ch, layout)
    };
    (total, name)
}

pub fn parse_frame(data: &[u8]) -> Option<Ac3Frame> {
    if data.len() < 8 { return None; }

    // sync word 0x0B77
    if data[0] != 0x0B || data[1] != 0x77 { return None; }

    let bsid = (data[5] >> 3) & 0x1F;

    if bsid <= 10 {
        parse_ac3(data, bsid)
    } else if bsid <= 16 {
        parse_eac3(data, bsid)
    } else {
        None
    }
}

fn parse_ac3(data: &[u8], bsid: u8) -> Option<Ac3Frame> {
    if data.len() < 7 { return None; }

    let fscod = (data[4] >> 6) & 0x03;
    let frmsizecod = (data[4] & 0x3F) as usize;

    if fscod >= 3 || frmsizecod / 2 >= AC3_BITRATES.len() {
        return None;
    }

    let sample_rate = AC3_SAMPLE_RATES[fscod as usize];
    let bitrate_kbps = AC3_BITRATES[frmsizecod / 2];
    let frame_size = if fscod == 0 {
        AC3_FRAME_SIZES_48K[frmsizecod / 2] as usize * 2
    } else {
        (bitrate_kbps as usize * 192000 / sample_rate as usize / 8) * 2
    };

    let bsmod = data[5] & 0x07;
    let acmod = (data[6] >> 5) & 0x07;

    let mut bit_offset = 51; // after acmod in bit position
    if acmod & 0x01 != 0 && acmod != 1 { bit_offset += 2; } // cmixlev
    if acmod & 0x04 != 0 { bit_offset += 2; } // surmixlev
    if acmod == 0x02 { bit_offset += 2; } // dsurmod

    let byte_pos = bit_offset / 8;
    let bit_pos = bit_offset % 8;
    let lfeon = if byte_pos < data.len() {
        (data[byte_pos] >> (7 - bit_pos)) & 1 != 0
    } else {
        false
    };

    let dialnorm_pos = bit_offset + 1;
    let db = dialnorm_pos / 8;
    let dp = dialnorm_pos % 8;
    let dialnorm = if db + 1 < data.len() {
        let val = ((data[db] as u16) << 8 | data[db+1] as u16) >> (11 - dp);
        -((val & 0x1F) as i8)
    } else {
        -31
    };

    let compre_pos = dialnorm_pos + 5;
    let compre = if compre_pos / 8 < data.len() {
        (data[compre_pos / 8] >> (7 - compre_pos % 8)) & 1 != 0
    } else {
        false
    };

    let (channels, channel_layout) = channel_layout(acmod, lfeon);

    Some(Ac3Frame {
        codec: "AC-3".into(),
        bsid, bsmod, acmod, lfeon,
        sample_rate, bitrate_kbps, frame_size,
        channels, channel_layout, dialnorm, compre,
        atmos_joc: false,
    })
}

fn parse_eac3(data: &[u8], bsid: u8) -> Option<Ac3Frame> {
    if data.len() < 6 { return None; }

    let frame_size = (((data[2] as usize & 0x07) << 8) | data[3] as usize) * 2 + 2;
    let fscod = (data[4] >> 6) & 0x03;
    let sample_rate = if fscod < 3 {
        AC3_SAMPLE_RATES[fscod as usize]
    } else {
        let fscod2 = (data[4] >> 4) & 0x03;
        match fscod2 { 0 => 24000, 1 => 22050, 2 => 16000, _ => 0 }
    };

    let acmod = (data[4] >> 1) & 0x07;
    let lfeon = data[4] & 0x01 != 0;
    let bsmod = 0; // not directly available in E-AC-3 at same position

    let bitrate_kbps = if sample_rate > 0 {
        (frame_size as u32 * 8 * sample_rate / 1536 / 1000) as u32
    } else { 0 };

    let dialnorm = if data.len() > 5 { -((data[5] >> 3) as i8 & 0x1F) } else { -31 };

    // Atmos JOC detection: check for JOC substream in extended data
    let atmos_joc = detect_atmos_joc(data);

    let (channels, channel_layout) = channel_layout(acmod, lfeon);

    Some(Ac3Frame {
        codec: if atmos_joc { "E-AC-3 + Atmos JOC".into() } else { "E-AC-3".into() },
        bsid, bsmod, acmod, lfeon,
        sample_rate, bitrate_kbps, frame_size,
        channels, channel_layout, dialnorm,
        compre: false,
        atmos_joc,
    })
}

fn detect_atmos_joc(data: &[u8]) -> bool {
    // JOC signaled via addbsi or EC-3 extension with strmtyp=1
    if data.len() < 4 { return false; }
    let strmtyp = (data[2] >> 6) & 0x03;
    if strmtyp == 1 { return true; }
    // heuristic: search for JOC magic bytes in auxdata
    data.windows(2).any(|w| w[0] == 0x4A && w[1] == 0x4F)
}

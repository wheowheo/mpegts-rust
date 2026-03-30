use serde::Serialize;
use crate::bitreader::BitReader;

#[derive(Debug, Clone, Serialize)]
pub struct Sps {
    pub profile_idc: u8,
    pub level_idc: u8,
    pub sps_id: u32,
    pub chroma_format_idc: u32,
    pub bit_depth_luma: u32,
    pub bit_depth_chroma: u32,
    pub width: u32,
    pub height: u32,
    pub max_num_ref_frames: u32,
    pub frame_mbs_only: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Pps {
    pub pps_id: u32,
    pub sps_id: u32,
    pub entropy_coding_mode: u8,
    pub num_ref_idx_l0: u32,
    pub num_ref_idx_l1: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SliceHeader {
    pub slice_type: u8,
    pub slice_type_name: String,
    pub frame_num: u32,
    pub pps_id: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct NalUnit {
    pub nal_type: u8,
    pub nal_type_name: String,
    pub nal_ref_idc: u8,
    pub size: usize,
    pub sps: Option<Sps>,
    pub pps: Option<Pps>,
    pub slice: Option<SliceHeader>,
}

fn nal_type_name(t: u8) -> &'static str {
    match t {
        1 => "Coded slice (non-IDR)",
        2 => "Coded slice part A",
        3 => "Coded slice part B",
        4 => "Coded slice part C",
        5 => "IDR slice",
        6 => "SEI",
        7 => "SPS",
        8 => "PPS",
        9 => "AUD",
        _ => "Other",
    }
}

fn slice_type_name(t: u8) -> &'static str {
    match t {
        0 | 5 => "P",
        1 | 6 => "B",
        2 | 7 => "I",
        3 | 8 => "SP",
        4 | 9 => "SI",
        _ => "?",
    }
}

fn remove_emulation_prevention(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut i = 0;
    while i < data.len() {
        if i + 2 < data.len() && data[i] == 0x00 && data[i+1] == 0x00 && data[i+2] == 0x03 {
            out.push(0x00);
            out.push(0x00);
            i += 3;
        } else {
            out.push(data[i]);
            i += 1;
        }
    }
    out
}

pub fn find_nal_units(data: &[u8]) -> Vec<(usize, usize)> {
    let mut units: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i + 2 < data.len() {
        if data[i] == 0x00 && data[i+1] == 0x00 {
            let (sc_len, start) = if i + 3 < data.len() && data[i+2] == 0x00 && data[i+3] == 0x01 {
                (4, i + 4)
            } else if data[i+2] == 0x01 {
                (3, i + 3)
            } else {
                i += 1;
                continue;
            };
            if let Some(last) = units.last_mut() {
                last.1 = i;
            }
            units.push((start, data.len()));
            i = start;
            let _ = sc_len;
            continue;
        }
        i += 1;
    }
    units
}

pub fn parse_nal(data: &[u8]) -> Option<NalUnit> {
    if data.is_empty() { return None; }
    let forbidden = (data[0] >> 7) & 1;
    if forbidden != 0 { return None; }
    let nal_ref_idc = (data[0] >> 5) & 0x03;
    let nal_type = data[0] & 0x1F;

    let mut unit = NalUnit {
        nal_type,
        nal_type_name: nal_type_name(nal_type).into(),
        nal_ref_idc,
        size: data.len(),
        sps: None,
        pps: None,
        slice: None,
    };

    let rbsp = remove_emulation_prevention(&data[1..]);

    match nal_type {
        7 => { unit.sps = parse_sps(&rbsp); }
        8 => { unit.pps = parse_pps(&rbsp); }
        1 | 5 => { unit.slice = parse_slice_header(&rbsp); }
        _ => {}
    }

    Some(unit)
}

fn parse_sps(data: &[u8]) -> Option<Sps> {
    if data.len() < 4 { return None; }
    let mut br = BitReader::new(data);

    let profile_idc = br.read_bits(8)? as u8;
    br.skip(8); // constraint flags + reserved
    let level_idc = br.read_bits(8)? as u8;
    let sps_id = br.read_ue()?;

    let mut chroma_format_idc = 1;
    let mut bit_depth_luma = 8;
    let mut bit_depth_chroma = 8;

    if matches!(profile_idc, 100 | 110 | 122 | 244 | 44 | 83 | 86 | 118 | 128 | 138 | 139 | 134) {
        chroma_format_idc = br.read_ue()?;
        if chroma_format_idc == 3 {
            br.skip(1);
        }
        bit_depth_luma = br.read_ue()? + 8;
        bit_depth_chroma = br.read_ue()? + 8;
        br.skip(1); // qpprime_y_zero_transform_bypass
        let scaling_list_present = br.read_bit()?;
        if scaling_list_present {
            let count = if chroma_format_idc != 3 { 8 } else { 12 };
            for _ in 0..count {
                if br.read_bit()? {
                    let size = if count < 6 { 16 } else { 64 };
                    skip_scaling_list(&mut br, size);
                }
            }
        }
    }

    let _log2_max_frame_num = br.read_ue()? + 4;
    let pic_order_cnt_type = br.read_ue()?;

    match pic_order_cnt_type {
        0 => { br.read_ue()?; }
        1 => {
            br.skip(1);
            br.read_se()?;
            br.read_se()?;
            let n = br.read_ue()?;
            for _ in 0..n { br.read_se()?; }
        }
        _ => {}
    }

    let max_num_ref_frames = br.read_ue()?;
    br.skip(1); // gaps_in_frame_num_value_allowed
    let pic_width_in_mbs = br.read_ue()? + 1;
    let pic_height_in_map_units = br.read_ue()? + 1;
    let frame_mbs_only = br.read_bit()?;

    let width = pic_width_in_mbs * 16;
    let height = pic_height_in_map_units * 16 * if frame_mbs_only { 1 } else { 2 };

    Some(Sps {
        profile_idc,
        level_idc,
        sps_id,
        chroma_format_idc,
        bit_depth_luma,
        bit_depth_chroma,
        width,
        height,
        max_num_ref_frames,
        frame_mbs_only,
    })
}

fn skip_scaling_list(br: &mut BitReader, size: usize) {
    let mut last_scale = 8i32;
    let mut next_scale = 8i32;
    for _ in 0..size {
        if next_scale != 0 {
            let delta = br.read_se().unwrap_or(0);
            next_scale = (last_scale + delta + 256) % 256;
        }
        last_scale = if next_scale == 0 { last_scale } else { next_scale };
    }
}

fn parse_pps(data: &[u8]) -> Option<Pps> {
    let mut br = BitReader::new(data);
    let pps_id = br.read_ue()?;
    let sps_id = br.read_ue()?;
    let entropy_coding_mode = br.read_bits(1)? as u8;
    br.skip(1); // bottom_field_pic_order_in_frame_present
    let num_slice_groups = br.read_ue()? + 1;
    if num_slice_groups > 1 {
        return Some(Pps { pps_id, sps_id, entropy_coding_mode, num_ref_idx_l0: 0, num_ref_idx_l1: 0 });
    }
    let num_ref_idx_l0 = br.read_ue()? + 1;
    let num_ref_idx_l1 = br.read_ue()? + 1;

    Some(Pps { pps_id, sps_id, entropy_coding_mode, num_ref_idx_l0, num_ref_idx_l1 })
}

fn parse_slice_header(data: &[u8]) -> Option<SliceHeader> {
    let mut br = BitReader::new(data);
    let _first_mb = br.read_ue()?;
    let st = br.read_ue()? as u8;
    let pps_id = br.read_ue()?;
    let frame_num = br.read_bits(4)?; // simplified, actually depends on SPS

    Some(SliceHeader {
        slice_type: st,
        slice_type_name: slice_type_name(st).into(),
        frame_num,
        pps_id,
    })
}

pub fn profile_name(idc: u8) -> &'static str {
    match idc {
        66 => "Baseline",
        77 => "Main",
        88 => "Extended",
        100 => "High",
        110 => "High 10",
        122 => "High 4:2:2",
        244 => "High 4:4:4 Pred",
        _ => "Unknown",
    }
}

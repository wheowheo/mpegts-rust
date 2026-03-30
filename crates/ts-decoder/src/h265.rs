use serde::Serialize;
use crate::bitreader::BitReader;

#[derive(Debug, Clone, Serialize)]
pub struct Vps {
    pub vps_id: u8,
    pub max_layers: u8,
    pub max_sub_layers: u8,
    pub profile_idc: u8,
    pub level_idc: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct Sps {
    pub sps_id: u32,
    pub vps_id: u8,
    pub chroma_format_idc: u32,
    pub width: u32,
    pub height: u32,
    pub bit_depth_luma: u32,
    pub bit_depth_chroma: u32,
    pub log2_max_poc_lsb: u32,
    pub max_sub_layers: u8,
    pub profile_idc: u8,
    pub level_idc: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct Pps {
    pub pps_id: u32,
    pub sps_id: u32,
    pub tiles_enabled: bool,
    pub wpp_enabled: bool,
    pub deblocking_override: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SliceHeader {
    pub slice_type: u8,
    pub slice_type_name: String,
    pub first_slice: bool,
    pub pps_id: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct NalUnit {
    pub nal_type: u8,
    pub nal_type_name: String,
    pub nuh_layer_id: u8,
    pub nuh_temporal_id: u8,
    pub size: usize,
    pub vps: Option<Vps>,
    pub sps: Option<Sps>,
    pub pps: Option<Pps>,
    pub slice: Option<SliceHeader>,
}

fn nal_type_name(t: u8) -> &'static str {
    match t {
        0..=9 => "Coded slice (non-TSA, non-STSA)",
        16..=21 => "Coded slice (BLA/IDR/CRA)",
        32 => "VPS",
        33 => "SPS",
        34 => "PPS",
        35 => "AUD",
        39 | 40 => "SEI",
        _ => "Other",
    }
}

fn slice_type_name(t: u8) -> &'static str {
    match t {
        0 => "B",
        1 => "P",
        2 => "I",
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

pub fn parse_nal(data: &[u8]) -> Option<NalUnit> {
    if data.len() < 2 { return None; }
    let forbidden = (data[0] >> 7) & 1;
    if forbidden != 0 { return None; }

    let nal_type = (data[0] >> 1) & 0x3F;
    let nuh_layer_id = ((data[0] & 1) << 5) | ((data[1] >> 3) & 0x1F);
    let nuh_temporal_id = (data[1] & 0x07).saturating_sub(1);

    let mut unit = NalUnit {
        nal_type,
        nal_type_name: nal_type_name(nal_type).into(),
        nuh_layer_id,
        nuh_temporal_id,
        size: data.len(),
        vps: None,
        sps: None,
        pps: None,
        slice: None,
    };

    let rbsp = remove_emulation_prevention(&data[2..]);

    match nal_type {
        32 => { unit.vps = parse_vps(&rbsp); }
        33 => { unit.sps = parse_sps(&rbsp); }
        34 => { unit.pps = parse_pps(&rbsp); }
        0..=9 | 16..=21 => { unit.slice = parse_slice_header(&rbsp, nal_type); }
        _ => {}
    }

    Some(unit)
}

fn parse_profile_tier_level(br: &mut BitReader, max_sub_layers: u8) -> Option<(u8, u8)> {
    br.skip(2); // general_profile_space
    br.skip(1); // general_tier_flag
    let profile_idc = br.read_bits(5)? as u8;
    br.skip(32); // general_profile_compatibility_flags
    br.skip(48); // general_constraint_indicator_flags
    let level_idc = br.read_bits(8)? as u8;

    if max_sub_layers > 1 {
        let mut sub_layer_profile_present = [false; 7];
        let mut sub_layer_level_present = [false; 7];
        for i in 0..(max_sub_layers - 1) as usize {
            sub_layer_profile_present[i] = br.read_bit()?;
            sub_layer_level_present[i] = br.read_bit()?;
        }
        if max_sub_layers < 8 {
            for _ in (max_sub_layers - 1)..8 {
                br.skip(2);
            }
        }
        for i in 0..(max_sub_layers - 1) as usize {
            if sub_layer_profile_present[i] {
                br.skip(88);
            }
            if sub_layer_level_present[i] {
                br.skip(8);
            }
        }
    }

    Some((profile_idc, level_idc))
}

fn parse_vps(data: &[u8]) -> Option<Vps> {
    let mut br = BitReader::new(data);
    let vps_id = br.read_bits(4)? as u8;
    br.skip(2); // vps_reserved_three_2bits
    let max_layers = br.read_bits(6)? as u8 + 1;
    let max_sub_layers = br.read_bits(3)? as u8 + 1;
    br.skip(1); // vps_temporal_id_nesting

    br.skip(16); // vps_reserved_0xffff_16bits

    let (profile_idc, level_idc) = parse_profile_tier_level(&mut br, max_sub_layers)?;

    Some(Vps { vps_id, max_layers, max_sub_layers, profile_idc, level_idc })
}

fn parse_sps(data: &[u8]) -> Option<Sps> {
    let mut br = BitReader::new(data);
    let vps_id = br.read_bits(4)? as u8;
    let max_sub_layers = br.read_bits(3)? as u8 + 1;
    br.skip(1); // sps_temporal_id_nesting

    let (profile_idc, level_idc) = parse_profile_tier_level(&mut br, max_sub_layers)?;

    let sps_id = br.read_ue()?;
    let chroma_format_idc = br.read_ue()?;
    if chroma_format_idc == 3 {
        br.skip(1);
    }
    let width = br.read_ue()?;
    let height = br.read_ue()?;

    let conformance_window = br.read_bit()?;
    if conformance_window {
        br.read_ue()?; // left
        br.read_ue()?; // right
        br.read_ue()?; // top
        br.read_ue()?; // bottom
    }

    let bit_depth_luma = br.read_ue()? + 8;
    let bit_depth_chroma = br.read_ue()? + 8;
    let log2_max_poc_lsb = br.read_ue()? + 4;

    Some(Sps {
        sps_id, vps_id, chroma_format_idc, width, height,
        bit_depth_luma, bit_depth_chroma, log2_max_poc_lsb,
        max_sub_layers, profile_idc, level_idc,
    })
}

fn parse_pps(data: &[u8]) -> Option<Pps> {
    let mut br = BitReader::new(data);
    let pps_id = br.read_ue()?;
    let sps_id = br.read_ue()?;
    br.skip(1); // dependent_slice_segments
    br.skip(1); // output_flag_present
    br.skip(3); // num_extra_slice_header_bits
    br.skip(1); // sign_data_hiding
    br.skip(1); // cabac_init_present
    br.read_ue()?; // num_ref_idx_l0
    br.read_ue()?; // num_ref_idx_l1
    br.read_se()?; // init_qp
    br.skip(1); // constrained_intra_pred
    br.skip(1); // transform_skip
    let _cu_qp_delta = br.read_bit()?;
    // skip ahead to tiles/wpp flags
    let _ = br.read_se(); // cb_qp_offset
    let _ = br.read_se(); // cr_qp_offset
    br.skip(1); // slice_chroma_qp_offsets
    br.skip(1); // weighted_pred
    br.skip(1); // weighted_bipred
    br.skip(1); // transquant_bypass
    let tiles_enabled = br.read_bit().unwrap_or(false);
    let wpp_enabled = br.read_bit().unwrap_or(false);
    br.skip(1); // ...
    let deblocking_override = br.read_bit().unwrap_or(false);

    Some(Pps { pps_id, sps_id, tiles_enabled, wpp_enabled, deblocking_override })
}

fn parse_slice_header(data: &[u8], nal_type: u8) -> Option<SliceHeader> {
    let mut br = BitReader::new(data);
    let first_slice = br.read_bit()?;

    if matches!(nal_type, 16..=23) {
        br.skip(1); // no_output_of_prior_pics
    }

    let pps_id = br.read_ue()?;

    // simplified: can't fully parse without SPS/PPS context
    let slice_type = if matches!(nal_type, 19 | 20) {
        2 // IDR → I
    } else {
        br.read_ue().unwrap_or(1) as u8
    };

    Some(SliceHeader {
        slice_type,
        slice_type_name: slice_type_name(slice_type).into(),
        first_slice,
        pps_id,
    })
}

pub fn profile_name(idc: u8) -> &'static str {
    match idc {
        1 => "Main",
        2 => "Main 10",
        3 => "Main Still Picture",
        4 => "Range Extensions",
        _ => "Unknown",
    }
}

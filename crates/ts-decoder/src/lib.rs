pub mod h264;
pub mod h265;
pub mod ac3;
pub mod aac;
pub mod bitreader;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub enum FrameInfo {
    Video(VideoFrame),
    Audio(AudioFrame),
}

#[derive(Debug, Clone, Serialize)]
pub struct VideoFrame {
    pub codec: String,
    pub frame_type: String,
    pub size_bytes: usize,
    pub pts: Option<f64>,
    pub dts: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub profile: Option<String>,
    pub level: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AudioFrame {
    pub codec: String,
    pub sample_rate: u32,
    pub channels: u8,
    pub channel_layout: String,
    pub bitrate_kbps: u32,
    pub frame_size: usize,
    pub pts: Option<f64>,
    pub dialog_norm: Option<i8>,
    pub atmos_joc: bool,
}

use serde::Serialize;
use ts_decoder::{h264, h265, ac3, aac, VideoFrame, AudioFrame, FrameInfo};
use ts_decoder::thumbnail::ThumbnailExtractor;

#[derive(Debug, Clone, Serialize)]
pub struct FrameEntry {
    pub index: usize,
    pub packet_index: u64,
    pub frame_type: String,
    pub size_bytes: usize,
    pub pts: Option<f64>,
    pub dts: Option<f64>,
    pub info: FrameInfo,
}

pub struct FrameIndexer {
    pub frames: Vec<FrameEntry>,
    pub thumb_extractor: ThumbnailExtractor,
    stream_type: Option<u8>,
    pes_buffer: Vec<u8>,
    last_pts: Option<f64>,
    last_dts: Option<f64>,
    h264_sps: Option<h264::Sps>,
    h265_sps: Option<h265::Sps>,
}

impl FrameIndexer {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            thumb_extractor: ThumbnailExtractor::new(30), // every 30 I-frames ~1sec@30fps
            stream_type: None,
            pes_buffer: Vec::new(),
            last_pts: None,
            last_dts: None,
            h264_sps: None,
            h265_sps: None,
        }
    }

    pub fn set_stream_type(&mut self, st: u8) {
        self.stream_type = Some(st);
    }

    pub fn feed_pes(&mut self, pes_data: &[u8], pts: Option<u64>, dts: Option<u64>, packet_index: u64) {
        self.last_pts = pts.map(|p| p as f64 / 90000.0);
        self.last_dts = dts.map(|d| d as f64 / 90000.0);
        self.pes_buffer.clear();
        self.pes_buffer.extend_from_slice(pes_data);

        match self.stream_type {
            Some(0x1B) => self.index_h264(packet_index),
            Some(0x24) => self.index_h265(packet_index),
            Some(0x81) | Some(0x87) => self.index_ac3(packet_index),
            Some(0x0F) | Some(0x11) => self.index_aac(packet_index),
            _ => {}
        }
    }

    fn index_h264(&mut self, packet_index: u64) {
        let nals = h264::find_nal_units(&self.pes_buffer);
        for (start, end) in nals {
            if start >= self.pes_buffer.len() { continue; }
            let nal_data = &self.pes_buffer[start..end.min(self.pes_buffer.len())];
            if let Some(nal) = h264::parse_nal(nal_data) {
                if let Some(ref sps) = nal.sps {
                    self.h264_sps = Some(sps.clone());
                }
                if let Some(ref slice) = nal.slice {
                    let (width, height) = self.h264_sps.as_ref()
                        .map(|s| (Some(s.width), Some(s.height)))
                        .unwrap_or((None, None));
                    let profile = self.h264_sps.as_ref()
                        .map(|s| h264::profile_name(s.profile_idc).to_string());
                    let level = self.h264_sps.as_ref()
                        .map(|s| format!("{:.1}", s.level_idc as f32 / 10.0));

                    let idx = self.frames.len();

                    // thumbnail capture on I-frames
                    if slice.slice_type_name == "I" {
                        self.thumb_extractor.inc_frame();
                        if self.thumb_extractor.should_capture() {
                            let w = width.unwrap_or(1920);
                            let h = height.unwrap_or(1080);
                            self.thumb_extractor.add_placeholder(idx, self.last_pts, w, h);
                        }
                    }

                    self.frames.push(FrameEntry {
                        index: idx,
                        packet_index,
                        frame_type: slice.slice_type_name.clone(),
                        size_bytes: nal.size,
                        pts: self.last_pts,
                        dts: self.last_dts,
                        info: FrameInfo::Video(VideoFrame {
                            codec: "H.264".into(),
                            frame_type: slice.slice_type_name.clone(),
                            size_bytes: nal.size,
                            pts: self.last_pts,
                            dts: self.last_dts,
                            width,
                            height,
                            profile,
                            level,
                        }),
                    });
                }
            }
        }
    }

    fn index_h265(&mut self, packet_index: u64) {
        let nals = h264::find_nal_units(&self.pes_buffer); // same start code format
        for (start, end) in nals {
            if start >= self.pes_buffer.len() { continue; }
            let nal_data = &self.pes_buffer[start..end.min(self.pes_buffer.len())];
            if let Some(nal) = h265::parse_nal(nal_data) {
                if let Some(ref sps) = nal.sps {
                    self.h265_sps = Some(sps.clone());
                }
                if let Some(ref slice) = nal.slice {
                    let (width, height) = self.h265_sps.as_ref()
                        .map(|s| (Some(s.width), Some(s.height)))
                        .unwrap_or((None, None));
                    let profile = self.h265_sps.as_ref()
                        .map(|s| h265::profile_name(s.profile_idc).to_string());
                    let level = self.h265_sps.as_ref()
                        .map(|s| format!("{:.1}", s.level_idc as f32 / 30.0));

                    let idx = self.frames.len();
                    self.frames.push(FrameEntry {
                        index: idx,
                        packet_index,
                        frame_type: slice.slice_type_name.clone(),
                        size_bytes: nal.size,
                        pts: self.last_pts,
                        dts: self.last_dts,
                        info: FrameInfo::Video(VideoFrame {
                            codec: "H.265".into(),
                            frame_type: slice.slice_type_name.clone(),
                            size_bytes: nal.size,
                            pts: self.last_pts,
                            dts: self.last_dts,
                            width,
                            height,
                            profile,
                            level,
                        }),
                    });
                }
            }
        }
    }

    fn index_ac3(&mut self, packet_index: u64) {
        if let Some(frame) = ac3::parse_frame(&self.pes_buffer) {
            let idx = self.frames.len();
            self.frames.push(FrameEntry {
                index: idx,
                packet_index,
                frame_type: "Audio".into(),
                size_bytes: frame.frame_size,
                pts: self.last_pts,
                dts: self.last_dts,
                info: FrameInfo::Audio(AudioFrame {
                    codec: frame.codec,
                    sample_rate: frame.sample_rate,
                    channels: frame.channels,
                    channel_layout: frame.channel_layout,
                    bitrate_kbps: frame.bitrate_kbps,
                    frame_size: frame.frame_size,
                    pts: self.last_pts,
                    dialog_norm: Some(frame.dialnorm),
                    atmos_joc: frame.atmos_joc,
                }),
            });
        }
    }

    fn index_aac(&mut self, packet_index: u64) {
        if let Some(frame) = aac::parse_adts(&self.pes_buffer) {
            let bitrate_kbps = if frame.sample_rate > 0 {
                (frame.frame_length as u32 * 8 * frame.sample_rate / 1024 / 1000) as u32
            } else { 0 };
            let idx = self.frames.len();
            self.frames.push(FrameEntry {
                index: idx,
                packet_index,
                frame_type: "Audio".into(),
                size_bytes: frame.frame_length as usize,
                pts: self.last_pts,
                dts: self.last_dts,
                info: FrameInfo::Audio(AudioFrame {
                    codec: frame.profile_name.clone(),
                    sample_rate: frame.sample_rate,
                    channels: frame.channels,
                    channel_layout: frame.channel_layout,
                    bitrate_kbps,
                    frame_size: frame.frame_length as usize,
                    pts: self.last_pts,
                    dialog_norm: None,
                    atmos_joc: false,
                }),
            });
        }
    }
}

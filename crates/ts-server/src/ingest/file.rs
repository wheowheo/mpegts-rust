use std::path::Path;
use ts_core::packet::TS_PACKET_SIZE;
use ts_analyzer::StreamAnalyzer;

pub fn analyze_file(path: &Path) -> std::io::Result<StreamAnalyzer> {
    let data = std::fs::read(path)?;
    let mut analyzer = StreamAnalyzer::new();

    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    analyzer.set_filename(&filename);

    let mut offset = 0;

    // sync byte 탐색
    while offset < data.len() {
        if data[offset] == 0x47 {
            break;
        }
        offset += 1;
    }

    while offset + TS_PACKET_SIZE <= data.len() {
        analyzer.feed_packet(&data[offset..offset + TS_PACKET_SIZE]);
        offset += TS_PACKET_SIZE;
    }

    Ok(analyzer)
}

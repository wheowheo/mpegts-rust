use std::path::Path;
use ts_core::packet::TS_PACKET_SIZE;
use ts_analyzer::StreamAnalyzer;

pub fn analyze_bytes(data: &[u8], filename: &str) -> StreamAnalyzer {
    let mut analyzer = StreamAnalyzer::new();
    analyzer.set_filename(filename);

    let mut offset = 0;

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

    analyzer.sync_pid_bitrates();
    analyzer
}

pub fn analyze_file_mmap(path: &Path) -> std::io::Result<StreamAnalyzer> {
    let file = std::fs::File::open(path)?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let filename = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    Ok(analyze_bytes(&mmap, &filename))
}

use serde::Serialize;
use super::system_stats::SystemSnapshot;

#[derive(Debug, Clone, Serialize)]
pub struct CapacityEstimate {
    pub cpu_headroom_pct: f64,
    pub net_headroom_bps: f64,
    pub estimated_additional_streams: u32,
    pub bottleneck: String,
}

pub fn estimate_capacity(
    sys: &SystemSnapshot,
    current_stream_bitrate: u64,
    current_stream_count: u32,
    max_network_bps: u64,
) -> CapacityEstimate {
    let cpu_headroom = (100.0 - sys.cpu_usage_pct).max(0.0);

    let net_used_bps = sys.net_tx_bytes_sec * 8.0;
    let net_headroom = (max_network_bps as f64 - net_used_bps).max(0.0);

    // CPU based estimate: how many more streams before hitting 90%
    let cpu_per_stream = if current_stream_count > 0 {
        sys.cpu_usage_pct / current_stream_count as f64
    } else {
        5.0 // default assumption
    };
    let cpu_additional = if cpu_per_stream > 0.0 {
        ((cpu_headroom - 10.0).max(0.0) / cpu_per_stream) as u32
    } else {
        0
    };

    // network based estimate
    let net_additional = if current_stream_bitrate > 0 {
        (net_headroom / current_stream_bitrate as f64) as u32
    } else {
        0
    };

    let (additional, bottleneck) = if cpu_additional <= net_additional {
        (cpu_additional, "cpu".to_string())
    } else {
        (net_additional, "network".to_string())
    };

    CapacityEstimate {
        cpu_headroom_pct: cpu_headroom,
        net_headroom_bps: net_headroom,
        estimated_additional_streams: additional,
        bottleneck,
    }
}

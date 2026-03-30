export interface StreamInfo {
	filename: string;
	duration_ms: number | null;
	total_packets: number;
	total_bitrate_bps: number;
	programs: ProgramInfo[];
}

export interface ProgramInfo {
	program_number: number;
	pmt_pid: number;
	streams: ElementaryStreamInfo[];
}

export interface ElementaryStreamInfo {
	pid: number;
	stream_type: number;
	stream_type_name: string;
	codec: string | null;
}

export interface PidInfo {
	pid: number;
	label: string;
	stream_type: number | null;
	packet_count: number;
	cc_errors: number;
	bitrate_bps: number;
	has_pcr: boolean;
	scrambled: boolean;
	percentage: number;
}

export interface RealtimeData {
	total_bitrate_bps?: number;
	pcr_jitter_ms?: number;
	cc_error?: {
		pid: number;
		expected: number;
		got: number;
	};
	pids?: PidInfo[];
	total_packets?: number;
	complete?: boolean;
}

export interface OutputConfig {
	source_type: 'file' | 'udp';
	source_path?: string;
	source_addr?: string;
	dest_addr: string;
	dest_port: number;
	protocol: 'udp' | 'rtp';
	bitrate_bps: number;
}

export interface OutputStatus {
	running: boolean;
	config: OutputConfig | null;
	packets_sent: number;
	bytes_sent: number;
	elapsed_sec: number;
	actual_bitrate_bps: number;
}

export interface SystemSnapshot {
	timestamp_sec: number;
	cpu_usage_pct: number;
	memory_used_bytes: number;
	memory_total_bytes: number;
	net_tx_bytes_sec: number;
	net_tx_drops: number;
}

export interface CapacityEstimate {
	cpu_headroom_pct: number;
	net_headroom_bps: number;
	estimated_additional_streams: number;
	bottleneck: string;
}

export interface SystemResponse {
	system: SystemSnapshot;
	capacity: CapacityEstimate;
}

export interface Scte35Event {
	pts: number;
	command_type: string;
	splice_event_id: number;
	duration_ms: number | null;
	out_of_network: boolean;
}

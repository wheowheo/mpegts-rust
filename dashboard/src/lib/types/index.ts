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

export interface Scte35Event {
	pts: number;
	command_type: string;
	splice_event_id: number;
	duration_ms: number | null;
	out_of_network: boolean;
}

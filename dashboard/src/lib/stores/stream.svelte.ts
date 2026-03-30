import type { StreamInfo, PidInfo, RealtimeData } from '../types';

let streamInfo = $state<StreamInfo | null>(null);
let pids = $state<PidInfo[]>([]);
let bitrateHistory = $state<{ ts: number; bps: number }[]>([]);
let pcrJitter = $state<{ ts: number; jitter_ms: number }[]>([]);
let ccErrors = $state<{ ts: number; pid: number; expected: number; got: number }[]>([]);
let analyzing = $state(false);

export function getStreamStore() {
	return {
		get streamInfo() { return streamInfo; },
		get pids() { return pids; },
		get bitrateHistory() { return bitrateHistory; },
		get pcrJitter() { return pcrJitter; },
		get ccErrors() { return ccErrors; },
		get analyzing() { return analyzing; },

		setStreamInfo(info: StreamInfo) {
			streamInfo = info;
		},

		setAnalyzing(v: boolean) {
			analyzing = v;
		},

		updatePids(newPids: PidInfo[]) {
			pids = newPids;
		},

		pushRealtime(data: RealtimeData) {
			const now = Date.now();

			if (data.total_bitrate_bps !== undefined) {
				bitrateHistory = [
					...bitrateHistory.slice(-299),
					{ ts: now, bps: data.total_bitrate_bps },
				];
			}

			if (data.pcr_jitter_ms !== undefined) {
				pcrJitter = [
					...pcrJitter.slice(-299),
					{ ts: now, jitter_ms: data.pcr_jitter_ms },
				];
			}

			if (data.cc_error) {
				ccErrors = [
					...ccErrors.slice(-999),
					{ ts: now, ...data.cc_error },
				];
			}

			if (data.pids) {
				pids = data.pids;
			}
		},

		reset() {
			streamInfo = null;
			pids = [];
			bitrateHistory = [];
			pcrJitter = [];
			ccErrors = [];
		},
	};
}

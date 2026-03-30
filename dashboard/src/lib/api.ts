import type { StreamInfo, PidInfo, OutputConfig, OutputStatus, SystemResponse } from './types';

const BASE = '/api';

export async function fetchStreamInfo(): Promise<StreamInfo> {
	const res = await fetch(`${BASE}/stream`);
	return res.json();
}

export async function fetchPids(): Promise<PidInfo[]> {
	const res = await fetch(`${BASE}/pids`);
	return res.json();
}

export async function fetchPidDetail(pid: number): Promise<PidInfo> {
	const res = await fetch(`${BASE}/pids/${pid}`);
	if (!res.ok) throw new Error(`PID ${pid} not found`);
	return res.json();
}

export async function uploadFile(file: File): Promise<{ status: string; total_packets: number; filename: string }> {
	const form = new FormData();
	form.append('file', file);
	const res = await fetch(`${BASE}/analyze`, { method: 'POST', body: form });
	if (!res.ok) throw new Error('Upload failed');
	return res.json();
}

export async function startOutput(config: OutputConfig): Promise<OutputStatus> {
	const res = await fetch(`${BASE}/output/start`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(config),
	});
	if (!res.ok) throw new Error(await res.text());
	return res.json();
}

export async function stopOutput(): Promise<OutputStatus> {
	const res = await fetch(`${BASE}/output/stop`, { method: 'POST' });
	return res.json();
}

export async function fetchOutputStatus(): Promise<OutputStatus> {
	const res = await fetch(`${BASE}/output/status`);
	return res.json();
}

export async function fetchSystemStats(): Promise<SystemResponse> {
	const res = await fetch(`${BASE}/system`);
	return res.json();
}

import type { StreamInfo, PidInfo } from './types';

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

import type { StreamInfo, PidInfo, OutputConfig, OutputStatus, SystemResponse, IngestStatus, Tr101290Summary, Tr101290Error } from './types';

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

export async function fetchPidFullDetail(pid: number): Promise<any> {
	const res = await fetch(`${BASE}/pids/${pid}/detail`);
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

export async function stopOutput(sessionId?: string): Promise<void> {
	const url = sessionId
		? `${BASE}/output/stop/${sessionId}`
		: `${BASE}/output/stop`;
	await fetch(url, { method: 'POST' });
}

export async function listOutputs(): Promise<OutputStatus[]> {
	const res = await fetch(`${BASE}/output/list`);
	return res.json();
}

export async function fetchSystemStats(): Promise<SystemResponse> {
	const res = await fetch(`${BASE}/system`);
	return res.json();
}

export interface PacketHex {
	index: number;
	hex: string;
}

export async function fetchPidPackets(pid: number, offset = 0, limit = 16): Promise<PacketHex[]> {
	const res = await fetch(`${BASE}/pids/${pid}/packets?offset=${offset}&limit=${limit}`);
	if (!res.ok) throw new Error(`PID ${pid} packets not found`);
	return res.json();
}

export interface FrameEntry {
	index: number;
	packet_index: number;
	frame_type: string;
	size_bytes: number;
	pts: number | null;
	dts: number | null;
	info: { Video?: VideoFrame; Audio?: AudioFrame };
}

export interface VideoFrame {
	codec: string;
	frame_type: string;
	size_bytes: number;
	pts: number | null;
	dts: number | null;
	width: number | null;
	height: number | null;
	profile: string | null;
	level: string | null;
}

export interface AudioFrame {
	codec: string;
	sample_rate: number;
	channels: number;
	channel_layout: string;
	bitrate_kbps: number;
	frame_size: number;
	pts: number | null;
	dialog_norm: number | null;
	atmos_joc: boolean;
}

export async function fetchPidFrames(pid: number, offset = 0, limit = 200): Promise<FrameEntry[]> {
	const res = await fetch(`${BASE}/pids/${pid}/frames?offset=${offset}&limit=${limit}`);
	if (!res.ok) throw new Error(`PID ${pid} frames not found`);
	return res.json();
}

export interface ThumbnailInfo {
	index: number;
	frame_index: number;
	pts: number | null;
	width: number;
	height: number;
	size_bytes: number;
}

export async function fetchPidThumbnails(pid: number): Promise<ThumbnailInfo[]> {
	const res = await fetch(`${BASE}/pids/${pid}/thumbnails`);
	if (!res.ok) return [];
	return res.json();
}

export function thumbnailUrl(pid: number, idx: number): string {
	return `${BASE}/pids/${pid}/thumbnail/${idx}`;
}

export async function startIngest(url: string, protocol?: string): Promise<IngestStatus> {
	const res = await fetch(`${BASE}/ingest/start`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ url, protocol }),
	});
	if (!res.ok) throw new Error(await res.text());
	return res.json();
}

export async function stopIngest(): Promise<IngestStatus> {
	const res = await fetch(`${BASE}/ingest/stop`, { method: 'POST' });
	return res.json();
}

export async function fetchIngestStatus(): Promise<IngestStatus> {
	const res = await fetch(`${BASE}/ingest/status`);
	return res.json();
}

export async function fetchTr101290(): Promise<Tr101290Summary> {
	const res = await fetch(`${BASE}/tr101290`);
	return res.json();
}

export interface SessionRecord {
	id: string;
	filename: string;
	start_time: string;
	end_time: string | null;
	duration_ms: number | null;
	total_packets: number;
	bitrate_bps: number;
	p1_errors: number;
	p2_errors: number;
	p3_errors: number;
}

export interface HistoryStats {
	total_sessions: number;
	total_errors: number;
	total_packets: number;
}

export async function fetchHistorySessions(limit = 50, offset = 0): Promise<SessionRecord[]> {
	const res = await fetch(`${BASE}/history/sessions?limit=${limit}&offset=${offset}`);
	return res.json();
}

export async function fetchHistorySession(id: string): Promise<any> {
	const res = await fetch(`${BASE}/history/sessions/${id}`);
	if (!res.ok) throw new Error('Session not found');
	return res.json();
}

export async function deleteHistorySession(id: string): Promise<void> {
	await fetch(`${BASE}/history/sessions/${id}`, { method: 'DELETE' });
}

export async function fetchHistoryStats(): Promise<HistoryStats> {
	const res = await fetch(`${BASE}/history/stats`);
	return res.json();
}

export async function fetchTr101290Errors(limit = 200, priority?: string): Promise<Tr101290Error[]> {
	const params = new URLSearchParams({ limit: String(limit) });
	if (priority) params.set('priority', priority);
	const res = await fetch(`${BASE}/tr101290/errors?${params}`);
	return res.json();
}

<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { listOutputs, fetchSystemStats } from '$lib/api';
	import type { OutputStatus, SystemResponse } from '$lib/types';
	import OutputControl from '$lib/components/OutputControl.svelte';
	import SystemLoad from '$lib/components/SystemLoad.svelte';
	import TxBitrateChart from '$lib/components/TxBitrateChart.svelte';

	let sessions = $state<OutputStatus[]>([]);
	let systemData = $state<SystemResponse | null>(null);
	let alertHistory = $state<{ time: string; level: string; message: string }[]>([]);
	let interval: ReturnType<typeof setInterval>;

	async function refresh() {
		try {
			const [outputs, system] = await Promise.all([
				listOutputs(),
				fetchSystemStats(),
			]);
			sessions = outputs;
			systemData = system;

			for (const s of outputs) {
				for (const a of s.alerts) {
					const timeStr = formatElapsed(a.timestamp_sec);
					const exists = alertHistory.some(h => h.message === a.message && h.time === timeStr);
					if (!exists) {
						alertHistory = [...alertHistory.slice(-99), { time: timeStr, level: a.level, message: `[${s.session_id}] ${a.message}` }];
					}
				}
			}
		} catch (e) {
			console.error('poll error', e);
		}
	}

	onMount(() => {
		refresh();
		interval = setInterval(refresh, 1000);
	});

	onDestroy(() => {
		clearInterval(interval);
	});

	const activeSession = $derived(sessions.find(s => s.running) ?? null);
	const hasActive = $derived(sessions.some(s => s.running));

	function formatBps(bps: number): string {
		if (bps >= 1_000_000) return (bps / 1_000_000).toFixed(2);
		if (bps >= 1_000) return (bps / 1_000).toFixed(1);
		return bps.toFixed(0);
	}

	function bpsUnit(bps: number): string {
		if (bps >= 1_000_000) return 'Mbps';
		if (bps >= 1_000) return 'Kbps';
		return 'bps';
	}

	function formatElapsed(sec: number): string {
		const h = Math.floor(sec / 3600);
		const m = Math.floor((sec % 3600) / 60);
		const s = Math.floor(sec % 60);
		if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
		return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
	}

	function formatBytes(bytes: number): string {
		if (bytes >= 1_073_741_824) return (bytes / 1_073_741_824).toFixed(2) + ' GB';
		if (bytes >= 1_048_576) return (bytes / 1_048_576).toFixed(1) + ' MB';
		if (bytes >= 1_024) return (bytes / 1_024).toFixed(1) + ' KB';
		return bytes + ' B';
	}

	function deviationColor(pct: number): string {
		const abs = Math.abs(pct);
		if (abs > 15) return 'var(--red)';
		if (abs > 5) return 'var(--amber)';
		return 'var(--green)';
	}

	function deviationGlow(pct: number): string {
		const abs = Math.abs(pct);
		if (abs > 15) return 'var(--glow-red)';
		if (abs > 5) return '0 0 8px rgba(255,184,0,0.3)';
		return 'var(--glow-green)';
	}

	function progressPct(s: OutputStatus): number {
		if (!s.source_packets) return 0;
		return (s.source_position / s.source_packets) * 100;
	}

	function ssrcHex(n: number): string {
		return '0x' + (n >>> 0).toString(16).toUpperCase().padStart(8, '0');
	}
</script>

<div class="output-page">
	<!-- Row 1: Control + System -->
	<div class="top-row">
		<div class="top-left">
			<OutputControl {sessions} onUpdate={refresh} />
		</div>
		<div class="top-right">
			<SystemLoad data={systemData} />
			{#if activeSession}
				<div class="card deviation-gauge">
					<h3>Bitrate Accuracy</h3>
					<div class="deviation-value">
						<span class="seg-number" style="color: {deviationColor(activeSession.bitrate_deviation_pct)}; text-shadow: {deviationGlow(activeSession.bitrate_deviation_pct)};">
							{activeSession.bitrate_deviation_pct >= 0 ? '+' : ''}{activeSession.bitrate_deviation_pct.toFixed(2)}
						</span>
						<span class="meter-unit">%</span>
					</div>
					<div class="meter-bar">
						<div class="meter-fill" style="width: {Math.min(Math.abs(activeSession.bitrate_deviation_pct) * 2, 100)}%; background: {deviationColor(activeSession.bitrate_deviation_pct)};"></div>
					</div>
					<div class="deviation-sub">
						Target: {formatBps(activeSession.config?.bitrate_bps ?? 0)} {bpsUnit(activeSession.config?.bitrate_bps ?? 0)}
						/ Actual: {formatBps(activeSession.actual_bitrate_bps)} {bpsUnit(activeSession.actual_bitrate_bps)}
					</div>
				</div>
			{/if}
		</div>
	</div>

	{#if hasActive && activeSession}
		<!-- Row 2: Meter Grid -->
		<div class="meter-grid">
			<div class="card meter">
				<h3>Status</h3>
				<div class="meter-value">
					<span class="led led-on"></span>
					<span class="seg-number green">LIVE</span>
				</div>
				<div class="meter-sub">{activeSession.session_id}</div>
			</div>

			<div class="card meter">
				<h3>Bitrate</h3>
				<div class="meter-value">
					<span class="seg-number cyan">{formatBps(activeSession.actual_bitrate_bps)}</span>
					<span class="meter-unit">{bpsUnit(activeSession.actual_bitrate_bps)}</span>
				</div>
				<div class="meter-bar">
					<div class="meter-fill" style="width: {Math.min(activeSession.actual_bitrate_bps / 50_000_000 * 100, 100)}%; background: var(--cyan);"></div>
				</div>
			</div>

			<div class="card meter">
				<h3>Uptime</h3>
				<div class="meter-value">
					<span class="seg-number green">{formatElapsed(activeSession.elapsed_sec)}</span>
				</div>
				<div class="meter-sub">{activeSession.elapsed_sec.toFixed(1)} sec</div>
			</div>

			<div class="card meter">
				<h3>Packets Sent</h3>
				<div class="meter-value">
					<span class="seg-number cyan">{activeSession.packets_sent.toLocaleString()}</span>
				</div>
				<div class="meter-sub">{formatBytes(activeSession.bytes_sent)}</div>
			</div>

			<div class="card meter">
				<h3>Data Sent</h3>
				<div class="meter-value">
					<span class="seg-number">{formatBytes(activeSession.bytes_sent)}</span>
				</div>
				<div class="meter-sub">{(activeSession.bytes_sent * 8 / 1_000_000).toFixed(1)} Mbit total</div>
			</div>

			<div class="card meter">
				<h3>Source Loop</h3>
				<div class="meter-value">
					<span class="seg-number amber">{activeSession.loop_count}</span>
				</div>
				<div class="loop-progress">
					<div class="meter-bar">
						<div class="meter-fill" style="width: {progressPct(activeSession)}%; background: var(--amber);"></div>
					</div>
					<div class="meter-sub">{activeSession.source_position.toLocaleString()} / {activeSession.source_packets.toLocaleString()} pkts</div>
				</div>
			</div>
		</div>

		<!-- Row 3: Chart + Detail -->
		<div class="mid-row">
			<div class="chart-area">
				<TxBitrateChart status={activeSession} />
			</div>
			<div class="detail-area">
				<!-- Transport Detail -->
				<div class="card detail-table">
					<h3>Transport</h3>
					<table>
						<tbody>
							<tr><td>Protocol</td><td><span class="badge">{activeSession.config?.protocol.toUpperCase()}</span></td></tr>
							<tr><td>Destination</td><td class="mono">{activeSession.config?.dest_addr}:{activeSession.config?.dest_port}</td></tr>
							<tr><td>Source</td><td class="mono truncate">{activeSession.config?.source_path ?? '-'}</td></tr>
							<tr><td>Source Packets</td><td class="mono">{activeSession.source_packets.toLocaleString()}</td></tr>
							<tr><td>Packets/Burst</td><td class="mono">{activeSession.packets_per_burst}</td></tr>
							<tr><td>Burst Interval</td><td class="mono">{activeSession.burst_interval_us.toLocaleString()} us</td></tr>
							<tr><td>Datagram Size</td><td class="mono">{activeSession.rtp_info ? '1,328' : '1,316'} bytes</td></tr>
						</tbody>
					</table>
				</div>

				<!-- RTP Detail (conditional) -->
				{#if activeSession.rtp_info}
					<div class="card detail-table">
						<h3>RTP</h3>
						<table>
							<tbody>
								<tr><td>SSRC</td><td class="mono accent">{ssrcHex(activeSession.rtp_info.ssrc)}</td></tr>
								<tr><td>Sequence</td><td class="mono">{activeSession.rtp_info.sequence.toLocaleString()}</td></tr>
								<tr><td>Timestamp</td><td class="mono">{activeSession.rtp_info.timestamp.toLocaleString()}</td></tr>
								<tr><td>Payload Type</td><td class="mono">{activeSession.rtp_info.payload_type} (MPEG-TS)</td></tr>
								<tr><td>TS/RTP Packet</td><td class="mono">{activeSession.rtp_info.ts_per_packet}</td></tr>
							</tbody>
						</table>
					</div>
				{/if}
			</div>
		</div>

		<!-- Row 4: Alerts Log -->
		<div class="card alert-log">
			<div class="alert-header">
				<h3>Alerts</h3>
				{#if alertHistory.length === 0}
					<span class="alert-status ok"><span class="led led-on"></span> NO ALERTS</span>
				{:else}
					<span class="alert-status warn"><span class="led led-warn"></span> {alertHistory.length} ALERT{alertHistory.length > 1 ? 'S' : ''}</span>
				{/if}
			</div>
			{#if alertHistory.length > 0}
				<div class="alert-list">
					{#each [...alertHistory].reverse() as alert}
						<div class="alert-row">
							<span class="alert-time mono">{alert.time}</span>
							<span class="alert-badge" class:critical={alert.level === 'critical'} class:warning={alert.level === 'warning'}>
								{alert.level.toUpperCase()}
							</span>
							<span class="alert-msg">{alert.message}</span>
						</div>
					{/each}
				</div>
			{:else}
				<div class="alert-empty">No alerts recorded during this session</div>
			{/if}
		</div>

		<!-- Multi-session summary (when >1 active) -->
		{#if sessions.filter(s => s.running).length > 1}
			<div class="card">
				<h3>All Sessions ({sessions.filter(s => s.running).length})</h3>
				<div class="multi-grid">
					{#each sessions.filter(s => s.running) as s}
						<div class="mini-session">
							<div class="mini-header">
								<span class="led led-on"></span>
								<span class="mono">{s.session_id}</span>
								<span class="badge">{s.config?.protocol.toUpperCase()}</span>
							</div>
							<div class="mini-stats">
								<span>{formatBps(s.actual_bitrate_bps)} {bpsUnit(s.actual_bitrate_bps)}</span>
								<span>{s.packets_sent.toLocaleString()} pkts</span>
								<span>{formatElapsed(s.elapsed_sec)}</span>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	{:else}
		<div class="empty-state">
			<div class="empty-icon">&#x25B6;</div>
			<div class="empty-label">NO ACTIVE OUTPUT</div>
			<div class="empty-sub">Configure and start an output session from the control panel</div>
		</div>
	{/if}
</div>

<style>
	.output-page {
		display: flex;
		flex-direction: column;
		gap: 0.6rem;
	}

	/* Row 1 */
	.top-row {
		display: grid;
		grid-template-columns: 340px 1fr;
		gap: 0.6rem;
	}
	.top-right {
		display: flex;
		flex-direction: column;
		gap: 0.6rem;
	}

	/* Meter Grid */
	.meter-grid {
		display: grid;
		grid-template-columns: repeat(6, 1fr);
		gap: 0.5rem;
	}
	.card {
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 0.6rem;
	}
	.card h3 {
		margin: 0 0 0.4rem;
		font-family: var(--font-mono);
		font-size: 0.55rem;
		font-weight: 600;
		color: var(--text-dim);
		letter-spacing: 0.12em;
		text-transform: uppercase;
	}
	.meter-value {
		display: flex;
		align-items: baseline;
		gap: 0.3rem;
	}
	.seg-number {
		font-family: var(--font-mono);
		font-size: 1.3rem;
		font-weight: 700;
		color: var(--text);
		line-height: 1;
	}
	.seg-number.cyan { color: var(--cyan); text-shadow: var(--glow-cyan); }
	.seg-number.green { color: var(--green); text-shadow: var(--glow-green); }
	.seg-number.amber { color: var(--amber); text-shadow: 0 0 8px rgba(255,184,0,0.3); }
	.meter-unit {
		font-family: var(--font-mono);
		font-size: 0.65rem;
		color: var(--text-muted);
	}
	.meter-bar {
		height: 4px;
		background: var(--bg-inset);
		border-radius: 2px;
		margin-top: 0.4rem;
		overflow: hidden;
	}
	.meter-fill {
		height: 100%;
		border-radius: 2px;
		transition: width 0.3s;
	}
	.meter-sub {
		font-family: var(--font-mono);
		font-size: 0.6rem;
		color: var(--text-muted);
		margin-top: 0.25rem;
	}

	/* LED */
	.led {
		display: inline-block;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		margin-right: 0.3rem;
	}
	.led-on {
		background: var(--green);
		box-shadow: var(--glow-green);
	}
	.led-warn {
		background: var(--amber);
		box-shadow: 0 0 8px rgba(255,184,0,0.4);
	}

	/* Deviation Gauge */
	.deviation-gauge { text-align: center; }
	.deviation-value {
		display: flex;
		align-items: baseline;
		justify-content: center;
		gap: 0.3rem;
		margin-bottom: 0.3rem;
	}
	.deviation-sub {
		font-family: var(--font-mono);
		font-size: 0.6rem;
		color: var(--text-muted);
		margin-top: 0.4rem;
	}

	/* Loop progress */
	.loop-progress { margin-top: 0.2rem; }

	/* Mid Row */
	.mid-row {
		display: grid;
		grid-template-columns: 2fr 1fr;
		gap: 0.6rem;
	}
	.detail-area {
		display: flex;
		flex-direction: column;
		gap: 0.6rem;
	}

	/* Detail Tables */
	.detail-table table {
		width: 100%;
		border-collapse: collapse;
	}
	.detail-table td {
		padding: 0.25rem 0;
		font-size: 0.7rem;
		border-bottom: 1px solid var(--border);
	}
	.detail-table td:first-child {
		color: var(--text-muted);
		font-family: var(--font-mono);
		font-size: 0.6rem;
		letter-spacing: 0.04em;
		width: 45%;
	}
	.detail-table td:last-child {
		text-align: right;
		color: var(--text);
	}
	.mono { font-family: var(--font-mono); }
	.accent { color: var(--accent) !important; }
	.truncate {
		max-width: 160px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		display: inline-block;
	}
	.badge {
		font-family: var(--font-mono);
		font-size: 0.55rem;
		font-weight: 700;
		padding: 0.1rem 0.35rem;
		border-radius: 3px;
		background: var(--accent-dim);
		color: var(--accent);
		letter-spacing: 0.06em;
	}

	/* Alert Log */
	.alert-log { max-height: 220px; }
	.alert-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}
	.alert-header h3 { margin: 0; }
	.alert-status {
		font-family: var(--font-mono);
		font-size: 0.6rem;
		font-weight: 600;
		display: flex;
		align-items: center;
		letter-spacing: 0.06em;
	}
	.alert-status.ok { color: var(--green); }
	.alert-status.warn { color: var(--amber); }
	.alert-list {
		max-height: 150px;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 0.2rem;
	}
	.alert-row {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.65rem;
		padding: 0.2rem 0.3rem;
		border-radius: 3px;
		background: var(--bg-inset);
	}
	.alert-time {
		color: var(--text-dim);
		font-size: 0.6rem;
		flex-shrink: 0;
	}
	.alert-badge {
		font-family: var(--font-mono);
		font-size: 0.5rem;
		font-weight: 700;
		padding: 0.1rem 0.25rem;
		border-radius: 2px;
		flex-shrink: 0;
	}
	.alert-badge.warning { background: var(--amber-dim); color: var(--amber); }
	.alert-badge.critical { background: var(--red-dim); color: var(--red); }
	.alert-msg { color: var(--text-muted); }
	.alert-empty {
		font-size: 0.65rem;
		color: var(--text-dim);
		font-style: italic;
	}

	/* Multi-session */
	.multi-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
		gap: 0.5rem;
	}
	.mini-session {
		padding: 0.4rem;
		border: 1px solid var(--border);
		border-radius: 4px;
		background: var(--bg-inset);
	}
	.mini-header {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.75rem;
		font-weight: 600;
		margin-bottom: 0.3rem;
	}
	.mini-stats {
		display: flex;
		gap: 0.8rem;
		font-family: var(--font-mono);
		font-size: 0.6rem;
		color: var(--text-muted);
	}

	/* Empty state */
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		min-height: 300px;
		gap: 0.5rem;
	}
	.empty-icon { font-size: 2rem; color: var(--text-dim); }
	.empty-label {
		font-family: var(--font-mono);
		font-size: 0.8rem;
		color: var(--text-dim);
		letter-spacing: 0.12em;
	}
	.empty-sub { font-size: 0.75rem; color: var(--text-muted); }

	/* Responsive */
	@media (max-width: 1100px) {
		.meter-grid { grid-template-columns: repeat(3, 1fr); }
		.mid-row { grid-template-columns: 1fr; }
	}
	@media (max-width: 768px) {
		.top-row { grid-template-columns: 1fr; }
		.meter-grid { grid-template-columns: repeat(2, 1fr); }
	}
</style>

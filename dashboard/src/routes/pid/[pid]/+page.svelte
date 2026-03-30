<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { fetchPidFullDetail } from '$lib/api';
	import HexViewer from '$lib/components/HexViewer.svelte';
	import FrameTimeline from '$lib/components/FrameTimeline.svelte';
	import ThumbnailStrip from '$lib/components/ThumbnailStrip.svelte';

	let detail = $state<any>(null);
	let error = $state('');
	const pid = $derived(Number(page.params.pid));

	onMount(async () => {
		try {
			detail = await fetchPidFullDetail(pid);
		} catch (e: any) {
			error = e.message;
		}
	});

	function pidHex(p: number): string {
		return `0x${p.toString(16).toUpperCase().padStart(4, '0')}`;
	}
	function hex8(v: number): string { return `0x${v.toString(16).toUpperCase().padStart(2, '0')}`; }
	function formatBps(bps: number): string {
		if (bps >= 1e6) return (bps / 1e6).toFixed(2) + ' Mbps';
		if (bps >= 1e3) return (bps / 1e3).toFixed(1) + ' kbps';
		return bps.toFixed(0) + ' bps';
	}
	function afcName(i: number): string {
		return ['Reserved', 'Payload only', 'AF only', 'AF + Payload'][i] ?? '?';
	}
</script>

<div class="pid-page">
	<a href="/" class="back">&larr; DASHBOARD</a>

	{#if error}
		<div class="card"><p style="color:var(--red)">{error}</p></div>
	{:else if !detail}
		<div class="card"><p style="color:var(--text-muted)">Loading...</p></div>
	{:else}

	<!-- Header -->
	<div class="pid-header">
		<div class="pid-id">
			<span class="seg-number cyan">{pidHex(detail.pid)}</span>
			<span class="pid-dec">({detail.pid})</span>
		</div>
		<div class="pid-meta">
			<span class="pid-label">{detail.label}</span>
			{#if detail.stream_type != null}
				<span class="pid-type">{hex8(detail.stream_type)} {detail.stream_type_name}</span>
			{/if}
		</div>
		<div class="pid-stats">
			<div class="mini-meter"><span class="seg-number green" style="font-size:1.2rem">{formatBps(detail.bitrate_bps)}</span></div>
			<div class="mini-meter"><span class="mono">{detail.percentage.toFixed(2)}%</span></div>
		</div>
	</div>

	<div class="grid-2">
		<!-- Transport Header Stats -->
		<div class="card">
			<h3>Transport Header</h3>
			<div class="kv-grid">
				<div class="kv"><span class="k">Total Packets</span><span class="v">{detail.transport.total_packets.toLocaleString()}</span></div>
				<div class="kv"><span class="k">PUSI Count</span><span class="v">{detail.transport.payload_unit_start_count.toLocaleString()}</span></div>
				<div class="kv"><span class="k">Transport Errors</span>
					<span class="v" class:red-text={detail.transport.transport_error_count > 0}>{detail.transport.transport_error_count}</span>
				</div>
				<div class="kv"><span class="k">Priority</span><span class="v">{detail.transport.transport_priority_count}</span></div>
				<div class="kv"><span class="k">Scrambled (Even)</span><span class="v">{detail.transport.scrambled_even_count}</span></div>
				<div class="kv"><span class="k">Scrambled (Odd)</span><span class="v">{detail.transport.scrambled_odd_count}</span></div>
			</div>
			<h3 style="margin-top:0.5rem">Adaptation Field Control</h3>
			<div class="kv-grid">
				{#each detail.transport.adaptation_field_control as count, i}
					<div class="kv"><span class="k">{i}: {afcName(i)}</span><span class="v mono">{count.toLocaleString()}</span></div>
				{/each}
			</div>
		</div>

		<!-- Adaptation Field Stats -->
		<div class="card">
			<h3>Adaptation Field</h3>
			<div class="kv-grid">
				<div class="kv"><span class="k">Total with AF</span><span class="v">{detail.adaptation.total_with_af}</span></div>
				<div class="kv"><span class="k">AF Only (stuffing)</span><span class="v">{detail.adaptation.af_only_count}</span></div>
				<div class="kv"><span class="k">Discontinuity</span>
					<span class="v" class:red-text={detail.adaptation.discontinuity_count > 0}>{detail.adaptation.discontinuity_count}</span>
				</div>
				<div class="kv"><span class="k">Random Access</span><span class="v">{detail.adaptation.random_access_count}</span></div>
				<div class="kv"><span class="k">PCR Count</span><span class="v">{detail.adaptation.pcr_count}</span></div>
				<div class="kv"><span class="k">OPCR Count</span><span class="v">{detail.adaptation.opcr_count}</span></div>
				<div class="kv"><span class="k">Splice Countdown</span><span class="v">{detail.adaptation.splice_countdown_count}</span></div>
			</div>
		</div>
	</div>

	<!-- PCR Section -->
	{#if detail.pcr_samples.length > 0}
	<div class="card">
		<h3>PCR Analysis ({detail.pcr_samples.length} samples)</h3>
		<div class="pcr-meters">
			<div class="pcr-m">
				<span class="k">PCR Bitrate</span>
				<span class="seg-number cyan" style="font-size:1.1rem">{detail.pcr_bitrate_bps ? formatBps(detail.pcr_bitrate_bps) : 'N/A'}</span>
			</div>
			<div class="pcr-m">
				<span class="k">Avg Jitter</span>
				<span class="seg-number green" style="font-size:1.1rem">{detail.pcr_jitter_avg_ms?.toFixed(4) ?? 'N/A'} ms</span>
			</div>
			<div class="pcr-m">
				<span class="k">Max Jitter</span>
				<span class="seg-number amber" style="font-size:1.1rem">{detail.pcr_jitter_max_ms?.toFixed(4) ?? 'N/A'} ms</span>
			</div>
		</div>
		<div class="table-scroll">
			<table>
				<thead><tr><th>Index</th><th>PCR Base</th><th>Ext</th><th>Value (27MHz)</th><th>Seconds</th><th>Interval</th></tr></thead>
				<tbody>
					{#each detail.pcr_samples.slice(0, 30) as s}
					<tr>
						<td class="mono">{s.packet_index}</td>
						<td class="mono">{s.pcr_base}</td>
						<td class="mono">{s.pcr_ext}</td>
						<td class="mono">{s.pcr_value.toLocaleString()}</td>
						<td class="mono">{s.pcr_seconds.toFixed(6)}</td>
						<td class="mono">{s.interval_ms != null ? s.interval_ms.toFixed(3) + ' ms' : '-'}</td>
					</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</div>
	{/if}

	<!-- PES Section -->
	{#if detail.pes_samples.length > 0}
	<div class="card">
		<h3>PES Headers ({detail.pes_samples.length} samples)</h3>
		<div class="table-scroll">
			<table>
				<thead><tr><th>Index</th><th>Stream ID</th><th>Type</th><th>Length</th><th>PTS (sec)</th><th>DTS (sec)</th><th>PTS (raw 90kHz)</th><th>DTS (raw 90kHz)</th></tr></thead>
				<tbody>
					{#each detail.pes_samples as p}
					<tr>
						<td class="mono">{p.packet_index}</td>
						<td class="mono">{hex8(p.stream_id)}</td>
						<td>{p.stream_id_name}</td>
						<td class="mono">{p.packet_length}</td>
						<td class="mono">{p.pts != null ? p.pts.toFixed(6) : '-'}</td>
						<td class="mono">{p.dts != null ? p.dts.toFixed(6) : '-'}</td>
						<td class="mono dim">{p.pts_raw ?? '-'}</td>
						<td class="mono dim">{p.dts_raw ?? '-'}</td>
					</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</div>
	{/if}

	<!-- Descriptors -->
	{#if detail.descriptors.length > 0}
	<div class="card">
		<h3>Descriptors ({detail.descriptors.length})</h3>
		{#each detail.descriptors as desc}
		<div class="desc-block">
			<div class="desc-header">
				<span class="desc-tag">{hex8(desc.tag)}</span>
				<span class="desc-name">{desc.tag_name}</span>
				<span class="desc-len">length={desc.length}</span>
			</div>
			{#if desc.fields.length > 0}
			<div class="desc-fields">
				{#each desc.fields as [k, v]}
				<div class="desc-field">
					<span class="k">{k}</span>
					<span class="v mono">{v}</span>
				</div>
				{/each}
			</div>
			{/if}
			<div class="desc-raw">
				<span class="k">Raw</span>
				<span class="mono dim">{desc.raw_hex}</span>
			</div>
		</div>
		{/each}
	</div>
	{/if}

	<!-- CC Errors -->
	<div class="card">
		<h3>Continuity Counter</h3>
		<div class="cc-summary">
			<div class="counter-display" class:has-error={detail.cc_errors.length > 0}>
				<span class="seg-number" class:red={detail.cc_errors.length > 0} class:green={detail.cc_errors.length === 0} style="font-size:1.3rem">
					{detail.cc_errors.length.toString().padStart(4, '0')}
				</span>
			</div>
			<div class="cc-info">
				<span class="led" class:led-green={detail.cc_errors.length === 0} class:led-red={detail.cc_errors.length > 0}></span>
				<span>{detail.cc_errors.length === 0 ? 'NO ERRORS' : `Error rate: ${(detail.cc_error_rate * 100).toFixed(6)}%`}</span>
			</div>
		</div>
		{#if detail.cc_errors.length > 0}
		<div class="table-scroll">
			<table>
				<thead><tr><th>Packet Index</th><th>Expected</th><th>Got</th></tr></thead>
				<tbody>
					{#each detail.cc_errors as e}
					<tr>
						<td class="mono">{e.packet_index}</td>
						<td class="mono">{e.expected}</td>
						<td class="mono red-text">{e.got}</td>
					</tr>
					{/each}
				</tbody>
			</table>
		</div>
		{/if}
	</div>

	<!-- Packet Range -->
	<div class="card">
		<h3>Packet Range</h3>
		<div class="kv-grid">
			<div class="kv"><span class="k">First Packet</span><span class="v mono">{detail.first_packet_index.toLocaleString()}</span></div>
			<div class="kv"><span class="k">Last Packet</span><span class="v mono">{detail.last_packet_index.toLocaleString()}</span></div>
			<div class="kv"><span class="k">Span</span><span class="v mono">{(detail.last_packet_index - detail.first_packet_index).toLocaleString()} packets</span></div>
		</div>
	</div>

	<!-- Frame Analysis -->
	<div class="card">
		<h3>Frame Analysis</h3>
		<FrameTimeline {pid} />
	</div>

	<!-- Thumbnail Strip -->
	<div class="card">
		<h3>Thumbnails</h3>
		<ThumbnailStrip {pid} />
	</div>

	<!-- HEX Viewer -->
	<div class="card">
		<h3>HEX Viewer</h3>
		<HexViewer {pid} />
	</div>

	{/if}
</div>

<style>
	.pid-page { display: flex; flex-direction: column; gap: 0.6rem; }
	.back {
		font-family: var(--font-mono); font-size: 0.7rem;
		color: var(--text-muted); text-decoration: none; letter-spacing: 0.08em;
	}
	.back:hover { color: var(--accent); }

	.pid-header {
		display: flex; align-items: center; gap: 1rem;
		background: var(--bg-card); border: 1px solid var(--border);
		border-radius: var(--radius); padding: 0.75rem 1rem;
		position: relative;
	}
	.pid-header::before {
		content: ''; position: absolute; top: 0; left: 12px; right: 12px;
		height: 1px; background: linear-gradient(90deg, transparent, var(--accent), transparent); opacity: 0.3;
	}
	.pid-id { display: flex; align-items: baseline; gap: 0.4rem; }
	.pid-dec { font-family: var(--font-mono); font-size: 0.72rem; color: var(--text-dim); }
	.pid-meta { flex: 1; display: flex; flex-direction: column; gap: 0.15rem; }
	.pid-label { font-weight: 600; font-size: 1rem; }
	.pid-type { font-family: var(--font-mono); font-size: 0.72rem; color: var(--text-muted); }
	.pid-stats { display: flex; gap: 1rem; align-items: center; }
	.mini-meter { text-align: right; }

	.grid-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 0.6rem; }

	.kv-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 0.15rem 0.5rem; }
	.kv { display: flex; justify-content: space-between; padding: 0.2rem 0; border-bottom: 1px solid var(--border); font-size: 0.78rem; }
	.k { color: var(--text-muted); }
	.v { font-family: var(--font-mono); color: var(--text); }
	.red-text { color: var(--red) !important; }
	.dim { color: var(--text-dim) !important; }

	.pcr-meters { display: flex; gap: 1.5rem; margin-bottom: 0.5rem; }
	.pcr-m { display: flex; flex-direction: column; gap: 0.15rem; }
	.pcr-m .k { font-size: 0.65rem; }

	.table-scroll { overflow: auto; max-height: 400px; border: 1px solid var(--border); border-radius: 4px; }

	.desc-block {
		background: var(--bg-inset); border: 1px solid var(--border);
		border-radius: 4px; padding: 0.5rem; margin-bottom: 0.4rem;
	}
	.desc-header { display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.3rem; }
	.desc-tag {
		font-family: var(--font-mono); font-size: 0.72rem; font-weight: 700;
		color: var(--accent); background: var(--accent-dim);
		padding: 0.1rem 0.3rem; border-radius: 3px;
	}
	.desc-name { font-weight: 600; font-size: 0.8rem; }
	.desc-len { font-family: var(--font-mono); font-size: 0.65rem; color: var(--text-dim); }
	.desc-fields { display: grid; grid-template-columns: 1fr 1fr; gap: 0.15rem 0.5rem; margin-bottom: 0.3rem; }
	.desc-field { display: flex; justify-content: space-between; font-size: 0.75rem; padding: 0.1rem 0.3rem; }
	.desc-field .k { color: var(--text-muted); }
	.desc-raw { font-size: 0.65rem; padding: 0.2rem 0.3rem; }

	.cc-summary { display: flex; align-items: center; gap: 0.8rem; margin-bottom: 0.5rem; }
	.counter-display {
		background: var(--bg-inset); border: 1px solid var(--border);
		border-radius: 4px; padding: 0.2rem 0.5rem;
	}
	.counter-display.has-error { border-color: rgba(255, 59, 92, 0.3); }
	.cc-info { display: flex; align-items: center; gap: 0.3rem; font-family: var(--font-mono); font-size: 0.7rem; color: var(--text-muted); }

	@media (max-width: 768px) {
		.grid-2 { grid-template-columns: 1fr; }
		.pid-header { flex-direction: column; align-items: flex-start; }
	}
</style>

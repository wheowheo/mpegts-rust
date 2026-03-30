<script lang="ts">
	import type { StreamInfo } from '../types';

	let { info }: { info: StreamInfo | null } = $props();

	function formatBitrate(bps: number): string {
		if (bps >= 1_000_000) return `${(bps / 1_000_000).toFixed(2)} Mbps`;
		if (bps >= 1_000) return `${(bps / 1_000).toFixed(1)} kbps`;
		return `${bps.toFixed(0)} bps`;
	}

	function formatDuration(ms: number | null): string {
		if (ms == null) return '-';
		const sec = ms / 1000;
		const m = Math.floor(sec / 60);
		const s = (sec % 60).toFixed(1);
		return m > 0 ? `${m}m ${s}s` : `${s}s`;
	}
</script>

{#if info}
<div class="summary-grid">
	<div class="card stat">
		<h3>File</h3>
		<span class="mono">{info.filename}</span>
	</div>
	<div class="card stat">
		<h3>Duration</h3>
		<span class="value">{formatDuration(info.duration_ms)}</span>
	</div>
	<div class="card stat">
		<h3>Packets</h3>
		<span class="value">{info.total_packets.toLocaleString()}</span>
	</div>
	<div class="card stat">
		<h3>Bitrate</h3>
		<span class="value">{formatBitrate(info.total_bitrate_bps)}</span>
	</div>
	<div class="card stat">
		<h3>Programs</h3>
		<span class="value">{info.programs.length}</span>
	</div>
</div>
{:else}
<div class="card">
	<p style="color: var(--text-muted)">No stream loaded. Upload a .ts file to start analysis.</p>
</div>
{/if}

<style>
	.summary-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 0.75rem;
	}
	.stat {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}
	.value {
		font-size: 1.25rem;
		font-weight: 600;
	}
</style>

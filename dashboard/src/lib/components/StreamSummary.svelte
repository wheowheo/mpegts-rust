<script lang="ts">
	import type { StreamInfo } from '../types';

	let { info }: { info: StreamInfo | null } = $props();

	function formatBitrate(bps: number): string {
		if (bps >= 1_000_000) return (bps / 1_000_000).toFixed(2);
		if (bps >= 1_000) return (bps / 1_000).toFixed(1);
		return bps.toFixed(0);
	}

	function bitrateUnit(bps: number): string {
		if (bps >= 1_000_000) return 'Mbps';
		if (bps >= 1_000) return 'kbps';
		return 'bps';
	}

	function formatDuration(ms: number | null): string {
		if (ms == null) return '--:--';
		const sec = Math.floor(ms / 1000);
		const m = Math.floor(sec / 60);
		const s = sec % 60;
		const h = Math.floor(m / 60);
		if (h > 0) return `${h}:${(m % 60).toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
		return `${m}:${s.toString().padStart(2, '0')}`;
	}

	function bitratePercent(bps: number): number {
		return Math.min((bps / 50_000_000) * 100, 100);
	}

	const totalCcErrors = $derived(
		info ? 0 : 0  // placeholder, actual from pids
	);
</script>

{#if info}
<div class="meter-grid">
	<div class="card meter">
		<h3>Bitrate</h3>
		<div class="meter-value">
			<span class="seg-number cyan">{formatBitrate(info.total_bitrate_bps)}</span>
			<span class="meter-unit">{bitrateUnit(info.total_bitrate_bps)}</span>
		</div>
		<div class="meter-bar">
			<div class="meter-fill" style="width: {bitratePercent(info.total_bitrate_bps)}%; background: var(--cyan);"></div>
		</div>
	</div>

	<div class="card meter">
		<h3>Duration</h3>
		<div class="meter-value">
			<span class="seg-number green">{formatDuration(info.duration_ms)}</span>
		</div>
		<div class="meter-sub">{info.duration_ms != null ? (info.duration_ms / 1000).toFixed(1) + ' sec' : 'N/A'}</div>
	</div>

	<div class="card meter">
		<h3>Packets</h3>
		<div class="meter-value">
			<span class="seg-number amber">{info.total_packets.toLocaleString()}</span>
		</div>
		<div class="meter-sub">{(info.total_packets * 188 / 1_048_576).toFixed(1)} MB</div>
	</div>

	<div class="card meter">
		<h3>Programs</h3>
		<div class="meter-value">
			<span class="seg-number cyan">{info.programs.length}</span>
		</div>
		<div class="meter-sub">
			{#each info.programs as prog}
				<span class="prog-tag">#{prog.program_number}</span>
			{/each}
		</div>
	</div>

	<div class="card meter file-meter">
		<h3>Source</h3>
		<div class="filename">{info.filename || 'N/A'}</div>
	</div>
</div>
{:else}
<div class="card empty-state">
	<p>No stream loaded</p>
</div>
{/if}

<style>
	.meter-grid {
		display: grid;
		grid-template-columns: 1.5fr 1fr 1fr 0.8fr 1.5fr;
		gap: 0.5rem;
	}
	.meter {
		display: flex;
		flex-direction: column;
		gap: 0.3rem;
		padding: 0.6rem 0.75rem;
	}
	.meter-value {
		display: flex;
		align-items: baseline;
		gap: 0.3rem;
	}
	.meter-unit {
		font-family: var(--font-mono);
		font-size: 0.65rem;
		color: var(--text-muted);
		letter-spacing: 0.05em;
	}
	.meter-sub {
		font-family: var(--font-mono);
		font-size: 0.65rem;
		color: var(--text-dim);
	}
	.file-meter { justify-content: center; }
	.filename {
		font-family: var(--font-mono);
		font-size: 0.72rem;
		color: var(--text);
		word-break: break-all;
		line-height: 1.3;
	}
	.prog-tag {
		font-family: var(--font-mono);
		font-size: 0.6rem;
		color: var(--accent);
		background: var(--accent-dim);
		padding: 0.05rem 0.3rem;
		border-radius: 2px;
		margin-right: 0.25rem;
	}
	.empty-state p {
		color: var(--text-dim);
		font-size: 0.8rem;
		text-align: center;
		padding: 1rem;
	}

	@media (max-width: 900px) {
		.meter-grid { grid-template-columns: repeat(2, 1fr); }
		.file-meter { grid-column: 1 / -1; }
	}
</style>

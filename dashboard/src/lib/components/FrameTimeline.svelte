<script lang="ts">
	import { onMount } from 'svelte';
	import { fetchPidFrames, type FrameEntry } from '$lib/api';

	let { pid }: { pid: number } = $props();

	let frames = $state<FrameEntry[]>([]);
	let selected = $state<FrameEntry | null>(null);
	let loading = $state(true);

	onMount(async () => {
		try {
			frames = await fetchPidFrames(pid);
		} catch { /* empty */ }
		loading = false;
	});

	function frameColor(type: string): string {
		switch (type) {
			case 'I': return 'var(--red)';
			case 'P': return 'var(--cyan)';
			case 'B': return 'var(--green)';
			case 'Audio': return 'var(--amber)';
			default: return 'var(--text-muted)';
		}
	}

	function frameBg(type: string): string {
		switch (type) {
			case 'I': return 'var(--red-dim)';
			case 'P': return 'var(--accent-dim)';
			case 'B': return 'var(--green-dim)';
			case 'Audio': return 'var(--amber-dim)';
			default: return 'transparent';
		}
	}

	function frameHeight(frame: FrameEntry): number {
		if (!frames.length) return 20;
		const maxSize = Math.max(...frames.map(f => f.size_bytes));
		return Math.max(8, (frame.size_bytes / maxSize) * 60);
	}

	let isVideo = $derived(frames.length > 0 && frames[0].info.Video != null);

	// GOP detection
	function isGopBoundary(i: number): boolean {
		return i > 0 && frames[i].frame_type === 'I';
	}

	function formatSec(v: number | null): string {
		if (v === null) return '-';
		return v.toFixed(6) + 's';
	}
</script>

{#if loading}
	<p class="muted">Loading frames...</p>
{:else if frames.length === 0}
	<p class="muted">No frames decoded</p>
{:else}
	<div class="frame-panel">
		<!-- Frame Sequence -->
		<div class="timeline-header">
			<span class="label">FRAME SEQUENCE</span>
			<span class="frame-count">{frames.length} frames</span>
			{#if isVideo}
				<div class="legend">
					<span class="lg" style="color:var(--red)">I</span>
					<span class="lg" style="color:var(--cyan)">P</span>
					<span class="lg" style="color:var(--green)">B</span>
				</div>
			{/if}
		</div>

		<div class="timeline-scroll">
			<div class="timeline">
				{#each frames as frame, i}
					{#if isGopBoundary(i)}
						<div class="gop-sep"></div>
					{/if}
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						class="frame-bar"
						class:active={selected?.index === frame.index}
						style="height:{frameHeight(frame)}px; background:{frameBg(frame.frame_type)}; border-color:{frameColor(frame.frame_type)}"
						onclick={() => selected = frame}
						title="{frame.frame_type} #{frame.index} ({frame.size_bytes}B)"
					>
						<span class="frame-label" style="color:{frameColor(frame.frame_type)}">{frame.frame_type}</span>
					</div>
				{/each}
			</div>
		</div>

		<!-- Size Bar Chart -->
		<div class="size-chart">
			<div class="timeline-header"><span class="label">FRAME SIZE</span></div>
			<div class="timeline-scroll">
				<div class="bars">
					{#each frames as frame, i}
						{#if isGopBoundary(i)}
							<div class="gop-sep-sm"></div>
						{/if}
						<div
							class="size-bar"
							style="height:{frameHeight(frame)}px; background:{frameColor(frame.frame_type)}"
						></div>
					{/each}
				</div>
			</div>
		</div>

		<!-- Detail Panel -->
		{#if selected}
			<div class="detail-panel">
				<h4>Frame #{selected.index}</h4>
				<div class="detail-grid">
					<div class="kv"><span class="k">Type</span><span class="v" style="color:{frameColor(selected.frame_type)}">{selected.frame_type}</span></div>
					<div class="kv"><span class="k">Size</span><span class="v">{selected.size_bytes.toLocaleString()} bytes</span></div>
					<div class="kv"><span class="k">Packet Index</span><span class="v">{selected.packet_index}</span></div>
					<div class="kv"><span class="k">PTS</span><span class="v">{formatSec(selected.pts)}</span></div>
					<div class="kv"><span class="k">DTS</span><span class="v">{formatSec(selected.dts)}</span></div>
					{#if selected.pts != null && selected.dts != null}
						<div class="kv"><span class="k">PTS-DTS diff</span><span class="v">{((selected.pts - selected.dts) * 1000).toFixed(2)} ms</span></div>
					{/if}

					{#if selected.info.Video}
						{@const v = selected.info.Video}
						<div class="kv"><span class="k">Codec</span><span class="v">{v.codec}</span></div>
						{#if v.width && v.height}
							<div class="kv"><span class="k">Resolution</span><span class="v">{v.width}x{v.height}</span></div>
						{/if}
						{#if v.profile}
							<div class="kv"><span class="k">Profile</span><span class="v">{v.profile}</span></div>
						{/if}
						{#if v.level}
							<div class="kv"><span class="k">Level</span><span class="v">{v.level}</span></div>
						{/if}
					{/if}

					{#if selected.info.Audio}
						{@const a = selected.info.Audio}
						<div class="kv"><span class="k">Codec</span><span class="v">{a.codec}</span></div>
						<div class="kv"><span class="k">Sample Rate</span><span class="v">{a.sample_rate} Hz</span></div>
						<div class="kv"><span class="k">Channels</span><span class="v">{a.channel_layout}</span></div>
						<div class="kv"><span class="k">Bitrate</span><span class="v">{a.bitrate_kbps} kbps</span></div>
						{#if a.dialog_norm != null}
							<div class="kv"><span class="k">Dialog Norm</span><span class="v">{a.dialog_norm} dB</span></div>
						{/if}
						{#if a.atmos_joc}
							<div class="kv"><span class="k">Atmos JOC</span><span class="v" style="color:var(--magenta)">Detected</span></div>
						{/if}
					{/if}
				</div>
			</div>
		{/if}

		<!-- PTS/DTS Graph -->
		{#if isVideo && frames.some(f => f.pts != null)}
			<div class="timing-section">
				<div class="timeline-header"><span class="label">PTS / DTS TIMING</span></div>
				<div class="timing-table">
					<table>
						<thead><tr><th>#</th><th>Type</th><th>PTS</th><th>DTS</th><th>Diff (ms)</th><th>Frame Interval (ms)</th></tr></thead>
						<tbody>
							{#each frames.slice(0, 100) as frame, i}
								<tr>
									<td class="mono">{frame.index}</td>
									<td style="color:{frameColor(frame.frame_type)}">{frame.frame_type}</td>
									<td class="mono">{formatSec(frame.pts)}</td>
									<td class="mono">{formatSec(frame.dts)}</td>
									<td class="mono">{frame.pts != null && frame.dts != null ? ((frame.pts - frame.dts) * 1000).toFixed(2) : '-'}</td>
									<td class="mono">{i > 0 && frame.pts != null && frames[i-1].pts != null ? ((frame.pts! - frames[i-1].pts!) * 1000).toFixed(2) : '-'}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			</div>
		{/if}
	</div>
{/if}

<style>
	.frame-panel { display: flex; flex-direction: column; gap: 0.6rem; }
	.muted { color: var(--text-dim); font-size: 0.75rem; font-family: var(--font-mono); }

	.timeline-header {
		display: flex; align-items: center; gap: 0.5rem;
	}
	.timeline-header .label {
		font-family: var(--font-mono); font-size: 0.6rem; font-weight: 600;
		color: var(--text-dim); letter-spacing: 0.1em;
	}
	.frame-count {
		font-family: var(--font-mono); font-size: 0.6rem; color: var(--text-muted);
	}
	.legend { display: flex; gap: 0.4rem; margin-left: auto; }
	.lg { font-family: var(--font-mono); font-size: 0.65rem; font-weight: 700; }

	.timeline-scroll { overflow-x: auto; }
	.timeline {
		display: flex; align-items: flex-end; gap: 1px; min-height: 70px; padding: 0.3rem 0;
	}
	.frame-bar {
		width: 10px; min-width: 6px; flex-shrink: 0;
		border: 1px solid; border-radius: 2px 2px 0 0;
		cursor: pointer; display: flex; align-items: flex-start; justify-content: center;
		transition: opacity 0.1s;
	}
	.frame-bar:hover { opacity: 0.8; }
	.frame-bar.active { outline: 2px solid var(--text); outline-offset: 1px; }
	.frame-label { font-family: var(--font-mono); font-size: 0.5rem; font-weight: 700; }
	.gop-sep { width: 2px; background: var(--red); opacity: 0.5; min-height: 70px; flex-shrink: 0; border-radius: 1px; }

	.bars { display: flex; align-items: flex-end; gap: 1px; min-height: 40px; padding: 0.2rem 0; }
	.size-bar { width: 6px; min-width: 4px; flex-shrink: 0; border-radius: 1px 1px 0 0; opacity: 0.7; }
	.gop-sep-sm { width: 1px; background: var(--red); opacity: 0.4; min-height: 40px; flex-shrink: 0; }

	.detail-panel {
		background: var(--bg-inset); border: 1px solid var(--border);
		border-radius: 4px; padding: 0.5rem 0.75rem;
	}
	.detail-panel h4 {
		font-family: var(--font-mono); font-size: 0.7rem; color: var(--accent);
		margin-bottom: 0.4rem; letter-spacing: 0.05em;
	}
	.detail-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 0.1rem 0.5rem; }
	.kv { display: flex; justify-content: space-between; font-size: 0.75rem; padding: 0.15rem 0; border-bottom: 1px solid var(--border); }
	.k { color: var(--text-muted); }
	.v { font-family: var(--font-mono); color: var(--text); }

	.timing-section { margin-top: 0.3rem; }
	.timing-table { overflow: auto; max-height: 400px; border: 1px solid var(--border); border-radius: 4px; }
	.mono { font-family: var(--font-mono); }
</style>

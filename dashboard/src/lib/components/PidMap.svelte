<script lang="ts">
	import type { PidInfo } from '../types';

	let { pids }: { pids: PidInfo[] } = $props();

	function formatBitrate(bps: number): string {
		if (bps >= 1_000_000) return `${(bps / 1_000_000).toFixed(2)}`;
		if (bps >= 1_000) return `${(bps / 1_000).toFixed(1)}k`;
		return `${bps.toFixed(0)}`;
	}

	function pidHex(pid: number): string {
		return `0x${pid.toString(16).toUpperCase().padStart(4, '0')}`;
	}

	function pidColor(pid: PidInfo): string {
		if (pid.label === 'Null') return 'null';
		if (pid.label === 'PAT' || pid.label === 'PMT' || pid.label === 'CAT' ||
			pid.label?.startsWith('NIT') || pid.label?.startsWith('SDT') || pid.label?.startsWith('EIT'))
			return 'psi';
		if (pid.stream_type != null) {
			const st = pid.stream_type;
			if (st === 0x1B || st === 0x24 || st === 0x01 || st === 0x02 || st === 0x10) return 'video';
			if (st === 0x03 || st === 0x04 || st === 0x0F || st === 0x11 || st === 0x81 || st === 0x87) return 'audio';
			if (st === 0x86) return 'scte';
		}
		return 'other';
	}

	function barWidth(pct: number): number {
		return Math.max(pct, 0.3);
	}
</script>

<div class="card">
	<h3>PID Map ({pids.length} PIDs)</h3>
	<div class="table-wrap">
		<table>
			<thead>
				<tr>
					<th style="width:28px"></th>
					<th>PID</th>
					<th>Label</th>
					<th>Type</th>
					<th>Packets</th>
					<th>Share</th>
					<th>Bitrate</th>
					<th>CC</th>
					<th>PCR</th>
				</tr>
			</thead>
			<tbody>
				{#each pids as pid}
				{@const color = pidColor(pid)}
				<tr class="pid-row">
					<td><span class="pid-dot {color}"></span></td>
					<td class="mono"><a href="/pid/{pid.pid}">{pidHex(pid.pid)}</a></td>
					<td class="pid-label">{pid.label}</td>
					<td class="mono dim">{pid.stream_type != null ? `0x${pid.stream_type.toString(16).padStart(2, '0')}` : ''}</td>
					<td class="mono">{pid.packet_count.toLocaleString()}</td>
					<td>
						<div class="share-cell">
							<div class="share-bar">
								<div class="share-fill {color}" style="width: {barWidth(pid.percentage)}%"></div>
							</div>
							<span class="mono share-val">{pid.percentage.toFixed(1)}%</span>
						</div>
					</td>
					<td class="mono">{formatBitrate(pid.bitrate_bps)}</td>
					<td>
						{#if pid.cc_errors > 0}
							<span class="badge badge-error">{pid.cc_errors}</span>
						{:else}
							<span class="cc-ok">0</span>
						{/if}
					</td>
					<td>{#if pid.has_pcr}<span class="led led-green"></span>{/if}</td>
				</tr>
				{/each}
			</tbody>
		</table>
	</div>
</div>

<style>
	.table-wrap { overflow-x: auto; }
	a { color: var(--accent); text-decoration: none; }
	a:hover { text-decoration: underline; }
	.dim { color: var(--text-dim); }
	.pid-label { font-size: 0.78rem; }
	.cc-ok { font-family: var(--font-mono); font-size: 0.72rem; color: var(--text-dim); }

	.pid-dot {
		display: inline-block;
		width: 8px; height: 8px;
		border-radius: 2px;
	}
	.pid-dot.video { background: var(--cyan); box-shadow: var(--glow-cyan); }
	.pid-dot.audio { background: var(--green); box-shadow: var(--glow-green); }
	.pid-dot.psi { background: var(--amber); box-shadow: 0 0 6px rgba(255, 184, 0, 0.3); }
	.pid-dot.scte { background: var(--magenta); box-shadow: 0 0 6px rgba(224, 64, 251, 0.3); }
	.pid-dot.null { background: #2a3040; }
	.pid-dot.other { background: var(--text-dim); }

	.share-cell { display: flex; align-items: center; gap: 0.4rem; min-width: 120px; }
	.share-bar {
		flex: 1; height: 5px;
		background: var(--bg-inset);
		border-radius: 3px;
		overflow: hidden;
	}
	.share-fill {
		height: 100%; border-radius: 2px;
		transition: width 0.3s;
	}
	.share-fill.video { background: var(--cyan); }
	.share-fill.audio { background: var(--green); }
	.share-fill.psi { background: var(--amber); }
	.share-fill.scte { background: var(--magenta); }
	.share-fill.null { background: #2a3040; }
	.share-fill.other { background: var(--text-dim); }
	.share-val { font-size: 0.68rem; color: var(--text-muted); min-width: 36px; text-align: right; }
</style>

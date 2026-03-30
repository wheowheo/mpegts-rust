<script lang="ts">
	import type { PidInfo } from '../types';

	let { pids }: { pids: PidInfo[] } = $props();

	function formatBitrate(bps: number): string {
		if (bps >= 1_000_000) return `${(bps / 1_000_000).toFixed(2)} Mbps`;
		if (bps >= 1_000) return `${(bps / 1_000).toFixed(1)} kbps`;
		return `${bps.toFixed(0)} bps`;
	}

	function pidHex(pid: number): string {
		return `0x${pid.toString(16).toUpperCase().padStart(4, '0')}`;
	}
</script>

<div class="card">
	<h3>PID Map ({pids.length} PIDs)</h3>
	<div class="table-wrap">
		<table>
			<thead>
				<tr>
					<th>PID</th>
					<th>Label</th>
					<th>Type</th>
					<th>Packets</th>
					<th>%</th>
					<th>Bitrate</th>
					<th>CC Err</th>
					<th>PCR</th>
					<th>Scrambled</th>
				</tr>
			</thead>
			<tbody>
				{#each pids as pid}
				<tr>
					<td class="mono"><a href="/pid/{pid.pid}">{pidHex(pid.pid)}</a></td>
					<td>{pid.label}</td>
					<td class="mono">{pid.stream_type != null ? `0x${pid.stream_type.toString(16).padStart(2, '0')}` : '-'}</td>
					<td class="mono">{pid.packet_count.toLocaleString()}</td>
					<td class="mono">{pid.percentage.toFixed(1)}%</td>
					<td class="mono">{formatBitrate(pid.bitrate_bps)}</td>
					<td>
						{#if pid.cc_errors > 0}
							<span class="badge badge-error">{pid.cc_errors}</span>
						{:else}
							<span class="badge badge-success">0</span>
						{/if}
					</td>
					<td>{pid.has_pcr ? 'Y' : '-'}</td>
					<td>{pid.scrambled ? 'Y' : '-'}</td>
				</tr>
				{/each}
			</tbody>
		</table>
	</div>
</div>

<style>
	.table-wrap {
		overflow-x: auto;
	}
	a {
		color: var(--accent);
		text-decoration: none;
	}
	a:hover {
		text-decoration: underline;
	}
</style>

<script lang="ts">
	import type { Scte35Event } from '../types';

	let { events }: { events: Scte35Event[] } = $props();
</script>

<div class="card">
	<h3>SCTE-35 Events ({events.length})</h3>
	{#if events.length === 0}
		<p style="color: var(--text-muted); font-size: 0.85rem;">No SCTE-35 splice events detected.</p>
	{:else}
		<div class="table-wrap">
			<table>
				<thead>
					<tr>
						<th>PTS</th>
						<th>Command</th>
						<th>Event ID</th>
						<th>Duration</th>
						<th>Out of Network</th>
					</tr>
				</thead>
				<tbody>
					{#each events as evt}
					<tr>
						<td class="mono">{evt.pts.toFixed(3)}</td>
						<td>{evt.command_type}</td>
						<td class="mono">{evt.splice_event_id}</td>
						<td class="mono">{evt.duration_ms != null ? `${evt.duration_ms.toFixed(0)}ms` : '-'}</td>
						<td>{evt.out_of_network ? 'Yes' : 'No'}</td>
					</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{/if}
</div>

<style>
	.table-wrap {
		overflow-x: auto;
	}
</style>

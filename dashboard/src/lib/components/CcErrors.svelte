<script lang="ts">
	let { errors }: { errors: { ts: number; pid: number; expected: number; got: number }[] } = $props();

	function pidHex(pid: number): string {
		return `0x${pid.toString(16).toUpperCase().padStart(4, '0')}`;
	}

	function formatTime(ts: number): string {
		const d = new Date(ts);
		return d.toLocaleTimeString();
	}
</script>

<div class="card">
	<h3>CC Errors ({errors.length})</h3>
	{#if errors.length === 0}
		<p style="color: var(--text-muted); font-size: 0.85rem;">No continuity counter errors detected.</p>
	{:else}
		<div class="table-wrap">
			<table>
				<thead>
					<tr>
						<th>Time</th>
						<th>PID</th>
						<th>Expected</th>
						<th>Got</th>
					</tr>
				</thead>
				<tbody>
					{#each errors.slice(-50).reverse() as err}
					<tr>
						<td class="mono">{formatTime(err.ts)}</td>
						<td class="mono">{pidHex(err.pid)}</td>
						<td class="mono">{err.expected}</td>
						<td class="mono error-val">{err.got}</td>
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
		max-height: 300px;
		overflow-y: auto;
	}
	.error-val {
		color: var(--error);
	}
</style>

<script lang="ts">
	let { errors }: { errors: { ts: number; pid: number; expected: number; got: number }[] } = $props();

	function pidHex(pid: number): string {
		return `0x${pid.toString(16).toUpperCase().padStart(4, '0')}`;
	}

	function formatTime(ts: number): string {
		const d = new Date(ts);
		return d.toTimeString().slice(0, 8);
	}
</script>

<div class="card">
	<h3>CC Errors</h3>
	<div class="counter-row">
		<div class="counter-display" class:has-error={errors.length > 0}>
			<span class="seg-number" class:red={errors.length > 0} class:green={errors.length === 0}>
				{errors.length.toString().padStart(4, '0')}
			</span>
		</div>
		<span class="counter-label">TOTAL ERRORS</span>
	</div>

	{#if errors.length > 0}
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
					{#each errors.slice(-30).reverse() as err}
					<tr>
						<td class="mono">{formatTime(err.ts)}</td>
						<td class="mono">{pidHex(err.pid)}</td>
						<td class="mono">{err.expected}</td>
						<td class="mono err-val">{err.got}</td>
					</tr>
					{/each}
				</tbody>
			</table>
		</div>
	{:else}
		<div class="ok-msg">
			<span class="led led-green"></span>
			No continuity errors detected
		</div>
	{/if}
</div>

<style>
	.counter-row {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		margin-bottom: 0.5rem;
	}
	.counter-display {
		background: var(--bg-inset);
		border: 1px solid var(--border);
		border-radius: 4px;
		padding: 0.2rem 0.6rem;
	}
	.counter-display.has-error {
		border-color: rgba(255, 59, 92, 0.3);
	}
	.counter-label {
		font-family: var(--font-mono);
		font-size: 0.6rem;
		color: var(--text-dim);
		letter-spacing: 0.1em;
	}
	.table-wrap {
		overflow: auto;
		max-height: 220px;
		border: 1px solid var(--border);
		border-radius: 4px;
	}
	.err-val { color: var(--red); }
	.ok-msg {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.75rem;
		color: var(--text-muted);
		padding: 0.5rem 0;
	}
</style>

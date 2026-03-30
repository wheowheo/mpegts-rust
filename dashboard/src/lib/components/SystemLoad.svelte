<script lang="ts">
	import type { SystemResponse } from '$lib/types';

	let { data }: { data: SystemResponse | null } = $props();

	function formatBytes(bytes: number): string {
		if (bytes >= 1_073_741_824) return (bytes / 1_073_741_824).toFixed(1) + ' GB';
		if (bytes >= 1_048_576) return (bytes / 1_048_576).toFixed(0) + ' MB';
		return (bytes / 1024).toFixed(0) + ' KB';
	}

	function memoryPct(): number {
		if (!data || data.system.memory_total_bytes === 0) return 0;
		return (data.system.memory_used_bytes / data.system.memory_total_bytes) * 100;
	}
</script>

<div class="system-load">
	<h3>System Resources</h3>

	{#if data}
		<div class="gauge-group">
			<div class="gauge">
				<div class="gauge-label">CPU</div>
				<div class="gauge-bar">
					<div class="gauge-fill" class:warning={data.system.cpu_usage_pct > 70} class:danger={data.system.cpu_usage_pct > 90}
						style="width: {Math.min(data.system.cpu_usage_pct, 100)}%"></div>
				</div>
				<div class="gauge-value">{data.system.cpu_usage_pct.toFixed(1)}%</div>
			</div>

			<div class="gauge">
				<div class="gauge-label">Memory</div>
				<div class="gauge-bar">
					<div class="gauge-fill" class:warning={memoryPct() > 70} class:danger={memoryPct() > 90}
						style="width: {Math.min(memoryPct(), 100)}%"></div>
				</div>
				<div class="gauge-value">{formatBytes(data.system.memory_used_bytes)} / {formatBytes(data.system.memory_total_bytes)}</div>
			</div>
		</div>

		<div class="capacity">
			<div class="capacity-number">{data.capacity.estimated_additional_streams}</div>
			<div class="capacity-label">additional streams possible</div>
			<div class="capacity-detail">
				CPU headroom: {data.capacity.cpu_headroom_pct.toFixed(1)}%
				&middot; Bottleneck: {data.capacity.bottleneck}
			</div>
		</div>
	{:else}
		<div class="no-data">Loading...</div>
	{/if}
</div>

<style>
	.system-load {
		background: var(--card-bg);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 1rem;
	}
	h3 { margin: 0 0 1rem; font-size: 0.9rem; }
	.gauge-group { display: flex; flex-direction: column; gap: 0.75rem; }
	.gauge { display: flex; flex-direction: column; gap: 0.25rem; }
	.gauge-label { font-size: 0.75rem; color: var(--text-muted); }
	.gauge-bar {
		height: 8px;
		background: var(--bg);
		border-radius: 4px;
		overflow: hidden;
	}
	.gauge-fill {
		height: 100%;
		background: #2563eb;
		border-radius: 4px;
		transition: width 0.3s;
	}
	.gauge-fill.warning { background: #f59e0b; }
	.gauge-fill.danger { background: #ef4444; }
	.gauge-value { font-size: 0.75rem; color: var(--text-muted); }
	.capacity {
		margin-top: 1rem;
		padding-top: 1rem;
		border-top: 1px solid var(--border);
		text-align: center;
	}
	.capacity-number {
		font-size: 2rem;
		font-weight: 700;
		color: var(--text);
	}
	.capacity-label {
		font-size: 0.8rem;
		color: var(--text-muted);
		margin-bottom: 0.25rem;
	}
	.capacity-detail {
		font-size: 0.7rem;
		color: var(--text-muted);
	}
	.no-data { color: var(--text-muted); font-size: 0.8rem; }
</style>

<script lang="ts">
	import { onMount } from 'svelte';
	import { fetchTr101290 } from '$lib/api';
	import type { Tr101290Summary, Tr101290Error } from '$lib/types';

	let summary = $state<Tr101290Summary | null>(null);
	let filter = $state<string>('all');
	let loading = $state(true);

	onMount(() => {
		load();
		const iv = setInterval(load, 3000);
		return () => clearInterval(iv);
	});

	async function load() {
		try {
			summary = await fetchTr101290();
		} catch { /* empty */ }
		loading = false;
	}

	let filteredErrors = $derived.by(() => {
		if (!summary) return [];
		if (filter === 'all') return summary.errors.slice(-200);
		return summary.errors.filter(e => e.priority === filter).slice(-200);
	});

	function priorityClass(p: string): string {
		switch (p) {
			case 'P1': return 'p1';
			case 'P2': return 'p2';
			case 'P3': return 'p3';
			default: return '';
		}
	}

	function pidHex(pid: number | null): string {
		if (pid === null) return '-';
		return `0x${pid.toString(16).toUpperCase().padStart(4, '0')}`;
	}
</script>

{#if loading}
	<p class="muted">Loading TR 101 290 data...</p>
{:else if !summary}
	<p class="muted">No data available</p>
{:else}
	<div class="tr-dashboard">
		<!-- Summary Counters -->
		<div class="counter-row">
			<div class="counter-card p1-bg">
				<div class="counter-label">PRIORITY 1</div>
				<div class="counter-value seg-number" class:red={summary.p1_count > 0}>{summary.p1_count}</div>
				<div class="counter-sub">Critical</div>
			</div>
			<div class="counter-card p2-bg">
				<div class="counter-label">PRIORITY 2</div>
				<div class="counter-value seg-number" class:amber={summary.p2_count > 0}>{summary.p2_count}</div>
				<div class="counter-sub">Warning</div>
			</div>
			<div class="counter-card p3-bg">
				<div class="counter-label">PRIORITY 3</div>
				<div class="counter-value seg-number" class:cyan={summary.p3_count > 0}>{summary.p3_count}</div>
				<div class="counter-sub">Info</div>
			</div>
		</div>

		<!-- Error Type Breakdown -->
		<div class="breakdown">
			{#if Object.keys(summary.p1_counters).length > 0}
				<div class="breakdown-group">
					<h4 class="p1-text">P1 Errors</h4>
					{#each Object.entries(summary.p1_counters) as [type, count]}
						<div class="breakdown-row"><span>{type}</span><span class="mono red">{count}</span></div>
					{/each}
				</div>
			{/if}
			{#if Object.keys(summary.p2_counters).length > 0}
				<div class="breakdown-group">
					<h4 class="p2-text">P2 Errors</h4>
					{#each Object.entries(summary.p2_counters) as [type, count]}
						<div class="breakdown-row"><span>{type}</span><span class="mono amber">{count}</span></div>
					{/each}
				</div>
			{/if}
			{#if Object.keys(summary.p3_counters).length > 0}
				<div class="breakdown-group">
					<h4 class="p3-text">P3 Info</h4>
					{#each Object.entries(summary.p3_counters) as [type, count]}
						<div class="breakdown-row"><span>{type}</span><span class="mono cyan">{count}</span></div>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Error Log -->
		<div class="error-log">
			<div class="log-header">
				<span class="label">ERROR LOG</span>
				<div class="filter-buttons">
					<button class:active={filter==='all'} onclick={() => filter='all'}>ALL</button>
					<button class="p1-btn" class:active={filter==='P1'} onclick={() => filter='P1'}>P1</button>
					<button class="p2-btn" class:active={filter==='P2'} onclick={() => filter='P2'}>P2</button>
					<button class="p3-btn" class:active={filter==='P3'} onclick={() => filter='P3'}>P3</button>
				</div>
			</div>
			<div class="log-table">
				<table>
					<thead><tr><th>Pri</th><th>Type</th><th>Description</th><th>PID</th><th>Pkt#</th></tr></thead>
					<tbody>
						{#each filteredErrors as err}
							<tr class={priorityClass(err.priority)}>
								<td><span class="priority-badge {priorityClass(err.priority)}">{err.priority}</span></td>
								<td class="mono">{err.error_type}</td>
								<td>{err.description}</td>
								<td class="mono">{pidHex(err.pid)}</td>
								<td class="mono">{err.packet_index.toLocaleString()}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	</div>
{/if}

<style>
	.tr-dashboard { display: flex; flex-direction: column; gap: 0.6rem; }
	.muted { color: var(--text-dim); font-size: 0.75rem; font-family: var(--font-mono); }

	.counter-row { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 0.5rem; }
	.counter-card {
		background: var(--bg-inset); border: 1px solid var(--border);
		border-radius: 4px; padding: 0.6rem; text-align: center;
	}
	.p1-bg { border-color: rgba(255, 59, 92, 0.3); }
	.p2-bg { border-color: rgba(255, 184, 0, 0.3); }
	.p3-bg { border-color: rgba(0, 212, 255, 0.2); }
	.counter-label { font-family: var(--font-mono); font-size: 0.55rem; color: var(--text-dim); letter-spacing: 0.12em; }
	.counter-value { font-size: 1.5rem; margin: 0.2rem 0; }
	.counter-sub { font-family: var(--font-mono); font-size: 0.55rem; color: var(--text-muted); }

	.red { color: var(--red) !important; text-shadow: var(--glow-red); }
	.amber { color: var(--amber) !important; }
	.cyan { color: var(--cyan) !important; }

	.breakdown { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 0.5rem; }
	.breakdown-group {
		background: var(--bg-inset); border: 1px solid var(--border);
		border-radius: 4px; padding: 0.5rem;
	}
	.breakdown-group h4 { font-family: var(--font-mono); font-size: 0.65rem; letter-spacing: 0.08em; margin-bottom: 0.3rem; }
	.p1-text { color: var(--red); }
	.p2-text { color: var(--amber); }
	.p3-text { color: var(--cyan); }
	.breakdown-row {
		display: flex; justify-content: space-between; font-size: 0.72rem;
		padding: 0.1rem 0; border-bottom: 1px solid var(--border);
	}
	.mono { font-family: var(--font-mono); }

	.error-log { display: flex; flex-direction: column; gap: 0.3rem; }
	.log-header { display: flex; align-items: center; justify-content: space-between; }
	.label { font-family: var(--font-mono); font-size: 0.6rem; font-weight: 600; color: var(--text-dim); letter-spacing: 0.1em; }

	.filter-buttons { display: flex; gap: 2px; }
	.filter-buttons button {
		font-family: var(--font-mono); font-size: 0.55rem;
		background: var(--bg-inset); color: var(--text-muted);
		border: 1px solid var(--border); border-radius: 3px;
		padding: 0.15rem 0.4rem; cursor: pointer;
	}
	.filter-buttons button:hover { border-color: var(--text-muted); }
	.filter-buttons button.active { background: var(--accent-dim); border-color: var(--accent); color: var(--accent); }
	.filter-buttons .p1-btn.active { background: var(--red-dim); border-color: var(--red); color: var(--red); }
	.filter-buttons .p2-btn.active { background: var(--amber-dim); border-color: var(--amber); color: var(--amber); }
	.filter-buttons .p3-btn.active { background: var(--accent-dim); border-color: var(--cyan); color: var(--cyan); }

	.log-table { overflow: auto; max-height: 500px; border: 1px solid var(--border); border-radius: 4px; }
	tr.p1 { background: rgba(255, 59, 92, 0.05); }
	tr.p2 { background: rgba(255, 184, 0, 0.05); }
	tr.p3 { background: transparent; }

	.priority-badge {
		font-family: var(--font-mono); font-size: 0.6rem; font-weight: 700;
		padding: 0.1rem 0.3rem; border-radius: 2px;
	}
	.priority-badge.p1 { background: var(--red-dim); color: var(--red); }
	.priority-badge.p2 { background: var(--amber-dim); color: var(--amber); }
	.priority-badge.p3 { background: var(--accent-dim); color: var(--cyan); }
</style>

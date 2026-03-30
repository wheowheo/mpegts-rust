<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { listOutputs, fetchSystemStats } from '$lib/api';
	import type { OutputStatus, SystemResponse } from '$lib/types';
	import OutputControl from '$lib/components/OutputControl.svelte';
	import SystemLoad from '$lib/components/SystemLoad.svelte';
	import TxBitrateChart from '$lib/components/TxBitrateChart.svelte';

	let sessions = $state<OutputStatus[]>([]);
	let systemData = $state<SystemResponse | null>(null);
	let interval: ReturnType<typeof setInterval>;

	async function refresh() {
		try {
			const [outputs, system] = await Promise.all([
				listOutputs(),
				fetchSystemStats(),
			]);
			sessions = outputs;
			systemData = system;
		} catch (e) {
			console.error('poll error', e);
		}
	}

	onMount(() => {
		refresh();
		interval = setInterval(refresh, 1000);
	});

	onDestroy(() => {
		clearInterval(interval);
	});

	const activeSession = $derived(sessions.find(s => s.running) ?? null);
</script>

<div class="output-page">
	<div class="left">
		<OutputControl {sessions} onUpdate={refresh} />
		<SystemLoad data={systemData} />
	</div>
	<div class="right">
		<TxBitrateChart status={activeSession} />

		{#if sessions.some(s => s.running)}
			<div class="summary-card">
				<h3>All Sessions Summary</h3>
				<div class="stat-grid">
					<div class="stat">
						<span class="stat-label">Active Streams</span>
						<span class="stat-value">{sessions.filter(s => s.running).length}</span>
					</div>
					<div class="stat">
						<span class="stat-label">Total Packets</span>
						<span class="stat-value">
							{sessions.reduce((a, s) => a + s.packets_sent, 0).toLocaleString()}
						</span>
					</div>
					<div class="stat">
						<span class="stat-label">Total Data</span>
						<span class="stat-value">
							{(sessions.reduce((a, s) => a + s.bytes_sent, 0) / 1_048_576).toFixed(1)} MB
						</span>
					</div>
					<div class="stat">
						<span class="stat-label">Total Alerts</span>
						<span class="stat-value" class:has-alert={sessions.some(s => s.alerts.length > 0)}>
							{sessions.reduce((a, s) => a + s.alerts.length, 0)}
						</span>
					</div>
				</div>
			</div>
		{/if}
	</div>
</div>

<style>
	.output-page {
		display: grid;
		grid-template-columns: 340px 1fr;
		gap: 1rem;
	}
	.left, .right {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
	.summary-card {
		background: var(--card-bg);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 1rem;
	}
	.summary-card h3 { margin: 0 0 0.75rem; font-size: 0.9rem; }
	.stat-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 0.75rem;
	}
	.stat { display: flex; flex-direction: column; gap: 0.2rem; }
	.stat-label { font-size: 0.75rem; color: var(--text-muted); }
	.stat-value { font-size: 1rem; font-weight: 600; }
	.stat-value.has-alert { color: #ef4444; }

	@media (max-width: 768px) {
		.output-page { grid-template-columns: 1fr; }
	}
</style>

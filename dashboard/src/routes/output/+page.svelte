<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { fetchOutputStatus, fetchSystemStats } from '$lib/api';
	import type { OutputStatus, SystemResponse } from '$lib/types';
	import OutputControl from '$lib/components/OutputControl.svelte';
	import SystemLoad from '$lib/components/SystemLoad.svelte';
	import TxBitrateChart from '$lib/components/TxBitrateChart.svelte';

	let outputStatus = $state<OutputStatus | null>(null);
	let systemData = $state<SystemResponse | null>(null);
	let interval: ReturnType<typeof setInterval>;

	async function refresh() {
		try {
			const [output, system] = await Promise.all([
				fetchOutputStatus(),
				fetchSystemStats(),
			]);
			outputStatus = output;
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
</script>

<div class="output-page">
	<div class="left">
		<OutputControl status={outputStatus} onUpdate={refresh} />
		<SystemLoad data={systemData} />
	</div>
	<div class="right">
		<TxBitrateChart status={outputStatus} />

		{#if outputStatus?.running}
			<div class="stats-card">
				<h3>Transmission Stats</h3>
				<div class="stat-grid">
					<div class="stat">
						<span class="stat-label">Packets Sent</span>
						<span class="stat-value">{outputStatus.packets_sent.toLocaleString()}</span>
					</div>
					<div class="stat">
						<span class="stat-label">Data Sent</span>
						<span class="stat-value">
							{(outputStatus.bytes_sent / 1_048_576).toFixed(1)} MB
						</span>
					</div>
					<div class="stat">
						<span class="stat-label">Elapsed</span>
						<span class="stat-value">{outputStatus.elapsed_sec.toFixed(1)}s</span>
					</div>
					<div class="stat">
						<span class="stat-label">Bitrate Accuracy</span>
						<span class="stat-value">
							{#if outputStatus.config}
								{((outputStatus.actual_bitrate_bps / outputStatus.config.bitrate_bps) * 100).toFixed(1)}%
							{/if}
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
		grid-template-columns: 320px 1fr;
		gap: 1rem;
	}
	.left {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
	.right {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
	.stats-card {
		background: var(--card-bg);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 1rem;
	}
	.stats-card h3 { margin: 0 0 0.75rem; font-size: 0.9rem; }
	.stat-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 0.75rem;
	}
	.stat { display: flex; flex-direction: column; gap: 0.2rem; }
	.stat-label { font-size: 0.75rem; color: var(--text-muted); }
	.stat-value { font-size: 1rem; font-weight: 600; }

	@media (max-width: 768px) {
		.output-page { grid-template-columns: 1fr; }
	}
</style>

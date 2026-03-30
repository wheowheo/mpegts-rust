<script lang="ts">
	import { onMount } from 'svelte';
	import type { OutputStatus } from '$lib/types';

	let { status }: { status: OutputStatus | null } = $props();

	let canvas: HTMLCanvasElement;
	let chart: any;
	let history = $state<{ t: number; actual: number; target: number }[]>([]);

	$effect(() => {
		if (status?.running) {
			const target = status.config?.bitrate_bps ?? 0;
			history = [
				...history.slice(-120),
				{ t: Date.now(), actual: status.actual_bitrate_bps, target }
			];
			updateChart();
		}
	});

	onMount(async () => {
		const { Chart, registerables } = await import('chart.js');
		Chart.register(...registerables);

		chart = new Chart(canvas, {
			type: 'line',
			data: {
				labels: [],
				datasets: [
					{
						label: 'Actual',
						data: [],
						borderColor: '#2563eb',
						borderWidth: 1.5,
						pointRadius: 0,
						fill: false,
					},
					{
						label: 'Target',
						data: [],
						borderColor: '#6b7280',
						borderWidth: 1,
						borderDash: [4, 4],
						pointRadius: 0,
						fill: false,
					},
				],
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				animation: false,
				scales: {
					x: { display: false },
					y: {
						title: { display: true, text: 'Mbps', font: { size: 10 } },
						ticks: {
							callback: (v: any) => (v / 1_000_000).toFixed(1),
							font: { size: 10 },
						},
					},
				},
				plugins: {
					legend: { labels: { font: { size: 10 } } },
				},
			},
		});
	});

	function updateChart() {
		if (!chart) return;
		chart.data.labels = history.map((_, i) => i);
		chart.data.datasets[0].data = history.map((h) => h.actual);
		chart.data.datasets[1].data = history.map((h) => h.target);
		chart.update();
	}
</script>

<div class="tx-chart">
	<h3>Output Bitrate</h3>
	<div class="chart-wrap">
		<canvas bind:this={canvas}></canvas>
	</div>
</div>

<style>
	.tx-chart {
		background: var(--card-bg);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 1rem;
	}
	h3 { margin: 0 0 0.5rem; font-size: 0.9rem; }
	.chart-wrap { height: 200px; }
</style>

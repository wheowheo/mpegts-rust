<script lang="ts">
	import { onMount } from 'svelte';
	import Chart from 'chart.js/auto';

	let { data }: { data: { ts: number; jitter_ms: number }[] } = $props();

	let canvas: HTMLCanvasElement;
	let chart: Chart | null = null;

	onMount(() => {
		chart = new Chart(canvas, {
			type: 'line',
			data: {
				labels: [],
				datasets: [{
					label: 'PCR Jitter (ms)',
					data: [],
					borderColor: '#f59e0b',
					backgroundColor: 'rgba(245, 158, 11, 0.1)',
					borderWidth: 1.5,
					fill: true,
					tension: 0.3,
					pointRadius: 0,
				}],
			},
			options: {
				responsive: true,
				maintainAspectRatio: false,
				animation: false,
				scales: {
					x: {
						display: true,
						ticks: { color: '#71717a', maxTicksLimit: 8 },
						grid: { color: '#27272a' },
					},
					y: {
						display: true,
						ticks: { color: '#71717a' },
						grid: { color: '#27272a' },
					},
				},
				plugins: {
					legend: { display: false },
				},
			},
		});

		return () => chart?.destroy();
	});

	$effect(() => {
		if (!chart) return;
		chart.data.labels = data.map((d) => {
			const dt = new Date(d.ts);
			return `${dt.getMinutes()}:${dt.getSeconds().toString().padStart(2, '0')}`;
		});
		chart.data.datasets[0].data = data.map((d) => d.jitter_ms);
		chart.update();
	});
</script>

<div class="card">
	<h3>PCR Jitter</h3>
	<div class="chart-container">
		<canvas bind:this={canvas}></canvas>
	</div>
</div>

<style>
	.chart-container {
		height: 200px;
		position: relative;
	}
</style>

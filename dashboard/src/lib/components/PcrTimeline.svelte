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
					label: 'Jitter',
					data: [],
					borderColor: '#ffb800',
					backgroundColor: 'rgba(255, 184, 0, 0.06)',
					borderWidth: 1.5,
					fill: true,
					tension: 0.2,
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
						ticks: { color: '#3a4560', font: { family: 'JetBrains Mono', size: 9 }, maxTicksLimit: 6 },
						grid: { color: '#1a2030', lineWidth: 0.5 },
					},
					y: {
						display: true,
						ticks: {
							color: '#3a4560',
							font: { family: 'JetBrains Mono', size: 9 },
						},
						grid: { color: '#1a2030', lineWidth: 0.5 },
						title: { display: true, text: 'ms', color: '#3a4560', font: { family: 'JetBrains Mono', size: 9 } },
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
	<div class="scope">
		<canvas bind:this={canvas}></canvas>
	</div>
</div>

<style>
	.scope {
		height: 180px;
		position: relative;
		background: var(--bg-inset);
		border: 1px solid var(--border);
		border-radius: 4px;
		padding: 4px;
	}
</style>

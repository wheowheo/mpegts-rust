<script lang="ts">
	import { onMount } from 'svelte';
	import { getStreamStore } from '$lib/stores/stream.svelte';
	import { fetchStreamInfo, fetchPids, uploadFile } from '$lib/api';
	import { createWsConnection } from '$lib/ws';
	import StreamSummary from '$lib/components/StreamSummary.svelte';
	import PidMap from '$lib/components/PidMap.svelte';
	import BitrateChart from '$lib/components/BitrateChart.svelte';
	import PcrTimeline from '$lib/components/PcrTimeline.svelte';
	import PsiViewer from '$lib/components/PsiViewer.svelte';
	import CcErrors from '$lib/components/CcErrors.svelte';

	const store = getStreamStore();
	let fileInput: HTMLInputElement;
	let uploadError = $state('');

	onMount(() => {
		const ws = createWsConnection((data) => {
			store.pushRealtime(data);
		});

		loadInitialData();

		return () => ws.close();
	});

	async function loadInitialData() {
		try {
			const [info, pids] = await Promise.all([fetchStreamInfo(), fetchPids()]);
			store.setStreamInfo(info);
			store.updatePids(pids);
		} catch {
			// server may not have data yet
		}
	}

	async function handleUpload() {
		const file = fileInput?.files?.[0];
		if (!file) return;

		uploadError = '';
		store.setAnalyzing(true);
		store.reset();

		try {
			await uploadFile(file);
			const [info, pids] = await Promise.all([fetchStreamInfo(), fetchPids()]);
			store.setStreamInfo(info);
			store.updatePids(pids);
		} catch (e) {
			uploadError = e instanceof Error ? e.message : 'Upload failed';
		} finally {
			store.setAnalyzing(false);
		}
	}
</script>

<div class="page">
	<div class="upload-section">
		<input type="file" accept=".ts,.mts,.m2ts" bind:this={fileInput} onchange={handleUpload} />
		{#if store.analyzing}
			<span class="analyzing">Analyzing...</span>
		{/if}
		{#if uploadError}
			<span class="error">{uploadError}</span>
		{/if}
	</div>

	<StreamSummary info={store.streamInfo} />

	<div class="charts-row">
		<BitrateChart data={store.bitrateHistory} />
		<PcrTimeline data={store.pcrJitter} />
	</div>

	<PsiViewer info={store.streamInfo} />
	<PidMap pids={store.pids} />
	<CcErrors errors={store.ccErrors} />
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
	.upload-section {
		display: flex;
		align-items: center;
		gap: 1rem;
	}
	.upload-section input[type="file"] {
		font-size: 0.85rem;
	}
	.analyzing {
		color: var(--accent);
		font-size: 0.85rem;
	}
	.error {
		color: var(--error);
		font-size: 0.85rem;
	}
	.charts-row {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1rem;
	}
	@media (max-width: 768px) {
		.charts-row {
			grid-template-columns: 1fr;
		}
	}
</style>

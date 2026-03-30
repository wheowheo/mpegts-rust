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
	let dragging = $state(false);
	let uploadProgress = $state('');

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

	async function processFile(file: File) {
		if (!file.name.match(/\.(ts|mts|m2ts)$/i)) {
			uploadError = 'Unsupported file type. Use .ts, .mts, or .m2ts';
			return;
		}

		uploadError = '';
		store.setAnalyzing(true);
		store.reset();

		const sizeMB = (file.size / 1_048_576).toFixed(1);
		uploadProgress = `Uploading ${file.name} (${sizeMB} MB)...`;

		try {
			await uploadFile(file);
			uploadProgress = 'Analyzing...';
			const [info, pids] = await Promise.all([fetchStreamInfo(), fetchPids()]);
			store.setStreamInfo(info);
			store.updatePids(pids);
			uploadProgress = '';
		} catch (e) {
			uploadError = e instanceof Error ? e.message : 'Upload failed';
			uploadProgress = '';
		} finally {
			store.setAnalyzing(false);
		}
	}

	async function handleUpload() {
		const file = fileInput?.files?.[0];
		if (!file) return;
		await processFile(file);
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		dragging = false;
		const file = e.dataTransfer?.files?.[0];
		if (file) processFile(file);
	}

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		dragging = true;
	}

	function handleDragLeave(e: DragEvent) {
		e.preventDefault();
		dragging = false;
	}

	function handleClick() {
		fileInput?.click();
	}
</script>

<div class="page">
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="dropzone"
		class:dragging
		class:has-data={!!store.streamInfo}
		ondrop={handleDrop}
		ondragover={handleDragOver}
		ondragleave={handleDragLeave}
		onclick={handleClick}
		role="button"
		tabindex="0"
		onkeydown={(e) => e.key === 'Enter' && handleClick()}
	>
		<input
			type="file"
			accept=".ts,.mts,.m2ts"
			bind:this={fileInput}
			onchange={handleUpload}
			class="hidden-input"
		/>

		{#if store.analyzing}
			<div class="drop-content">
				<div class="spinner"></div>
				<span class="drop-text">{uploadProgress || 'Analyzing...'}</span>
			</div>
		{:else if dragging}
			<div class="drop-content">
				<span class="drop-icon">+</span>
				<span class="drop-text">Drop TS file here</span>
			</div>
		{:else}
			<div class="drop-content">
				<span class="drop-icon">&uarr;</span>
				<span class="drop-text">
					Drop MPEG-TS file here or <span class="link">browse</span>
				</span>
				<span class="drop-hint">.ts .mts .m2ts</span>
			</div>
		{/if}

		{#if uploadError}
			<div class="error">{uploadError}</div>
		{/if}
	</div>

	{#if store.streamInfo}
		<StreamSummary info={store.streamInfo} />

		<div class="charts-row">
			<BitrateChart data={store.bitrateHistory} />
			<PcrTimeline data={store.pcrJitter} />
		</div>

		<PsiViewer info={store.streamInfo} />
		<PidMap pids={store.pids} />
		<CcErrors errors={store.ccErrors} />
	{/if}
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.dropzone {
		border: 2px dashed var(--border);
		border-radius: 12px;
		padding: 2rem;
		text-align: center;
		cursor: pointer;
		transition: all 0.2s;
		background: var(--card-bg);
	}

	.dropzone:hover {
		border-color: #2563eb;
		background: color-mix(in srgb, #2563eb 5%, var(--card-bg));
	}

	.dropzone.dragging {
		border-color: #2563eb;
		border-style: solid;
		background: color-mix(in srgb, #2563eb 10%, var(--card-bg));
		transform: scale(1.01);
	}

	.dropzone.has-data {
		padding: 1rem;
	}

	.drop-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
	}

	.drop-icon {
		font-size: 2rem;
		line-height: 1;
		color: #2563eb;
		font-weight: 300;
	}

	.dropzone.has-data .drop-icon {
		font-size: 1.2rem;
	}

	.drop-text {
		font-size: 0.9rem;
		color: var(--text-muted);
	}

	.dropzone.has-data .drop-text {
		font-size: 0.8rem;
	}

	.drop-hint {
		font-size: 0.75rem;
		color: var(--text-muted);
		opacity: 0.6;
	}

	.link {
		color: #2563eb;
		text-decoration: underline;
	}

	.hidden-input {
		display: none;
	}

	.spinner {
		width: 24px;
		height: 24px;
		border: 2px solid var(--border);
		border-top-color: #2563eb;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.error {
		color: #ef4444;
		font-size: 0.8rem;
		margin-top: 0.5rem;
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

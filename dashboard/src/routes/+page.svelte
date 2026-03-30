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
		} catch {}
	}

	async function processFile(file: File) {
		if (!file.name.match(/\.(ts|mts|m2ts)$/i)) {
			uploadError = 'Unsupported format. Accepts .ts .mts .m2ts';
			return;
		}
		uploadError = '';
		store.setAnalyzing(true);
		store.reset();
		const sizeMB = (file.size / 1_048_576).toFixed(1);
		uploadProgress = `${file.name} (${sizeMB} MB)`;
		try {
			await uploadFile(file);
			uploadProgress = '';
			const [info, pids] = await Promise.all([fetchStreamInfo(), fetchPids()]);
			store.setStreamInfo(info);
			store.updatePids(pids);
		} catch (e) {
			uploadError = e instanceof Error ? e.message : 'Upload failed';
			uploadProgress = '';
		} finally {
			store.setAnalyzing(false);
		}
	}

	function handleUpload() {
		const file = fileInput?.files?.[0];
		if (file) processFile(file);
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		dragging = false;
		const file = e.dataTransfer?.files?.[0];
		if (file) processFile(file);
	}
</script>

<div class="page">
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="dropzone"
		class:dragging
		class:compact={!!store.streamInfo}
		ondrop={handleDrop}
		ondragover={(e) => { e.preventDefault(); dragging = true; }}
		ondragleave={(e) => { e.preventDefault(); dragging = false; }}
		onclick={() => fileInput?.click()}
		role="button"
		tabindex="0"
		onkeydown={(e) => e.key === 'Enter' && fileInput?.click()}
	>
		<input type="file" accept=".ts,.mts,.m2ts" bind:this={fileInput} onchange={handleUpload} class="sr-only" />

		{#if store.analyzing}
			<div class="drop-inner">
				<div class="spinner"></div>
				<div class="drop-label">ANALYZING</div>
				<div class="drop-sub">{uploadProgress}</div>
			</div>
		{:else if dragging}
			<div class="drop-inner">
				<div class="drop-icon pulse">+</div>
				<div class="drop-label">DROP FILE</div>
			</div>
		{:else}
			<div class="drop-inner">
				<div class="drop-icon">&uarr;</div>
				<div class="drop-label">INPUT</div>
				<div class="drop-sub">Drop MPEG-TS file or <span class="link">browse</span></div>
				<div class="drop-formats">.ts .mts .m2ts</div>
			</div>
		{/if}
		{#if uploadError}
			<div class="drop-error">{uploadError}</div>
		{/if}
	</div>

	{#if store.streamInfo}
		<StreamSummary info={store.streamInfo} pids={store.pids} />

		<div class="scope-row">
			<BitrateChart data={store.bitrateHistory} />
			<PcrTimeline data={store.pcrJitter} />
		</div>

		<div class="detail-row">
			<div class="detail-left">
				<PsiViewer info={store.streamInfo} />
				<CcErrors errors={store.ccErrors} />
			</div>
			<div class="detail-right">
				<PidMap pids={store.pids} />
			</div>
		</div>
	{/if}
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 0.6rem;
	}

	/* Drop zone - equipment slot style */
	.dropzone {
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: var(--radius);
		padding: 2.5rem 1rem;
		text-align: center;
		cursor: pointer;
		transition: all 0.2s;
		position: relative;
	}
	.dropzone::before {
		content: '';
		position: absolute;
		top: 0; left: 12px; right: 12px;
		height: 1px;
		background: linear-gradient(90deg, transparent, var(--accent), transparent);
		opacity: 0.2;
	}
	.dropzone:hover {
		border-color: rgba(0, 212, 255, 0.3);
	}
	.dropzone.dragging {
		border-color: var(--accent);
		background: rgba(0, 212, 255, 0.05);
	}
	.dropzone.compact {
		padding: 0.8rem 1rem;
	}

	.drop-inner {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.3rem;
	}
	.dropzone.compact .drop-inner {
		flex-direction: row;
		gap: 0.6rem;
	}
	.drop-icon {
		font-size: 1.8rem;
		color: var(--accent);
		font-weight: 300;
		line-height: 1;
	}
	.dropzone.compact .drop-icon { font-size: 1rem; }
	.drop-icon.pulse { animation: pulse 1s ease-in-out infinite; }
	@keyframes pulse { 50% { opacity: 0.4; } }

	.drop-label {
		font-family: var(--font-mono);
		font-size: 0.65rem;
		font-weight: 600;
		color: var(--accent);
		letter-spacing: 0.15em;
	}
	.drop-sub {
		font-size: 0.75rem;
		color: var(--text-muted);
	}
	.drop-formats {
		font-family: var(--font-mono);
		font-size: 0.6rem;
		color: var(--text-dim);
		letter-spacing: 0.1em;
	}
	.link { color: var(--accent); text-decoration: underline; }
	.drop-error {
		color: var(--red);
		font-size: 0.75rem;
		margin-top: 0.4rem;
	}

	.spinner {
		width: 20px; height: 20px;
		border: 2px solid var(--border);
		border-top-color: var(--accent);
		border-radius: 50%;
		animation: spin 0.7s linear infinite;
	}
	@keyframes spin { to { transform: rotate(360deg); } }

	.sr-only {
		position: absolute;
		width: 1px; height: 1px;
		padding: 0; margin: -1px;
		overflow: hidden;
		clip: rect(0,0,0,0);
		border: 0;
	}

	/* Layout grids */
	.scope-row {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 0.6rem;
	}
	.detail-row {
		display: grid;
		grid-template-columns: 380px 1fr;
		gap: 0.6rem;
	}
	.detail-left {
		display: flex;
		flex-direction: column;
		gap: 0.6rem;
	}
	.detail-right {
		display: flex;
		flex-direction: column;
		gap: 0.6rem;
	}

	@media (max-width: 900px) {
		.scope-row { grid-template-columns: 1fr; }
		.detail-row { grid-template-columns: 1fr; }
	}
</style>

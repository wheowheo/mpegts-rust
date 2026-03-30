<script lang="ts">
	import { onMount } from 'svelte';
	import { getStreamStore } from '$lib/stores/stream.svelte';
	import { fetchStreamInfo, fetchPids, uploadFile, fetchTr101290 } from '$lib/api';
	import { createWsConnection } from '$lib/ws';
	import type { Tr101290Summary } from '$lib/types';
	import StreamSummary from '$lib/components/StreamSummary.svelte';
	import PidMap from '$lib/components/PidMap.svelte';
	import BitrateChart from '$lib/components/BitrateChart.svelte';
	import PcrTimeline from '$lib/components/PcrTimeline.svelte';
	import PsiViewer from '$lib/components/PsiViewer.svelte';
	import CcErrors from '$lib/components/CcErrors.svelte';
	import IngestControl from '$lib/components/IngestControl.svelte';
	import Tr101290Dashboard from '$lib/components/Tr101290Dashboard.svelte';

	const store = getStreamStore();
	let fileInput: HTMLInputElement;
	let uploadError = $state('');
	let dragging = $state(false);
	let uploadProgress = $state('');
	let activeTab = $state('transport');
	let tr101290 = $state<Tr101290Summary | null>(null);

	const tabs = [
		{ id: 'transport', label: 'TRANSPORT' },
		{ id: 'programs', label: 'PROGRAMS' },
		{ id: 'pids', label: 'PIDS' },
		{ id: 'timing', label: 'TIMING' },
		{ id: 'errors', label: 'ERRORS' },
		{ id: 'output', label: 'OUTPUT' },
	];

	onMount(() => {
		const ws = createWsConnection((data) => {
			store.pushRealtime(data);
		});
		loadInitialData();
		const iv = setInterval(refreshTr101290, 5000);
		return () => { ws.close(); clearInterval(iv); };
	});

	async function loadInitialData() {
		try {
			const [info, pids] = await Promise.all([fetchStreamInfo(), fetchPids()]);
			store.setStreamInfo(info);
			store.updatePids(pids);
		} catch {}
		refreshTr101290();
	}

	async function refreshTr101290() {
		try { tr101290 = await fetchTr101290(); } catch {}
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
			refreshTr101290();
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

<div class="cma-layout">
	<!-- Left Panel: Stream Selector -->
	<aside class="sidebar">
		<div class="sidebar-section">
			<h4>FILE INPUT</h4>
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="dropzone-mini"
				class:dragging
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
					<div class="spinner-sm"></div>
					<span class="dm-label">ANALYZING...</span>
				{:else}
					<span class="dm-icon">+</span>
					<span class="dm-label">DROP / BROWSE</span>
				{/if}
			</div>
			{#if uploadError}
				<div class="dm-error">{uploadError}</div>
			{/if}
			{#if store.streamInfo?.filename}
				<div class="active-file">{store.streamInfo.filename}</div>
			{/if}
		</div>

		<div class="sidebar-section">
			<h4>LIVE INPUT</h4>
			<IngestControl />
		</div>

		<!-- TR 101 290 Mini Summary -->
		{#if tr101290}
			<div class="sidebar-section">
				<h4>TR 101 290</h4>
				<div class="tr-mini">
					<div class="tr-mini-row">
						<span class="tr-badge p1">P1</span>
						<span class="tr-count" class:has-error={tr101290.p1_count > 0}>{tr101290.p1_count}</span>
					</div>
					<div class="tr-mini-row">
						<span class="tr-badge p2">P2</span>
						<span class="tr-count p2-count" class:has-warning={tr101290.p2_count > 0}>{tr101290.p2_count}</span>
					</div>
					<div class="tr-mini-row">
						<span class="tr-badge p3">P3</span>
						<span class="tr-count">{tr101290.p3_count}</span>
					</div>
				</div>
			</div>
		{/if}
	</aside>

	<!-- Main Content -->
	<main class="main-content">
		{#if store.streamInfo}
			<!-- Top Row: Summary + Scopes -->
			<div class="top-row">
				<div class="scope-panel">
					<StreamSummary info={store.streamInfo} pids={store.pids} />
				</div>
			</div>

			<div class="scope-row">
				<BitrateChart data={store.bitrateHistory} />
				<PcrTimeline data={store.pcrJitter} />
			</div>

			<!-- Tab Bar -->
			<div class="tab-bar">
				{#each tabs as tab}
					<button
						class="tab-btn"
						class:active={activeTab === tab.id}
						onclick={() => activeTab = tab.id}
					>{tab.label}</button>
				{/each}
			</div>

			<!-- Tab Content -->
			<div class="tab-content">
				{#if activeTab === 'transport'}
					<div class="detail-row">
						<CcErrors errors={store.ccErrors} />
					</div>
				{:else if activeTab === 'programs'}
					<PsiViewer info={store.streamInfo} />
				{:else if activeTab === 'pids'}
					<PidMap pids={store.pids} />
				{:else if activeTab === 'timing'}
					<div class="scope-row">
						<BitrateChart data={store.bitrateHistory} />
						<PcrTimeline data={store.pcrJitter} />
					</div>
				{:else if activeTab === 'errors'}
					<Tr101290Dashboard />
				{:else if activeTab === 'output'}
					<p class="muted">Output control available on the <a href="/output">OUTPUT</a> page</p>
				{/if}
			</div>
		{:else}
			<div class="empty-state">
				<div class="empty-icon">&#x25B6;</div>
				<div class="empty-label">NO STREAM LOADED</div>
				<div class="empty-sub">Upload a file or start live ingest from the sidebar</div>
			</div>
		{/if}
	</main>
</div>

<style>
	.cma-layout {
		display: grid;
		grid-template-columns: 260px 1fr;
		gap: 0;
		min-height: calc(100vh - 60px);
	}

	/* Sidebar */
	.sidebar {
		background: var(--bg-panel);
		border-right: 1px solid var(--border);
		padding: 0.6rem;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		overflow-y: auto;
	}
	.sidebar-section {
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: 4px;
		padding: 0.5rem;
	}
	.sidebar-section h4 {
		font-family: var(--font-mono);
		font-size: 0.55rem;
		font-weight: 600;
		color: var(--text-dim);
		letter-spacing: 0.12em;
		margin-bottom: 0.4rem;
	}

	.dropzone-mini {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		padding: 0.5rem;
		background: var(--bg-inset);
		border: 1px dashed var(--border);
		border-radius: 4px;
		cursor: pointer;
		transition: border-color 0.15s;
	}
	.dropzone-mini:hover, .dropzone-mini.dragging {
		border-color: var(--accent);
	}
	.dm-icon { color: var(--accent); font-size: 1rem; font-weight: 300; }
	.dm-label { font-family: var(--font-mono); font-size: 0.6rem; color: var(--text-muted); letter-spacing: 0.08em; }
	.dm-error { font-size: 0.6rem; color: var(--red); margin-top: 0.2rem; }
	.active-file {
		font-family: var(--font-mono); font-size: 0.6rem;
		color: var(--green); margin-top: 0.3rem;
		padding: 0.2rem 0.4rem;
		background: var(--green-dim);
		border-radius: 3px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.spinner-sm {
		width: 14px; height: 14px;
		border: 2px solid var(--border);
		border-top-color: var(--accent);
		border-radius: 50%;
		animation: spin 0.7s linear infinite;
	}
	@keyframes spin { to { transform: rotate(360deg); } }
	.sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0,0,0,0); border: 0; }

	/* TR 101 290 mini */
	.tr-mini { display: flex; flex-direction: column; gap: 0.2rem; }
	.tr-mini-row { display: flex; align-items: center; justify-content: space-between; }
	.tr-badge {
		font-family: var(--font-mono); font-size: 0.55rem; font-weight: 700;
		padding: 0.1rem 0.3rem; border-radius: 2px;
	}
	.tr-badge.p1 { background: var(--red-dim); color: var(--red); }
	.tr-badge.p2 { background: var(--amber-dim); color: var(--amber); }
	.tr-badge.p3 { background: var(--accent-dim); color: var(--cyan); }
	.tr-count { font-family: var(--font-mono); font-size: 0.85rem; font-weight: 700; color: var(--text-dim); }
	.tr-count.has-error { color: var(--red); text-shadow: var(--glow-red); }
	.tr-count.has-warning { color: var(--amber); }

	/* Main Content */
	.main-content {
		padding: 0.6rem;
		display: flex;
		flex-direction: column;
		gap: 0.6rem;
		overflow-y: auto;
	}

	.top-row { display: flex; gap: 0.6rem; }
	.scope-panel { flex: 1; }

	.scope-row {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 0.6rem;
	}

	/* Tabs */
	.tab-bar {
		display: flex;
		gap: 0;
		border-bottom: 1px solid var(--border);
	}
	.tab-btn {
		font-family: var(--font-mono);
		font-size: 0.62rem;
		font-weight: 500;
		color: var(--text-muted);
		background: transparent;
		border: none;
		border-bottom: 2px solid transparent;
		padding: 0.4rem 0.7rem;
		cursor: pointer;
		letter-spacing: 0.06em;
		transition: all 0.15s;
	}
	.tab-btn:hover { color: var(--text); }
	.tab-btn.active {
		color: var(--accent);
		border-bottom-color: var(--accent);
	}

	.tab-content {
		min-height: 200px;
	}

	.detail-row { display: flex; flex-direction: column; gap: 0.6rem; }

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		min-height: 400px;
		gap: 0.5rem;
	}
	.empty-icon { font-size: 2rem; color: var(--text-dim); }
	.empty-label { font-family: var(--font-mono); font-size: 0.8rem; color: var(--text-dim); letter-spacing: 0.12em; }
	.empty-sub { font-size: 0.75rem; color: var(--text-muted); }
	.muted { color: var(--text-muted); font-size: 0.75rem; }
	.muted a { color: var(--accent); }

	@media (max-width: 900px) {
		.cma-layout { grid-template-columns: 1fr; }
		.sidebar { border-right: none; border-bottom: 1px solid var(--border); }
		.scope-row { grid-template-columns: 1fr; }
	}
</style>

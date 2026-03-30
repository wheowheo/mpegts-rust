<script lang="ts">
	import { onMount } from 'svelte';
	import { startIngest, stopIngest, fetchIngestStatus } from '$lib/api';
	import type { IngestStatus } from '$lib/types';

	let status = $state<IngestStatus | null>(null);
	let url = $state('udp://239.1.1.1:5000');
	let protocol = $state('');
	let error = $state('');
	let loading = $state(false);

	const protocols = [
		{ value: '', label: 'Auto-detect' },
		{ value: 'udp', label: 'UDP' },
		{ value: 'rtp', label: 'RTP' },
		{ value: 'http', label: 'HTTP/HLS' },
	];

	const presets = [
		'udp://239.1.1.1:5000',
		'rtp://239.1.1.1:5004',
	];

	onMount(() => {
		refresh();
		const iv = setInterval(refresh, 2000);
		return () => clearInterval(iv);
	});

	async function refresh() {
		try {
			status = await fetchIngestStatus();
		} catch { /* empty */ }
	}

	async function handleStart() {
		error = '';
		loading = true;
		try {
			status = await startIngest(url, protocol || undefined);
		} catch (e: any) {
			error = e.message;
		}
		loading = false;
	}

	async function handleStop() {
		loading = true;
		try {
			status = await stopIngest();
		} catch (e: any) {
			error = e.message;
		}
		loading = false;
	}
</script>

<div class="ingest-control">
	{#if status?.running}
		<div class="status-card running">
			<div class="status-header">
				<span class="led led-green"></span>
				<span class="status-label">RECEIVING</span>
			</div>
			<div class="status-details">
				<div class="kv"><span class="k">URL</span><span class="v mono">{status.url}</span></div>
				<div class="kv"><span class="k">Protocol</span><span class="v">{status.protocol.toUpperCase()}</span></div>
				<div class="kv"><span class="k">Packets</span><span class="v seg-number green">{status.packets_received.toLocaleString()}</span></div>
			</div>
			<button class="btn btn-stop" onclick={handleStop} disabled={loading}>STOP</button>
		</div>
	{:else}
		<div class="input-form">
			<div class="form-row">
				<label class="form-label">Stream URL</label>
				<input type="text" bind:value={url} placeholder="udp://239.x.x.x:port" class="form-input" />
			</div>
			<div class="form-row">
				<label class="form-label">Protocol</label>
				<select bind:value={protocol} class="form-select">
					{#each protocols as p}
						<option value={p.value}>{p.label}</option>
					{/each}
				</select>
			</div>
			<div class="presets">
				{#each presets as p}
					<button class="preset-btn" onclick={() => url = p}>{p}</button>
				{/each}
			</div>
			<button class="btn btn-start" onclick={handleStart} disabled={loading}>
				{loading ? 'STARTING...' : 'START INGEST'}
			</button>
		</div>
	{/if}

	{#if error}
		<div class="error">{error}</div>
	{/if}
</div>

<style>
	.ingest-control { display: flex; flex-direction: column; gap: 0.5rem; }

	.status-card {
		background: var(--bg-inset); border: 1px solid var(--border);
		border-radius: 4px; padding: 0.75rem;
	}
	.status-card.running { border-color: rgba(0, 255, 136, 0.3); }
	.status-header { display: flex; align-items: center; gap: 0.4rem; margin-bottom: 0.5rem; }
	.status-label {
		font-family: var(--font-mono); font-size: 0.7rem; font-weight: 600;
		color: var(--green); letter-spacing: 0.1em;
	}

	.status-details { display: flex; flex-direction: column; gap: 0.2rem; margin-bottom: 0.5rem; }
	.kv { display: flex; justify-content: space-between; font-size: 0.75rem; }
	.k { color: var(--text-muted); }
	.v { font-family: var(--font-mono); }
	.mono { font-family: var(--font-mono); }

	.input-form { display: flex; flex-direction: column; gap: 0.4rem; }
	.form-row { display: flex; flex-direction: column; gap: 0.15rem; }
	.form-label {
		font-family: var(--font-mono); font-size: 0.6rem; font-weight: 600;
		color: var(--text-dim); letter-spacing: 0.1em;
	}
	.form-input, .form-select {
		font-family: var(--font-mono); font-size: 0.75rem;
		background: var(--bg-inset); color: var(--text);
		border: 1px solid var(--border); border-radius: 4px;
		padding: 0.4rem 0.5rem;
	}
	.form-input:focus, .form-select:focus {
		outline: none; border-color: var(--accent);
	}

	.presets { display: flex; gap: 0.3rem; }
	.preset-btn {
		font-family: var(--font-mono); font-size: 0.55rem;
		background: var(--bg-card); color: var(--text-muted);
		border: 1px solid var(--border); border-radius: 3px;
		padding: 0.15rem 0.4rem; cursor: pointer;
	}
	.preset-btn:hover { border-color: var(--accent); color: var(--accent); }

	.btn {
		font-family: var(--font-mono); font-size: 0.72rem; font-weight: 600;
		padding: 0.4rem 1rem; border-radius: 4px; cursor: pointer;
		border: 1px solid; letter-spacing: 0.05em;
	}
	.btn-start { background: var(--green-dim); color: var(--green); border-color: rgba(0,255,136,0.3); }
	.btn-start:hover { border-color: var(--green); }
	.btn-stop { background: var(--red-dim); color: var(--red); border-color: rgba(255,59,92,0.3); }
	.btn-stop:hover { border-color: var(--red); }
	.btn:disabled { opacity: 0.5; cursor: not-allowed; }

	.error {
		font-family: var(--font-mono); font-size: 0.7rem;
		color: var(--red); padding: 0.3rem;
	}
</style>

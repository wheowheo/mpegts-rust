<script lang="ts">
	import { startOutput, stopOutput } from '$lib/api';
	import type { OutputStatus } from '$lib/types';

	let { status, onUpdate }: { status: OutputStatus | null; onUpdate: () => void } = $props();

	let destAddr = $state('239.1.1.1');
	let destPort = $state(5000);
	let protocol = $state<'udp' | 'rtp'>('udp');
	let bitrateMbps = $state(5);
	let sourcePath = $state('');
	let loading = $state(false);
	let error = $state('');

	async function handleStart() {
		if (!sourcePath) {
			error = 'source file path required';
			return;
		}
		loading = true;
		error = '';
		try {
			await startOutput({
				source_type: 'file',
				source_path: sourcePath,
				dest_addr: destAddr,
				dest_port: destPort,
				protocol,
				bitrate_bps: bitrateMbps * 1_000_000,
			});
			onUpdate();
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	}

	async function handleStop() {
		loading = true;
		try {
			await stopOutput();
			onUpdate();
		} finally {
			loading = false;
		}
	}

	function formatBps(bps: number): string {
		if (bps >= 1_000_000) return (bps / 1_000_000).toFixed(2) + ' Mbps';
		if (bps >= 1_000) return (bps / 1_000).toFixed(1) + ' Kbps';
		return bps.toFixed(0) + ' bps';
	}
</script>

<div class="output-control">
	<h3>Output Control</h3>

	{#if status?.running}
		<div class="status-card running">
			<div class="status-label">STREAMING</div>
			<div class="stat-row">
				<span>Protocol</span>
				<strong>{status.config?.protocol.toUpperCase()}</strong>
			</div>
			<div class="stat-row">
				<span>Destination</span>
				<strong>{status.config?.dest_addr}:{status.config?.dest_port}</strong>
			</div>
			<div class="stat-row">
				<span>Packets Sent</span>
				<strong>{status.packets_sent.toLocaleString()}</strong>
			</div>
			<div class="stat-row">
				<span>Actual Bitrate</span>
				<strong>{formatBps(status.actual_bitrate_bps)}</strong>
			</div>
			<div class="stat-row">
				<span>Elapsed</span>
				<strong>{status.elapsed_sec.toFixed(1)}s</strong>
			</div>
			<button class="btn stop" onclick={handleStop} disabled={loading}>Stop</button>
		</div>
	{:else}
		<div class="form">
			<label>
				Source File
				<input type="text" bind:value={sourcePath} placeholder="/path/to/file.ts" />
			</label>
			<label>
				Destination
				<div class="row">
					<input type="text" bind:value={destAddr} placeholder="239.1.1.1" />
					<input type="number" bind:value={destPort} min="1" max="65535" />
				</div>
			</label>
			<label>
				Protocol
				<select bind:value={protocol}>
					<option value="udp">UDP</option>
					<option value="rtp">RTP</option>
				</select>
			</label>
			<label>
				Bitrate (Mbps)
				<input type="number" bind:value={bitrateMbps} min="0.1" max="100" step="0.1" />
			</label>
			{#if error}
				<div class="error">{error}</div>
			{/if}
			<button class="btn start" onclick={handleStart} disabled={loading}>
				{loading ? 'Starting...' : 'Start Output'}
			</button>
		</div>
	{/if}
</div>

<style>
	.output-control {
		background: var(--card-bg);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 1rem;
	}
	h3 { margin: 0 0 1rem; font-size: 0.9rem; }
	.form {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}
	label {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		font-size: 0.8rem;
		color: var(--text-muted);
	}
	input, select {
		background: var(--bg);
		border: 1px solid var(--border);
		border-radius: 4px;
		padding: 0.4rem 0.5rem;
		color: var(--text);
		font-size: 0.85rem;
	}
	.row {
		display: flex;
		gap: 0.5rem;
	}
	.row input:first-child { flex: 1; }
	.row input:last-child { width: 80px; }
	.btn {
		padding: 0.5rem;
		border: none;
		border-radius: 4px;
		font-size: 0.85rem;
		cursor: pointer;
		font-weight: 600;
	}
	.btn.start { background: #2563eb; color: white; }
	.btn.stop { background: #dc2626; color: white; margin-top: 0.5rem; }
	.btn:disabled { opacity: 0.5; }
	.status-card { display: flex; flex-direction: column; gap: 0.5rem; }
	.status-card.running { }
	.status-label {
		font-size: 0.75rem;
		font-weight: 700;
		color: #22c55e;
		letter-spacing: 0.05em;
	}
	.stat-row {
		display: flex;
		justify-content: space-between;
		font-size: 0.8rem;
	}
	.stat-row span { color: var(--text-muted); }
	.error { color: #ef4444; font-size: 0.8rem; }
</style>

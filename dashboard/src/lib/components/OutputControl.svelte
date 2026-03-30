<script lang="ts">
	import { startOutput, stopOutput } from '$lib/api';
	import type { OutputStatus } from '$lib/types';

	let { sessions, onUpdate }: { sessions: OutputStatus[]; onUpdate: () => void } = $props();

	let destAddr = $state('239.1.1.1');
	let destPort = $state(5000);
	let protocol = $state<'udp' | 'rtp'>('udp');
	let bitrateMbps = $state(5);
	let sourcePath = $state('');
	let sessionId = $state('');
	let loading = $state(false);
	let error = $state('');

	async function handleStart() {
		if (!sourcePath) { error = 'source file path required'; return; }
		loading = true;
		error = '';
		try {
			await startOutput({
				session_id: sessionId || undefined,
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

	async function handleStop(id: string) {
		await stopOutput(id);
		onUpdate();
	}

	async function handleStopAll() {
		await stopOutput();
		onUpdate();
	}

	function formatBps(bps: number): string {
		if (bps >= 1_000_000) return (bps / 1_000_000).toFixed(2) + ' Mbps';
		if (bps >= 1_000) return (bps / 1_000).toFixed(1) + ' Kbps';
		return bps.toFixed(0) + ' bps';
	}

	const runningSessions = $derived(sessions.filter(s => s.running));
</script>

<div class="output-control">
	<h3>New Output</h3>
	<div class="form">
		<label>
			Session ID (optional)
			<input type="text" bind:value={sessionId} placeholder="auto" />
		</label>
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
		<div class="row">
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
		</div>
		{#if error}
			<div class="error">{error}</div>
		{/if}
		<button class="btn start" onclick={handleStart} disabled={loading}>
			{loading ? 'Starting...' : 'Start Output'}
		</button>
	</div>

	{#if runningSessions.length > 0}
		<div class="sessions">
			<div class="sessions-header">
				<h3>Active Sessions ({runningSessions.length})</h3>
				{#if runningSessions.length > 1}
					<button class="btn-sm stop" onclick={handleStopAll}>Stop All</button>
				{/if}
			</div>
			{#each runningSessions as s}
				<div class="session-card">
					<div class="session-header">
						<span class="session-id">{s.session_id}</span>
						<button class="btn-sm stop" onclick={() => handleStop(s.session_id)}>Stop</button>
					</div>
					<div class="stat-row">
						<span>{s.config?.protocol.toUpperCase()} &rarr; {s.config?.dest_addr}:{s.config?.dest_port}</span>
					</div>
					<div class="stat-row">
						<span>Sent</span>
						<strong>{s.packets_sent.toLocaleString()} pkts</strong>
					</div>
					<div class="stat-row">
						<span>Bitrate</span>
						<strong>{formatBps(s.actual_bitrate_bps)}</strong>
					</div>
					{#if s.alerts.length > 0}
						{#each s.alerts as alert}
							<div class="alert" class:critical={alert.level === 'critical'}>
								{alert.message}
							</div>
						{/each}
					{/if}
				</div>
			{/each}
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
	h3 { margin: 0 0 0.75rem; font-size: 0.9rem; }
	.form { display: flex; flex-direction: column; gap: 0.6rem; }
	label {
		display: flex; flex-direction: column; gap: 0.2rem;
		font-size: 0.8rem; color: var(--text-muted);
	}
	input, select {
		background: var(--bg); border: 1px solid var(--border);
		border-radius: 4px; padding: 0.35rem 0.5rem;
		color: var(--text); font-size: 0.85rem;
	}
	.row { display: flex; gap: 0.5rem; }
	.row input:first-child, .row label:first-child { flex: 1; }
	.row input:last-child { width: 80px; }
	.btn {
		padding: 0.45rem; border: none; border-radius: 4px;
		font-size: 0.85rem; cursor: pointer; font-weight: 600;
	}
	.btn.start { background: #2563eb; color: white; }
	.btn:disabled { opacity: 0.5; }
	.btn-sm {
		padding: 0.2rem 0.5rem; border: none; border-radius: 3px;
		font-size: 0.75rem; cursor: pointer;
	}
	.btn-sm.stop { background: #dc2626; color: white; }
	.sessions { margin-top: 1rem; border-top: 1px solid var(--border); padding-top: 0.75rem; }
	.sessions-header { display: flex; justify-content: space-between; align-items: center; }
	.session-card {
		margin-top: 0.5rem; padding: 0.5rem;
		border: 1px solid var(--border); border-radius: 6px;
		display: flex; flex-direction: column; gap: 0.3rem;
	}
	.session-header { display: flex; justify-content: space-between; align-items: center; }
	.session-id { font-weight: 600; font-size: 0.85rem; color: #22c55e; }
	.stat-row { display: flex; justify-content: space-between; font-size: 0.75rem; }
	.stat-row span { color: var(--text-muted); }
	.alert {
		font-size: 0.7rem; padding: 0.25rem 0.4rem;
		border-radius: 3px; background: #fef3c7; color: #92400e;
	}
	.alert.critical { background: #fecaca; color: #991b1b; }
	.error { color: #ef4444; font-size: 0.8rem; }
</style>

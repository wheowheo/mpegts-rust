<script lang="ts">
	import { onMount } from 'svelte';
	import { fetchHistorySessions, fetchHistoryStats, deleteHistorySession, type SessionRecord, type HistoryStats } from '$lib/api';

	let sessions = $state<SessionRecord[]>([]);
	let stats = $state<HistoryStats | null>(null);
	let loading = $state(true);

	onMount(async () => {
		await load();
	});

	async function load() {
		loading = true;
		try {
			[sessions, stats] = await Promise.all([
				fetchHistorySessions(),
				fetchHistoryStats(),
			]);
		} catch { /* empty */ }
		loading = false;
	}

	async function handleDelete(id: string) {
		await deleteHistorySession(id);
		await load();
	}

	function formatBps(bps: number): string {
		if (bps >= 1e6) return (bps / 1e6).toFixed(2) + ' Mbps';
		if (bps >= 1e3) return (bps / 1e3).toFixed(1) + ' kbps';
		return bps.toFixed(0) + ' bps';
	}

	function formatDuration(ms: number | null): string {
		if (ms === null) return '-';
		const sec = ms / 1000;
		if (sec < 60) return sec.toFixed(1) + 's';
		return (sec / 60).toFixed(1) + 'min';
	}

	function formatDate(iso: string): string {
		return new Date(iso).toLocaleString();
	}
</script>

<div class="page">
	{#if stats}
		<div class="stats-row">
			<div class="stat-card">
				<div class="stat-label">TOTAL SESSIONS</div>
				<div class="stat-value seg-number cyan">{stats.total_sessions}</div>
			</div>
			<div class="stat-card">
				<div class="stat-label">TOTAL ERRORS</div>
				<div class="stat-value seg-number" class:red={stats.total_errors > 0}>{stats.total_errors.toLocaleString()}</div>
			</div>
			<div class="stat-card">
				<div class="stat-label">TOTAL PACKETS</div>
				<div class="stat-value seg-number green">{stats.total_packets.toLocaleString()}</div>
			</div>
		</div>
	{/if}

	<div class="card">
		<h3>Session History</h3>
		{#if loading}
			<p class="muted">Loading...</p>
		{:else if sessions.length === 0}
			<p class="muted">No sessions recorded yet. Analyze a file to create a session.</p>
		{:else}
			<div class="table-scroll">
				<table>
					<thead>
						<tr>
							<th>Date</th>
							<th>Filename</th>
							<th>Duration</th>
							<th>Packets</th>
							<th>Bitrate</th>
							<th>P1</th>
							<th>P2</th>
							<th>P3</th>
							<th></th>
						</tr>
					</thead>
					<tbody>
						{#each sessions as s}
							<tr>
								<td class="mono">{formatDate(s.start_time)}</td>
								<td class="filename">{s.filename}</td>
								<td class="mono">{formatDuration(s.duration_ms)}</td>
								<td class="mono">{s.total_packets.toLocaleString()}</td>
								<td class="mono">{formatBps(s.bitrate_bps)}</td>
								<td class="mono" class:red={s.p1_errors > 0}>{s.p1_errors}</td>
								<td class="mono" class:amber={s.p2_errors > 0}>{s.p2_errors}</td>
								<td class="mono">{s.p3_errors}</td>
								<td>
									<button class="del-btn" onclick={() => handleDelete(s.id)}>x</button>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</div>
</div>

<style>
	.page { display: flex; flex-direction: column; gap: 0.6rem; }

	.stats-row { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 0.5rem; }
	.stat-card {
		background: var(--bg-card); border: 1px solid var(--border);
		border-radius: var(--radius); padding: 0.6rem; text-align: center;
	}
	.stat-label { font-family: var(--font-mono); font-size: 0.55rem; color: var(--text-dim); letter-spacing: 0.1em; }
	.stat-value { font-size: 1.3rem; margin-top: 0.2rem; }

	.table-scroll { overflow: auto; max-height: 600px; }
	.mono { font-family: var(--font-mono); }
	.filename { max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
	.red { color: var(--red) !important; }
	.amber { color: var(--amber) !important; }
	.muted { color: var(--text-dim); font-size: 0.75rem; font-family: var(--font-mono); }

	.del-btn {
		font-family: var(--font-mono); font-size: 0.6rem;
		background: transparent; color: var(--text-dim);
		border: 1px solid var(--border); border-radius: 3px;
		padding: 0.1rem 0.4rem; cursor: pointer;
	}
	.del-btn:hover { color: var(--red); border-color: var(--red); }
</style>

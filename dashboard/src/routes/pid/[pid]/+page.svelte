<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { fetchPidDetail } from '$lib/api';
	import type { PidInfo } from '$lib/types';

	let pidInfo = $state<PidInfo | null>(null);
	let error = $state('');

	const pidParam = $derived(Number(page.params.pid));

	onMount(() => {
		loadPid();
	});

	async function loadPid() {
		try {
			pidInfo = await fetchPidDetail(pidParam);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load';
		}
	}

	function formatBitrate(bps: number): string {
		if (bps >= 1_000_000) return `${(bps / 1_000_000).toFixed(2)} Mbps`;
		if (bps >= 1_000) return `${(bps / 1_000).toFixed(1)} kbps`;
		return `${bps.toFixed(0)} bps`;
	}

	function pidHex(pid: number): string {
		return `0x${pid.toString(16).toUpperCase().padStart(4, '0')}`;
	}
</script>

<div class="page">
	<a href="/" class="back">&larr; Back to Dashboard</a>

	{#if error}
		<div class="card">
			<p style="color: var(--error)">{error}</p>
		</div>
	{:else if pidInfo}
		<h2>{pidInfo.label} <span class="mono">{pidHex(pidInfo.pid)}</span></h2>

		<div class="detail-grid">
			<div class="card stat">
				<h3>Stream Type</h3>
				<span class="mono">{pidInfo.stream_type != null ? `0x${pidInfo.stream_type.toString(16).padStart(2, '0')}` : '-'}</span>
			</div>
			<div class="card stat">
				<h3>Packet Count</h3>
				<span class="value">{pidInfo.packet_count.toLocaleString()}</span>
			</div>
			<div class="card stat">
				<h3>Percentage</h3>
				<span class="value">{pidInfo.percentage.toFixed(2)}%</span>
			</div>
			<div class="card stat">
				<h3>Bitrate</h3>
				<span class="value">{formatBitrate(pidInfo.bitrate_bps)}</span>
			</div>
			<div class="card stat">
				<h3>CC Errors</h3>
				<span class="value" class:error-val={pidInfo.cc_errors > 0}>{pidInfo.cc_errors}</span>
			</div>
			<div class="card stat">
				<h3>Has PCR</h3>
				<span class="value">{pidInfo.has_pcr ? 'Yes' : 'No'}</span>
			</div>
			<div class="card stat">
				<h3>Scrambled</h3>
				<span class="value">{pidInfo.scrambled ? 'Yes' : 'No'}</span>
			</div>
		</div>
	{:else}
		<div class="card">
			<p style="color: var(--text-muted)">Loading...</p>
		</div>
	{/if}
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
	.back {
		color: var(--accent);
		text-decoration: none;
		font-size: 0.85rem;
	}
	.back:hover {
		text-decoration: underline;
	}
	h2 {
		font-size: 1.25rem;
	}
	.detail-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 0.75rem;
	}
	.stat {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}
	.value {
		font-size: 1.25rem;
		font-weight: 600;
	}
	.error-val {
		color: var(--error);
	}
</style>

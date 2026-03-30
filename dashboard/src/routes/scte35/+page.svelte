<script lang="ts">
	import Scte35Log from '$lib/components/Scte35Log.svelte';
	import type { Scte35Event } from '$lib/types';

	// TODO: fetch from /api/scte35 when endpoint is available
	let events = $state<Scte35Event[]>([]);
</script>

<div class="page">
	<h2>SCTE-35 Events</h2>
	<Scte35Log {events} />
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
	h2 {
		font-size: 1.25rem;
	}
</style>

<script lang="ts">
	import Tr101290Dashboard from '$lib/components/Tr101290Dashboard.svelte';
</script>

<div class="page">
	<div class="card">
		<h2>TR 101 290 Monitoring</h2>
		<Tr101290Dashboard />
	</div>
</div>

<style>
	.page { display: flex; flex-direction: column; gap: 0.6rem; }
	h2 { font-family: var(--font-mono); font-size: 0.85rem; color: var(--accent); letter-spacing: 0.08em; margin-bottom: 0.3rem; }
</style>

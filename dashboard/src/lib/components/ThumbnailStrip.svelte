<script lang="ts">
	import { onMount } from 'svelte';
	import { fetchPidThumbnails, thumbnailUrl, type ThumbnailInfo } from '$lib/api';

	let { pid }: { pid: number } = $props();

	let thumbs = $state<ThumbnailInfo[]>([]);
	let loading = $state(true);

	onMount(async () => {
		try {
			thumbs = await fetchPidThumbnails(pid);
		} catch { /* empty */ }
		loading = false;
	});

	function formatPts(v: number | null): string {
		if (v === null) return '';
		return v.toFixed(3) + 's';
	}
</script>

{#if loading}
	<p class="muted">Loading thumbnails...</p>
{:else if thumbs.length === 0}
	<p class="muted">No thumbnails available (requires ffmpeg for actual decoding)</p>
{:else}
	<div class="strip">
		{#each thumbs as thumb}
			<div class="thumb-card">
				<img src={thumbnailUrl(pid, thumb.index)} alt="Frame {thumb.frame_index}" width={thumb.width} height={thumb.height} />
				<div class="thumb-info">
					<span>#{thumb.frame_index}</span>
					<span>{formatPts(thumb.pts)}</span>
				</div>
			</div>
		{/each}
	</div>
{/if}

<style>
	.muted { color: var(--text-dim); font-size: 0.75rem; font-family: var(--font-mono); }
	.strip {
		display: flex; gap: 0.4rem; overflow-x: auto; padding: 0.3rem 0;
	}
	.thumb-card {
		flex-shrink: 0; border: 1px solid var(--border); border-radius: 4px;
		overflow: hidden; background: var(--bg-inset);
	}
	.thumb-card img {
		display: block; width: 120px; height: auto; object-fit: cover;
		image-rendering: pixelated;
	}
	.thumb-info {
		display: flex; justify-content: space-between; padding: 0.15rem 0.3rem;
		font-family: var(--font-mono); font-size: 0.55rem; color: var(--text-muted);
	}
</style>

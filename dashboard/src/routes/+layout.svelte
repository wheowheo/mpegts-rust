<script>
	import '../app.css';
	import { onMount } from 'svelte';

	let { children } = $props();
	let clock = $state('');
	let connected = $state(false);

	onMount(() => {
		const tick = () => {
			const d = new Date();
			clock = d.toTimeString().slice(0, 8);
		};
		tick();
		const iv = setInterval(tick, 1000);

		// connection check
		const check = async () => {
			try {
				const r = await fetch('/api/stream');
				connected = r.ok;
			} catch { connected = false; }
		};
		check();
		const cv = setInterval(check, 5000);

		return () => { clearInterval(iv); clearInterval(cv); };
	});
</script>

<div class="shell">
	<header>
		<div class="header-inner">
			<div class="brand-group">
				<div class="brand">TS-ENGINE</div>
				<div class="brand-sub">MPEG-TS ANALYZER</div>
			</div>
			<nav>
				<a href="/">ANALYZE</a>
				<a href="/output">OUTPUT</a>
				<a href="/scte35">SCTE-35</a>
				<a href="/errors">TR 101 290</a>
				<a href="/history">HISTORY</a>
			</nav>
			<div class="status-group">
				<div class="status-item">
					<span class="led" class:led-green={connected} class:led-off={!connected}></span>
					<span class="status-label">{connected ? 'ONLINE' : 'OFFLINE'}</span>
				</div>
				<div class="clock">{clock}</div>
			</div>
		</div>
	</header>
	<main>
		{@render children()}
	</main>
</div>

<style>
	.shell {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
	}
	header {
		background: var(--bg-panel);
		border-bottom: 1px solid var(--border);
		padding: 0 1rem;
		position: sticky;
		top: 0;
		z-index: 100;
	}
	.header-inner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: 44px;
		max-width: 1600px;
		margin: 0 auto;
		width: 100%;
	}
	.brand-group {
		display: flex;
		align-items: baseline;
		gap: 0.6rem;
	}
	.brand {
		font-family: var(--font-mono);
		font-weight: 700;
		font-size: 0.9rem;
		color: var(--accent);
		letter-spacing: 0.1em;
		text-shadow: var(--glow-cyan);
	}
	.brand-sub {
		font-family: var(--font-mono);
		font-size: 0.55rem;
		color: var(--text-dim);
		letter-spacing: 0.15em;
		text-transform: uppercase;
	}
	nav {
		display: flex;
		gap: 0;
	}
	nav a {
		font-family: var(--font-mono);
		font-size: 0.7rem;
		font-weight: 500;
		color: var(--text-muted);
		text-decoration: none;
		padding: 0.3rem 0.8rem;
		letter-spacing: 0.06em;
		border: 1px solid transparent;
		border-radius: 3px;
		transition: all 0.15s;
	}
	nav a:hover {
		color: var(--accent);
		background: var(--accent-dim);
		border-color: rgba(0, 212, 255, 0.2);
	}
	.status-group {
		display: flex;
		align-items: center;
		gap: 1rem;
	}
	.status-item {
		display: flex;
		align-items: center;
		gap: 0.35rem;
	}
	.status-label {
		font-family: var(--font-mono);
		font-size: 0.6rem;
		color: var(--text-muted);
		letter-spacing: 0.08em;
	}
	.clock {
		font-family: var(--font-mono);
		font-size: 0.75rem;
		font-weight: 600;
		color: var(--text-dim);
		letter-spacing: 0.05em;
	}
	main {
		flex: 1;
		padding: 1rem;
		max-width: 1600px;
		margin: 0 auto;
		width: 100%;
	}
</style>

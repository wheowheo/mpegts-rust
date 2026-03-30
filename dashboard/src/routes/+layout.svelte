<script>
	import '../app.css';

	let { children } = $props();
</script>

<div class="layout">
	<header>
		<nav>
			<a href="/" class="brand">TS Engine</a>
			<div class="nav-links">
				<a href="/">Dashboard</a>
				<a href="/output">Output</a>
				<a href="/scte35">SCTE-35</a>
			</div>
		</nav>
	</header>
	<main>
		{@render children()}
	</main>
</div>

<style>
	.layout {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
	}
	header {
		border-bottom: 1px solid var(--border);
		padding: 0 1.5rem;
	}
	nav {
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: 48px;
		max-width: 1400px;
		margin: 0 auto;
		width: 100%;
	}
	.brand {
		font-weight: 700;
		font-size: 1rem;
		color: var(--text);
		text-decoration: none;
	}
	.nav-links {
		display: flex;
		gap: 1.5rem;
	}
	.nav-links a {
		color: var(--text-muted);
		text-decoration: none;
		font-size: 0.85rem;
	}
	.nav-links a:hover {
		color: var(--text);
	}
	main {
		flex: 1;
		padding: 1.5rem;
		max-width: 1400px;
		margin: 0 auto;
		width: 100%;
	}
</style>

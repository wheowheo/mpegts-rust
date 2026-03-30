<script lang="ts">
	import type { StreamInfo } from '../types';

	let { info }: { info: StreamInfo | null } = $props();

	function pidHex(pid: number): string {
		return `0x${pid.toString(16).toUpperCase().padStart(4, '0')}`;
	}

	function typeHex(t: number): string {
		return `0x${t.toString(16).toUpperCase().padStart(2, '0')}`;
	}

	function streamIcon(codec: string | null): string {
		if (!codec) return '';
		if (codec.includes('H.26') || codec.includes('MPEG')) return 'V';
		if (codec.includes('AC-3') || codec.includes('AAC') || codec.includes('Audio')) return 'A';
		return 'D';
	}

	function streamColor(codec: string | null): string {
		const icon = streamIcon(codec);
		if (icon === 'V') return 'video';
		if (icon === 'A') return 'audio';
		return 'other';
	}
</script>

{#if info && info.programs.length > 0}
<div class="card">
	<h3>PSI / Program Structure</h3>
	<div class="programs">
		{#each info.programs as prog}
		<div class="program-block">
			<div class="prog-header">
				<span class="prog-num">PROGRAM {prog.program_number}</span>
				<span class="prog-pmt">PMT {pidHex(prog.pmt_pid)}</span>
			</div>
			<div class="stream-list">
				{#each prog.streams as stream}
				<div class="stream-entry">
					<span class="stream-icon {streamColor(stream.codec)}">{streamIcon(stream.codec)}</span>
					<span class="stream-pid mono">{pidHex(stream.pid)}</span>
					<span class="stream-type">{stream.stream_type_name}</span>
					<span class="stream-codec">{stream.codec ?? ''}</span>
					<span class="stream-hex mono">{typeHex(stream.stream_type)}</span>
				</div>
				{/each}
			</div>
		</div>
		{/each}
	</div>
</div>
{/if}

<style>
	.programs { display: flex; flex-direction: column; gap: 0.5rem; }
	.program-block {
		background: var(--bg-inset);
		border: 1px solid var(--border);
		border-radius: 4px;
		overflow: hidden;
	}
	.prog-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.4rem 0.6rem;
		background: var(--bg-panel);
		border-bottom: 1px solid var(--border);
	}
	.prog-num {
		font-family: var(--font-mono);
		font-size: 0.7rem;
		font-weight: 600;
		color: var(--accent);
		letter-spacing: 0.08em;
	}
	.prog-pmt {
		font-family: var(--font-mono);
		font-size: 0.65rem;
		color: var(--text-dim);
	}
	.stream-list { padding: 0.3rem 0; }
	.stream-entry {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.3rem 0.6rem;
		font-size: 0.78rem;
	}
	.stream-entry:hover { background: var(--bg-hover); }
	.stream-icon {
		width: 18px; height: 18px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 3px;
		font-family: var(--font-mono);
		font-size: 0.6rem;
		font-weight: 700;
	}
	.stream-icon.video { background: var(--accent-dim); color: var(--cyan); border: 1px solid rgba(0, 212, 255, 0.3); }
	.stream-icon.audio { background: var(--green-dim); color: var(--green); border: 1px solid rgba(0, 255, 136, 0.3); }
	.stream-icon.other { background: rgba(100, 100, 120, 0.15); color: var(--text-muted); border: 1px solid var(--border); }
	.stream-pid { color: var(--text); min-width: 50px; }
	.stream-type { color: var(--text-muted); flex: 1; }
	.stream-codec { font-weight: 600; color: var(--text); min-width: 50px; }
	.stream-hex { color: var(--text-dim); font-size: 0.65rem; }
</style>

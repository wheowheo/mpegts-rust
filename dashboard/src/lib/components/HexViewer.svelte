<script lang="ts">
	import { onMount } from 'svelte';
	import { fetchPidPackets, type PacketHex } from '$lib/api';

	let { pid }: { pid: number } = $props();

	let packets = $state<PacketHex[]>([]);
	let selectedIdx = $state(0);
	let loading = $state(true);
	let hoverByte = $state<number | null>(null);

	const BYTES_PER_ROW = 16;

	// TS packet field regions
	interface Region {
		start: number;
		end: number;
		name: string;
		cls: string;
		detail: (bytes: number[]) => string;
	}

	function getRegions(bytes: number[]): Region[] {
		const regions: Region[] = [];

		// sync byte
		regions.push({ start: 0, end: 1, name: 'Sync Byte', cls: 'r-sync', detail: () => '0x47' });

		// TS header bytes 1-3
		if (bytes.length >= 4) {
			const pid = ((bytes[1] & 0x1F) << 8) | bytes[2];
			const tei = (bytes[1] & 0x80) !== 0;
			const pusi = (bytes[1] & 0x40) !== 0;
			const cc = bytes[3] & 0x0F;
			const afc = (bytes[3] >> 4) & 0x03;
			regions.push({
				start: 1, end: 4, name: 'TS Header',
				cls: 'r-header',
				detail: () => `PID=0x${pid.toString(16).toUpperCase().padStart(4,'0')} TEI=${tei?1:0} PUSI=${pusi?1:0} AFC=${afc} CC=${cc}`
			});
		}

		if (bytes.length <= 4) return regions;

		const afc = (bytes[3] >> 4) & 0x03;
		let payloadStart = 4;

		// adaptation field
		if (afc === 2 || afc === 3) {
			const afLen = bytes[4];
			const afEnd = 5 + afLen;
			regions.push({
				start: 4, end: Math.min(afEnd, bytes.length), name: 'Adaptation Field',
				cls: 'r-adapt',
				detail: () => `length=${afLen}`
			});

			// PCR within adaptation field
			if (afLen >= 7 && bytes.length >= 11 && (bytes[5] & 0x10)) {
				regions.push({
					start: 6, end: 12, name: 'PCR',
					cls: 'r-pcr',
					detail: () => {
						const base = ((bytes[6] << 25) | (bytes[7] << 17) | (bytes[8] << 9) | (bytes[9] << 1) | (bytes[10] >> 7)) >>> 0;
						const ext = ((bytes[10] & 0x01) << 8) | bytes[11];
						return `base=${base} ext=${ext}`;
					}
				});
			}
			payloadStart = Math.min(afEnd, bytes.length);
		}

		// payload
		if ((afc === 1 || afc === 3) && payloadStart < bytes.length) {
			// PES start code detection
			if (payloadStart + 3 <= bytes.length &&
				bytes[payloadStart] === 0x00 && bytes[payloadStart+1] === 0x00 && bytes[payloadStart+2] === 0x01) {
				regions.push({
					start: payloadStart, end: payloadStart + 3, name: 'PES Start Code',
					cls: 'r-pes-start',
					detail: () => '00 00 01'
				});
				if (payloadStart + 9 <= bytes.length) {
					const streamId = bytes[payloadStart + 3];
					const pesLen = (bytes[payloadStart+4] << 8) | bytes[payloadStart+5];
					const hdrLen = bytes[payloadStart + 8];
					const pesHdrEnd = Math.min(payloadStart + 9 + hdrLen, bytes.length);
					regions.push({
						start: payloadStart + 3, end: pesHdrEnd, name: 'PES Header',
						cls: 'r-pes-hdr',
						detail: () => `stream_id=0x${streamId.toString(16).toUpperCase()} len=${pesLen} hdr_len=${hdrLen}`
					});
					regions.push({
						start: pesHdrEnd, end: bytes.length, name: 'Payload',
						cls: 'r-payload',
						detail: () => `${bytes.length - pesHdrEnd} bytes`
					});
				}
			} else {
				regions.push({
					start: payloadStart, end: bytes.length, name: 'Payload',
					cls: 'r-payload',
					detail: () => `${bytes.length - payloadStart} bytes`
				});
			}
		}

		return regions;
	}

	function getByteRegion(regions: Region[], idx: number): Region | null {
		for (const r of regions) {
			if (idx >= r.start && idx < r.end) return r;
		}
		return null;
	}

	function regionClass(regions: Region[], idx: number): string {
		const r = getByteRegion(regions, idx);
		return r ? r.cls : '';
	}

	let selectedBytes = $derived.by(() => {
		const pkt = packets[selectedIdx];
		if (!pkt) return [];
		return pkt.hex.split(' ').map(h => parseInt(h, 16));
	});

	let regions = $derived(getRegions(selectedBytes));

	let tooltip = $derived.by(() => {
		if (hoverByte === null) return '';
		const r = getByteRegion(regions, hoverByte);
		if (!r) return `Byte ${hoverByte}: 0x${selectedBytes[hoverByte]?.toString(16).toUpperCase().padStart(2,'0') ?? '??'}`;
		return `${r.name}: ${r.detail(selectedBytes)}`;
	});

	onMount(async () => {
		try {
			packets = await fetchPidPackets(pid);
		} catch { /* empty */ }
		loading = false;
	});

	function rows(bytes: number[]): { addr: number; hex: string[]; ascii: string }[] {
		const result = [];
		for (let i = 0; i < bytes.length; i += BYTES_PER_ROW) {
			const chunk = bytes.slice(i, i + BYTES_PER_ROW);
			result.push({
				addr: i,
				hex: chunk.map(b => b.toString(16).toUpperCase().padStart(2, '0')),
				ascii: chunk.map(b => (b >= 0x20 && b <= 0x7E) ? String.fromCharCode(b) : '.').join('')
			});
		}
		return result;
	}
</script>

{#if loading}
	<p class="muted">Loading packets...</p>
{:else if packets.length === 0}
	<p class="muted">No packets captured</p>
{:else}
	<div class="hex-viewer">
		<div class="pkt-selector">
			<span class="label">PACKET</span>
			<div class="pkt-buttons">
				{#each packets as pkt, i}
					<button class="pkt-btn" class:active={i === selectedIdx} onclick={() => selectedIdx = i}>
						{pkt.index}
					</button>
				{/each}
			</div>
		</div>

		{#if tooltip}
			<div class="tooltip-bar">{tooltip}</div>
		{/if}

		<div class="legend">
			<span class="lg r-sync">Sync</span>
			<span class="lg r-header">Header</span>
			<span class="lg r-adapt">Adaptation</span>
			<span class="lg r-pcr">PCR</span>
			<span class="lg r-pes-start">PES Start</span>
			<span class="lg r-pes-hdr">PES Header</span>
			<span class="lg r-payload">Payload</span>
		</div>

		<div class="hex-table" role="grid" onmouseleave={() => hoverByte = null}>
			<div class="hex-row hex-hdr">
				<span class="addr">ADDR</span>
				<span class="hex-cols">
					{#each Array(BYTES_PER_ROW) as _, i}
						<span class="col-hdr">{i.toString(16).toUpperCase().padStart(2, '0')}</span>
					{/each}
				</span>
				<span class="ascii-hdr">ASCII</span>
			</div>
			{#each rows(selectedBytes) as row}
				<div class="hex-row">
					<span class="addr">{row.addr.toString(16).toUpperCase().padStart(4, '0')}</span>
					<span class="hex-cols">
						{#each row.hex as byte, i}
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<span
								class="hex-byte {regionClass(regions, row.addr + i)}"
								class:hover={hoverByte === row.addr + i}
								onmouseenter={() => hoverByte = row.addr + i}
							>{byte}</span>
						{/each}
						{#if row.hex.length < BYTES_PER_ROW}
							{#each Array(BYTES_PER_ROW - row.hex.length) as _}
								<span class="hex-byte empty">  </span>
							{/each}
						{/if}
					</span>
					<span class="ascii">
						{#each row.ascii.split('') as ch, i}
							<span class="{regionClass(regions, row.addr + i)}">{ch}</span>
						{/each}
					</span>
				</div>
			{/each}
		</div>
	</div>
{/if}

<style>
	.hex-viewer { display: flex; flex-direction: column; gap: 0.4rem; }

	.pkt-selector { display: flex; align-items: center; gap: 0.5rem; }
	.pkt-selector .label {
		font-family: var(--font-mono); font-size: 0.6rem; font-weight: 600;
		color: var(--text-dim); letter-spacing: 0.1em;
	}
	.pkt-buttons { display: flex; gap: 2px; flex-wrap: wrap; }
	.pkt-btn {
		font-family: var(--font-mono); font-size: 0.6rem;
		background: var(--bg-inset); color: var(--text-muted);
		border: 1px solid var(--border); border-radius: 3px;
		padding: 0.15rem 0.35rem; cursor: pointer;
	}
	.pkt-btn:hover { border-color: var(--accent); color: var(--accent); }
	.pkt-btn.active { background: var(--accent-dim); border-color: var(--accent); color: var(--accent); }

	.tooltip-bar {
		font-family: var(--font-mono); font-size: 0.7rem;
		background: var(--bg-panel); border: 1px solid var(--border-bright);
		border-radius: 3px; padding: 0.25rem 0.5rem; color: var(--text);
		min-height: 1.6rem;
	}

	.legend { display: flex; gap: 0.5rem; flex-wrap: wrap; }
	.lg {
		font-family: var(--font-mono); font-size: 0.55rem; font-weight: 600;
		padding: 0.1rem 0.3rem; border-radius: 2px;
		letter-spacing: 0.05em;
	}

	.hex-table {
		font-family: var(--font-mono); font-size: 0.72rem;
		background: var(--bg-inset); border: 1px solid var(--border);
		border-radius: 4px; overflow: auto; max-height: 500px;
	}
	.hex-row { display: flex; align-items: center; padding: 0 0.4rem; line-height: 1.6; }
	.hex-row:hover { background: var(--bg-hover); }
	.hex-hdr { color: var(--text-dim); border-bottom: 1px solid var(--border); font-size: 0.6rem; }
	.addr { width: 3.5rem; color: var(--text-dim); flex-shrink: 0; }
	.hex-cols { display: flex; gap: 0.3rem; flex: 1; }
	.col-hdr { width: 1.6rem; text-align: center; }
	.hex-byte { width: 1.6rem; text-align: center; border-radius: 2px; cursor: crosshair; }
	.hex-byte.hover { outline: 1px solid var(--text); }
	.hex-byte.empty { cursor: default; }
	.ascii { color: var(--text-muted); letter-spacing: 0.05em; margin-left: 0.5rem; white-space: pre; }
	.ascii-hdr { margin-left: 0.5rem; }
	.muted { color: var(--text-dim); font-size: 0.75rem; font-family: var(--font-mono); }

	/* region colors */
	.r-sync { background: rgba(255, 59, 92, 0.3); color: var(--red); }
	.r-header { background: rgba(0, 212, 255, 0.2); color: var(--cyan); }
	.r-adapt { background: rgba(255, 184, 0, 0.2); color: var(--amber); }
	.r-pcr { background: rgba(224, 64, 251, 0.25); color: var(--magenta); }
	.r-pes-start { background: rgba(0, 255, 136, 0.25); color: var(--green); }
	.r-pes-hdr { background: rgba(180, 255, 0, 0.2); color: #b4ff00; }
	.r-payload { background: transparent; color: var(--text); }
</style>

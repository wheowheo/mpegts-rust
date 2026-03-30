<script lang="ts">
	import type { StreamInfo } from '../types';

	let { info }: { info: StreamInfo | null } = $props();
</script>

{#if info && info.programs.length > 0}
<div class="card">
	<h3>PSI - Programs</h3>
	{#each info.programs as prog}
	<div class="program">
		<div class="program-header">
			Program {prog.program_number} <span class="mono">(PMT PID: 0x{prog.pmt_pid.toString(16).toUpperCase().padStart(4, '0')})</span>
		</div>
		<table>
			<thead>
				<tr>
					<th>PID</th>
					<th>Stream Type</th>
					<th>Codec</th>
				</tr>
			</thead>
			<tbody>
				{#each prog.streams as stream}
				<tr>
					<td class="mono">0x{stream.pid.toString(16).toUpperCase().padStart(4, '0')}</td>
					<td>{stream.stream_type_name} <span class="mono">(0x{stream.stream_type.toString(16).padStart(2, '0')})</span></td>
					<td>{stream.codec ?? '-'}</td>
				</tr>
				{/each}
			</tbody>
		</table>
	</div>
	{/each}
</div>
{/if}

<style>
	.program {
		margin-bottom: 1rem;
	}
	.program-header {
		font-weight: 600;
		margin-bottom: 0.5rem;
		font-size: 0.9rem;
	}
</style>

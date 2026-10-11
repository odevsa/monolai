<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		forId = '',
		required = false,
		hint = '',
		class: className = '',
		children
	}: {
		forId?: string;
		required?: boolean;
		hint?: string;
		class?: string;
		children?: Snippet;
	} = $props();
</script>

<div class="flex items-center justify-between gap-2 mb-1.5 {className}">
	<label
		for={forId}
		class="text-xs font-semibold text-[var(--text-primary)] select-none flex items-center gap-1"
	>
		{@render children?.()}
		{#if required}
			<span class="text-red-400 font-bold" title="Required">*</span>
		{/if}
	</label>
	{#if hint}
		<span class="text-[11px] text-[var(--text-muted)] font-normal">{hint}</span>
	{/if}
</div>

<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title = undefined,
		description = undefined,
		children
	}: {
		title?: string;
		description?: string;
		children: Snippet;
	} = $props();
</script>

<div class="flex flex-col gap-1.5 w-full">
	{#if title}
		<h5
			class="m-0 text-[11px] font-semibold tracking-wider uppercase text-[var(--text-muted)] px-3 select-none"
		>
			{title}
		</h5>
	{/if}

	<div
		class="w-full bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-2xl divide-y divide-[var(--border-color)]/60 shadow-2xs"
	>
		{@render children()}
	</div>

	{#if description}
		<p class="m-0 text-[11px] text-[var(--text-muted)] px-3 leading-relaxed">
			{description}
		</p>
	{/if}
</div>

<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title = '',
		description = '',
		icon: IconComponent,
		class: className = '',
		actions
	}: {
		title: string;
		description?: string;
		icon?: any;
		class?: string;
		actions?: Snippet;
	} = $props();
</script>

<div
	class="flex flex-col gap-4 items-center justify-center p-8 sm:p-12 text-center rounded-2xl border border-dashed border-[var(--border-color)] bg-[var(--bg-surface)]/40 {className}"
>
	{#if IconComponent}
		<div
			class="w-12 h-12 rounded-2xl bg-[var(--bg-surface)] text-[var(--text-muted)] flex items-center justify-center border border-[var(--border-color)]"
		>
			<IconComponent size={24} strokeWidth={1.5} />
		</div>
	{/if}

	<div class="flex flex-col gap-1">
		<h4 class="text-sm sm:text-base font-bold text-[var(--text-primary)]">{title}</h4>
		
		{#if description}
			<p class="text-xs text-[var(--text-muted)] max-w-sm leading-relaxed">
				{description}
			</p>
		{/if}
	</div>

	{#if actions}
		<div class="mt-4 flex items-center gap-3">
			{@render actions()}
		</div>
	{/if}
</div>

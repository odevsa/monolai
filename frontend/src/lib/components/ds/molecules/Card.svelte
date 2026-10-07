<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		variant = 'surface',
		padding = 'md',
		title = '',
		class: className = '',
		header,
		actions,
		footer,
		children
	}: {
		variant?: 'surface' | 'glass';
		padding?: 'none' | 'sm' | 'md' | 'lg';
		title?: string;
		class?: string;
		header?: Snippet;
		actions?: Snippet;
		footer?: Snippet;
		children?: Snippet;
	} = $props();

	const paddingClasses = {
		none: 'p-0',
		sm: 'p-3.5',
		md: 'p-5',
		lg: 'p-6'
	};

	const variantClasses = {
		surface: 'bg-[var(--bg-surface)]',
		glass: 'bg-[var(--bg-surface)]/60 backdrop-blur-md'
	};
</script>

<div
	class="rounded-2xl flex flex-col transition-all duration-150 {variantClasses[
		variant
	]} {paddingClasses[padding]} {className}"
>
	{#if title || header || actions}
		<div class="flex items-center justify-between gap-3 mb-4 shrink-0">
			{#if header}
				{@render header()}
			{:else if title}
				<h3 class="m-0 text-sm font-bold text-[var(--text-primary)] leading-tight">{title}</h3>
			{/if}

			{#if actions}
				<div class="flex items-center gap-2 shrink-0">
					{@render actions()}
				</div>
			{/if}
		</div>
	{/if}

	<div class="flex-1 min-w-0">
		{@render children?.()}
	</div>

	{#if footer}
		<div class="mt-4 pt-3.5 border-t border-[var(--border-color)] shrink-0">
			{@render footer()}
		</div>
	{/if}
</div>

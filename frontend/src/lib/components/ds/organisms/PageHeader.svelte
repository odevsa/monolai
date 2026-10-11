<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title,
		subtitle,
		icon: IconComponent = null,
		iconSize = 38,
		children
	}: {
		title: string;
		subtitle?: string;
		icon?: any;
		iconSize?: number;
		children?: Snippet;
	} = $props();
</script>

<header class="flex items-center justify-between gap-4 mb-6 flex-wrap shrink-0">
	<div class="flex items-center gap-3.5 min-w-0">
		{#if IconComponent}
			<IconComponent size={iconSize > 48 ? 48 : iconSize} strokeWidth={1.75} />
		{/if}
		<div class="min-w-0">
			<h2
				class="text-lg sm:text-xl font-bold tracking-tight text-[var(--text-primary)] m-0 truncate"
			>
				{title}
			</h2>
			{#if subtitle}
				<p class="text-xs text-[var(--text-muted)] m-0 mt-0.5 truncate">{subtitle}</p>
			{/if}
		</div>
	</div>

	{#if children}
		<div class="flex items-center gap-2.5 shrink-0 flex-wrap">
			{@render children()}
		</div>
	{/if}
</header>

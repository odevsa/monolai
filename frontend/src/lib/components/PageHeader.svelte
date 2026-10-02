<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title,
		subtitle,
		icon: IconComponent = null,
		iconSize = 48,
		children
	}: {
		title: string;
		subtitle?: string;
		icon?: any;
		iconSize?: number;
		children?: Snippet;
	} = $props();
</script>

<header class="page-header">
	<div class="flex items-center gap-4">
		{#if IconComponent}
			<IconComponent size={iconSize} strokeWidth={1} class="text-white shrink-0" />
		{/if}
		<div>
			<h2 class="page-title">{title}</h2>
			{#if subtitle}
				<p class="page-subtitle">{subtitle}</p>
			{/if}
		</div>
	</div>
	{#if children}
		<div class="flex items-center gap-2">
			{@render children()}
		</div>
	{/if}
</header>

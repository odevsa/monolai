<script lang="ts">
	import type { Snippet } from 'svelte';
	import { PageHeader } from './ds';

	let {
		title,
		subtitle,
		icon = null,
		iconSize,
		children
	}: {
		title: string;
		subtitle?: string;
		icon?: any;
		iconSize?: number;
		children?: Snippet;
	} = $props();
</script>

<PageHeader {title} {subtitle} {icon} {iconSize}>
	{#if children}
		{@render children()}
	{/if}
</PageHeader>

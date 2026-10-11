<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		level = 2,
		class: className = '',
		children
	}: {
		level?: 1 | 2 | 3 | 4;
		class?: string;
		children?: Snippet;
	} = $props();

	const levelClasses = {
		1: 'text-2xl font-bold tracking-tight text-[var(--text-primary)]',
		2: 'text-lg font-bold text-[var(--text-primary)]',
		3: 'text-base font-semibold text-[var(--text-primary)]',
		4: 'text-sm font-semibold text-[var(--text-primary)]'
	};
</script>

{#if level === 1}
	<h1 class="{levelClasses[1]} {className}">{@render children?.()}</h1>
{:else if level === 2}
	<h2 class="{levelClasses[2]} {className}">{@render children?.()}</h2>
{:else if level === 3}
	<h3 class="{levelClasses[3]} {className}">{@render children?.()}</h3>
{:else}
	<h4 class="{levelClasses[4]} {className}">{@render children?.()}</h4>
{/if}

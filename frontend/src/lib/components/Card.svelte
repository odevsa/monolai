<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		class: className = '',
		hover = false,
		children
	}: {
		class?: string;
		hover?: boolean;
		children?: Snippet;
	} = $props();
</script>

<div class="app-card {hover ? 'app-card-hover' : ''} {className}">
	{@render children?.()}
</div>

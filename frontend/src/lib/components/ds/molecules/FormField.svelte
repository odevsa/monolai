<script lang="ts">
	import type { Snippet } from 'svelte';
	import Label from '../atoms/Label.svelte';

	let {
		label = '',
		forId = '',
		required = false,
		hint = '',
		error = '',
		class: className = '',
		children
	}: {
		label?: string;
		forId?: string;
		required?: boolean;
		hint?: string;
		error?: string;
		class?: string;
		children?: Snippet;
	} = $props();
</script>

<div class="flex flex-col gap-1 w-full {className}">
	{#if label}
		<Label {forId} {required} {hint}>
			{label}
		</Label>
	{/if}

	<div class="w-full">
		{@render children?.()}
	</div>

	{#if error}
		<span class="text-[11px] text-red-400 font-medium mt-0.5">{error}</span>
	{/if}
</div>

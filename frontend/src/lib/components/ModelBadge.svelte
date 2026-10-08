<script lang="ts">
	import { Box } from '@lucide/svelte';
	import { parseModelDetails } from '$lib/utils/model';
	import Badge from './ds/atoms/Badge.svelte';

	let {
		model,
		showIcon = true,
		iconSize = 14,
		variant = 'badge',
		class: className = ''
	}: {
		model: string | string[] | undefined | null;
		showIcon?: boolean;
		iconSize?: number;
		variant?: 'badge' | 'inline' | 'clean' | 'badge-full' | 'full';
		class?: string;
	} = $props();

	let parsed = $derived(parseModelDetails(model));
</script>

{#if parsed.name}
	<span class="model-badge-container {variant} {className}">
		{#if showIcon}
			<Box size={iconSize} class="model-badge-icon" />
		{/if}

		<span class="model-name-text" title={parsed.name}>{parsed.name}</span>

		{#each parsed.tags as tag}
			<span class="model-tag">{tag}</span>
		{/each}
	</span>
{/if}

<style>
	.model-badge-container {
		display: inline-flex;
		align-items: center;
		gap: 0.25rem;
		width: fit-content;
		max-width: 100%;
		min-width: 0;
		vertical-align: middle;
		overflow: hidden;
	}

	.model-badge-container.badge {
		background: color-mix(in srgb, var(--bg-surface) 80%, black 20%);
		border: 1px solid var(--border-color);
		border-radius: 0.5rem;
		padding: 0.25rem 0.25rem;
		padding-left: 0.4rem;
	}

	.model-badge-container.badge-full {
		background: color-mix(in srgb, var(--bg-surface) 80%, black 20%);
		border: 1px solid var(--border-color);
		border-radius: 0.5rem;
		display: flex;
		justify-content: space-between;
		width: 100%;
		padding: 0.25rem 0.25rem;
		padding-left: 0.4rem;
	}

	.model-badge-container.full {
		display: flex;
		justify-content: space-between;
		width: 100%;
	}

	.model-badge-container.inline {
		background: transparent;
		border: none;
		padding: 0;
	}

	:global(.model-badge-icon) {
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.model-name-text {
		font-size: 0.8125rem;
		font-weight: 500;
		color: var(--text-primary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		min-width: 0;
		flex-shrink: 1;
		margin-right: auto;
	}

	.model-tag {
		background: color-mix(in srgb, var(--text-primary) 10%, transparent);
		border-radius: 9999px;
		padding: 0.1rem 0.45rem;
		font-size: 0.7rem;
		font-weight: 500;
		font-variant-numeric: tabular-nums;
		color: var(--text-secondary);
		line-height: 1.1;
		white-space: nowrap;
		flex-shrink: 0;
	}
</style>

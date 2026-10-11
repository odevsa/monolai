<script lang="ts">
	import { parseModelDetails } from '$lib/utils/model';
	import { Box } from '@lucide/svelte';

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
		flex-wrap: nowrap;
		gap: 0.25rem;
		max-width: 100%;
		min-width: 0;
		vertical-align: middle;
		overflow: hidden;
	}

	.model-badge-container.badge {
		background: color-mix(in srgb, var(--bg-surface) 80%, black 20%);
		border: 1px solid var(--border-color);
		border-radius: 0.5rem;
		padding: 0.35rem 0.5rem;
		padding-left: 0.5rem;
	}

	.model-badge-container.badge-full {
		background: color-mix(in srgb, var(--bg-surface) 80%, black 20%);
		border: 1px solid var(--border-color);
		border-radius: 0.5rem;
		padding: 0.35rem 0.5rem;
		padding-left: 0.5rem;
	}

	.model-badge-container.clean {
		background: transparent;
		border: none;
		padding: 0;
	}

	.model-badge-container.inline {
		width: fit-content;
		border: none;
		background: transparent;
		padding: 0;
	}

	:global(.model-badge-icon) {
		color: var(--primary);
		flex-shrink: 0;
		display: inline-block;
		vertical-align: middle;
	}

	.model-name-text {
		font-weight: 600;
		font-size: 0.775rem;
		color: var(--text-primary);
		line-height: 1.1;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		margin-right: auto;
	}

	.clean .model-name-text,
	.inline .model-name-text {
		font-size: 0.825rem;
		color: inherit;
	}

	.model-tag {
		font-size: 0.65rem;
		font-weight: 600;
		padding: 0.1rem 0.35rem;
		border-radius: 0.25rem;
		background: var(--border-color);
		color: var(--text-secondary);
		line-height: 1;
		white-space: nowrap;
		flex-shrink: 0;
	}

	.inline .model-tag {
		background: var(--primary-focus);
		color: var(--primary-hover);
	}

	.badge-full .model-tag {
		background: var(--primary-focus);
		color: var(--primary-hover);
	}
</style>

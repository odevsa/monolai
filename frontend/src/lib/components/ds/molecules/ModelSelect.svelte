<script lang="ts">
	import { getFeatureModels, type FeatureModelItem } from '$lib/api/models';
	import { ModelBadge } from '$lib/components/ds';
	import { t } from '$lib/i18n';
	import { featuresState } from '$lib/state/features.svelte';
	import { Box, Check, ChevronDown } from '@lucide/svelte';
	import { onMount } from 'svelte';

	let {
		value = $bindable(''),
		models = undefined,
		features = undefined,
		placeholder = '',
		label = '',
		emptyText = '',
		disabled = false,
		placement = 'bottom',
		id = undefined,
		name = undefined,
		class: className = '',
		onchange = undefined,
		onopen = undefined
	}: {
		value?: string;
		models?: FeatureModelItem[];
		features?: string[];
		placeholder?: string;
		label?: string;
		emptyText?: string;
		disabled?: boolean;
		placement?: 'bottom' | 'top';
		id?: string;
		name?: string;
		class?: string;
		onchange?: (val: string) => void;
		onopen?: () => void;
	} = $props();

	let open = $state(false);
	let containerRef = $state<HTMLDivElement | null>(null);
	let internalModels = $state<FeatureModelItem[]>([]);

	async function loadModels() {
		try {
			internalModels = await getFeatureModels(features || []);
		} catch (err) {
			console.error('Error fetching models in ModelSelect:', err);
		}
	}

	$effect(() => {
		if (features && models === undefined && featuresState.isInitialized) {
			const _ = featuresState.lastUpdated;
			loadModels();
		}
	});

	let displayModels = $derived(models !== undefined ? models : internalModels);
	let resolvedPlaceholder = $derived(placeholder || t('common.selectModel') || 'Select Model');
	let resolvedLabel = $derived(label || t('common.selectActiveModel') || 'SELECT ACTIVE MODEL');
	let resolvedEmptyText = $derived(
		emptyText || t('common.noModelsRegistered') || 'No models registered'
	);

	function toggleOpen(e?: MouseEvent) {
		if (e) e.stopPropagation();
		if (disabled) return;
		open = !open;
		if (open) {
			if (onopen) onopen();
			if (features && models === undefined) {
				loadModels();
			}
		}
	}

	function selectOption(modelId: string) {
		value = modelId;
		open = false;
		if (onchange) {
			onchange(modelId);
		}
	}

	function handleClickOutside(event: MouseEvent) {
		if (containerRef && !containerRef.contains(event.target as Node)) {
			open = false;
		}
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.key === 'Escape' && open) {
			open = false;
		}
	}

	onMount(() => {
		if (features && models === undefined) {
			loadModels();
		}
		document.addEventListener('click', handleClickOutside);
		document.addEventListener('keydown', handleKeyDown);
		return () => {
			document.removeEventListener('click', handleClickOutside);
			document.removeEventListener('keydown', handleKeyDown);
		};
	});
</script>

<div class="model-select-wrapper {className}" bind:this={containerRef}>
	<button
		{id}
		{name}
		type="button"
		class="model-select-btn"
		class:open
		onclick={toggleOpen}
		{disabled}
		aria-expanded={open}
		aria-label={resolvedPlaceholder}
	>
		<div class="model-select-value">
			{#if value}
				<ModelBadge model={value} variant="inline" />
			{:else}
				<Box size={14} class="model-box-icon" />
				<span class="model-placeholder-text">{resolvedPlaceholder}</span>
			{/if}
		</div>

		<ChevronDown size={14} class="chevron-icon {open ? 'open' : ''}" />
	</button>

	{#if open}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="mobile-backdrop sm:hidden" onclick={() => (open = false)} role="presentation"></div>

		<div class="popover-menu placement-{placement}">
			<div class="menu-label">{resolvedLabel}</div>
			{#if displayModels.length === 0}
				<div class="empty-menu-text">{resolvedEmptyText}</div>
			{:else}
				{#each displayModels as model (model.id)}
					<button
						type="button"
						class="popover-item {value === model.id ? 'selected' : ''}"
						onclick={() => selectOption(model.id)}
					>
						<div class="item-left">
							<span class="check-slot">
								{#if value === model.id}
									<Check size={14} class="check-icon" />
								{/if}
							</span>
							<ModelBadge model={model.id} variant="inline" />
						</div>
						<span class="mini-tag">{model.runtime}</span>
					</button>
				{/each}
			{/if}
		</div>
	{/if}
</div>

<style>
	.model-select-wrapper {
		position: relative;
		width: 100%;
	}

	.model-select-btn {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.5rem;
		width: 100%;
		height: 34px;
		padding: 0 0.65rem;
		font-size: 0.825rem;
		font-weight: 500;
		color: var(--text-secondary);
		background: var(--bg-primary);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-lg);
		cursor: pointer;
		transition: all 0.15s ease;
		box-sizing: border-box;
		text-align: left;
		user-select: none;
		overflow: hidden;
	}

	.model-select-btn:hover:not(:disabled) {
		color: var(--text-primary);
		border-color: var(--border-hover);
		background: var(--bg-hover);
	}

	.model-select-btn.open {
		border-color: var(--primary);
	}

	.model-select-btn:disabled {
		opacity: 0.45;
		cursor: not-allowed;
	}

	.model-select-value {
		display: flex;
		align-items: center;
		gap: 0.35rem;
		min-width: 0;
		flex: 1 1 auto;
		overflow: hidden;
	}

	.model-select-value :global(.model-badge-container) {
		max-width: 100%;
		min-width: 0;
		overflow: hidden;
	}

	.model-select-value :global(.model-name-text) {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.model-placeholder-text {
		color: var(--text-muted);
		font-size: 0.825rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	:global(.model-box-icon) {
		color: var(--text-muted);
		flex-shrink: 0;
	}

	:global(.chevron-icon) {
		color: var(--text-muted);
		transition: transform 0.15s ease;
		flex-shrink: 0;
	}

	:global(.chevron-icon.open) {
		transform: rotate(180deg);
	}

	.mobile-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		-webkit-backdrop-filter: blur(2px);
		backdrop-filter: blur(2px);
		z-index: 55;
	}

	.popover-menu {
		position: absolute;
		left: 0;
		right: 0;
		width: 100%;
		min-width: min(300px, 100%);
		background-color: var(--bg-surface);
		border: 1px solid var(--border-color);
		border-radius: var(--radius-xl, 0.75rem);
		padding: 0.375rem;
		box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
		z-index: 60;
		max-height: 280px;
		overflow-y: auto;
		overflow-x: hidden;
		display: flex;
		flex-direction: column;
		gap: 0.125rem;
		animation: popoverFadeIn 0.15s ease-out;
		box-sizing: border-box;
	}

	.popover-menu.placement-bottom {
		top: calc(100% + 0.35rem);
		bottom: auto;
	}

	.popover-menu.placement-top {
		bottom: calc(100% + 0.35rem);
		top: auto;
	}

	.menu-label {
		padding: 0.5rem 0.75rem 0.25rem 0.75rem;
		font-size: 0.7rem;
		font-weight: 600;
		color: var(--text-muted);
		letter-spacing: 0.05em;
	}

	.empty-menu-text {
		padding: 0.75rem;
		font-size: 0.85rem;
		color: var(--text-muted);
		text-align: center;
	}

	.popover-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.75rem;
		padding: 0.5rem 0.65rem;
		border-radius: var(--radius-md, 0.5rem);
		background: transparent;
		border: none;
		color: var(--text-primary);
		font-size: 0.875rem;
		cursor: pointer;
		width: 100%;
		max-width: 100%;
		min-width: 0;
		text-align: left;
		transition: background-color 0.15s ease;
		box-sizing: border-box;
		white-space: nowrap;
		overflow: hidden;
	}

	.popover-item:hover:not(:disabled) {
		background-color: var(--bg-hover);
	}

	.popover-item.selected {
		background-color: var(--primary-light);
		color: var(--primary);
		font-weight: 500;
	}

	.popover-item.selected :global(.model-name-text) {
		color: var(--primary);
		font-weight: 600;
	}

	.item-left {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		min-width: 0;
		flex: 1 1 auto;
		overflow: hidden;
	}

	.item-left :global(.model-badge-container) {
		max-width: 100%;
		min-width: 0;
		overflow: hidden;
	}

	.item-left :global(.model-name-text) {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.check-slot {
		width: 14px;
		height: 14px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: var(--primary);
		flex-shrink: 0;
	}

	:global(.check-icon) {
		color: var(--primary);
	}

	.mini-tag {
		font-size: 0.7rem;
		font-weight: 500;
		color: var(--primary);
		background: var(--primary-light);
		border: 1px solid var(--primary);
		padding: 0.15rem 0.45rem;
		border-radius: 0.375rem;
		white-space: nowrap;
		flex-shrink: 0;
		line-height: 1.2;
	}

	@keyframes popoverFadeIn {
		from {
			opacity: 0;
			transform: translateY(-4px);
		}
		to {
			opacity: 1;
			transform: translateY(0);
		}
	}
</style>

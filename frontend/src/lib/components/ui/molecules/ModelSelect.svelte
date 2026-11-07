<script lang="ts">
	import { getFeatureModels, type FeatureModelItem } from '$lib/api/models';
	import { ModelBadge } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { featuresState } from '$lib/state/features.svelte';
	import { portal } from '$lib/utils/portal';
	import { Box, ChevronDown } from '@lucide/svelte';
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
		align = 'left',
		variant = 'default',
		centerOnScreen = false,
		showChevron = undefined,
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
		align?: 'left' | 'right';
		variant?: 'default' | 'pill';
		centerOnScreen?: boolean;
		showChevron?: boolean;
		id?: string;
		name?: string;
		class?: string;
		onchange?: (val: string) => void;
		onopen?: () => void;
	} = $props();

	let resolvedShowChevron = $derived(showChevron !== undefined ? showChevron : variant !== 'pill');

	let open = $state(false);
	let containerRef = $state<HTMLDivElement | null>(null);
	let dropdownRef = $state<HTMLDivElement | null>(null);
	let popoverStyle = $state('');
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
	let selectedModel = $derived(
		displayModels.find((m) => m.id === value) || displayModels.find((m) => m.modelId === value)
	);
	let resolvedPlaceholder = $derived(placeholder || t('common.selectModel'));
	let resolvedLabel = $derived(label || t('common.selectActiveModel'));
	let resolvedEmptyText = $derived(emptyText || t('common.noModelsRegistered'));

	function updateCoords() {
		if (!centerOnScreen || !containerRef) return;
		const rect = containerRef.getBoundingClientRect();
		const isMobile = window.innerWidth <= 640;

		const useBottom = placement === 'top';
		const bottom = Math.max(8, window.innerHeight - rect.top + 8);
		const top = Math.max(8, rect.bottom + 8);

		if (isMobile) {
			const maxHeight = useBottom
				? Math.min(280, Math.max(120, rect.top - 16))
				: Math.min(280, Math.max(120, window.innerHeight - rect.bottom - 16));

			popoverStyle = [
				'position: fixed',
				'left: 1rem',
				'right: 1rem',
				'margin-left: auto',
				'margin-right: auto',
				'width: auto',
				'max-width: 440px',
				'min-width: 0',
				useBottom ? `bottom: ${bottom}px` : `top: ${top}px`,
				useBottom ? 'top: auto' : 'bottom: auto',
				`max-height: ${maxHeight}px`,
				'z-index: 9999'
			].join('; ');
		} else {
			const maxHeight = useBottom
				? Math.min(280, Math.max(120, rect.top - 16))
				: Math.min(280, Math.max(120, window.innerHeight - rect.bottom - 16));

			const isPill = variant === 'pill';
			let horizontalStyles: string[];

			if (isPill) {
				if (align === 'right') {
					const popoverWidth = Math.min(440, window.innerWidth - 32);
					const idealLeft = rect.right - popoverWidth;
					const left = Math.max(16, Math.min(idealLeft, window.innerWidth - popoverWidth - 16));
					horizontalStyles = [
						`left: ${left}px`,
						'right: auto',
						`width: ${popoverWidth}px`,
						'min-width: 0',
						'max-width: calc(100vw - 2rem)'
					];
				} else {
					const popoverWidth = Math.min(380, window.innerWidth - 32);
					const idealLeft = rect.left;
					const left = Math.max(16, Math.min(idealLeft, window.innerWidth - popoverWidth - 16));
					horizontalStyles = [
						`left: ${left}px`,
						'right: auto',
						`width: ${popoverWidth}px`,
						'min-width: 0',
						'max-width: calc(100vw - 2rem)'
					];
				}
			} else {
				const width = Math.min(Math.max(rect.width, 280), window.innerWidth - 32);
				const left = Math.max(16, Math.min(rect.left, window.innerWidth - width - 16));
				horizontalStyles = [
					`left: ${left}px`,
					'right: auto',
					`width: ${width}px`,
					'min-width: 0',
					'max-width: calc(100vw - 2rem)'
				];
			}

			popoverStyle = [
				'position: fixed',
				...horizontalStyles,
				useBottom ? `bottom: ${bottom}px` : `top: ${top}px`,
				useBottom ? 'top: auto' : 'bottom: auto',
				`max-height: ${maxHeight}px`,
				'z-index: 9999'
			].join('; ');
		}
	}

	function toggleOpen(e?: MouseEvent) {
		if (e) e.stopPropagation();
		if (disabled) return;
		open = !open;
		if (open) {
			if (centerOnScreen) {
				updateCoords();
			}
			if (onopen) onopen();
			if (features && models === undefined) {
				loadModels();
			}
		}
	}

	$effect(() => {
		if (open && centerOnScreen) {
			updateCoords();
			const handleScroll = (e: Event) => {
				if (dropdownRef && dropdownRef.contains(e.target as Node)) {
					return;
				}
				updateCoords();
			};
			const handleResize = () => {
				updateCoords();
			};
			window.addEventListener('scroll', handleScroll, true);
			window.addEventListener('resize', handleResize);
			return () => {
				window.removeEventListener('scroll', handleScroll, true);
				window.removeEventListener('resize', handleResize);
			};
		}
	});

	function selectOption(modelId: string) {
		value = modelId;
		open = false;
		if (onchange) {
			onchange(modelId);
		}
	}

	function handleClickOutside(event: MouseEvent) {
		const target = event.target as Node;
		if (
			containerRef &&
			!containerRef.contains(target) &&
			(!dropdownRef || !dropdownRef.contains(target))
		) {
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

<div class="model-select-wrapper variant-{variant} {className}" bind:this={containerRef}>
	<button
		{id}
		{name}
		type="button"
		class="model-select-btn variant-{variant}"
		class:open
		onclick={toggleOpen}
		{disabled}
		aria-expanded={open}
		aria-label={resolvedPlaceholder}
	>
		<div class="model-select-value">
			{#if value}
				<ModelBadge
					model={selectedModel?.modelId || value}
					preset={selectedModel?.presetName}
					variant="inline"
					hideTagsOnMobile={variant === 'pill'}
				/>
			{:else}
				<Box size={14} class="model-box-icon" />
				<span class="model-placeholder-text">{resolvedPlaceholder}</span>
			{/if}
		</div>

		{#if resolvedShowChevron}
			<ChevronDown size={14} class="chevron-icon {open ? 'open' : ''}" />
		{/if}
	</button>

	{#snippet menuContent()}
		<div class="menu-label">{resolvedLabel}</div>
		{#if displayModels.length === 0}
			<div class="empty-menu-text">{resolvedEmptyText}</div>
		{:else}
			{#each displayModels as model (model.id)}
				<button
					type="button"
					role="option"
					aria-selected={value === model.id}
					class="popover-item {value === model.id ? 'selected' : ''}"
					onclick={() => selectOption(model.id)}
				>
					<div class="item-left">
						<ModelBadge
							model={model.modelId || model.id}
							preset={model.presetName}
							variant="inline"
						/>
					</div>
					<span class="mini-tag">{model.runtime}</span>
				</button>
			{/each}
		{/if}
	{/snippet}

	{#if open}
		{#if centerOnScreen}
			<div
				use:portal
				bind:this={dropdownRef}
				class="popover-menu popover-portaled placement-{placement} align-{align} variant-{variant}"
				style={popoverStyle}
				role="listbox"
				tabindex="-1"
			>
				{@render menuContent()}
			</div>
		{:else}
			<div
				bind:this={dropdownRef}
				class="popover-menu placement-{placement} align-{align} variant-{variant}"
				role="listbox"
				tabindex="-1"
			>
				{@render menuContent()}
			</div>
		{/if}
	{/if}
</div>

<style>
	.model-select-wrapper {
		position: relative;
		width: 100%;
	}

	.model-select-wrapper.variant-pill {
		width: auto;
		display: inline-flex;
		align-items: center;
		max-width: 100%;
		min-width: 0;
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

	.model-select-btn.variant-pill {
		height: auto;
		width: auto;
		max-width: 100%;
		min-width: 0;
		padding: 0.35rem 0.65rem;
		border-radius: 9999px;
		background: var(--bg-surface);
		font-size: 0.75rem;
		gap: 0.375rem;
	}

	.model-select-btn.variant-pill:hover:not(:disabled) {
		background: var(--bg-surface-hover);
		border-color: var(--border-hover);
		color: var(--text-primary);
	}

	@media (max-width: 640px) {
		.model-select-btn.variant-pill {
			padding: 0.35rem 0.5rem;
		}
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

	.popover-menu.popover-portaled {
		position: fixed;
		z-index: 9999;
	}

	.popover-menu.placement-bottom {
		top: calc(100% + 0.35rem);
		bottom: auto;
	}

	.popover-menu.placement-top {
		bottom: calc(100% + 0.35rem);
		top: auto;
	}

	.popover-menu.align-right {
		left: auto;
		right: 0;
		width: max-content;
		min-width: 280px;
		max-width: min(380px, calc(100vw - 2rem));
	}

	.popover-menu.align-left.variant-pill {
		left: 0;
		right: auto;
		width: max-content;
		min-width: 280px;
		max-width: min(380px, calc(100vw - 2rem));
	}

	@media (min-width: 640px) {
		.popover-menu.align-right,
		.popover-menu.align-left.variant-pill {
			min-width: 440px;
			max-width: min(640px, calc(100vw - 4rem));
		}
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

	.preset-badge {
		font-size: 0.65rem;
		font-weight: 600;
		padding: 0.1rem 0.4rem;
		border-radius: var(--radius-sm, 0.25rem);
		background: var(--bg-surface-hover);
		color: var(--text-secondary);
		border: 1px solid var(--border-color);
		line-height: 1.2;
		white-space: nowrap;
		flex-shrink: 0;
	}

	.popover-item.selected .preset-badge {
		background: var(--primary-light);
		color: var(--primary);
		border-color: var(--primary);
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

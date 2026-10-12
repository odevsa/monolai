<script lang="ts">
	import { ModelBadge } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { Box, Check } from '@lucide/svelte';
	import { onMount } from 'svelte';

	interface RegisteredModel {
		id: string;
		runtime: string;
		flags: string;
	}

	interface ChatModelSelectorProps {
		selectedModel: string;
		models: RegisteredModel[];
		onSelectModel: (modelId: string) => void;
	}

	let {
		selectedModel = $bindable(''),
		models = [],
		onSelectModel
	}: ChatModelSelectorProps = $props();

	let isMenuOpen = $state(false);
	let wrapperRef: HTMLDivElement | null = $state(null);

	function toggleMenu() {
		isMenuOpen = !isMenuOpen;
	}

	function handleOutsideClick(e: MouseEvent) {
		if (wrapperRef && !wrapperRef.contains(e.target as Node)) {
			isMenuOpen = false;
		}
	}

	onMount(() => {
		document.addEventListener('click', handleOutsideClick);
		return () => {
			document.removeEventListener('click', handleOutsideClick);
		};
	});
</script>

<div class="relative inline-flex items-center" bind:this={wrapperRef}>
	<button
		type="button"
		class="inline-flex items-center gap-1.5 py-1.5 px-2.5 rounded-full bg-[var(--bg-surface)] border border-[var(--border-color)] text-[var(--text-secondary)] text-xs cursor-pointer transition-all duration-150 hover:bg-[var(--bg-surface-hover)] hover:border-[var(--border-hover)] hover:text-[var(--text-primary)]"
		onclick={toggleMenu}
		aria-label={t('common.selectModel')}
	>
		{#if selectedModel}
			<ModelBadge model={selectedModel} variant="inline" />
		{:else}
			<Box size={14} class="text-[var(--text-muted)]" />
			<span class="text-[var(--text-secondary)] font-medium">{t('common.selectModel')}</span>
		{/if}
	</button>

	{#if isMenuOpen}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div
			class="fixed inset-0 bg-transparent z-[99] sm:hidden"
			onclick={() => (isMenuOpen = false)}
			role="presentation"
		></div>
		<div
			class="absolute bottom-[calc(100%+8px)] right-0 bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-xl shadow-[0_8px_24px_rgba(0,0,0,0.3)] p-1.5 flex flex-col min-w-[220px] max-w-[320px] z-[100]"
		>
			<div
				class="text-[0.65rem] font-bold uppercase tracking-wider text-[var(--text-muted)] pt-1.5 px-2.5 pb-1"
			>
				{t('common.selectActiveModel')}
			</div>
			{#if models.length === 0}
				<div class="text-xs text-[var(--text-muted)] p-2.5 text-center">
					{t('common.noModelsRegistered')}
				</div>
			{:else}
				<div class="flex flex-col gap-0.5 max-h-[220px] overflow-y-auto">
					{#each models as model}
						<button
							type="button"
							class="flex items-center justify-between gap-2 py-1.5 px-2.5 rounded-lg border-0 text-xs cursor-pointer w-full transition-all duration-150 text-left {selectedModel ===
							model.id
								? 'bg-[var(--primary-light)] text-[var(--text-primary)]'
								: 'bg-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]'}"
							onclick={() => {
								selectedModel = model.id;
								onSelectModel(model.id);
								isMenuOpen = false;
							}}
						>
							<div class="flex items-center gap-1.5 min-w-0 flex-1">
								<span class="w-3.5 h-3.5 flex items-center justify-center shrink-0">
									{#if selectedModel === model.id}
										<Check size={14} class="text-[var(--primary)]" />
									{/if}
								</span>
								<ModelBadge model={model.id} variant="inline" />
							</div>
							<span
								class="text-[0.65rem] py-0.5 px-1.5 rounded bg-[var(--bg-hover)] text-[var(--text-muted)] border border-[var(--border-color)] shrink-0"
							>
								{model.runtime}
							</span>
						</button>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</div>

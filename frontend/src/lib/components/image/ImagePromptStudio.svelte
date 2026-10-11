<script lang="ts">
	import type { FeatureModelItem } from '$lib/api/models';
	import { Alert, Badge, Card, ModelSelect, Select } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { formatElapsedTime } from '$lib/utils/format';
	import { Dices, RefreshCw, Sparkles } from '@lucide/svelte';
	import ImageAdvancedParams from './ImageAdvancedParams.svelte';

	interface ImagePromptStudioProps {
		selectedModel: string;
		prompt: string;
		negativePrompt: string;
		selectedSize: string;
		selectedCount: string;
		steps: number;
		cfgScale: number;
		seed: number | string;
		isGenerating: boolean;
		isModelRunning: boolean;
		generationElapsed: number;
		generationError: string | null;
		models: FeatureModelItem[];
		onGenerate: () => void;
		onLoadModels: () => void;
		onInsertRandomPrompt: () => void;
	}

	let {
		selectedModel = $bindable(''),
		prompt = $bindable(''),
		negativePrompt = $bindable(''),
		selectedSize = $bindable('512x512'),
		selectedCount = $bindable('1'),
		steps = $bindable(20),
		cfgScale = $bindable(7.5),
		seed = $bindable(''),
		isGenerating = false,
		isModelRunning = false,
		generationElapsed = 0,
		generationError = null,
		models = [],
		onGenerate,
		onLoadModels,
		onInsertRandomPrompt
	}: ImagePromptStudioProps = $props();

	let sizeOptions = $derived([
		{ value: '256x256', label: t('image.sizes.squareDraft') },
		{ value: '512x512', label: t('image.sizes.squareFast') },
		{ value: '768x768', label: t('image.sizes.squareStandard') },
		{ value: '1024x1024', label: t('image.sizes.squareSdxl') },
		{ value: '768x512', label: t('image.sizes.landscape32') },
		{ value: '512x768', label: t('image.sizes.portrait23') },
		{ value: '1024x768', label: t('image.sizes.landscape43') },
		{ value: '768x1024', label: t('image.sizes.portrait34') }
	]);

	let countOptions = $derived([
		{ value: '1', label: t('image.counts.one') },
		{ value: '2', label: t('image.counts.two') },
		{ value: '4', label: t('image.counts.four') }
	]);

	function handleKeyDown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
			e.preventDefault();
			onGenerate();
		}
	}
</script>

<Card>
	<div class="flex flex-col gap-4">
		<!-- Studio Header -->
		<div class="flex items-center justify-between pb-1 border-b border-[var(--border-color)]">
			<div class="flex items-center gap-2">
				<Sparkles size={16} class="text-[var(--primary)]" />
				<h3 class="m-0 text-sm font-bold text-[var(--text-primary)]">
					{t('image.promptStudio')}
				</h3>
			</div>
			{#if isModelRunning}
				<Badge variant="success" pulse>{t('image.active')}</Badge>
			{/if}
		</div>

		<!-- Model Selector -->
		<div class="flex flex-col gap-1.5">
			<label for="image-model-select" class="text-xs font-semibold text-[var(--text-secondary)]">
				{t('image.modelLabel')}
			</label>
			<ModelSelect
				id="image-model-select"
				bind:value={selectedModel}
				{models}
				onopen={onLoadModels}
				placeholder={t('image.selectModelPlaceholder')}
				label={t('image.selectActiveModel')}
				emptyText={t('image.noModelsRegistered')}
			/>
		</div>

		<!-- Prompt Input -->
		<div class="flex flex-col gap-1">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<label for="image-prompt" class="text-xs font-semibold text-[var(--text-secondary)]">
						{t('image.promptLabel')}
					</label>
					<span class="text-[11px] text-[var(--text-muted)]">{t('image.ctrlEnterHint')}</span>
				</div>
				<button
					type="button"
					class="flex items-center gap-1.5 px-2.5 py-1 text-[11px] font-medium rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] text-[var(--text-secondary)] hover:text-[var(--primary)] hover:border-[var(--primary)] transition-colors cursor-pointer"
					onclick={onInsertRandomPrompt}
					title={t('image.randomPromptTitle')}
					disabled={isGenerating}
				>
					<Dices size={13} class="text-[var(--primary)]" />
					<span>{t('image.randomPromptBtn')}</span>
				</button>
			</div>
			<textarea
				id="image-prompt"
				bind:value={prompt}
				onkeydown={handleKeyDown}
				rows={4}
				placeholder={t('image.promptPlaceholder')}
				class="w-full p-3 rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] text-sm text-[var(--text-primary)] placeholder:[var(--text-muted)] outline-none resize-y transition-colors focus:border-[var(--primary)] focus:ring-1 focus:ring-[var(--primary)] disabled:opacity-50"
				disabled={isGenerating}></textarea>
		</div>

		<!-- Negative Prompt -->
		<div class="flex flex-col gap-1">
			<label for="image-negative-prompt" class="text-xs font-semibold text-[var(--text-secondary)]">
				{t('image.negativePromptTitle')}
			</label>
			<textarea
				id="image-negative-prompt"
				bind:value={negativePrompt}
				rows={2}
				placeholder={t('image.negativePromptPlaceholder')}
				class="w-full p-2.5 rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] text-xs text-[var(--text-primary)] placeholder:[var(--text-muted)] outline-none resize-y transition-colors focus:border-[var(--primary)] disabled:opacity-50"
				disabled={isGenerating}></textarea>
		</div>

		<!-- Dimensions & Quantity Row -->
		<div class="grid grid-cols-2 gap-3 pt-1">
			<div class="flex flex-col gap-1.5">
				<label for="image-size-select" class="text-xs font-semibold text-[var(--text-secondary)]">
					{t('image.dimensions')}
				</label>
				<Select
					bind:value={selectedSize}
					options={sizeOptions}
					placeholder={t('image.selectDimensions')}
					disabled={isGenerating}
				/>
			</div>

			<div class="flex flex-col gap-1.5">
				<label for="image-count-select" class="text-xs font-semibold text-[var(--text-secondary)]">
					{t('image.quantity')}
				</label>
				<Select
					bind:value={selectedCount}
					options={countOptions}
					placeholder={t('image.selectQuantity')}
					disabled={isGenerating}
				/>
			</div>
		</div>

		<!-- Advanced Parameters Component -->
		<ImageAdvancedParams bind:steps bind:cfgScale bind:seed disabled={isGenerating} />

		<!-- Generate Action Button -->
		<div class="pt-2">
			<button
				type="button"
				class="app-btn app-btn-primary w-full py-2.5 text-sm font-semibold rounded-lg flex items-center justify-center gap-2 shadow-xs transition-all cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
				disabled={isGenerating || !prompt.trim() || !selectedModel}
				onclick={onGenerate}
			>
				{#if isGenerating}
					<RefreshCw size={16} class="animate-spin" />
					<span
						>{t('image.generatingWithTime', { time: formatElapsedTime(generationElapsed) })}</span
					>
				{:else}
					<Sparkles size={16} />
					<span>{t('image.generateImage')}</span>
				{/if}
			</button>
		</div>

		<!-- Error Alert -->
		{#if generationError}
			<div class="pt-2">
				<Alert variant="error" title={t('image.generationError')} message={generationError} />
			</div>
		{/if}
	</div>
</Card>

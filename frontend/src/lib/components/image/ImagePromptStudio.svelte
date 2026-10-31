<script lang="ts">
	import type { FeatureModelItem } from '$lib/api/models';
	import { Alert, Badge, Button, Card, ModelSelect, Select, Textarea } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { formatElapsedTime } from '$lib/utils/format';
	import { Dices, Sparkles } from '@lucide/svelte';
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
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-2">
				<Sparkles size={16} class="text-[var(--primary)]" />
				<h3 class="m-0 text-sm font-bold text-[var(--text-primary)]">
					{t('image.promptStudio')}
				</h3>
			</div>
			{#if isModelRunning}
				<Badge variant="success" pulse>{t('common.active')}</Badge>
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
				placeholder={t('common.selectModel')}
				label={t('common.selectActiveModel')}
				emptyText={t('common.noModelsRegistered')}
			/>
		</div>

		<!-- Prompt Input -->
		<div class="flex flex-col gap-1">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<label for="image-prompt" class="text-xs font-semibold text-[var(--text-secondary)]">
						{t('common.prompt')}
					</label>
					<span class="text-[11px] text-[var(--text-muted)]">{t('image.ctrlEnterHint')}</span>
				</div>
				<Button
					variant="secondary"
					size="xs"
					onclick={onInsertRandomPrompt}
					title={t('image.randomPromptTitle')}
					ariaLabel={t('image.randomPromptTitle')}
					disabled={isGenerating}
				>
					<Dices size={13} class="text-[var(--primary)]" />
					<span>{t('image.randomPromptBtn')}</span>
				</Button>
			</div>
			<Textarea
				id="image-prompt"
				bind:value={prompt}
				onkeydown={handleKeyDown}
				rows={4}
				size="md"
				resize="y"
				placeholder={t('image.promptPlaceholder')}
				disabled={isGenerating}
			/>
		</div>

		<!-- Negative Prompt -->
		<div class="flex flex-col gap-1">
			<label for="image-negative-prompt" class="text-xs font-semibold text-[var(--text-secondary)]">
				{t('image.negativePromptLabel')}
			</label>
			<Textarea
				id="image-negative-prompt"
				bind:value={negativePrompt}
				rows={2}
				size="sm"
				resize="y"
				placeholder={t('image.negativePromptPlaceholder')}
				disabled={isGenerating}
			/>
		</div>

		<!-- Dimensions & Quantity Row -->
		<div class="grid grid-cols-2 gap-3 pt-1">
			<div class="flex flex-col gap-1.5">
				<label for="image-size-select" class="text-xs font-semibold text-[var(--text-secondary)]">
					{t('image.dimensions')}
				</label>
				<Select
					id="image-size-select"
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
					id="image-count-select"
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
			<Button
				variant="primary"
				size="lg"
				class="w-full"
				loading={isGenerating}
				disabled={isGenerating || !prompt.trim() || !selectedModel}
				onclick={onGenerate}
			>
				{#if isGenerating}
					<span
						>{t('image.generatingWithTime', { time: formatElapsedTime(generationElapsed) })}</span
					>
				{:else}
					<Sparkles size={16} />
					<span>{t('image.generateImage')}</span>
				{/if}
			</Button>
		</div>

		<!-- Error Alert -->
		{#if generationError}
			<div class="pt-2">
				<Alert variant="error" title={t('image.generationError')} message={generationError} />
			</div>
		{/if}
	</div>
</Card>

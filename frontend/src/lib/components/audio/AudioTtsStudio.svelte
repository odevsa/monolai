<script lang="ts">
	import type { FeatureModelItem } from '$lib/api/models';
	import { generateSpeech } from '$lib/api/openai';
	import {
		Alert,
		AudioPlayer,
		Badge,
		Button,
		EmptyState,
		Loader,
		ModelSelect,
		RandomPromptButton,
		Select,
		Textarea
	} from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { audioState } from '$lib/state';
	import { copyToClipboard } from '$lib/utils/clipboard';
	import { formatElapsedTime } from '$lib/utils/format';
	import { Check, Copy, Download, Headphones, Sparkles } from '@lucide/svelte';
	import { onDestroy, untrack } from 'svelte';

	interface AudioTtsStudioProps {
		models: FeatureModelItem[];
		selectedModel: string;
		isModelRunning?: boolean;
		onLoadModels?: () => void;
	}

	let {
		models = [],
		selectedModel = $bindable(''),
		isModelRunning = false,
		onLoadModels
	}: AudioTtsStudioProps = $props();

	let promptText = $state('');
	let selectedVoice = $state('af_heart');
	let selectedFormat = $state('mp3');
	let speed = $state(1.0);

	let isGenerating = $state(false);
	let generationError = $state<string | null>(null);
	let generationElapsed = $state(0);
	let generationTimer: ReturnType<typeof setInterval> | null = null;
	let abortController: AbortController | null = null;

	let lastGeneratedAudioUrl = $state<string | null>(null);
	let lastGeneratedDuration = $state<number>(0);
	let copied = $state(false);

	function handleDownload() {
		if (!lastGeneratedAudioUrl) return;
		const a = document.createElement('a');
		a.href = lastGeneratedAudioUrl;
		a.download = `monolai-audio-${Date.now()}.${selectedFormat || 'wav'}`;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
	}

	async function handleCopy() {
		if (!promptText) return;
		const success = await copyToClipboard(promptText);
		if (success) {
			copied = true;
			setTimeout(() => (copied = false), 2000);
		}
	}

	// TTS voice options including Kokoro & OpenAI defaults
	const voiceOptions = [
		{ value: 'af_heart', label: 'Heart (Female, en-US)' },
		{ value: 'af_bella', label: 'Bella (Female, en-US)' },
		{ value: 'af_nicole', label: 'Nicole (Female, en-US)' },
		{ value: 'af_sarah', label: 'Sarah (Female, en-US)' },
		{ value: 'af_sky', label: 'Sky (Female, en-US)' },
		{ value: 'am_adam', label: 'Adam (Male, en-US)' },
		{ value: 'am_michael', label: 'Michael (Male, en-US)' },
		{ value: 'bf_emma', label: 'Emma (Female, en-GB)' },
		{ value: 'bf_isabella', label: 'Isabella (Female, en-GB)' },
		{ value: 'bm_george', label: 'George (Male, en-GB)' },
		{ value: 'bm_lewis', label: 'Lewis (Male, en-GB)' },
		{ value: 'alloy', label: 'Alloy (OpenAI Standard)' },
		{ value: 'echo', label: 'Echo (OpenAI Standard)' },
		{ value: 'fable', label: 'Fable (OpenAI Standard)' },
		{ value: 'onyx', label: 'Onyx (OpenAI Standard)' },
		{ value: 'nova', label: 'Nova (OpenAI Standard)' },
		{ value: 'shimmer', label: 'Shimmer (OpenAI Standard)' }
	];

	const formatOptions = [
		{ value: 'mp3', label: 'MP3 (MPEG Audio)' },
		{ value: 'wav', label: 'WAV (Uncompressed PCM)' },
		{ value: 'opus', label: 'Opus (Low Latency)' },
		{ value: 'flac', label: 'FLAC (Lossless)' }
	];

	const samplePrompts = [
		'Welcome to Monolai! The local, private, and high-performance AI engine for text, image, and audio.',
		'Artificial intelligence is evolving rapidly, bringing powerful open-source models right into your machine.',
		'The quick brown fox jumps over the lazy dog, showcasing the clarity and natural intonation of our audio synthesis engine.'
	];

	function insertSampleText() {
		const random = samplePrompts[Math.floor(Math.random() * samplePrompts.length)];
		promptText = random;
	}

	async function blobToDataUrl(blob: Blob): Promise<string> {
		return new Promise((resolve, reject) => {
			const reader = new FileReader();
			reader.onloadend = () => resolve(reader.result as string);
			reader.onerror = reject;
			reader.readAsDataURL(blob);
		});
	}

	let lastSelectionVersion = 0;

	// Listen to item selection from sidebar audio history
	$effect(() => {
		const currentVersion = audioState.selectionVersion;
		if (currentVersion > 0 && currentVersion !== lastSelectionVersion) {
			lastSelectionVersion = currentVersion;
			untrack(() => {
				const item = audioState.selectedItem;
				if (item?.type === 'tts') {
					promptText = item.prompt || '';
					if (item.model) {
						selectedModel = item.model;
					}
					if (item.voice && voiceOptions.some((v) => v.value === item.voice)) {
						selectedVoice = item.voice;
					}
					if (item.format && formatOptions.some((f) => f.value === item.format)) {
						selectedFormat = item.format;
					}
					if (item.speed !== undefined) {
						speed = item.speed;
					}
					lastGeneratedAudioUrl = item.audioUrl || null;
					lastGeneratedDuration = item.durationSecs || 0;
					generationError = null;
				}
			});
		}
	});

	async function handleSynthesize() {
		if (!promptText.trim() || !selectedModel || isGenerating) return;

		isGenerating = true;
		generationError = null;
		generationElapsed = 0;
		abortController = new AbortController();

		const startTime = Date.now();
		generationTimer = setInterval(() => {
			generationElapsed = Math.floor((Date.now() - startTime) / 100) / 10;
		}, 100);

		try {
			const blob = await generateSpeech(
				{
					model: selectedModel,
					input: promptText.trim(),
					voice: selectedVoice as any,
					response_format: selectedFormat as any,
					speed: Number(speed) || 1.0
				},
				abortController.signal
			);

			const totalSecs = Math.round((Date.now() - startTime) / 100) / 10;
			lastGeneratedDuration = totalSecs;

			let audioUrl = '';
			try {
				audioUrl = await blobToDataUrl(blob);
			} catch {
				audioUrl = URL.createObjectURL(blob);
			}
			lastGeneratedAudioUrl = audioUrl;

			// Save to global audio state
			audioState.addItem({
				id: `tts-${Date.now()}`,
				type: 'tts',
				model: selectedModel,
				timestamp: Date.now(),
				prompt: promptText.trim(),
				audioUrl,
				voice: selectedVoice,
				speed: Number(speed) || 1.0,
				format: selectedFormat,
				durationSecs: totalSecs
			});
		} catch (err: unknown) {
			console.error('TTS speech generation error:', err);
			if (err instanceof Error && err.name === 'AbortError') {
				generationError = t('audio.interrupted');
			} else {
				const msg = err instanceof Error ? err.message : String(err);
				generationError = msg || t('audio.ttsError');
			}
		} finally {
			if (generationTimer) {
				clearInterval(generationTimer);
				generationTimer = null;
			}
			isGenerating = false;
			abortController = null;
		}
	}

	function handleCancel() {
		if (abortController) {
			abortController.abort();
			abortController = null;
		}
	}

	onDestroy(() => {
		if (abortController) {
			abortController.abort();
		}
		if (generationTimer) {
			clearInterval(generationTimer);
		}
	});
</script>

<div class="grid grid-cols-1 lg:grid-cols-12 gap-5 sm:gap-6 items-start">
	<!-- Left Column: TTS Controls Studio (Mirroring ImagePromptStudio) -->
	<div class="lg:col-span-5 flex flex-col gap-4">
		<!-- Model Selection -->
		<div class="flex flex-col gap-1.5">
			<div class="flex items-center justify-between">
				<label for="tts-model-select" class="text-xs font-semibold text-[var(--text-secondary)]">
					{t('audio.selectTtsModel')}
				</label>
				{#if isModelRunning}
					<Badge variant="success" class="text-[10px]">
						{t('common.active')}
					</Badge>
				{/if}
			</div>
			<ModelSelect
				id="tts-model-select"
				bind:value={selectedModel}
				{models}
				placeholder={t('audio.selectModelPlaceholder')}
				onopen={onLoadModels}
			/>
		</div>

		<!-- Voice and Format Controls -->
		<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
			<!-- Voice Selection -->
			<div class="flex flex-col gap-1.5">
				<label for="tts-voice-select" class="text-xs font-semibold text-[var(--text-secondary)]">
					{t('audio.voice')}
				</label>
				<Select
					id="tts-voice-select"
					bind:value={selectedVoice}
					options={voiceOptions}
					placeholder={t('audio.selectVoice')}
					disabled={isGenerating}
				/>
			</div>

			<!-- Output Format -->
			<div class="flex flex-col gap-1.5">
				<label for="tts-format-select" class="text-xs font-semibold text-[var(--text-secondary)]">
					{t('audio.format')}
				</label>
				<Select
					id="tts-format-select"
					bind:value={selectedFormat}
					options={formatOptions}
					placeholder={t('audio.selectFormat')}
					disabled={isGenerating}
				/>
			</div>
		</div>

		<!-- Speed Slider -->
		<div class="flex flex-col gap-1.5">
			<div class="flex items-center justify-between text-xs">
				<label for="tts-speed-slider" class="font-semibold text-[var(--text-secondary)]">
					{t('audio.speed')}
				</label>
				<span class="font-mono text-[var(--text-muted)]">{speed.toFixed(2)}x</span>
			</div>
			<input
				id="tts-speed-slider"
				type="range"
				min="0.5"
				max="2.0"
				step="0.05"
				bind:value={speed}
				disabled={isGenerating}
				class="w-full h-1.5 rounded-lg appearance-none cursor-pointer bg-[var(--bg-hover)] accent-[var(--primary)] focus:outline-hidden"
			/>
		</div>

		<!-- Input Text Prompt Area -->
		<div class="flex flex-col gap-1.5">
			<div class="flex items-center justify-between">
				<label for="tts-prompt-input" class="text-xs font-semibold text-[var(--text-secondary)]">
					{t('audio.textToSpeak')}
				</label>
				<RandomPromptButton onclick={insertSampleText} disabled={isGenerating} />
			</div>
			<Textarea
				id="tts-prompt-input"
				bind:value={promptText}
				placeholder={t('audio.ttsPromptPlaceholder')}
				rows={4}
				disabled={isGenerating}
				class="w-full text-sm"
			/>
			<div class="flex justify-end text-[11px] text-[var(--text-muted)]">
				<span>{promptText.length} {t('audio.characters')}</span>
			</div>
		</div>

		<!-- Generate Action Button (Mirroring /image Generate Button) -->
		<div class="pt-2">
			<Button
				variant="primary"
				size="lg"
				class="w-full font-semibold"
				loading={isGenerating}
				disabled={isGenerating || !promptText.trim() || !selectedModel}
				onclick={handleSynthesize}
			>
				{#if isGenerating}
					<span
						>{t('image.generatingWithTime', { time: formatElapsedTime(generationElapsed) })}</span
					>
				{:else}
					<Sparkles size={16} class="shrink-0" />
					<span>{t('audio.generateSpeech')}</span>
				{/if}
			</Button>
		</div>

		<!-- Error Feedback -->
		{#if generationError}
			<div class="pt-2">
				<Alert variant="error" title={t('audio.generationError')} message={generationError} />
			</div>
		{/if}
	</div>

	<!-- Right Column: Audio Output Player & Preview (Mirroring ImagePreviewCard) -->
	<div class="lg:col-span-7 flex flex-col gap-4">
		<!-- Header with Action Toolbar -->
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-2">
				<Headphones size={16} class="text-[var(--primary)] shrink-0" />
				<h3 class="m-0 text-sm font-bold text-[var(--text-primary)]">
					{t('audio.audioOutput')}
				</h3>
			</div>

			{#if lastGeneratedAudioUrl}
				<div class="flex items-center gap-1.5 ml-auto">
					<Button
						variant="secondary"
						size="icon-sm"
						onclick={handleDownload}
						title={t('common.download')}
						ariaLabel={t('common.download')}
					>
						<Download size={13} />
					</Button>

					<Button
						variant="secondary"
						size="icon-sm"
						onclick={handleCopy}
						title={t('common.copy')}
						ariaLabel={t('common.copy')}
					>
						{#if copied}
							<Check size={13} class="text-[var(--success-text)]" />
						{:else}
							<Copy size={13} />
						{/if}
					</Button>
				</div>
			{/if}
		</div>

		<!-- Main Audio Display Canvas -->
		<div
			class="w-full min-h-[380px] max-h-[580px] rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] flex flex-col items-center justify-center relative overflow-hidden group select-none p-6"
		>
			{#if isGenerating}
				<!-- Loading State -->
				<div class="flex flex-col items-center justify-center gap-3 p-8 text-center">
					<div class="relative w-16 h-16 flex items-center justify-center">
						<Loader size={64} />
					</div>
					<span class="text-xs text-[var(--text-muted)] animate-pulse">
						{t('audio.generatingAudio')}
					</span>
				</div>
			{:else if lastGeneratedAudioUrl}
				<!-- Center Playback Player using AudioPlayer variant="preview" -->
				<div class="my-auto w-full">
					<AudioPlayer
						src={lastGeneratedAudioUrl}
						durationSecs={lastGeneratedDuration}
						variant="preview"
					/>
				</div>

				<!-- Meta Overlay Pill at Bottom -->
				<div
					class="absolute bottom-3 left-3 right-3 p-3 rounded-lg bg-[var(--bg-surface)]/90 backdrop-blur-md border border-[var(--border-color)]/80 flex items-center justify-between gap-3 text-xs pointer-events-auto"
				>
					<span class="truncate text-[var(--text-primary)] font-medium">
						"{promptText}"
					</span>
					<div class="flex items-center gap-2 truncate">
						{#if selectedModel}
							<Badge variant="muted" rounded="rounded-md">{selectedModel}</Badge>
						{/if}
						{#if lastGeneratedDuration}
							<Badge variant="muted" rounded="rounded-md">
								{formatElapsedTime(lastGeneratedDuration)}
							</Badge>
						{/if}
					</div>
				</div>
			{:else}
				<!-- Empty State Placeholder -->
				<EmptyState
					title={t('audio.noAudioGeneratedYet')}
					description={t('audio.ttsEmptyGuide')}
					icon={Headphones}
					class="border-0 bg-transparent max-w-sm"
				/>
			{/if}
		</div>
	</div>
</div>

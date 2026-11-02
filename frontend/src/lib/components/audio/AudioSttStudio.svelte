<script lang="ts">
	import type { FeatureModelItem } from '$lib/api/models';
	import { transcribeAudio } from '$lib/api/openai';
	import { Alert, Badge, Button, ModelSelect, Select, Tabs } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { audioState } from '$lib/state';
	import { convertBlobToWav } from '$lib/utils/audio';
	import { formatElapsedTime } from '$lib/utils/format';
	import { FileHeadphone, Mic, Sparkles, Upload } from '@lucide/svelte';
	import { onDestroy, untrack } from 'svelte';
	import AudioPlayerCard from './AudioPlayerCard.svelte';
	import AudioRecorder from './AudioRecorder.svelte';
	import AudioTranscriptionCard from './AudioTranscriptionCard.svelte';

	interface AudioSttStudioProps {
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
	}: AudioSttStudioProps = $props();

	let inputMethod = $state<'file' | 'mic'>('file');
	let selectedLanguage = $state('auto');

	// File input state
	let uploadedFile = $state<File | null>(null);
	let fileAudioUrl = $state<string | null>(null);

	// Recorded audio state
	let recordedBlob = $state<Blob | null>(null);
	let recordedDuration = $state(0);
	let recordedAudioUrl = $state<string | null>(null);

	// Execution state
	let isTranscribing = $state(false);
	let transcriptionError = $state<string | null>(null);
	let transcriptionElapsed = $state(0);
	let transcriptionTimer: ReturnType<typeof setInterval> | null = null;
	let abortController: AbortController | null = null;

	let lastTranscript = $state<string>('');

	let lastSelectionVersion = 0;

	// Listen to item selection from sidebar audio history
	$effect(() => {
		const currentVersion = audioState.selectionVersion;
		if (currentVersion > 0 && currentVersion !== lastSelectionVersion) {
			lastSelectionVersion = currentVersion;
			untrack(() => {
				const item = audioState.selectedItem;
				if (item?.type === 'stt') {
					lastTranscript = item.transcript || '';
					if (item.model) {
						selectedModel = item.model;
					}
					if (item.language && languageOptions.some((l) => l.value === item.language)) {
						selectedLanguage = item.language;
					}
					transcriptionError = null;
				}
			});
		}
	});

	const languageOptions = [
		{ value: 'auto', label: 'Auto Detect' },
		{ value: 'en', label: 'English (en)' },
		{ value: 'pt', label: 'Português (pt)' },
		{ value: 'es', label: 'Español (es)' },
		{ value: 'fr', label: 'Français (fr)' },
		{ value: 'de', label: 'Deutsch (de)' },
		{ value: 'it', label: 'Italiano (it)' },
		{ value: 'ja', label: 'Japanese (ja)' },
		{ value: 'zh', label: 'Chinese (zh)' },
		{ value: 'ru', label: 'Russian (ru)' }
	];

	const inputTabs = [
		{ id: 'file', label: t('audio.uploadFile'), icon: Upload },
		{ id: 'mic', label: t('audio.recordMicrophone'), icon: Mic }
	];

	function handleFileSelected(e: Event) {
		const target = e.target as HTMLInputElement;
		if (target.files && target.files.length > 0) {
			const file = target.files[0];
			uploadedFile = file;
			if (fileAudioUrl) {
				URL.revokeObjectURL(fileAudioUrl);
			}
			fileAudioUrl = URL.createObjectURL(file);
		}
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		if (e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files.length > 0) {
			const file = e.dataTransfer.files[0];
			uploadedFile = file;
			if (fileAudioUrl) {
				URL.revokeObjectURL(fileAudioUrl);
			}
			fileAudioUrl = URL.createObjectURL(file);
		}
	}

	function handleDragOver(e: DragEvent) {
		e.preventDefault();
	}

	function handleRecordingComplete(blob: Blob, durationSecs: number) {
		recordedBlob = blob;
		recordedDuration = durationSecs;
		if (recordedAudioUrl) {
			URL.revokeObjectURL(recordedAudioUrl);
		}
		recordedAudioUrl = URL.createObjectURL(blob);
	}

	let activeAudioFile = $derived.by(() => {
		if (inputMethod === 'file') return uploadedFile;
		if (inputMethod === 'mic' && recordedBlob) {
			return new File([recordedBlob], `recording-${Date.now()}.wav`, {
				type: 'audio/wav'
			});
		}
		return null;
	});

	let activePreviewUrl = $derived(inputMethod === 'file' ? fileAudioUrl : recordedAudioUrl);

	async function handleTranscribe() {
		const file = activeAudioFile;
		if (!file || !selectedModel || isTranscribing) return;

		isTranscribing = true;
		transcriptionError = null;
		transcriptionElapsed = 0;
		abortController = new AbortController();

		const startTime = Date.now();
		transcriptionTimer = setInterval(() => {
			transcriptionElapsed = Math.floor((Date.now() - startTime) / 100) / 10;
		}, 100);

		try {
			let uploadFile = file;
			if (!file.name.toLowerCase().endsWith('.wav')) {
				try {
					const { blob: convertedWav, duration: convDur } = await convertBlobToWav(file);
					const baseName = file.name.substring(0, file.name.lastIndexOf('.')) || file.name;
					uploadFile = new File([convertedWav], `${baseName}.wav`, { type: 'audio/wav' });
					if (!recordedDuration && convDur) {
						recordedDuration = convDur;
					}
				} catch (convErr) {
					console.warn('Could not pre-convert uploaded audio to WAV:', convErr);
				}
			}

			const params: any = {
				file: uploadFile,
				model: selectedModel
			};
			if (selectedLanguage && selectedLanguage !== 'auto') {
				params.language = selectedLanguage;
			}

			const res = await transcribeAudio(params, abortController.signal);
			const textResult = res.text || '';
			lastTranscript = textResult;

			// Add to global audio state
			const wordCount = textResult.trim() ? textResult.trim().split(/\s+/).length : 0;
			audioState.addItem({
				id: `stt-${Date.now()}`,
				type: 'stt',
				model: selectedModel,
				timestamp: Date.now(),
				transcript: textResult,
				filename: file.name,
				language: selectedLanguage !== 'auto' ? selectedLanguage : undefined,
				wordCount,
				durationSecs: recordedDuration || undefined,
				audioUrl: activePreviewUrl || undefined
			});
		} catch (err: unknown) {
			console.error('STT transcription error:', err);
			if (err instanceof Error && err.name === 'AbortError') {
				transcriptionError = t('audio.interrupted');
			} else {
				const msg = err instanceof Error ? err.message : String(err);
				transcriptionError = msg || t('audio.sttError');
			}
		} finally {
			if (transcriptionTimer) {
				clearInterval(transcriptionTimer);
				transcriptionTimer = null;
			}
			isTranscribing = false;
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
		if (transcriptionTimer) {
			clearInterval(transcriptionTimer);
		}
		if (fileAudioUrl) {
			URL.revokeObjectURL(fileAudioUrl);
		}
		if (recordedAudioUrl) {
			URL.revokeObjectURL(recordedAudioUrl);
		}
	});
</script>

<div class="grid grid-cols-1 lg:grid-cols-12 gap-5 sm:gap-6 items-start">
	<!-- Left Column: Audio Input & STT Controls -->
	<div class="lg:col-span-5 flex flex-col gap-4">
		<!-- Model Selection -->
		<div class="flex flex-col gap-1.5">
			<div class="flex items-center justify-between">
				<label for="stt-model-select" class="text-xs font-semibold text-[var(--text-secondary)]">
					{t('audio.selectSttModel')}
				</label>
				{#if isModelRunning}
					<Badge variant="success" class="text-[10px]">
						{t('common.active')}
					</Badge>
				{/if}
			</div>
			<ModelSelect
				id="stt-model-select"
				bind:value={selectedModel}
				{models}
				placeholder={t('audio.selectModelPlaceholder')}
				onopen={onLoadModels}
			/>
		</div>

		<!-- Language Selection -->
		<div class="flex flex-col gap-1.5">
			<label for="stt-lang-select" class="text-xs font-semibold text-[var(--text-secondary)]">
				{t('audio.language')}
			</label>
			<Select
				id="stt-lang-select"
				bind:value={selectedLanguage}
				options={languageOptions}
				placeholder={t('audio.selectLanguage')}
				disabled={isTranscribing}
			/>
		</div>

		<!-- Audio Source Selection (Tabs: Upload or Mic) -->
		<div class="flex flex-col gap-2 pt-1">
			<span class="text-xs font-semibold text-[var(--text-secondary)]">
				{t('audio.audioSource')}
			</span>
			<Tabs variant="pills" items={inputTabs} bind:value={inputMethod} />

			{#if inputMethod === 'file'}
				<!-- File Upload Dropzone -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					ondrop={handleDrop}
					ondragover={handleDragOver}
					class="flex flex-col items-center justify-center p-6 border-2 border-dashed border-[var(--border-color)] rounded-xl hover:border-[var(--primary)] transition-colors cursor-pointer bg-[var(--bg-hover)]/30 text-center relative"
				>
					<input
						type="file"
						accept="audio/*"
						onchange={handleFileSelected}
						disabled={isTranscribing}
						class="absolute inset-0 opacity-0 cursor-pointer w-full h-full disabled:cursor-not-allowed"
						aria-label={t('audio.uploadAudioFile')}
					/>
					<FileHeadphone size={28} class="text-[var(--text-muted)] mb-2 shrink-0" />
					{#if uploadedFile}
						<span class="text-xs font-semibold text-[var(--primary)] truncate max-w-xs">
							{uploadedFile.name}
						</span>
						<span class="text-[11px] text-[var(--text-muted)]">
							{(uploadedFile.size / (1024 * 1024)).toFixed(2)} MB
						</span>
					{:else}
						<span class="text-xs font-semibold text-[var(--text-primary)]">
							{t('audio.dragDropAudio')}
						</span>
						<span class="text-[11px] text-[var(--text-muted)] mt-1">
							{t('audio.supportedFormats')}
						</span>
					{/if}
				</div>
			{:else}
				<!-- Microphone Recording Studio -->
				<AudioRecorder onRecordingComplete={handleRecordingComplete} />
			{/if}
		</div>

		<!-- Audio Playback Preview if file or recording is loaded -->
		{#if activePreviewUrl}
			<div class="pt-1">
				<AudioPlayerCard
					src={activePreviewUrl}
					title={uploadedFile ? uploadedFile.name : t('audio.recordedClip')}
					durationSecs={inputMethod === 'mic' ? recordedDuration : 0}
				/>
			</div>
		{/if}

		<!-- Transcribe Action Button (Mirroring /image Generate Button) -->
		<div class="pt-2">
			<Button
				variant="primary"
				size="lg"
				class="w-full font-semibold"
				loading={isTranscribing}
				disabled={isTranscribing || !activeAudioFile || !selectedModel}
				onclick={handleTranscribe}
			>
				{#if isTranscribing}
					<span
						>{t('image.generatingWithTime', {
							time: formatElapsedTime(transcriptionElapsed)
						})}</span
					>
				{:else}
					<Sparkles size={16} class="shrink-0" />
					<span>{t('audio.transcribeAudio')}</span>
				{/if}
			</Button>
		</div>

		<!-- Error Feedback -->
		{#if transcriptionError}
			<div class="pt-2">
				<Alert variant="error" title={t('audio.transcriptionError')} message={transcriptionError} />
			</div>
		{/if}
	</div>

	<!-- Right Column: Transcription Output Result (Mirroring ImagePreviewCard) -->
	<div class="lg:col-span-7 flex flex-col gap-5">
		<AudioTranscriptionCard transcript={lastTranscript} model={selectedModel} {isTranscribing} />
	</div>
</div>

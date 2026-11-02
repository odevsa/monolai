<script lang="ts">
	import { getFeatureModels, type FeatureModelItem } from '$lib/api/models';
	import { AudioSttStudio, AudioTtsStudio } from '$lib/components/audio';
	import { FeatureGate, PageHeader, Tabs } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { runningModelsState } from '$lib/runningModelsStore';
	import { featuresState } from '$lib/state/features.svelte';
	import { audioState } from '$lib/state';
	import { Headphones, Mic } from '@lucide/svelte';
	import { onMount, untrack } from 'svelte';

	// Tab configuration for audio modes (extensible for future features)
	let activeTab = $state('tts');

	let audioTabs = $derived([
		{ id: 'tts', label: t('audio.ttsTab'), icon: Headphones },
		{ id: 'stt', label: t('audio.sttTab'), icon: Mic }
	]);

	// Model state
	let registeredAudioModels = $state<FeatureModelItem[]>([]);
	let selectedTtsModel = $state<string>('');
	let selectedSttModel = $state<string>('');

	// Separate models by capability / modality
	let ttsModels = $derived.by(() => {
		const filtered = registeredAudioModels.filter(
			(m) =>
				m.modality === 'text-to-speech' ||
				(!m.modality && m.runtime.includes('audio')) ||
				m.id.toLowerCase().includes('tts') ||
				m.id.toLowerCase().includes('kokoro')
		);
		return filtered.length > 0 ? filtered : registeredAudioModels;
	});

	let sttModels = $derived.by(() => {
		const filtered = registeredAudioModels.filter(
			(m) =>
				m.modality === 'speech-to-text' ||
				(!m.modality && m.runtime.includes('audio')) ||
				m.id.toLowerCase().includes('asr') ||
				m.id.toLowerCase().includes('whisper') ||
				m.id.toLowerCase().includes('sensevoice')
		);
		return filtered.length > 0 ? filtered : registeredAudioModels;
	});

	let activeRunningModels = $derived(runningModelsState.runningModels);

	let isTtsModelRunning = $derived(
		selectedTtsModel ? activeRunningModels.some((m) => m.model_id === selectedTtsModel) : false
	);

	let isSttModelRunning = $derived(
		selectedSttModel ? activeRunningModels.some((m) => m.model_id === selectedSttModel) : false
	);

	async function loadAudioModels() {
		try {
			registeredAudioModels = await getFeatureModels(['audio', 'text-to-speech', 'speech-to-text']);
		} catch (err) {
			console.error('Error fetching audio models via OpenAI SDK:', err);
		}
	}

	$effect(() => {
		if (featuresState.isInitialized) {
			const _ = featuresState.lastUpdated;
			loadAudioModels();
		}
	});

	// Restore selected models from localStorage
	$effect(() => {
		if (ttsModels.length > 0) {
			const saved =
				typeof localStorage !== 'undefined'
					? localStorage.getItem('monolai:selected_tts_model')
					: null;
			if (saved && ttsModels.some((m) => m.id === saved)) {
				selectedTtsModel = saved;
			} else if (!selectedTtsModel || !ttsModels.some((m) => m.id === selectedTtsModel)) {
				selectedTtsModel = ttsModels[0].id;
			}
		}
	});

	$effect(() => {
		if (sttModels.length > 0) {
			const saved =
				typeof localStorage !== 'undefined'
					? localStorage.getItem('monolai:selected_stt_model')
					: null;
			if (saved && sttModels.some((m) => m.id === saved)) {
				selectedSttModel = saved;
			} else if (!selectedSttModel || !sttModels.some((m) => m.id === selectedSttModel)) {
				selectedSttModel = sttModels[0].id;
			}
		}
	});

	// Save selected models to localStorage
	$effect(() => {
		if (selectedTtsModel && typeof localStorage !== 'undefined') {
			localStorage.setItem('monolai:selected_tts_model', selectedTtsModel);
		}
	});

	$effect(() => {
		if (selectedSttModel && typeof localStorage !== 'undefined') {
			localStorage.setItem('monolai:selected_stt_model', selectedSttModel);
		}
	});

	let lastSelectionVersion = 0;

	// Sync active tab only when an item is explicitly selected from sidebar history
	$effect(() => {
		const currentVersion = audioState.selectionVersion;
		if (currentVersion > 0 && currentVersion !== lastSelectionVersion) {
			lastSelectionVersion = currentVersion;
			untrack(() => {
				const item = audioState.selectedItem;
				if (item) {
					if (item.type === 'tts') {
						activeTab = 'tts';
					} else if (item.type === 'stt') {
						activeTab = 'stt';
					}
				}
			});
		}
	});

	onMount(() => {
		loadAudioModels();
	});
</script>

<svelte:head>
	<title>{t('app.title')} - {t('navigation.audio')}</title>
</svelte:head>

<FeatureGate features={['audio']} showCard={true}>
	<div class="w-full px-4 sm:px-8 py-5 md:py-8 pt-16 md:pt-16 max-w-7xl mx-auto box-border">
		<div class="flex flex-col gap-5 sm:gap-6">
			<!-- Navigation Tabs & Studio Card Container -->
			<Tabs
				variant="card"
				items={audioTabs}
				bind:value={activeTab}
				cardClass="p-5 sm:p-6 shadow-sm"
			>
				{#snippet header()}
					<PageHeader
						title={t('audio.pageTitle')}
						subtitle={t('audio.pageSubtitle')}
						icon={Headphones}
					/>
				{/snippet}

				{#if activeTab === 'tts'}
					<AudioTtsStudio
						models={ttsModels}
						bind:selectedModel={selectedTtsModel}
						isModelRunning={isTtsModelRunning}
						onLoadModels={loadAudioModels}
					/>
				{:else if activeTab === 'stt'}
					<AudioSttStudio
						models={sttModels}
						bind:selectedModel={selectedSttModel}
						isModelRunning={isSttModelRunning}
						onLoadModels={loadAudioModels}
					/>
				{/if}
			</Tabs>
		</div>
	</div>
</FeatureGate>

<script lang="ts">
	import { getFeatureModels, type FeatureModelItem } from '$lib/api/models';
	import { generateImages } from '$lib/api/openai';
	import { ImageLightboxModal, ImagePreviewCard, ImagePromptStudio } from '$lib/components/image';
	import { FeatureGate, PageHeader } from '$lib/components/ui';
	import { t, tArray } from '$lib/i18n';
	import { runningModelsState } from '$lib/runningModelsStore';
	import { imageGalleryState } from '$lib/state';
	import { featuresState, onFeaturesChange } from '$lib/state/features.svelte';
	import type { GeneratedImageItem as GeneratedImage } from '$lib/types/chat';
	import { copyImageDataUrlToClipboard } from '$lib/utils/clipboard';
	import { Image as ImageIcon } from '@lucide/svelte';
	import { onMount } from 'svelte';

	// Component & Generation State
	let selectedModel = $state<string>('');
	let randomPrompts = $derived(tArray('image.randomPrompts'));

	let prompt = $state('');
	let negativePrompt = $state('');
	let selectedSize = $state('512x512');
	let selectedCount = $state('1');
	let steps = $state(20);
	let cfgScale = $state(7.5);
	let seed = $state<number | string>('');

	// Execution & Output State
	let isGenerating = $state(false);
	let generationError = $state<string | null>(null);
	let generationElapsed = $state(0);
	let generationTimer: ReturnType<typeof setInterval> | null = null;
	let generationAbortController: AbortController | null = null;
	let currentResult = $state<GeneratedImage[]>([]);
	let selectedResultImage = $state<GeneratedImage | null>(null);

	// UI feedback states
	let lightboxOpen = $state(false);
	let copiedImage = $state(false);

	let activeRunningModels = $derived(runningModelsState.runningModels);
	let isModelRunning = $derived(
		selectedModel ? activeRunningModels.some((m) => m.model_id === selectedModel) : false
	);

	let registeredImageModels = $state<FeatureModelItem[]>([]);

	async function loadImageModels() {
		try {
			registeredImageModels = await getFeatureModels(['image-generation']);
		} catch (err) {
			console.error('Error fetching image models via OpenAI SDK:', err);
		}
	}

	$effect(() => {
		if (featuresState.isInitialized) {
			const _ = featuresState.lastUpdated;
			loadImageModels();
		}
	});

	$effect(() => {
		if (registeredImageModels.length > 0) {
			const saved =
				typeof localStorage !== 'undefined'
					? localStorage.getItem('monolai:selected_image_model')
					: null;
			if (saved && registeredImageModels.some((m) => m.id === saved)) {
				selectedModel = saved;
			} else if (!selectedModel || !registeredImageModels.some((m) => m.id === selectedModel)) {
				selectedModel = registeredImageModels[0].id;
			}
		}
	});

	$effect(() => {
		if (selectedModel && typeof localStorage !== 'undefined') {
			localStorage.setItem('monolai:selected_image_model', selectedModel);
		}
	});

	$effect(() => {
		if (imageGalleryState.selectionVersion > 0 && imageGalleryState.selectedImage) {
			selectedResultImage = imageGalleryState.selectedImage;
			currentResult = [imageGalleryState.selectedImage];
		}
	});

	function handleModelUnloadInterruption(targetModelId?: string) {
		if (!isGenerating) return;
		if (!targetModelId || targetModelId === '*' || targetModelId === selectedModel) {
			if (generationAbortController) {
				generationAbortController.abort();
				generationAbortController = null;
			}
			if (generationTimer) {
				clearInterval(generationTimer);
				generationTimer = null;
			}
			isGenerating = false;
			generationError = t('image.interrupted');
		}
	}

	async function handleGenerate() {
		if (!prompt.trim() || !selectedModel || isGenerating) return;

		isGenerating = true;
		generationError = null;
		generationElapsed = 0;
		generationAbortController = new AbortController();

		const startTime = Date.now();
		generationTimer = setInterval(() => {
			generationElapsed = Math.floor((Date.now() - startTime) / 100) / 10;
		}, 100);

		const rawPrompt = prompt.trim();
		const userProvidedSeed =
			seed !== '' && !isNaN(Number(seed)) && Number(seed) >= 0 ? Number(seed) : undefined;
		const effectiveSeed =
			userProvidedSeed !== undefined ? userProvidedSeed : Math.floor(Math.random() * 2147483647);
		const n = parseInt(selectedCount, 10) || 1;

		const extraArgs: Record<string, any> = {
			seed: effectiveSeed,
			sample_steps: Number(steps) || 20,
			cfg_scale: Number(cfgScale) || 7.5
		};
		if (negativePrompt.trim()) {
			extraArgs.negative_prompt = negativePrompt.trim();
		}

		const promptWithExtraArgs = rawPrompt.includes('<sd_cpp_extra_args>')
			? rawPrompt
			: `${rawPrompt} <sd_cpp_extra_args>${JSON.stringify(extraArgs)}</sd_cpp_extra_args>`;

		const requestBody: Record<string, any> = {
			model: selectedModel,
			prompt: promptWithExtraArgs,
			n,
			size: selectedSize,
			response_format: 'b64_json',
			seed: effectiveSeed
		};

		if (negativePrompt.trim()) {
			requestBody.negative_prompt = negativePrompt.trim();
		}
		if (steps) {
			requestBody.steps = Number(steps);
		}
		if (cfgScale) {
			requestBody.cfg_scale = Number(cfgScale);
		}

		try {
			const data = await generateImages(requestBody as any, generationAbortController.signal);
			const totalSecs = Math.round((Date.now() - startTime) / 100) / 10;

			if (!data.data || !Array.isArray(data.data) || data.data.length === 0) {
				throw new Error('No image data returned from image generation endpoint');
			}

			const generatedList: GeneratedImage[] = data.data.map((item: any, idx: number) => {
				let imageSrc = '';
				if (item.b64_json) {
					imageSrc = item.b64_json.startsWith('data:')
						? item.b64_json
						: `data:image/png;base64,${item.b64_json}`;
				} else if (item.url) {
					imageSrc = item.url;
				}

				return {
					id: `img-${Date.now()}-${idx}`,
					timestamp: Date.now(),
					prompt: rawPrompt,
					negativePrompt: negativePrompt.trim() || undefined,
					model: selectedModel,
					size: selectedSize,
					steps,
					cfgScale,
					seed: effectiveSeed,
					src: imageSrc,
					generationTimeSecs: totalSecs
				};
			});

			currentResult = generatedList;
			selectedResultImage = generatedList[0] || null;

			// Add to global gallery store
			imageGalleryState.addImage(generatedList);
		} catch (err: any) {
			if (err.name === 'AbortError') {
				generationError = generationError || t('image.interrupted');
			} else {
				generationError = err.message || 'Image generation failed';
			}
		} finally {
			if (generationTimer) {
				clearInterval(generationTimer);
				generationTimer = null;
			}
			generationAbortController = null;
			isGenerating = false;
		}
	}

	function insertRandomPrompt() {
		const list = randomPrompts;
		if (list.length === 0) return;
		let nextPrompt = prompt;
		let attempts = 0;
		while (attempts < 10) {
			const candidate = list[Math.floor(Math.random() * list.length)];
			if (candidate !== prompt || list.length === 1) {
				nextPrompt = candidate;
				break;
			}
			attempts++;
		}
		prompt = nextPrompt;
	}

	function reuseParameters(item: GeneratedImage) {
		prompt = item.prompt;
		if (item.negativePrompt) {
			negativePrompt = item.negativePrompt;
		}
		if (item.size) {
			selectedSize = item.size;
		}
		if (item.steps) {
			steps = item.steps;
		}
		if (item.cfgScale) {
			cfgScale = item.cfgScale;
		}
		if (item.seed !== undefined) {
			seed = item.seed;
		}
		if (registeredImageModels.some((m) => m.id === item.model)) {
			selectedModel = item.model;
		}
	}

	function downloadImage(item: GeneratedImage) {
		const a = document.createElement('a');
		a.href = item.src;
		const sanitized = item.prompt.slice(0, 30).replace(/[^a-zA-Z0-9_-]/g, '_');
		a.download = `monolai-${item.model}-${sanitized}-${Date.now()}.png`;
		document.body.appendChild(a);
		a.click();
		document.body.removeChild(a);
	}

	async function copyImageToClipboard(item: GeneratedImage) {
		const success = await copyImageDataUrlToClipboard(item.src);
		if (success) {
			copiedImage = true;
			setTimeout(() => (copiedImage = false), 2000);
		}
	}

	$effect(() => {
		if (isGenerating && selectedModel) {
			if (runningModelsState.unloadingModelIds.has(selectedModel)) {
				handleModelUnloadInterruption(selectedModel);
			}
		}
	});

	onMount(() => {
		loadImageModels();
		const unsubFeatures = onFeaturesChange(() => {
			loadImageModels();
		});
		imageGalleryState.loadFromStorage();
		const onUnload = (e: Event) => {
			const customEvent = e as CustomEvent<{ modelId?: string }>;
			handleModelUnloadInterruption(customEvent.detail?.modelId);
		};
		if (typeof window !== 'undefined') {
			window.addEventListener('monolai:model-unload', onUnload);
		}

		return () => {
			unsubFeatures();
			if (typeof window !== 'undefined') {
				window.removeEventListener('monolai:model-unload', onUnload);
			}
			if (generationAbortController) {
				generationAbortController.abort();
			}
			if (generationTimer) {
				clearInterval(generationTimer);
			}
		};
	});
</script>

<svelte:head>
	<title>{t('app.title')} - {t('navigation.image')}</title>
</svelte:head>

<FeatureGate features={['image-generation']} showCard={true}>
	<div class="w-full px-4 sm:px-8 py-5 md:py-8 pt-16 md:pt-16 max-w-7xl mx-auto box-border">
		<div class="flex flex-col gap-5 sm:gap-6">
			<!-- Page Header -->
			<PageHeader
				title={t('image.pageTitle')}
				subtitle={t('image.pageSubtitle')}
				icon={ImageIcon}
			/>

			<!-- Main Generation Studio Grid -->
			<div class="grid grid-cols-1 lg:grid-cols-12 gap-5 sm:gap-6 items-start">
				<!-- Left Column: Controls & Prompt Studio -->
				<div class="lg:col-span-5 flex flex-col gap-4">
					<ImagePromptStudio
						bind:selectedModel
						bind:prompt
						bind:negativePrompt
						bind:selectedSize
						bind:selectedCount
						bind:steps
						bind:cfgScale
						bind:seed
						{isGenerating}
						{isModelRunning}
						{generationElapsed}
						{generationError}
						models={registeredImageModels}
						onGenerate={handleGenerate}
						onLoadModels={loadImageModels}
						onInsertRandomPrompt={insertRandomPrompt}
					/>
				</div>

				<!-- Right Column: Showcase & Preview Card -->
				<div class="lg:col-span-7 flex flex-col gap-5">
					<ImagePreviewCard
						selectedImage={selectedResultImage}
						variations={currentResult}
						{isGenerating}
						{copiedImage}
						onDownload={() => selectedResultImage && downloadImage(selectedResultImage)}
						onReuse={() => selectedResultImage && reuseParameters(selectedResultImage)}
						onCopy={() => selectedResultImage && copyImageToClipboard(selectedResultImage)}
						onOpenLightbox={() => (lightboxOpen = true)}
						onSelectVariation={(item) => (selectedResultImage = item)}
					/>
				</div>
			</div>
		</div>
	</div>

	<!-- Fullscreen Lightbox Modal -->
	<ImageLightboxModal
		image={selectedResultImage}
		isOpen={lightboxOpen}
		onClose={() => (lightboxOpen = false)}
		onDownload={downloadImage}
	/>
</FeatureGate>

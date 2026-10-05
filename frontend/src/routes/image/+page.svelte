<script lang="ts">
	import Alert from '$lib/components/Alert.svelte';
	import Badge from '$lib/components/Badge.svelte';
	import Card from '$lib/components/Card.svelte';
	import ModelBadge from '$lib/components/ModelBadge.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import Select from '$lib/components/Select.svelte';
	import FeatureGate from '$lib/components/FeatureGate.svelte';
	import { featuresStore } from '$lib/featuresStore';
	import { runningModels } from '$lib/runningModelsStore';
	import { formatElapsedTime } from '$lib/utils/format';
	import {
		Box,
		Boxes,
		Check,
		ChevronDown,
		ChevronUp,
		Copy,
		Dices,
		Download,
		Image as ImageIcon,
		Maximize2,
		RefreshCw,
		RotateCcwClock,
		Sliders,
		Sparkles,
		Trash,
		X
	} from '@lucide/svelte';
	import { onMount } from 'svelte';

	interface ModelRecord {
		id: string;
		runtime: string;
		flags: string;
		file_exists: boolean;
		created_at?: string;
	}

	interface RuntimeManifest {
		id: string;
		name: string;
		features: string[];
	}

	interface GeneratedImage {
		id: string;
		timestamp: number;
		prompt: string;
		negativePrompt?: string;
		model: string;
		size: string;
		steps: number;
		cfgScale: number;
		seed?: number;
		src: string; // data:image/png;base64,... or url
		generationTimeSecs?: number;
	}

	const SIZE_OPTIONS = [
		{ value: '256x256', label: '256 × 256 (Square - Draft)' },
		{ value: '512x512', label: '512 × 512 (Square - Fast)' },
		{ value: '768x768', label: '768 × 768 (Square - Standard)' },
		{ value: '1024x1024', label: '1024 × 1024 (Square - SDXL / FLUX)' },
		{ value: '768x512', label: '768 × 512 (Landscape 3:2)' },
		{ value: '512x768', label: '512 × 768 (Portrait 2:3)' },
		{ value: '1024x768', label: '1024 × 768 (Landscape 4:3)' },
		{ value: '768x1024', label: '768 × 1024 (Portrait 3:4)' }
	];

	const COUNT_OPTIONS = [
		{ value: '1', label: '1 Image' },
		{ value: '2', label: '2 Images' },
		{ value: '4', label: '4 Images' }
	];

	const RANDOM_PROMPTS: string[] = [
		'A futuristic cyberpunk city with neon reflections in rain, cinematic lighting, volumetric fog, 8k resolution',
		'Majestic snow-capped mountain range under a galaxy night sky with the Milky Way, ultra-detailed landscape',
		'Studio portrait of a cute red panda wearing vintage aviator goggles and a leather jacket, 35mm photograph',
		'Serene Japanese zen garden with blooming cherry blossoms, mossy stones, Koi pond, soft morning mist',
		'Hyper-realistic macro photography of a crystal butterfly resting on a glowing bioluminescent flower',
		'Epic fantasy castle perched on a floating island surrounded by golden sunset clouds and waterfalls',
		'Cozy coffee shop interior on a rainy afternoon, warm ambient lighting, wooden furniture, steam from coffee cup',
		'Steampunk airship soaring through thunderous storm clouds with brass gears and glowing copper conduits',
		'Enchanted ancient forest with giant glowing mushrooms, ethereal light rays filtering through trees, fairies',
		'Portrait of an astronaut floating in deep space looking back at Earth, visor reflecting stars, cinematic 8k',
		'A cute baby dragon sleeping curled up around a pile of shiny gold coins and gemstones, 3d render style',
		'Gothic cathedral interior with colorful stained-glass windows projecting kaleidoscopic light onto stone floor',
		'Vibrant coral reef teeming with exotic sea creatures, manta rays, sunbeams penetrating clear turquoise water',
		'Futuristic sports car speeding across a desert highway at twilight, motion blur, taillight light trails',
		'Mythical white phoenix with glowing feathered wings rising from gentle blue flames, fantasy art illustration',
		'Modern minimalist villa built into ocean cliffside with infinity pool and panoramic sunset horizon',
		'A cybernetic samurai warrior standing in a neon-drenched Tokyo alleyway at night, reflections, blade glowing',
		'Whimsical treehouse village connected by rope bridges in an autumn forest with falling golden leaves',
		'Intricate mechanical watch mechanism with exposed brass gears, tourbillon movement, macro lens photography',
		'Ancient Egyptian temple ruins during a golden hour sandstorm, mysterious statues partially buried in dunes',
		'Cute fluffy cat wearing an astronaut helmet, floating weightlessly in space with tiny floating fish snacks',
		'Watercolor painting of a quaint European cobblestone canal street with bicycles and flower pots',
		'Bioluminescent underwater cavern with glowing jellyfishes illuminating ancient submerged stone statues',
		'Retro 1980s synthwave sunset grid landscape with neon palm trees, wireframe mountains, chrome aesthetics',
		'Dramatic cinematic shot of a lone wanderer with a staff standing before an ancient colossal stone portal',
		'Isometric 3D diorama of a cozy miniature bookstore room, warm reading lamp, detailed bookshelves, tiny cat',
		'Close-up portrait of an elven princess with silver braided hair and delicate emerald jewelry, fantasy concept art',
		'Nordic fjord landscape with a small wooden red cabin on the shore, mirror reflections in calm fjord water',
		'A friendly robot gardener watering colorful alien flowers in an orbital glass greenhouse dome',
		'Surreal dreamscape with melting pocket watches draped over branches, floating clockwork gears, Dali style',
		'Golden retriever puppy sitting proudly in a meadow of lavender flowers, golden hour soft bokeh lighting',
		'Victorian library with towering dark wood bookcases, spiral staircases, leather armchairs, rolling ladders',
		'An imposing storm giant forged of lightning and storm clouds towering over a rugged mountain ridge',
		'Cyberpunk street food stall with vapor steam rising, neon Japanese signage, appetizing noodle bowls, rain',
		'Art nouveau illustration of a maiden surrounded by climbing vines, stylized floral patterns and gold leaf accents',
		'A cozy rustic cabin surrounded by deep winter snow with warm glowing windows and smoke rising from chimney',
		'Majestic stag with glowing crystal antlers standing in a misty twilight forest glade',
		'Sci-fi laboratory with holographic displays, glowing particle accelerator ring, high-tech cleanroom aesthetic',
		'Portrait of a wise owl with steampunk brass monocle, gears, and leather collar against dark library backdrop',
		'Tropical island paradise with turquoise water, overwater wooden bungalows, pristine white sand beach, sunny day',
		'Candid street photograph in Paris at dusk, wet pavement reflecting street lamps, vintage Citroen car parked',
		'Futuristic modular lunar colony base with glass biodomes and solar arrays under starry black space sky',
		'An ornate porcelain teacup filled with swirling galaxies and miniature stars, ethereal tabletop photography',
		'Vintage oil painting of a majestic clipper ship battling stormy ocean waves with torn sails and lightning',
		'Charming small bakery window display filled with artisan sourdough breads, croissants, and fruit pastries',
		'A cute little robot sitting alone on a grassy hill watching a colorful sunset, Pixar style animation render',
		'Dramatic portrait of an ancient Norse warrior in fur armor standing on a snowy mountain peak at dusk',
		'Floating sky island archipelago with waterfalls dropping into infinite sky, vibrant lush greenery, sunny daylight',
		'Vibrant Holi festival celebration with colorful powder explosions in air, smiling people, joyful cinematic capture',
		'Abstract geometric architectural sculpture of smooth concrete and glass intersecting with desert sunlight and shadows'
	];

	// Component State
	let selectedModel = $state<string>('');

	// Generation Form State
	let prompt = $state('');
	let negativePrompt = $state('');
	let selectedSize = $state('512x512');
	let selectedCount = $state('1');
	let steps = $state(20);
	let cfgScale = $state(7.5);
	let seed = $state<number | string>('');
	let showAdvanced = $state(false);
	let showNegative = $state(false);

	// Execution & Output State
	let isGenerating = $state(false);
	let generationError = $state<string | null>(null);
	let generationElapsed = $state(0);
	let generationTimer: ReturnType<typeof setInterval> | null = null;
	let currentResult = $state<GeneratedImage[]>([]);
	let selectedResultImage = $state<GeneratedImage | null>(null);
	let history = $state<GeneratedImage[]>([]);

	// UI feedback states
	let lightboxOpen = $state(false);
	let copiedPrompt = $state(false);
	let copiedImage = $state(false);

	let activeRunningModels = $derived($runningModels);
	let isModelRunning = $derived(
		selectedModel ? activeRunningModels.some((m) => m.model_id === selectedModel) : false
	);

	let registeredImageModels = $derived.by(() => {
		if (!$featuresStore.isInitialized) return [];
		return $featuresStore.models.filter((m) => {
			const runtimeItem = $featuresStore.runtimes.find(
				(r) => r.id.toLowerCase() === m.runtime.toLowerCase()
			);
			if (runtimeItem?.features) {
				return runtimeItem.features.includes('image-generation');
			}
			return (
				m.runtime.toLowerCase() === 'sd-cpp' || m.runtime.toLowerCase().includes('diffusion')
			);
		});
	});

	let modelSelectOptions = $derived(
		registeredImageModels.map((m) => ({
			value: m.id,
			label: `${m.id} (${m.runtime})`
		}))
	);

	let currentModelRecord = $derived(registeredImageModels.find((m) => m.id === selectedModel));

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

	function loadHistoryFromStorage() {
		if (typeof localStorage !== 'undefined') {
			try {
				const saved = localStorage.getItem('monolai:image_history');
				if (saved) {
					history = JSON.parse(saved);
				}
			} catch (err) {
				console.error('Failed to load image history:', err);
			}
		}
	}

	function saveHistoryToStorage(newHistory: GeneratedImage[]) {
		history = newHistory;
		if (typeof localStorage !== 'undefined') {
			try {
				localStorage.setItem('monolai:image_history', JSON.stringify(newHistory.slice(0, 30)));
			} catch (err) {
				console.error('Failed to save image history to storage:', err);
			}
		}
	}

	function clearHistory() {
		history = [];
		if (typeof localStorage !== 'undefined') {
			localStorage.removeItem('monolai:image_history');
		}
	}

	async function handleGenerate() {
		if (!prompt.trim() || !selectedModel || isGenerating) return;

		isGenerating = true;
		generationError = null;
		generationElapsed = 0;

		const startTime = Date.now();
		generationTimer = setInterval(() => {
			generationElapsed = Math.floor((Date.now() - startTime) / 100) / 10;
		}, 100);

		const parsedSeed = seed !== '' && !isNaN(Number(seed)) ? Number(seed) : undefined;
		const n = parseInt(selectedCount, 10) || 1;

		// Standard OpenAI Images Generations Request body
		const requestBody: Record<string, any> = {
			model: selectedModel,
			prompt: prompt.trim(),
			n,
			size: selectedSize,
			response_format: 'b64_json'
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
		if (parsedSeed !== undefined && parsedSeed >= 0) {
			requestBody.seed = parsedSeed;
		}

		try {
			const res = await fetch('/v1/images/generations', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify(requestBody)
			});

			if (!res.ok) {
				const errData = await res.json().catch(() => null);
				const msg = errData?.error?.message || `Server returned ${res.status}: ${res.statusText}`;
				throw new Error(msg);
			}

			const data = await res.json();
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
					prompt: prompt.trim(),
					negativePrompt: negativePrompt.trim() || undefined,
					model: selectedModel,
					size: selectedSize,
					steps,
					cfgScale,
					seed: parsedSeed,
					src: imageSrc,
					generationTimeSecs: totalSecs
				};
			});

			currentResult = generatedList;
			selectedResultImage = generatedList[0] || null;

			// Add to history
			saveHistoryToStorage([...generatedList, ...history]);
		} catch (err: any) {
			generationError = err.message || 'Image generation failed';
		} finally {
			if (generationTimer) {
				clearInterval(generationTimer);
				generationTimer = null;
			}
			isGenerating = false;
		}
	}

	function handleKeyDown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
			e.preventDefault();
			handleGenerate();
		}
	}

	function insertRandomPrompt() {
		if (RANDOM_PROMPTS.length === 0) return;
		let nextPrompt = prompt;
		let attempts = 0;
		while (attempts < 10) {
			const candidate = RANDOM_PROMPTS[Math.floor(Math.random() * RANDOM_PROMPTS.length)];
			if (candidate !== prompt || RANDOM_PROMPTS.length === 1) {
				nextPrompt = candidate;
				break;
			}
			attempts++;
		}
		prompt = nextPrompt;
	}

	function openHistoryPreview(item: GeneratedImage) {
		selectedResultImage = item;
		currentResult = [item];
	}

	function reuseParameters(item: GeneratedImage) {
		prompt = item.prompt;
		if (item.negativePrompt) {
			negativePrompt = item.negativePrompt;
			showNegative = true;
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
			showAdvanced = true;
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

	async function copyPromptToClipboard(text: string) {
		try {
			await navigator.clipboard.writeText(text);
			copiedPrompt = true;
			setTimeout(() => (copiedPrompt = false), 2000);
		} catch (err) {
			console.error('Failed to copy prompt:', err);
		}
	}

	async function copyImageToClipboard(item: GeneratedImage) {
		try {
			if (item.src.startsWith('data:image')) {
				const res = await fetch(item.src);
				const blob = await res.blob();
				await navigator.clipboard.write([new ClipboardItem({ [blob.type]: blob })]);
				copiedImage = true;
				setTimeout(() => (copiedImage = false), 2000);
			}
		} catch (err) {
			console.error('Failed to copy image to clipboard:', err);
		}
	}

	onMount(() => {
		loadHistoryFromStorage();
	});
</script>

<svelte:head>
	<title>Monolai - Image</title>
</svelte:head>

<FeatureGate features={['image-generation']} showCard={true}>
	<div class="page-container">
	<div class="page-inner gap-5 sm:gap-6">
		<!-- Page Header matching Monolai design standards -->
		<PageHeader
			title="Image"
			subtitle="Generate images using local models via OpenAI-compatible API standard"
			icon={ImageIcon}
		/>

			<!-- Main Generation Studio Grid -->
			<div class="grid grid-cols-1 lg:grid-cols-12 gap-5 sm:gap-6 items-start">
				<!-- Left Column: Controls & Prompt Studio (5 cols on lg) -->
				<div class="lg:col-span-5 flex flex-col gap-4">
					<Card>
						<div class="app-card-header pb-1">
							<div class="flex items-center gap-2">
								<Sparkles size={16} class="text-[var(--primary)]" />
								<h3 class="app-card-title text-sm">Prompt Studio</h3>
							</div>
							{#if isModelRunning}
								<Badge variant="success" dot>Active</Badge>
							{/if}
						</div>

						<!-- Model Selector -->
						<div class="flex flex-col gap-1.5">
							<div class="flex items-center justify-between">
								<label
									for="image-model-select"
									class="text-xs font-semibold text-[var(--text-secondary)]"
								>
									Model
								</label>
								{#if currentModelRecord}
									<span class="text-[11px] text-[var(--text-muted)] font-mono">
										{currentModelRecord.runtime}
									</span>
								{/if}
							</div>
							<Select
								bind:value={selectedModel}
								options={modelSelectOptions}
								placeholder="Select an image model..."
							/>
						</div>

						<!-- Prompt Input -->
						<div class="flex flex-col gap-1.5">
							<div class="flex items-center justify-between">
								<div class="flex items-center gap-2">
									<label
										for="image-prompt"
										class="text-xs font-semibold text-[var(--text-secondary)]"
									>
										Prompt
									</label>
									<button
										type="button"
										class="flex items-center gap-1.5 px-2.5 py-1 text-[11px] font-medium rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] text-[var(--text-secondary)] hover:text-[var(--primary)] hover:border-[var(--primary)] transition-colors cursor-pointer"
										onclick={insertRandomPrompt}
										title="Generate random prompt from the list"
									>
										<Dices size={13} class="text-[var(--primary)]" />
										<span>Random</span>
									</button>
								</div>
								<span class="text-[11px] text-[var(--text-muted)]"> Ctrl+Enter to generate </span>
							</div>
							<textarea
								id="image-prompt"
								bind:value={prompt}
								onkeydown={handleKeyDown}
								rows={4}
								placeholder="Describe what you want to see in detail..."
								class="w-full p-3 rounded-xl bg-[var(--bg-primary)] border border-[var(--border-color)] text-sm text-[var(--text-primary)] placeholder:[var(--text-muted)] outline-none resize-y transition-colors focus:border-[var(--primary)] focus:ring-1 focus:ring-[var(--primary)]"
								disabled={isGenerating}></textarea>
						</div>

						<!-- Negative Prompt Toggle -->
						<div class="flex flex-col gap-1.5">
							<button
								type="button"
								class="flex items-center justify-between text-xs font-semibold text-[var(--text-secondary)] bg-transparent border-0 p-0 cursor-pointer hover:text-[var(--text-primary)]"
								onclick={() => (showNegative = !showNegative)}
							>
								<span>Negative Prompt</span>
								{#if showNegative}
									<ChevronUp size={14} />
								{:else}
									<ChevronDown size={14} />
								{/if}
							</button>

							{#if showNegative}
								<textarea
									bind:value={negativePrompt}
									rows={2}
									placeholder="Items or styles to avoid (e.g. blurry, deformed, low quality, artifacts)..."
									class="w-full p-2.5 rounded-xl bg-[var(--bg-primary)] border border-[var(--border-color)] text-xs text-[var(--text-primary)] placeholder:[var(--text-muted)] outline-none resize-y transition-colors focus:border-[var(--primary)]"
									disabled={isGenerating}></textarea>
							{/if}
						</div>

						<!-- Dimensions & Quantity Row -->
						<div class="grid grid-cols-2 gap-3 pt-1">
							<div class="flex flex-col gap-1.5">
								<label
									for="image-size-select"
									class="text-xs font-semibold text-[var(--text-secondary)]"
								>
									Dimensions
								</label>
								<Select
									bind:value={selectedSize}
									options={SIZE_OPTIONS}
									placeholder="Select size..."
								/>
							</div>

							<div class="flex flex-col gap-1.5">
								<label
									for="image-count-select"
									class="text-xs font-semibold text-[var(--text-secondary)]"
								>
									Quantity (n)
								</label>
								<Select
									bind:value={selectedCount}
									options={COUNT_OPTIONS}
									placeholder="Select count..."
								/>
							</div>
						</div>

						<!-- Advanced Parameters Accordion -->
						<div class="border-t border-[var(--border-color)] pt-3 flex flex-col gap-3">
							<button
								type="button"
								class="flex items-center justify-between text-xs font-semibold text-[var(--text-secondary)] bg-transparent border-0 p-0 cursor-pointer hover:text-[var(--text-primary)]"
								onclick={() => (showAdvanced = !showAdvanced)}
							>
								<div class="flex items-center gap-1.5">
									<Sliders size={13} />
									<span>Advanced Parameters</span>
								</div>
								{#if showAdvanced}
									<ChevronUp size={14} />
								{:else}
									<ChevronDown size={14} />
								{/if}
							</button>

							{#if showAdvanced}
								<div class="grid grid-cols-2 gap-3 pt-1">
									<div class="flex flex-col gap-1">
										<div class="flex justify-between text-[11px] text-[var(--text-muted)]">
											<span>Steps</span>
											<span class="font-mono text-[var(--text-primary)]">{steps}</span>
										</div>
										<input
											type="range"
											min="5"
											max="50"
											step="1"
											bind:value={steps}
											class="w-full accent-[var(--primary)] cursor-pointer"
										/>
									</div>

									<div class="flex flex-col gap-1">
										<div class="flex justify-between text-[11px] text-[var(--text-muted)]">
											<span>CFG Scale</span>
											<span class="font-mono text-[var(--text-primary)]">{cfgScale}</span>
										</div>
										<input
											type="range"
											min="1"
											max="20"
											step="0.5"
											bind:value={cfgScale}
											class="w-full accent-[var(--primary)] cursor-pointer"
										/>
									</div>

									<div class="col-span-2 flex flex-col gap-1">
										<label for="image-seed-input" class="text-[11px] text-[var(--text-muted)]">
											Seed (leave empty for random)
										</label>
										<input
											id="image-seed-input"
											type="number"
											bind:value={seed}
											placeholder="-1 (Random)"
											class="w-full px-3 py-1.5 rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] text-xs text-[var(--text-primary)] outline-none focus:border-[var(--primary)]"
										/>
									</div>
								</div>
							{/if}
						</div>

						<!-- Generate Action Button -->
						<div class="pt-2">
							<button
								type="button"
								class="app-btn app-btn-primary w-full py-2.5 text-sm font-semibold rounded-xl flex items-center justify-center gap-2 shadow-xs transition-all"
								disabled={isGenerating || !prompt.trim() || !selectedModel}
								onclick={handleGenerate}
							>
								{#if isGenerating}
									<RefreshCw size={16} class="animate-spin" />
									<span>Generating... ({formatElapsedTime(generationElapsed)})</span>
								{:else}
									<Sparkles size={16} />
									<span>Generate Image</span>
								{/if}
							</button>
						</div>

						{#if generationError}
							<div class="pt-2">
								<Alert variant="error" title="Generation Error:" message={generationError} />
							</div>
						{/if}
					</Card>
				</div>

				<!-- Right Column: Showcase & Gallery (7 cols on lg) -->
				<div class="lg:col-span-7 flex flex-col gap-5">
					<!-- Active / Latest Image Card -->
					<Card class="overflow-hidden">
						<div class="app-card-header pb-1">
							<div class="flex items-center gap-2">
								<ImageIcon size={16} class="text-[var(--primary)]" />
								<h3 class="app-card-title text-sm">Preview</h3>
							</div>

							{#if selectedResultImage}
								<div class="flex items-center gap-1.5">
									<button
										type="button"
										class="app-btn app-btn-secondary app-btn-sm"
										onclick={() => downloadImage(selectedResultImage!)}
										title="Download image"
									>
										<Download size={13} />
										<span class="hidden sm:inline">Download</span>
									</button>

									<button
										type="button"
										class="app-btn app-btn-secondary app-btn-sm"
										onclick={() => copyPromptToClipboard(selectedResultImage!.prompt)}
										title="Copy prompt"
									>
										{#if copiedPrompt}
											<Check size={13} class="text-emerald-500" />
											<span class="text-emerald-500 hidden sm:inline">Copied</span>
										{:else}
											<Copy size={13} />
											<span class="hidden sm:inline">Prompt</span>
										{/if}
									</button>

									<button
										type="button"
										class="app-btn app-btn-secondary app-btn-sm"
										onclick={() => (lightboxOpen = true)}
										title="Fullscreen preview"
									>
										<Maximize2 size={13} />
									</button>
								</div>
							{/if}
						</div>

						<!-- Main Image Display Container -->
						<div
							class="w-full min-h-[380px] max-h-[580px] rounded-xl bg-[var(--bg-primary)] border border-[var(--border-color)] flex items-center justify-center relative overflow-hidden group select-none"
						>
							{#if isGenerating}
								<!-- Loading State -->
								<div class="flex flex-col items-center justify-center gap-3 p-8 text-center">
									<div class="relative w-16 h-16 flex items-center justify-center">
										<div
											class="absolute inset-0 rounded-full border-2 border-[var(--border-color)] border-t-[var(--primary)] animate-spin"
										></div>
										<Sparkles size={24} class="text-[var(--primary)] animate-pulse" />
									</div>
								</div>
							{:else if selectedResultImage}
								<!-- Display Image -->
								<!-- svelte-ignore a11y_click_events_have_key_events -->
								<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
								<img
									src={selectedResultImage.src}
									alt={selectedResultImage.prompt}
									class="w-full h-full object-contain max-h-[560px] cursor-pointer transition-transform duration-200 group-hover:scale-[1.01]"
									onclick={() => (lightboxOpen = true)}
								/>

								<!-- Hover Overlay Pill with Meta -->
								<div
									class="absolute bottom-3 left-3 right-3 p-3 rounded-xl bg-[var(--bg-surface)]/90 backdrop-blur-md border border-[var(--border-color)]/80 flex items-center justify-between gap-3 text-xs opacity-0 group-hover:opacity-100 transition-opacity duration-150 pointer-events-auto"
								>
									<span class="truncate text-[var(--text-primary)] font-medium">
										"{selectedResultImage.prompt}"
									</span>
									<div class="flex items-center gap-2 shrink-0">
										<Badge variant="pill">{selectedResultImage.size}</Badge>
										{#if selectedResultImage.generationTimeSecs}
											<Badge variant="pill">
												{formatElapsedTime(selectedResultImage.generationTimeSecs)}
											</Badge>
										{/if}
									</div>
								</div>
							{:else}
								<!-- Empty State -->
								<div
									class="flex flex-col items-center justify-center gap-3 p-8 text-center max-w-sm"
								>
									<div
										class="w-12 h-12 rounded-2xl bg-[var(--bg-surface)] text-[var(--text-muted)] flex items-center justify-center border border-[var(--border-color)]"
									>
										<ImageIcon size={24} />
									</div>
									<div class="flex flex-col gap-1">
										<span class="text-sm font-semibold text-[var(--text-primary)]">
											No Image Generated Yet
										</span>
										<span class="text-xs text-[var(--text-muted)] leading-relaxed">
											Enter a prompt and click "Generate Image" to render images with your local
											diffusion model.
										</span>
									</div>
								</div>
							{/if}
						</div>

						<!-- Multiple Outputs Carousel/Strip if n > 1 -->
						{#if currentResult.length > 1}
							<div class="flex flex-col gap-2 pt-2">
								<span class="text-xs font-semibold text-[var(--text-muted)]">
									Generated Variations ({currentResult.length})
								</span>
								<div class="flex gap-2.5 overflow-x-auto pb-1">
									{#each currentResult as item (item.id)}
										<button
											type="button"
											class="w-18 h-18 rounded-lg overflow-hidden shrink-0 border-2 cursor-pointer transition-all bg-[var(--bg-primary)] p-0 {selectedResultImage?.id ===
											item.id
												? 'border-[var(--primary)] shadow-sm'
												: 'border-transparent opacity-70 hover:opacity-100'}"
											onclick={() => (selectedResultImage = item)}
										>
											<img src={item.src} alt="Variation" class="w-full h-full object-cover" />
										</button>
									{/each}
								</div>
							</div>
						{/if}
					</Card>

					<!-- Session Gallery / History -->
					{#if history.length > 0}
						<Card>
							<div class="app-card-header pb-1">
								<div class="flex items-center gap-2">
									<RotateCcwClock size={16} class="text-[var(--text-muted)]" />
									<h3 class="app-card-title text-sm">Session History ({history.length})</h3>
								</div>

								<button
									type="button"
									class="app-btn app-btn-secondary app-btn-sm"
									onclick={clearHistory}
									title="Clear image history"
								>
									<Trash size={13} />
									<span>Clear</span>
								</button>
							</div>

							<div
								class="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-5 gap-3 max-h-[300px] overflow-y-auto p-1"
							>
								{#each history as item (item.id)}
									<!-- svelte-ignore a11y_click_events_have_key_events -->
									<!-- svelte-ignore a11y_no_static_element_interactions -->
									<div
										class="group relative rounded-xl overflow-hidden aspect-square bg-[var(--bg-primary)] border border-[var(--border-color)] cursor-pointer transition-all duration-150 hover:border-[var(--border-hover)] {selectedResultImage?.id ===
										item.id
											? 'ring-2 ring-[var(--primary)]'
											: ''}"
										onclick={() => openHistoryPreview(item)}
									>
										<img
											src={item.src}
											alt={item.prompt}
											class="w-full h-full object-cover transition-transform duration-200 group-hover:scale-105"
										/>

										<!-- Hover overlay actions -->
										<div
											class="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity p-2 flex flex-col justify-between"
										>
											<p class="text-[10px] text-white line-clamp-2 m-0 leading-tight">
												{item.prompt}
											</p>
											<div class="flex items-center justify-end gap-1">
												<button
													type="button"
													class="p-1 rounded bg-white/20 text-white hover:bg-white/40 border-0 cursor-pointer"
													onclick={(e) => {
														e.stopPropagation();
														reuseParameters(item);
													}}
													title="Reuse prompt & settings"
												>
													<Sparkles size={11} />
												</button>
												<button
													type="button"
													class="p-1 rounded bg-white/20 text-white hover:bg-white/40 border-0 cursor-pointer"
													onclick={(e) => {
														e.stopPropagation();
														downloadImage(item);
													}}
													title="Download image"
												>
													<Download size={11} />
												</button>
											</div>
										</div>
									</div>
								{/each}
							</div>
						</Card>
					{/if}
				</div>
			</div>
		</div>
	</div>

<!-- Lightbox Modal Preview -->
{#if lightboxOpen && selectedResultImage}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-[200] bg-black/85 backdrop-blur-sm flex flex-col items-center justify-center p-4"
		onclick={() => (lightboxOpen = false)}
	>
		<div
			class="relative max-w-5xl max-h-[90vh] flex flex-col items-center bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-2xl overflow-hidden shadow-2xl p-4 gap-3 pointer-events-auto"
			onclick={(e) => e.stopPropagation()}
		>
			<!-- Top Bar inside Modal -->
			<div
				class="w-full flex items-center justify-between pb-2 border-b border-[var(--border-color)]"
			>
				<div class="flex items-center gap-2 min-w-0">
					<ModelBadge model={selectedResultImage.model} variant="inline" class="text-xs" />
					<Badge variant="pill">{selectedResultImage.size}</Badge>
					{#if selectedResultImage.generationTimeSecs}
						<Badge variant="pill">{formatElapsedTime(selectedResultImage.generationTimeSecs)}</Badge
						>
					{/if}
				</div>

				<div class="flex items-center gap-2">
					<button
						type="button"
						class="app-btn app-btn-secondary app-btn-sm"
						onclick={() => downloadImage(selectedResultImage!)}
						title="Download image"
					>
						<Download size={13} />
						<span>Download</span>
					</button>
					<button
						type="button"
						class="w-7 h-7 flex items-center justify-center rounded-lg bg-[var(--bg-hover)] text-[var(--text-muted)] border-0 cursor-pointer hover:text-[var(--text-primary)]"
						onclick={() => (lightboxOpen = false)}
						title="Close modal"
					>
						<X size={16} />
					</button>
				</div>
			</div>

			<!-- Image in Modal -->
			<div class="flex-1 min-h-0 flex items-center justify-center max-h-[70vh]">
				<img
					src={selectedResultImage.src}
					alt={selectedResultImage.prompt}
					class="max-w-full max-h-[70vh] object-contain rounded-lg"
				/>
			</div>

			<!-- Prompt details at bottom -->
			<div
				class="w-full p-2 rounded-xl bg-[var(--bg-primary)] border border-[var(--border-color)] text-xs text-[var(--text-secondary)]"
			>
				<span class="font-semibold text-[var(--text-primary)]">Prompt:</span>
				{selectedResultImage.prompt}
				{#if selectedResultImage.negativePrompt}
					<div class="mt-1">
						<span class="font-semibold text-[var(--text-primary)]">Negative Prompt:</span>
						{selectedResultImage.negativePrompt}
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
</FeatureGate>

<script lang="ts">
	import { onMount } from 'svelte';
	import type { Snippet } from 'svelte';
	import {
		featuresState,
		refreshFeatures,
		type FeatureCheckResult
	} from '$lib/state/features.svelte';
	import {
		MessageSquare,
		Image as ImageIcon,
		Code,
		Binary,
		FileText,
		Headphones,
		Mic,
		Eye,
		Sparkles,
		Box,
		Boxes,
		ChevronRight,
		CircleAlert,
		RefreshCw
	} from '@lucide/svelte';
	import Card from '$lib/components/Card.svelte';
	import Alert from '$lib/components/Alert.svelte';

	interface Props {
		features: string[] | string;
		showCard?: boolean;
		title?: string;
		description?: string;
		children?: Snippet;
	}

	let { features, showCard = true, title, description, children }: Props = $props();

	// De-para mapping of feature to icon, label, and fallback info
	const FEATURE_MAP: Record<string, { label: string; icon: any; sampleRuntimes?: string }> = {
		chat: {
			label: 'Chat',
			icon: MessageSquare,
			sampleRuntimes: 'LLaMA C++ Server / llama-cpp'
		},
		'image-generation': {
			label: 'Image Generation',
			icon: ImageIcon,
			sampleRuntimes: 'Stable Diffusion C++ / sd-cpp'
		},
		image: {
			label: 'Image Generation',
			icon: ImageIcon,
			sampleRuntimes: 'Stable Diffusion C++ / sd-cpp'
		},
		completion: {
			label: 'Text Completion',
			icon: FileText,
			sampleRuntimes: 'LLaMA C++ Server / llama-cpp'
		},
		'text-generation': {
			label: 'Text Generation',
			icon: FileText,
			sampleRuntimes: 'LLaMA C++ Server / llama-cpp'
		},
		embeddings: {
			label: 'Embeddings',
			icon: Binary,
			sampleRuntimes: 'LLaMA C++ Server / llama-cpp'
		},
		embedding: {
			label: 'Embeddings',
			icon: Binary,
			sampleRuntimes: 'LLaMA C++ Server / llama-cpp'
		},
		code: {
			label: 'Code Generation',
			icon: Code
		},
		'code-generation': {
			label: 'Code Generation',
			icon: Code
		},
		audio: {
			label: 'Audio',
			icon: Headphones
		},
		'speech-to-text': {
			label: 'Speech to Text',
			icon: Mic
		},
		'text-to-speech': {
			label: 'Text to Speech',
			icon: Headphones
		},
		vision: {
			label: 'Vision',
			icon: Eye
		}
	};

	function getFeatureMeta(feat: string) {
		const key = feat.toLowerCase().trim();
		return (
			FEATURE_MAP[key] || {
				label: feat.charAt(0).toUpperCase() + feat.slice(1),
				icon: Sparkles
			}
		);
	}

	let reqFeatures = $derived(Array.isArray(features) ? features : [features]);

	// Reactive derivation based on featuresState
	let checkResults = $derived(
		reqFeatures.map((f) => ({
			feature: f,
			meta: getFeatureMeta(f),
			...featuresState.checkFeature(f)
		}))
	);

	let isSatisfied = $derived(
		featuresState.isInitialized && checkResults.every((r) => r.isSatisfied)
	);

	let missingRuntimes = $derived(checkResults.filter((r) => r.missingRuntime));
	let missingModels = $derived(checkResults.filter((r) => r.missingModel));

	let candidateRuntimeNames = $derived(
		checkResults
			.map((r) => r.candidateNames)
			.filter(Boolean)
			.join(' / ')
	);

	let isLoading = $derived(!featuresState.isInitialized || featuresState.isLoading);

	onMount(() => {
		if (!featuresState.isInitialized) {
			refreshFeatures();
		}
	});

	let cardTitle = $derived.by(() => {
		if (title) return title;
		if (reqFeatures.length === 1) {
			const feat = reqFeatures[0].toLowerCase().trim();
			if (feat === 'image-generation' || feat === 'image') {
				if (missingRuntimes.length > 0 && missingModels.length === 0) {
					return 'No Image Runtime Installed';
				}
				return 'No Image Models Found';
			}
			if (feat === 'chat') {
				if (missingRuntimes.length > 0 && missingModels.length === 0) {
					return 'No Chat Runtime Installed';
				}
				return 'No Chat Models Found';
			}
			const label = getFeatureMeta(feat).label;
			if (missingRuntimes.length > 0 && missingModels.length === 0) {
				return `No ${label} Runtime Installed`;
			}
			return `No ${label} Models Found`;
		}
		return 'Feature Requirements Not Met';
	});
</script>

{#if isLoading}
	{#if showCard}
		<div
			class="flex flex-col items-center justify-center p-8 sm:p-12 gap-3 text-[var(--text-muted)] w-full h-full min-h-[50vh]"
		>
			<RefreshCw class="animate-spin text-[var(--primary)]" size={24} />
			<span class="text-xs font-medium">Verifying feature requirements...</span>
		</div>
	{/if}
{:else if isSatisfied}
	{@render children?.()}
{:else if showCard}
	<div
		class="flex flex-col items-center justify-center p-4 sm:p-6 w-full h-full min-h-[calc(100dvh-4rem)]"
	>
		<Card class="border border-dashed border-[var(--border-color)] max-w-xl w-full">
			<div class="flex flex-col items-center justify-center text-center p-6 sm:p-10 mx-auto w-full">
				<!-- Feature Icons: one icon for each required feature -->
				{#if reqFeatures.length === 1}
					<div
						class="w-14 h-14 rounded-2xl bg-[var(--primary-light)] text-[var(--primary)] flex items-center justify-center mb-4"
					>
						{#snippet singleIcon()}
							{@const Icon = getFeatureMeta(reqFeatures[0]).icon}
							<Icon size={28} />
						{/snippet}
						{@render singleIcon()}
					</div>
				{:else}
					<div class="flex items-center justify-center gap-3 mb-4 flex-wrap">
						{#each reqFeatures as feat}
							<div
								class="w-12 h-12 rounded-2xl bg-[var(--primary-light)] text-[var(--primary)] flex items-center justify-center"
								title={getFeatureMeta(feat).label}
							>
								{#snippet multiIcon()}
									{@const Icon = getFeatureMeta(feat).icon}
									<Icon size={24} />
								{/snippet}
								{@render multiIcon()}
							</div>
						{/each}
					</div>
				{/if}

				<!-- Title -->
				<h3 class="text-lg font-bold text-[var(--text-primary)] m-0 mb-2">
					{cardTitle}
				</h3>

				<!-- Description -->
				<p class="text-sm text-[var(--text-muted)] leading-relaxed m-0 mb-4 max-w-md">
					{#if description}
						{description}
					{:else if reqFeatures.length === 1}
						{@const feat = reqFeatures[0]}
						{@const meta = getFeatureMeta(feat)}
						To {feat === 'chat' ? 'chat' : `use ${meta.label.toLowerCase()}`}, register a model that
						uses a runtime backend with the
						<code
							class="px-1.5 py-0.5 rounded bg-[var(--bg-primary)] text-[var(--text-primary)] font-mono text-xs"
						>
							{feat}
						</code>
						feature
						{#if candidateRuntimeNames}
							(such as <strong class="text-[var(--text-primary)]">{candidateRuntimeNames}</strong>).
						{:else}
							.
						{/if}
					{:else}
						To use this feature, register a model with a runtime backend supporting:
						{reqFeatures.map((f) => getFeatureMeta(f).label).join(', ')}.
					{/if}
				</p>

				<!-- Missing Requirements Checklist -->
				{#if missingRuntimes.length > 0 || missingModels.length > 0}
					<div
						class="w-full max-w-md my-3 p-3 rounded-xl bg-[var(--bg-primary)] border border-[var(--border-color)] text-left flex flex-col gap-2.5"
					>
						<div
							class="text-[11px] font-semibold uppercase tracking-wider text-[var(--text-muted)] flex items-center gap-1.5"
						>
							<CircleAlert size={13} class="text-amber-500" />
							<span>Missing Requirements</span>
						</div>

						{#if missingRuntimes.length > 0}
							<div
								class="flex items-center justify-between gap-3 p-2.5 rounded-lg bg-[var(--bg-surface)] border border-[var(--border-color)]"
							>
								<div class="flex items-center gap-2.5 min-w-0">
									<div
										class="w-8 h-8 rounded-lg bg-amber-500/10 text-amber-500 flex items-center justify-center shrink-0"
									>
										<Boxes size={16} />
									</div>
									<div class="min-w-0">
										<div class="text-xs font-semibold text-[var(--text-primary)]">
											Runtime Not Installed
										</div>
										<div class="text-[11px] text-[var(--text-muted)] truncate">
											{missingRuntimes.map((m) => m.candidateNames).join(' / ')}
										</div>
									</div>
								</div>
								<a href="/runtimes" class="app-btn app-btn-secondary app-btn-sm shrink-0">
									<Boxes size={13} />
									<span>Runtimes</span>
									<ChevronRight size={13} />
								</a>
							</div>
						{/if}

						{#if missingModels.length > 0}
							<div
								class="flex items-center justify-between gap-3 p-2.5 rounded-lg bg-[var(--bg-surface)] border border-[var(--border-color)]"
							>
								<div class="flex items-center gap-2.5 min-w-0">
									<div
										class="w-8 h-8 rounded-lg bg-amber-500/10 text-amber-500 flex items-center justify-center shrink-0"
									>
										<Box size={16} />
									</div>
									<div class="min-w-0">
										<div class="text-xs font-semibold text-[var(--text-primary)]">
											Model Not Registered
										</div>
										<div class="text-[11px] text-[var(--text-muted)] truncate">
											No model configured for {missingModels.map((m) => m.meta.label).join(', ')}
										</div>
									</div>
								</div>
								<a href="/models" class="app-btn app-btn-secondary app-btn-sm shrink-0">
									<Box size={13} />
									<span>Models</span>
									<ChevronRight size={13} />
								</a>
							</div>
						{/if}
					</div>
				{/if}
			</div>
		</Card>
	</div>
{/if}

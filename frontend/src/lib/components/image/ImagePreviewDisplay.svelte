<script lang="ts">
	import { Badge } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { GeneratedImageItem as GeneratedImage } from '$lib/types/chat';
	import { formatElapsedTime } from '$lib/utils/format';
	import { Image as ImageIcon, Sparkles } from '@lucide/svelte';

	interface ImagePreviewDisplayProps {
		selectedImage: GeneratedImage | null;
		isGenerating: boolean;
		onOpenLightbox: () => void;
	}

	let {
		selectedImage = null,
		isGenerating = false,
		onOpenLightbox
	}: ImagePreviewDisplayProps = $props();
</script>

<div
	class="w-full min-h-[380px] max-h-[580px] rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] flex items-center justify-center relative overflow-hidden group select-none"
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
			<span class="text-xs text-[var(--text-muted)] animate-pulse">
				{t('image.generatingBtn')}
			</span>
		</div>
	{:else if selectedImage}
		<!-- Display Active Image -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
		<img
			src={selectedImage.src}
			alt={selectedImage.prompt}
			class="w-full h-full object-contain max-h-[560px] cursor-pointer transition-transform duration-200 group-hover:scale-[1.01]"
			onclick={onOpenLightbox}
		/>

		<!-- Hover Overlay Pill with Meta -->
		<div
			class="absolute bottom-3 left-3 right-3 p-3 rounded-lg bg-[var(--bg-surface)]/90 backdrop-blur-md border border-[var(--border-color)]/80 flex items-center justify-between gap-3 text-xs opacity-0 group-hover:opacity-100 transition-opacity duration-150 pointer-events-auto"
		>
			<span class="truncate text-[var(--text-primary)] font-medium">
				"{selectedImage.prompt}"
			</span>
			<div class="flex items-center gap-2 shrink-0">
				<Badge variant="muted" rounded="rounded-md">{selectedImage.size}</Badge>
				{#if selectedImage.generationTimeSecs}
					<Badge variant="muted" rounded="rounded-md">
						{formatElapsedTime(selectedImage.generationTimeSecs)}
					</Badge>
				{/if}
			</div>
		</div>
	{:else}
		<!-- Empty State Placeholder -->
		<div class="flex flex-col items-center justify-center gap-3 p-8 text-center max-w-sm">
			<div
				class="w-12 h-12 rounded-lg bg-[var(--bg-surface)] text-[var(--text-muted)] flex items-center justify-center border border-[var(--border-color)]"
			>
				<ImageIcon size={24} />
			</div>
			<div class="flex flex-col gap-1">
				<span class="text-sm font-semibold text-[var(--text-primary)]">
					{t('image.noImageTitle')}
				</span>
				<span class="text-xs text-[var(--text-muted)] leading-relaxed">
					{t('image.noImageDesc')}
				</span>
			</div>
		</div>
	{/if}
</div>

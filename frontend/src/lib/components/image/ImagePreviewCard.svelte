<script lang="ts">
	import { Button, Card } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { GeneratedImageItem as GeneratedImage } from '$lib/types/chat';
	import { Check, Copy, Download, Image as ImageIcon, Maximize } from '@lucide/svelte';
	import ImagePreviewDisplay from './ImagePreviewDisplay.svelte';
	import ImageVariationStrip from './ImageVariationStrip.svelte';

	interface ImagePreviewCardProps {
		selectedImage: GeneratedImage | null;
		variations: GeneratedImage[];
		isGenerating: boolean;
		copiedImage?: boolean;
		onDownload: () => void;
		onReuse: () => void;
		onCopy: () => void;
		onOpenLightbox: () => void;
		onSelectVariation: (item: GeneratedImage) => void;
	}

	let {
		selectedImage = null,
		variations = [],
		isGenerating = false,
		copiedImage = false,
		onDownload,
		onReuse,
		onCopy,
		onOpenLightbox,
		onSelectVariation
	}: ImagePreviewCardProps = $props();
</script>

<Card class="overflow-hidden">
	<div class="flex flex-col gap-4">
		<!-- Header with Action Toolbar -->
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-2">
				<ImageIcon size={16} class="text-[var(--primary)]" />
				<h3 class="m-0 text-sm font-bold text-[var(--text-primary)]">
					{t('image.preview')}
				</h3>
			</div>

			{#if selectedImage}
				<div class="flex items-center gap-1.5 ml-auto">
					<Button
						variant="secondary"
						size="icon-sm"
						onclick={onDownload}
						title={t('common.download')}
						ariaLabel={t('common.download')}
					>
						<Download size={13} />
					</Button>

					<Button
						variant="secondary"
						size="icon-sm"
						onclick={onReuse}
						title={t('image.reusePrompt')}
						ariaLabel={t('image.reusePrompt')}
					>
						<Copy size={13} />
					</Button>

					<Button
						variant="secondary"
						size="icon-sm"
						onclick={onCopy}
						title={t('common.copy')}
						ariaLabel={t('common.copy')}
					>
						{#if copiedImage}
							<Check size={13} class="text-[var(--success-text)]" />
						{:else}
							<ImageIcon size={13} />
						{/if}
					</Button>

					<Button
						variant="secondary"
						size="icon-sm"
						onclick={onOpenLightbox}
						title={t('image.fullscreenPreview')}
						ariaLabel={t('image.fullscreenPreview')}
					>
						<Maximize size={13} />
					</Button>
				</div>
			{/if}
		</div>

		<!-- Main Image Display Canvas -->
		<ImagePreviewDisplay {selectedImage} {isGenerating} {onOpenLightbox} />

		<!-- Multiple Variations Strip -->
		<ImageVariationStrip
			items={variations}
			selectedId={selectedImage?.id}
			onSelect={onSelectVariation}
		/>
	</div>
</Card>

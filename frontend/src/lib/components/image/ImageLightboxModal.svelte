<script lang="ts">
	import { Badge, Button, ModelBadge } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { GeneratedImageItem as GeneratedImage } from '$lib/types/chat';
	import { formatElapsedTime } from '$lib/utils/format';
	import { Download, X } from '@lucide/svelte';

	interface ImageLightboxModalProps {
		image: GeneratedImage | null;
		isOpen: boolean;
		onClose: () => void;
		onDownload: (item: GeneratedImage) => void;
	}

	let { image, isOpen = false, onClose, onDownload }: ImageLightboxModalProps = $props();

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Escape' && isOpen) {
			onClose();
		}
	}
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen && image}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 z-[200] bg-[var(--bg-primary)]/85 backdrop-blur-md flex flex-col items-center justify-center p-4 transition-opacity duration-200"
		onclick={onClose}
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
					<ModelBadge model={image.model} variant="inline" class="text-xs" />
					<Badge variant="muted" rounded="rounded-md">{image.size}</Badge>
					{#if image.generationTimeSecs}
						<Badge variant="muted" rounded="rounded-md">
							{formatElapsedTime(image.generationTimeSecs)}
						</Badge>
					{/if}
				</div>

				<div class="flex items-center gap-2">
					<Button
						variant="secondary"
						size="sm"
						onclick={() => onDownload(image!)}
						title={t('image.download')}
						ariaLabel={t('image.download')}
					>
						<Download size={13} />
						<span>{t('image.download')}</span>
					</Button>

					<Button
						variant="ghost"
						size="icon-sm"
						onclick={onClose}
						title={t('common.close')}
						ariaLabel={t('common.close')}
					>
						<X size={16} />
					</Button>
				</div>
			</div>

			<!-- Image in Modal -->
			<div class="flex-1 min-h-0 flex items-center justify-center max-h-[70vh]">
				<img
					src={image.src}
					alt={image.prompt}
					class="max-w-full max-h-[70vh] object-contain rounded-lg"
				/>
			</div>

			<!-- Prompt details at bottom -->
			<div
				class="w-full p-2.5 rounded-lg bg-[var(--bg-surface-hover)] text-xs text-[var(--text-secondary)] leading-relaxed"
			>
				<span class="font-semibold text-[var(--text-primary)]">{t('image.promptPrefix')}</span>
				{image.prompt}
				{#if image.negativePrompt}
					<div class="mt-1">
						<span class="font-semibold text-[var(--text-primary)]">
							{t('image.negativePromptPrefix')}
						</span>
						{image.negativePrompt}
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

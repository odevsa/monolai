<script lang="ts">
	import { Button, EmptyState, FeatureGate, Input } from '$lib/components/ui';
	import { askConfirm } from '$lib/confirmStore';
	import { t } from '$lib/i18n';
	import { imageGalleryState } from '$lib/state';
	import type { GeneratedImageItem } from '$lib/types/chat';
	import { Search, Trash } from '@lucide/svelte';

	interface SidebarImageGalleryProps {
		onCloseMobile?: () => void;
	}

	let { onCloseMobile = () => {} }: SidebarImageGalleryProps = $props();

	let imageSearchQuery = $state('');

	let filteredImages = $derived.by(() => {
		const q = imageSearchQuery.trim().toLowerCase();
		if (!q) return imageGalleryState.history;
		return imageGalleryState.history.filter(
			(img) =>
				img.prompt.toLowerCase().includes(q) || (img.model && img.model.toLowerCase().includes(q))
		);
	});

	function handleImageSelect(item: GeneratedImageItem) {
		imageGalleryState.selectImage(item);
		onCloseMobile();
	}

	async function deleteSingleImage(id: string, e: MouseEvent) {
		e.stopPropagation();
		e.preventDefault();
		const confirmed = await askConfirm(
			t('common.delete') + '?',
			t('common.delete'),
			'danger',
			t('common.delete'),
			t('common.cancel')
		);
		if (confirmed) {
			imageGalleryState.deleteImage(id);
		}
	}

	async function clearImageHistory() {
		const confirmed = await askConfirm(
			t('image.clearHistoryConfirmMsg'),
			t('image.clearHistoryConfirmTitle'),
			'danger',
			t('common.delete'),
			t('common.cancel')
		);
		if (confirmed) {
			imageGalleryState.clearHistory();
		}
	}
</script>

{#snippet searchPrefix()}
	<Search size={14} />
{/snippet}

<FeatureGate features={['image-generation']} showCard={false}>
	<div class="flex flex-col gap-3">
		<div class="flex items-center justify-between px-1">
			<span class="text-[0.75rem] font-bold text-[var(--text-muted)] uppercase tracking-wider">
				{t('image.sessionHistory')}
			</span>
			{#if imageGalleryState.history.length > 0}
				<Button
					variant="ghost"
					size="icon-sm"
					class="text-[var(--text-muted)] hover:text-[var(--danger-text)]"
					onclick={clearImageHistory}
					title={t('common.clear')}
					ariaLabel={t('common.clear')}
				>
					<Trash size={13} />
				</Button>
			{/if}
		</div>

		<Input
			size="sm"
			variant="surface"
			bind:value={imageSearchQuery}
			placeholder={t('image.searchPrompts')}
			prefix={searchPrefix}
		/>

		<div class="flex flex-col gap-1">
			{#if filteredImages.length === 0}
				<EmptyState
					size="sm"
					title={imageGalleryState.history.length === 0
						? t('image.noImagesFound')
						: t('image.noImagesFound')}
				/>
			{:else}
				{#each filteredImages as item (item.id)}
					<div
						class="group flex items-center gap-2 p-1.5 rounded-xl transition-all duration-150 relative hover:bg-[var(--bg-hover)] {imageGalleryState
							.selectedImage?.id === item.id
							? 'bg-[var(--bg-surface-hover)]'
							: ''}"
					>
						<button
							type="button"
							class="flex items-center gap-2.5 flex-1 min-w-0 bg-transparent border-0 p-0 text-left cursor-pointer"
							onclick={() => handleImageSelect(item)}
							title={item.prompt}
						>
							<div
								class="w-10 h-10 rounded-lg overflow-hidden bg-[var(--bg-primary)] border border-[var(--border-color)] shrink-0"
							>
								<img
									src={item.src}
									alt={item.prompt}
									class="w-full h-full object-cover transition-transform duration-200 group-hover:scale-105"
								/>
							</div>

							<div class="flex-1 min-w-0">
								<span class="text-xs font-medium text-[var(--text-primary)] truncate block">
									{item.prompt}
								</span>
								<div class="flex items-center gap-1.5 text-[10px] text-[var(--text-muted)] mt-0.5">
									<span>{item.size}</span>
									{#if item.model}
										<span>•</span>
										<span class="truncate max-w-[80px]">{item.model}</span>
									{/if}
								</div>
							</div>
						</button>

						<Button
							variant="ghost"
							size="icon-sm"
							class="text-[var(--text-muted)] opacity-100 md:opacity-0 group-hover:opacity-100 hover:text-[var(--danger-text)] shrink-0"
							onclick={(e) => deleteSingleImage(item.id, e)}
							title={t('common.delete')}
							ariaLabel={t('common.delete')}
						>
							<Trash size={13} />
						</Button>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</FeatureGate>

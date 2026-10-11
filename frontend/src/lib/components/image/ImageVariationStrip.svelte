<script lang="ts">
	import { t } from '$lib/i18n';
	import type { GeneratedImageItem as GeneratedImage } from '$lib/types/chat';

	interface ImageVariationStripProps {
		items: GeneratedImage[];
		selectedId?: string;
		onSelect: (item: GeneratedImage) => void;
	}

	let { items = [], selectedId, onSelect }: ImageVariationStripProps = $props();
</script>

{#if items.length > 1}
	<div class="flex flex-col gap-2 pt-2 border-t border-[var(--border-color)]">
		<span class="text-xs font-semibold text-[var(--text-muted)]">
			{t('image.generatedVariations', { count: items.length })}
		</span>
		<div class="flex gap-2.5 overflow-x-auto pb-1">
			{#each items as item (item.id)}
				<button
					type="button"
					class="w-16 h-16 sm:w-18 sm:h-18 rounded-lg overflow-hidden shrink-0 border-2 cursor-pointer transition-all bg-[var(--bg-primary)] p-0 {selectedId ===
					item.id
						? 'border-[var(--primary)] shadow-sm opacity-100 scale-102'
						: 'border-transparent opacity-70 hover:opacity-100 hover:border-[var(--border-hover)]'}"
					onclick={() => onSelect(item)}
					title={item.prompt}
				>
					<img src={item.src} alt={item.prompt} class="w-full h-full object-cover" />
				</button>
			{/each}
		</div>
	</div>
{/if}

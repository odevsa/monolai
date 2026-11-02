<script lang="ts">
	import { Badge, Button, EmptyState, FeatureGate, Input } from '$lib/components/ui';
	import { askConfirm } from '$lib/confirmStore';
	import { t } from '$lib/i18n';
	import { audioState } from '$lib/state';
	import type { AudioGenerationItem } from '$lib/types/audio';
	import { formatAudioDuration } from '$lib/utils/audio';
	import { Headphones, Mic, Search, Trash } from '@lucide/svelte';

	interface SidebarAudioHistoryProps {
		onCloseMobile?: () => void;
	}

	let { onCloseMobile = () => {} }: SidebarAudioHistoryProps = $props();

	let searchQuery = $state('');

	let filteredItems = $derived.by(() => {
		const q = searchQuery.trim().toLowerCase();
		if (!q) return audioState.history;
		return audioState.history.filter((item) => {
			const text = (item.prompt || item.transcript || '').toLowerCase();
			const model = item.model.toLowerCase();
			return text.includes(q) || model.includes(q);
		});
	});

	function handleItemSelect(item: AudioGenerationItem) {
		audioState.selectItem(item);
		onCloseMobile();
	}

	async function deleteSingleItem(id: string, e: MouseEvent) {
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
			audioState.deleteItem(id);
		}
	}

	async function clearAudioHistory() {
		const confirmed = await askConfirm(
			t('audio.clearHistoryConfirmMsg'),
			t('audio.clearHistoryConfirmTitle'),
			'danger',
			t('common.delete'),
			t('common.cancel')
		);
		if (confirmed) {
			audioState.clearHistory();
		}
	}
</script>

{#snippet searchPrefix()}
	<Search size={14} />
{/snippet}

<FeatureGate features={['audio']} showCard={false}>
	<div class="flex flex-col gap-3">
		<div class="flex items-center justify-between px-1">
			<span class="text-[0.75rem] font-bold text-[var(--text-muted)] uppercase tracking-wider">
				{t('audio.sessionHistory')}
			</span>
			{#if audioState.history.length > 0}
				<Button
					variant="ghost"
					size="icon-sm"
					class="text-[var(--text-muted)] hover:text-[var(--danger-text)]"
					onclick={clearAudioHistory}
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
			bind:value={searchQuery}
			placeholder={t('audio.searchHistory')}
			prefix={searchPrefix}
		/>

		<div class="flex flex-col gap-1">
			{#if filteredItems.length === 0}
				<EmptyState
					size="sm"
					title={t('audio.noHistoryFound')}
					description={t('audio.noHistoryFoundDesc')}
				/>
			{:else}
				{#each filteredItems as item (item.id)}
					{@const isSelected = audioState.selectedItem?.id === item.id}
					<div
						class="group flex items-center gap-2 p-1.5 rounded-xl transition-all duration-150 relative hover:bg-[var(--bg-hover)] {isSelected
							? 'bg-[var(--bg-surface-hover)]'
							: ''}"
					>
						<button
							type="button"
							onclick={() => handleItemSelect(item)}
							class="flex items-center gap-2.5 flex-1 min-w-0 bg-transparent border-0 p-0 text-left cursor-pointer"
						>
							<div
								class="w-10 h-10 rounded-lg flex flex-col gap-1 items-center justify-center shrink-0 bg-[var(--primary-light)] text-[var(--primary)]"
							>
								{#if item.type === 'tts'}
									<Headphones size={18} />
								{:else}
									<Mic size={18} />
								{/if}

								<span class="uppercase text-[9px]">
									{item.type}
								</span>
							</div>

							<div class="flex-1 min-w-0">
								<p class="text-xs font-medium text-[var(--text-primary)] truncate m-0">
									{item.prompt || item.transcript || item.filename || item.model}
								</p>
								<div class="flex items-center gap-1.5 mt-0.5 text-[10px] text-[var(--text-muted)]">
									<span>{formatAudioDuration(item.durationSecs)}</span>
									{#if item.voice}
										<span>•</span>
										<span>{item.voice}</span>
									{/if}
									{#if item.language}
										<span>•</span>
										<span class="uppercase">{item.language}</span>
									{/if}
								</div>
							</div>
						</button>

						<Button
							variant="ghost"
							size="icon-sm"
							class="text-[var(--text-muted)] opacity-100 md:opacity-0 group-hover:opacity-100 hover:text-[var(--danger-text)] shrink-0"
							onclick={(e) => deleteSingleItem(item.id, e)}
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

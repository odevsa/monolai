<script lang="ts">
	import { Button, EmptyState, FeatureGate, Input, Badge } from '$lib/components/ui';
	import { askConfirm } from '$lib/confirmStore';
	import { t } from '$lib/i18n';
	import { audioState } from '$lib/state';
	import type { AudioGenerationItem } from '$lib/types/audio';
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
						class="group flex items-center gap-2 p-1.5 rounded-xl transition-all duration-150 relative border {isSelected
							? 'bg-[var(--bg-surface-hover)] border-[var(--primary)]'
							: 'border-transparent hover:bg-[var(--bg-surface)] hover:border-[var(--border-color)]'}"
					>
						<button
							type="button"
							onclick={() => handleItemSelect(item)}
							class="flex items-start gap-2.5 flex-1 min-w-0 bg-transparent border-0 p-0 text-left cursor-pointer"
						>
							<div
								class="w-7 h-7 rounded-lg flex items-center justify-center shrink-0 mt-0.5 {item.type ===
								'tts'
									? 'bg-[var(--primary-light)] text-[var(--primary)]'
									: 'bg-[var(--info-light)] text-[var(--info-text)]'}"
							>
								{#if item.type === 'tts'}
									<Headphones size={14} />
								{:else}
									<Mic size={14} />
								{/if}
							</div>

							<div class="flex-1 min-w-0 pr-6">
								<p class="text-xs font-medium text-[var(--text-primary)] truncate m-0">
									{item.prompt || item.transcript || item.filename || item.model}
								</p>
								<div class="flex items-center gap-1.5 mt-0.5 text-[10px] text-[var(--text-muted)]">
									<Badge variant="muted" class="text-[9px] px-1 py-0 uppercase">
										{item.type}
									</Badge>
									{#if item.voice}
										<span>{item.voice}</span>
										<span>•</span>
									{/if}
									{#if item.language}
										<span class="uppercase">{item.language}</span>
										<span>•</span>
									{/if}
									<span
										>{new Date(item.timestamp).toLocaleTimeString([], {
											hour: '2-digit',
											minute: '2-digit'
										})}</span
									>
								</div>
							</div>
						</button>

						<button
							type="button"
							class="absolute right-2 top-2 p-1 text-[var(--text-muted)] hover:text-[var(--danger-text)] opacity-0 group-hover:opacity-100 transition-opacity rounded-md"
							onclick={(e) => deleteSingleItem(item.id, e)}
							title={t('common.delete')}
							aria-label={t('common.delete')}
						>
							<Trash size={12} />
						</button>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</FeatureGate>

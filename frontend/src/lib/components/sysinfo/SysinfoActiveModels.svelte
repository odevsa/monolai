<script lang="ts">
	import { t } from '$lib/i18n';
	import type { RunningModelStatus } from '$lib/types/models';
	import { Box, Power } from '@lucide/svelte';
	import SysinfoModelCard from './SysinfoModelCard.svelte';

	interface SysinfoActiveModelsProps {
		runningModels: RunningModelStatus[];
		unloadingIds: Set<string>;
		onUnloadModel: (modelId: string) => void;
		onUnloadAll: () => void;
	}

	let { runningModels, unloadingIds, onUnloadModel, onUnloadAll }: SysinfoActiveModelsProps =
		$props();
</script>

<section class="bg-[var(--bg-surface)] rounded-2xl p-6 flex flex-col gap-4 border-0 shadow-xs">
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-3">
			<div class="flex items-center gap-2">
				<Box size={16} class="text-[var(--primary)]" />
				<h3 class="app-card-title text-sm">{t('sysinfo.activeModels')}</h3>
			</div>
		</div>

		{#if runningModels.length > 0}
			<button
				type="button"
				class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold text-[var(--danger-text)] bg-[var(--danger-light)] border border-[var(--danger-border)] cursor-pointer transition-all hover:bg-[var(--danger-hover)] hover:text-[var(--text-primary)] active:scale-95"
				onclick={onUnloadAll}
				title={t('sysinfo.unloadAllTitle')}
			>
				<Power size={13} />
				<span>{t('sysinfo.unloadAll')}</span>
			</button>
		{/if}
	</div>

	{#if runningModels.length === 0}
		<div
			class="flex flex-col stretch items-center gap-2 p-4 rounded-xl bg-[var(--bg-surface-hover)] border-0 text-[var(--text-muted)]"
		>
			<div
				class="flex items-center justify-center shrink-0 w-8 h-8 rounded-lg bg-[var(--bg-surface)] text-[var(--text-muted)] border border-[var(--border-color)]"
			>
				<Box size={16} />
			</div>
			<div>
				<div class="text-xs text-center font-semibold text-[var(--text-primary)]">
					{t('sysinfo.noModelsLoaded')}
				</div>
				<div class="text-[0.75rem] text-center text-[var(--text-muted)] max-w-md">
					{t('sysinfo.noModelsLoadedDesc')}
				</div>
			</div>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 gap-3.5">
			{#each runningModels as model (model.model_id)}
				<SysinfoModelCard
					{model}
					isUnloading={unloadingIds.has(model.model_id)}
					onUnload={onUnloadModel}
				/>
			{/each}
		</div>
	{/if}
</section>

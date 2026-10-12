<script lang="ts">
	import { Button, EmptyState } from '$lib/components/ui';
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
			<Button
				variant="danger"
				size="sm"
				onclick={onUnloadAll}
				title={t('sysinfo.unloadAllTitle')}
				ariaLabel={t('sysinfo.unloadAllTitle')}
			>
				<Power size={13} />
				<span>{t('sysinfo.unloadAll')}</span>
			</Button>
		{/if}
	</div>

	{#if runningModels.length === 0}
		<EmptyState
			icon={Box}
			title={t('sysinfo.noModelsLoaded')}
			description={t('sysinfo.noModelsLoadedDesc')}
			class="border-0 bg-[var(--bg-surface-hover)]"
		/>
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

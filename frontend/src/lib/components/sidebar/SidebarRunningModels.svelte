<script lang="ts">
	import { ModelBadge } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import {
		getStatusType,
		runningModelsState,
		startRunningStatePolling,
		unloadAllModels,
		unloadModel
	} from '$lib/runningModelsStore';
	import { Power } from '@lucide/svelte';
	import { onMount } from 'svelte';

	interface SidebarRunningModelsProps {
		isExpanded: boolean;
	}

	let { isExpanded }: SidebarRunningModelsProps = $props();

	let loadedModels = $derived(runningModelsState.runningModels);
	let unloadingIds = $derived(runningModelsState.unloadingModelIds);

	onMount(() => {
		const stopPolling = startRunningStatePolling(2500);
		return () => {
			stopPolling();
		};
	});
</script>

{#if isExpanded}
	<div class="px-3.5 py-3 border-t border-[var(--border-color)] flex flex-col gap-2">
		<div class="flex items-center justify-between">
			<span class="text-[0.7rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]">
				{t('navigation.runningModels')}
			</span>
			{#if loadedModels.length > 0}
				<button
					type="button"
					class="bg-transparent border-0 text-[var(--text-muted)] text-[0.7rem] cursor-pointer px-1 py-0.5 rounded transition-colors hover:text-red-500"
					onclick={unloadAllModels}
					title={t('navigation.unloadAll')}
				>
					{t('navigation.unloadAll')}
				</button>
			{/if}
		</div>

		{#if loadedModels.length === 0}
			<div
				class="bg-[var(--bg-surface)] border border-dashed border-[var(--border-color)] rounded-xl p-2.5"
			>
				<p class="text-[0.75rem] text-[var(--text-muted)] leading-snug m-0">
					{t('navigation.noRunningModels')}
				</p>
			</div>
		{:else}
			<div class="flex flex-col gap-1.5 max-h-[180px] overflow-y-auto">
				{#each loadedModels as model (model.model_id)}
					{@const isUnloading = unloadingIds.has(model.model_id)}
					{@const rawStatus = getStatusType(model.state)}
					{@const statusType = isUnloading ? 'loading' : rawStatus}
					<div
						class="flex items-center justify-between bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-lg px-2.5 py-1.5 gap-2"
					>
						<div class="flex items-center gap-2 min-w-0">
							<span
								class="w-2 h-2 rounded-full shrink-0 {statusType === 'ready'
									? 'bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.4)] animate-pulse'
									: statusType === 'loading'
										? 'bg-amber-500 animate-pulse'
										: 'bg-red-500'}"
								title={statusType === 'ready'
									? 'Active'
									: statusType === 'loading'
										? 'Loading / Unloading...'
										: 'Error'}
							></span>
							<ModelBadge
								model={model.model_id}
								variant="clean"
								showIcon={false}
								class="truncate text-xs font-medium"
							/>
						</div>

						<button
							type="button"
							class="bg-transparent border-0 text-[var(--text-muted)] cursor-pointer p-1 rounded-md flex items-center justify-center transition-colors shrink-0 hover:bg-red-500/15 hover:text-red-500 disabled:opacity-50 disabled:cursor-not-allowed"
							disabled={isUnloading}
							onclick={() => unloadModel(model.model_id)}
							title="Unload model"
							aria-label="Unload model"
						>
							<Power size={13} />
						</button>
					</div>
				{/each}
			</div>
		{/if}
	</div>
{:else}
	{#if loadedModels.length > 0}
		<div
			class="flex items-center justify-center py-2"
			title="{loadedModels.length} model(s) loaded"
		>
			<span
				class="w-2.5 h-2.5 rounded-full bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.4)] animate-pulse"
			></span>
		</div>
	{/if}
{/if}

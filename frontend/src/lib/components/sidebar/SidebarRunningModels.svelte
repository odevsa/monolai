<script lang="ts">
	import { Button, EmptyState, ModelBadge, StatusDot } from '$lib/components/ui';
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

	function getStatusDotPropsByType(statusType: string) {
		const variant: "success" | "warning" | "danger" | "primary" | "muted" | undefined = statusType === 'ready' 
			? 'success' 
			: statusType === 'loading'
				? 'warning'
				: 'danger';
		const pulse = statusType === 'ready' || statusType === 'loading';
		const title = statusType === 'ready'
			? t('common.active')
			: statusType === 'loading'
				? t('sysinfo.statusLoading')
				: t('common.error');
		return {variant, pulse, title}
	}
</script>

{#if isExpanded}
	<div class="px-3.5 py-3 flex flex-col gap-2">
		<div class="flex items-center justify-between">
			<span class="text-[0.7rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]">
				{t('navigation.runningModels')}
			</span>
			{#if loadedModels.length > 0}
				<Button
					variant="ghost"
					size="xs"
					class="text-[var(--text-muted)] text-[0.7rem] px-1 py-0.5 hover:text-[var(--danger-text)]"
					onclick={unloadAllModels}
					title={t('common.unloadAll')}
				>
					{t('common.unloadAll')}
				</Button>
			{/if}
		</div>

		{#if loadedModels.length === 0}
			<EmptyState size="sm" title={t('navigation.noRunningModels')} />
		{:else}
			<div class="flex flex-col gap-1.5 max-h-[180px] overflow-y-auto">
				{#each loadedModels as model (model.model_id)}
					{@const isUnloading = unloadingIds.has(model.model_id)}
					{@const rawStatus = getStatusType(model.state)}
					{@const statusType = isUnloading ? 'loading' : rawStatus}
					{@const {variant, pulse, title} = getStatusDotPropsByType(statusType)}
					<div
						class="flex items-center justify-between bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-lg px-2.5 py-1.5 gap-2"
					>
						<div class="flex items-center gap-2 min-w-0">
							<StatusDot {variant} {pulse} {title} />
							<ModelBadge
								model={model.model_id}
								variant="clean"
								showIcon={false}
								class="truncate text-xs font-medium"
							/>
						</div>

						<Button
							variant="ghost"
							size="icon-sm"
							class="text-[var(--text-muted)] hover:bg-[var(--danger-light)] hover:text-[var(--danger-text)] shrink-0"
							disabled={isUnloading}
							onclick={() => unloadModel(model.model_id)}
							title={t('common.unload')}
							ariaLabel={t('common.unload')}
						>
							<Power size={13} />
						</Button>
					</div>
				{/each}
			</div>
		{/if}
	</div>
{:else}
	{#if loadedModels.length > 0}
		{#each loadedModels as model (model.model_id)}
			{@const isUnloading = unloadingIds.has(model.model_id)}
			{@const rawStatus = getStatusType(model.state)}
			{@const statusType = isUnloading ? 'loading' : rawStatus}
			{@const {variant, pulse, title} = getStatusDotPropsByType(statusType)}
			<div
				class="flex items-center justify-center py-2"
				title={t('navigation.modelsLoaded', { count: loadedModels.length })}
			>
				<StatusDot variant={variant} pulse={pulse} title={title} size="lg" />
			</div>
		{/each}
	{/if}
{/if}

<script lang="ts">
	import { Input, Select } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { Search } from '@lucide/svelte';

	let {
		searchQuery = $bindable(''),
		selectedRuntimeFilter = $bindable('all'),
		selectedStatusFilter = $bindable('all'),
		runtimeFilterOptions = [],
		statusFilterOptions = []
	}: {
		searchQuery?: string;
		selectedRuntimeFilter?: string;
		selectedStatusFilter?: 'all' | 'running' | 'missing';
		runtimeFilterOptions: { value: string; label: string }[];
		statusFilterOptions: { value: 'all' | 'running' | 'missing'; label: string }[];
	} = $props();
</script>

<div
	class="flex flex-col sm:flex-row items-stretch sm:items-center gap-2.5 p-2 bg-[var(--bg-surface)] rounded-xl shadow-xs"
>
	<!-- Search Input -->
	<div class="relative flex-1 min-w-[180px]">
		<Input bind:value={searchQuery} placeholder={t('models.searchPlaceholder')} clearable={true}>
			{#snippet prefix()}
				<Search size={14} />
			{/snippet}
		</Input>
	</div>

	<!-- Runtime Custom Selectbox -->
	<div class="w-full sm:w-52">
		<Select
			bind:value={selectedRuntimeFilter}
			options={runtimeFilterOptions}
			placeholder={t('models.allRuntimes')}
			ariaLabel={t('models.filterByRuntime')}
		/>
	</div>

	<!-- Status Custom Selectbox -->
	<div class="w-full sm:w-44">
		<Select
			bind:value={selectedStatusFilter}
			options={statusFilterOptions}
			placeholder={t('models.allStatuses')}
			ariaLabel={t('models.filterByStatus')}
			align="right"
		/>
	</div>
</div>

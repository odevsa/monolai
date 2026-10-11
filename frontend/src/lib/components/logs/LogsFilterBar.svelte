<script lang="ts">
	import { Input, Select } from '$lib/components/ui';
	import type { SelectOption } from '$lib/components/ui/molecules/Select.svelte';
	import { t } from '$lib/i18n';
	import { Funnel, Search, Terminal } from '@lucide/svelte';

	interface LogsFilterBarProps {
		selectedService: string;
		selectedLevel: string;
		searchQuery: string;
		serviceOptions: SelectOption[];
		levelOptions: SelectOption[];
		onServiceChange: (service: string) => void;
	}

	let {
		selectedService = $bindable('all'),
		selectedLevel = $bindable('ALL'),
		searchQuery = $bindable(''),
		serviceOptions,
		levelOptions,
		onServiceChange
	}: LogsFilterBarProps = $props();
</script>

<div class="bg-[var(--bg-surface)] rounded-xl p-3.5 shadow-none border-0 shrink-0">
	<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-12 gap-3 items-center">
		<!-- Service Select -->
		<div class="lg:col-span-4 flex items-center gap-2">
			<span class="text-xs text-[var(--text-muted)] font-medium shrink-0 flex items-center gap-1.5">
				<Terminal size={14} class="text-[var(--primary)]" />
				{t('logs.serviceLabel')}
			</span>
			<div class="flex-1 min-w-0">
				<Select
					value={selectedService}
					options={serviceOptions}
					searchable={true}
					onchange={onServiceChange}
				/>
			</div>
		</div>

		<!-- Level Select -->
		<div class="lg:col-span-3 flex items-center gap-2">
			<span class="text-xs text-[var(--text-muted)] font-medium shrink-0 flex items-center gap-1.5">
				<Funnel size={14} class="text-[var(--primary)]" />
				{t('logs.levelLabel')}
			</span>
			<div class="flex-1 min-w-0">
				<Select
					value={selectedLevel}
					options={levelOptions}
					onchange={(val) => (selectedLevel = val)}
				/>
			</div>
		</div>

		<!-- Search Filter Input -->
		<div class="sm:col-span-2 lg:col-span-5">
			<Input bind:value={searchQuery} placeholder={t('logs.filterLogs')} clearable={true}>
				{#snippet prefix()}
					<Search size={14} />
				{/snippet}
			</Input>
		</div>
	</div>
</div>

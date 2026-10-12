<script lang="ts">
	import { Badge } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { HardwareReport } from '$lib/types/config';
	import { CircleCheck, Zap } from '@lucide/svelte';
	import SetupHardwareCard from './SetupHardwareCard.svelte';
	import { HARDWARE_OPTIONS, getHardwareName } from './types';

	let {
		hardwareReport = null,
		selectedHardware = $bindable('auto'),
		isStorageConfigured = false
	}: {
		hardwareReport?: HardwareReport | null;
		selectedHardware?: string;
		isStorageConfigured?: boolean;
	} = $props();

	let recommendedName = $derived(
		hardwareReport ? getHardwareName(hardwareReport.recommended_acceleration) : ''
	);
</script>

<div
	class="flex flex-col gap-3 p-4 rounded-xl border border-[var(--border-color)] bg-[var(--bg-surface-hover)]/30"
>
	<div class="flex items-center justify-between">
		<span
			class="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-[var(--text-secondary)]"
		>
			<Zap size={15} class="text-[var(--primary)]" />
			<span>
				{t(isStorageConfigured ? 'setup.hardwareLabel' : 'setup.stepHardware')}
			</span>
		</span>
		{#if hardwareReport}
			<span class="text-[11px] text-[var(--success-text)] font-mono flex items-center gap-1.5">
				<span class="w-1.5 h-1.5 rounded-full bg-[var(--success)] animate-pulse"></span>
				<span>{t('setup.detectedPill', { name: recommendedName })}</span>
			</span>
		{/if}
	</div>

	{#if hardwareReport}
		<div
			class="flex flex-col sm:flex-row sm:items-center justify-between gap-2.5 p-3 rounded-xl bg-[var(--bg-primary)] border border-[var(--border-color)] text-xs"
		>
			<div class="flex items-center gap-2.5 min-w-0">
				<div
					class="w-8 h-8 rounded-lg bg-[var(--success-light)] text-[var(--success-text)] border border-[var(--success-border)] flex items-center justify-center shrink-0"
				>
					<CircleCheck size={16} />
				</div>
				<div class="flex flex-col min-w-0">
					<div class="flex items-center gap-2 flex-wrap">
						<span class="text-[var(--text-primary)] font-semibold text-xs">
							{t('setup.detectedAcceleration')}
						</span>
						<Badge variant="success" rounded="rounded-full">
							{recommendedName}
						</Badge>
						{#if hardwareReport.detected_gpus.length > 0}
							<span class="text-[11px] text-[var(--text-muted)] font-mono truncate">
								• {hardwareReport.detected_gpus.join(', ')}
							</span>
						{/if}
					</div>
					<span class="text-[11px] text-[var(--text-muted)] mt-0.5">
						{t('setup.supportedOnHost', {
							engines: hardwareReport.available_accelerations.map(getHardwareName).join(', '),
							os: hardwareReport.os,
							arch: hardwareReport.arch
						})}
					</span>
				</div>
			</div>
		</div>
	{/if}

	<div class="grid grid-cols-1 sm:grid-cols-2 gap-2 mt-1">
		{#each HARDWARE_OPTIONS as opt}
			{@const isSelected = selectedHardware === opt.id}
			{@const isSupported =
				opt.id === 'auto' ||
				Boolean(hardwareReport && hardwareReport.available_accelerations.includes(opt.id))}
			{@const isRecommended =
				opt.id === 'auto' ||
				Boolean(hardwareReport && hardwareReport.recommended_acceleration === opt.id)}

			<SetupHardwareCard
				option={opt}
				{isSelected}
				{isRecommended}
				{isSupported}
				hasReport={!!hardwareReport}
				{recommendedName}
				onclick={() => (selectedHardware = opt.id)}
			/>
		{/each}
	</div>
</div>

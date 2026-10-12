<script lang="ts">
	import { Badge } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { HardwareOption } from './types';

	let {
		option,
		isSelected = false,
		isRecommended = false,
		isSupported = false,
		hasReport = false,
		recommendedName = '',
		onclick
	}: {
		option: HardwareOption;
		isSelected?: boolean;
		isRecommended?: boolean;
		isSupported?: boolean;
		hasReport?: boolean;
		recommendedName?: string;
		onclick?: () => void;
	} = $props();
</script>

<button
	type="button"
	{onclick}
	class="p-3 rounded-xl border text-left flex flex-col gap-1.5 transition cursor-pointer {isSelected
		? 'border-[var(--primary)] bg-[var(--primary-light)] text-[var(--text-primary)] ring-1 ring-[var(--primary)]'
		: 'border-[var(--border-color)] bg-[var(--bg-surface-hover)]/30 hover:bg-[var(--bg-surface-hover)] text-[var(--text-muted)]'}"
>
	<div class="flex items-center justify-between gap-1.5 flex-wrap">
		<span class="font-semibold text-xs text-[var(--text-primary)]">
			{t(option.titleKey)}
			{#if option.id === 'auto' && recommendedName}
				<span class="text-[var(--text-muted)] font-normal text-[11px]">
					({recommendedName})
				</span>
			{/if}
		</span>
		<div class="flex items-center gap-1">
			{#if isRecommended}
				<Badge variant="success" rounded="rounded-sm">
					{t('common.recommended')}
				</Badge>
			{:else if isSupported}
				<Badge variant="muted" rounded="rounded-sm">
					{t('setup.available')}
				</Badge>
			{:else if hasReport}
				<Badge variant="danger" rounded="rounded-sm">
					{t('setup.notDetected')}
				</Badge>
			{/if}
		</div>
	</div>
	<p class="m-0 text-[11px] leading-tight text-[var(--text-muted)]">
		{t(option.descKey)}
	</p>
</button>

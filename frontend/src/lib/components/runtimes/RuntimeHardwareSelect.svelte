<script lang="ts">
	import { Select } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { AccelerationOption } from '$lib/types/runtimes';

	interface RuntimeHardwareSelectProps {
		runtimeId: string;
		accelerations: AccelerationOption[];
		selectedAcceleration: string;
		onSelect: (val: string) => void;
	}

	let { runtimeId, accelerations, selectedAcceleration, onSelect }: RuntimeHardwareSelectProps =
		$props();

	let options = $derived(
		accelerations.map((opt) => ({
			value: opt.id,
			label: opt.label + (opt.is_recommended ? ` (${t('runtimes.recommendedBadge')})` : '')
		}))
	);
</script>

<div
	class="flex flex-col gap-2 p-2.5 rounded-xl bg-[var(--bg-primary)] border border-[var(--border-color)]"
>
	<div class="flex items-center justify-between gap-2">
		<span class="text-xs font-semibold text-[var(--text-primary)] whitespace-nowrap shrink-0">
			{t('runtimes.accelerationLabel')}
		</span>
		<span class="text-[11px] text-[var(--text-muted)] truncate text-right">
			{t('runtimes.selectHardwareDesc')}
		</span>
	</div>

	<div class="w-full">
		<Select
			value={selectedAcceleration}
			onchange={onSelect}
			{options}
			placeholder={t('runtimes.selectHardwarePlaceholder')}
			ariaLabel={t('runtimes.accelerationLabel')}
		/>
	</div>
</div>

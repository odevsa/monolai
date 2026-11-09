<script lang="ts">
	import { Input, Select, type SelectOption } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { ChevronDown, ChevronUp, Dices, SlidersVertical } from '@lucide/svelte';

	interface ImageAdvancedParamsProps {
		steps: number;
		cfgScale: number;
		samplingMethod?: string;
		scheduleMethod?: string;
		seed: number | string;
		disabled?: boolean;
	}

	let {
		steps = $bindable(20),
		cfgScale = $bindable(7.5),
		samplingMethod = $bindable('euler_a'),
		scheduleMethod = $bindable('default'),
		seed = $bindable(''),
		disabled = false
	}: ImageAdvancedParamsProps = $props();

	let showAdvanced = $state(false);

	const samplerOptions = $derived<SelectOption[]>([
		{ value: 'euler_a', label: t('image.samplers.euler_a') },
		{ value: 'euler', label: t('image.samplers.euler') },
		{ value: 'heun', label: t('image.samplers.heun') },
		{ value: 'dpm2', label: t('image.samplers.dpm2') },
		{ value: 'dpm++2s_a', label: t('image.samplers.dpm2s_a') },
		{ value: 'dpm++2m', label: t('image.samplers.dpm2m') },
		{ value: 'dpm++2mv2', label: t('image.samplers.dpm2mv2') },
		{ value: 'lcm', label: t('image.samplers.lcm') },
		{ value: 'ipndm', label: t('image.samplers.ipndm') },
		{ value: 'ipndm_v', label: t('image.samplers.ipndm_v') }
	]);

	const schedulerOptions = $derived<SelectOption[]>([
		{ value: 'default', label: t('image.schedulers.default') },
		{ value: 'discrete', label: t('image.schedulers.discrete') },
		{ value: 'karras', label: t('image.schedulers.karras') },
		{ value: 'exponential', label: t('image.schedulers.exponential') },
		{ value: 'ays', label: t('image.schedulers.ays') },
		{ value: 'gits', label: t('image.schedulers.gits') }
	]);
</script>

<div class="border-t border-[var(--border-color)] pt-3 flex flex-col gap-3">
	<button
		type="button"
		class="flex items-center justify-between text-xs font-semibold text-[var(--text-secondary)] bg-transparent border-0 p-0 cursor-pointer hover:text-[var(--text-primary)] transition-colors select-none"
		onclick={() => (showAdvanced = !showAdvanced)}
		aria-expanded={showAdvanced}
	>
		<div class="flex items-center gap-1.5">
			<SlidersVertical size={13} />
			<span>{t('image.advancedParams')}</span>
		</div>
		{#if showAdvanced}
			<ChevronUp size={14} />
		{:else}
			<ChevronDown size={14} />
		{/if}
	</button>

	{#if showAdvanced}
		<div class="grid grid-cols-2 gap-3 pt-1">
			<!-- Sampling Steps -->
			<div class="flex flex-col gap-1">
				<div class="flex justify-between text-[11px] text-[var(--text-muted)]">
					<span>{t('image.steps')}</span>
					<span class="font-mono text-[var(--text-primary)]">{steps}</span>
				</div>
				<input
					type="range"
					min="5"
					max="50"
					step="1"
					bind:value={steps}
					{disabled}
					class="w-full accent-[var(--primary)] cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
				/>
			</div>

			<!-- CFG Scale -->
			<div class="flex flex-col gap-1">
				<div class="flex justify-between text-[11px] text-[var(--text-muted)]">
					<span>{t('image.cfgScale')}</span>
					<span class="font-mono text-[var(--text-primary)]">{cfgScale}</span>
				</div>
				<input
					type="range"
					min="1"
					max="20"
					step="0.5"
					bind:value={cfgScale}
					{disabled}
					class="w-full accent-[var(--primary)] cursor-pointer disabled:cursor-not-allowed disabled:opacity-50"
				/>
			</div>

			<!-- Sampling Method -->
			<div class="flex flex-col gap-1.5">
				<label for="image-sampling-method" class="text-[11px] text-[var(--text-muted)] font-medium">
					{t('image.samplingMethod')}
				</label>
				<Select
					id="image-sampling-method"
					bind:value={samplingMethod}
					options={samplerOptions}
					{disabled}
				/>
			</div>

			<!-- Schedule Method -->
			<div class="flex flex-col gap-1.5">
				<label for="image-schedule-method" class="text-[11px] text-[var(--text-muted)] font-medium">
					{t('image.scheduleMethod')}
				</label>
				<Select
					id="image-schedule-method"
					bind:value={scheduleMethod}
					options={schedulerOptions}
					{disabled}
				/>
			</div>

			<!-- Seed Input -->
			<div class="col-span-2 flex flex-col gap-1">
				<div class="flex items-center justify-between">
					<label for="image-seed-input" class="text-[11px] text-[var(--text-muted)]">
						{t('image.seedLabel')}
					</label>
					<div class="flex items-center gap-2">
						<button
							type="button"
							class="text-[10px] text-[var(--primary)] hover:underline flex items-center gap-1 cursor-pointer bg-transparent border-0 p-0"
							onclick={() => {
								seed = Math.floor(Math.random() * 2147483647);
							}}
							title={t('image.randomizeSeed')}
							aria-label={t('image.randomizeSeed')}
							{disabled}
						>
							<Dices size={12} />
							<span>{t('image.randomizeSeed')}</span>
						</button>
						{#if seed !== ''}
							<button
								type="button"
								class="text-[10px] text-[var(--text-muted)] hover:text-[var(--text-primary)] cursor-pointer bg-transparent border-0 p-0"
								onclick={() => {
									seed = '';
								}}
								title={t('common.clear')}
								aria-label={t('common.clear')}
								{disabled}
							>
								{t('common.clear')}
							</button>
						{/if}
					</div>
				</div>
				<Input
					id="image-seed-input"
					type="number"
					bind:value={seed}
					placeholder={t('image.seedPlaceholder')}
					size="sm"
					{disabled}
				/>
			</div>
		</div>
	{/if}
</div>

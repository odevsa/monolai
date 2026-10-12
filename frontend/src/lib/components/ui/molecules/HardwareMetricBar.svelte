<script lang="ts">
	import type { Component } from 'svelte';

	interface HardwareMetricBarProps {
		label: string;
		detail?: string | null;
		value?: number | null;
		formattedValue?: string;
		color?: string;
		icon?: Component<{ size?: number; style?: string; class?: string }>;
	}

	let {
		label,
		detail = null,
		value = 0,
		formattedValue,
		color = 'var(--primary)',
		icon: Icon
	}: HardwareMetricBarProps = $props();

	let safeValue = $derived(value !== null && value !== undefined ? value : 0);
	let widthPercent = $derived(Math.min(100, Math.max(0, safeValue)).toFixed(0));
	let displayPercent = $derived(formattedValue ?? `${safeValue.toFixed(0)}%`);
</script>

<div class="flex flex-col gap-1.5">
	<div class="flex items-center justify-between text-xs gap-2">
		<div class="flex items-center gap-2 text-[var(--text-secondary)]">
			{#if Icon}
				<Icon size={14} style="color: {color}" class="shrink-0" />
			{/if}
			<span class="font-medium">{label}</span>
		</div>
		{#if detail}
			<span class="text-[var(--text-muted)] truncate">
				{detail}
			</span>
		{/if}
	</div>
	<div class="flex items-center gap-2">
		<div class="h-1.5 flex-1 rounded-full bg-[var(--progress-bg)] overflow-hidden">
			<div
				class="h-full rounded-full transition-all duration-500"
				style="background-color: {color}; width: {widthPercent}%"
			></div>
		</div>
		<span
			class="text-[12px] text-[var(--text-muted)] font-bold w-8 text-right shrink-0 tabular-nums"
		>
			{displayPercent}
		</span>
	</div>
</div>

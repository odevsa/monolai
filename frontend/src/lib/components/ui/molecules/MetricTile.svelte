<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title = '',
		value = '',
		subtitle = '',
		percentage = 0,
		icon: IconComponent,
		color = 'primary',
		class: className = '',
		footer
	}: {
		title: string;
		value: string;
		subtitle?: string;
		percentage?: number;
		icon?: any;
		color?: 'primary' | 'emerald' | 'amber' | 'cyan' | 'purple';
		class?: string;
		footer?: Snippet;
	} = $props();

	const colorClasses = {
		primary: 'bg-[var(--primary)] text-[var(--primary)]',
		emerald: 'bg-emerald-500 text-emerald-400',
		amber: 'bg-amber-500 text-amber-400',
		cyan: 'bg-cyan-500 text-cyan-400',
		purple: 'bg-purple-500 text-purple-400'
	};
</script>

<div
	class="p-4 rounded-xl bg-[var(--bg-surface)] border border-[var(--border-color)] flex flex-col justify-between gap-3 {className}"
>
	<div class="flex items-center justify-between gap-2">
		<span class="text-xs font-semibold text-[var(--text-muted)] truncate">{title}</span>
		{#if IconComponent}
			<IconComponent size={16} class="{colorClasses[color].split(' ')[1]} shrink-0" />
		{/if}
	</div>

	<div>
		<div class="text-xl font-bold tracking-tight text-[var(--text-primary)] leading-none">
			{value}
		</div>
		{#if subtitle}
			<div class="text-[11px] text-[var(--text-muted)] mt-1.5 truncate">{subtitle}</div>
		{/if}
	</div>

	{#if percentage !== undefined}
		<div class="w-full bg-[var(--progress-bg)] h-1.5 rounded-full overflow-hidden">
			<div
				class="h-full rounded-full transition-all duration-300 {colorClasses[color].split(' ')[0]}"
				style="width: {Math.min(100, Math.max(0, percentage))}%"
			></div>
		</div>
	{/if}

	{#if footer}
		<div class="pt-2 border-t border-[var(--border-color)] text-[11px]">
			{@render footer()}
		</div>
	{/if}
</div>

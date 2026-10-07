<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		variant = 'muted',
		pulse = false,
		class: className = '',
		children
	}: {
		variant?: 'success' | 'warning' | 'danger' | 'primary' | 'muted' | 'pill';
		pulse?: boolean;
		class?: string;
		children?: Snippet;
	} = $props();

	const variantClasses = {
		success: 'bg-emerald-500/15 text-emerald-400 border-emerald-500/30',
		warning: 'bg-amber-500/15 text-amber-400 border-amber-500/30',
		danger: 'bg-red-500/15 text-red-400 border-red-500/30',
		primary: 'bg-[var(--primary-light)] text-[var(--primary)] border-[var(--primary-focus)]',
		muted: 'bg-white/5 text-[var(--text-muted)] border-[var(--border-color)]',
		pill: 'font-mono text-[11px] px-1.5 py-0.5 rounded-md bg-white/[0.04] text-[var(--text-muted)] border border-white/[0.06]'
	};

	const dotClasses = {
		success: 'bg-emerald-400 shadow-[0_0_6px_rgba(16,185,129,0.5)]',
		warning: 'bg-amber-400 shadow-[0_0_6px_rgba(245,158,11,0.5)]',
		danger: 'bg-red-400 shadow-[0_0_6px_rgba(239,68,68,0.5)]',
		primary: 'bg-[var(--primary)] shadow-[0_0_6px_rgba(99,102,241,0.5)]',
		muted: 'bg-[var(--text-muted)]',
		pill: 'bg-[var(--text-muted)]'
	};
</script>

<span
	class="inline-flex items-center gap-1.5 px-2 py-0.5 text-[11px] font-semibold rounded-full border border-solid select-none whitespace-nowrap {variantClasses[
		variant
	]} {className}"
>
	{#if pulse}
		<span class="w-1.5 h-1.5 rounded-full shrink-0 {dotClasses[variant]} animate-pulse"></span>
	{/if}
	{@render children?.()}
</span>

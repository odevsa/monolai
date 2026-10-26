<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		variant = 'muted',
		rounded = 'rounded-full',
		size = 'md',
		pulse = false,
		blur = false,
		class: className = '',
		children
	}: {
		variant?: 'success' | 'warning' | 'danger' | 'primary' | 'muted' | 'info';
		rounded?: 'rounded-full' | 'rounded-md' | 'rounded-sm';
		size?: 'xs' | 'sm' | 'md';
		pulse?: boolean;
		blur?: boolean;
		class?: string;
		children?: Snippet;
	} = $props();

	const variantClasses = {
		success: 'bg-[var(--success-light)] text-[var(--success-text)] border-[var(--success-border)]',
		warning: 'bg-[var(--warning-light)] text-[var(--warning-text)] border-[var(--warning-border)]',
		danger: 'bg-[var(--danger-light)] text-[var(--danger-text)] border-[var(--danger-border)]',
		primary: 'bg-[var(--primary-light)] text-[var(--primary)] border-[var(--primary-focus)]',
		info: 'bg-[var(--info-light)] text-[var(--info-text)] border-[var(--info-border)]',
		muted: 'bg-[var(--bg-surface)] text-[var(--text-muted)] border-[var(--border-color)]'
	};

	const dotClasses = {
		success: 'bg-[var(--success)] shadow-[0_0_6px_var(--success)]',
		warning: 'bg-[var(--warning)] shadow-[0_0_6px_var(--warning)]',
		danger: 'bg-[var(--danger)] shadow-[0_0_6px_var(--danger)]',
		primary: 'bg-[var(--primary)] shadow-[0_0_6px_var(--primary-focus)]',
		info: 'bg-[var(--info)] shadow-[0_0_6px_var(--info)]',
		muted: 'bg-[var(--text-muted)]'
	};

	const sizeClasses: Record<typeof size, string> = {
		xs: 'px-0.5 py-0 text-[9px]',
		sm: 'px-1 py-0.25 text-[10px]',
		md: 'px-1.5 py-0.5 text-xs'
	};
</script>

<span
	class="inline-flex items-center gap-1.5 font-semibold {rounded} border border-solid select-none whitespace-nowrap {variantClasses[
		variant
	]} {sizeClasses[size]} {blur ? 'backdrop-blur-md' : ''} {className}"
>
	{#if pulse}
		<span class="w-1.5 h-1.5 rounded-full shrink-0 {dotClasses[variant]} animate-pulse"></span>
	{/if}
	{@render children?.()}
</span>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import Spinner from './Spinner.svelte';

	let {
		variant = 'secondary',
		size = 'md',
		type = 'button',
		disabled = false,
		loading = false,
		class: className = '',
		title = '',
		ariaLabel = '',
		onclick,
		children
	}: {
		variant?: 'primary' | 'secondary' | 'danger' | 'ghost' | 'glass';
		size?: 'sm' | 'md' | 'lg' | 'icon';
		type?: 'button' | 'submit' | 'reset';
		disabled?: boolean;
		loading?: boolean;
		class?: string;
		title?: string;
		ariaLabel?: string;
		onclick?: (event: MouseEvent) => void;
		children?: Snippet;
	} = $props();

	const variantClasses = {
		primary:
			'bg-[var(--primary)] text-white hover:bg-[var(--primary-hover)] border border-transparent shadow-sm',
		secondary:
			'bg-[var(--btn-bg)] text-[var(--text-primary)] border border-[var(--border-color)] hover:bg-[var(--btn-hover)] hover:border-[var(--border-hover)]',
		danger:
			'bg-red-500/12 text-red-400 border border-red-500/25 hover:bg-red-500/22 hover:border-red-500/35',
		ghost:
			'bg-transparent text-[var(--text-secondary)] border border-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]',
		glass:
			'bg-[var(--bg-surface)]/50 backdrop-blur-md text-[var(--text-primary)] border border-[var(--border-color)]/70 hover:bg-[var(--bg-surface)]/80 hover:border-[var(--border-hover)]'
	};

	const sizeClasses = {
		sm: 'px-2.5 py-1 text-xs rounded-lg gap-1.5',
		md: 'px-3.5 py-1.5 text-xs font-medium rounded-lg gap-2',
		lg: 'px-4 py-2.5 text-sm font-semibold rounded-xl gap-2.5',
		icon: 'w-8 h-8 p-0 rounded-lg flex items-center justify-center shrink-0'
	};
</script>

<button
	{type}
	disabled={disabled || loading}
	class="inline-flex items-center justify-center transition-all duration-150 cursor-pointer select-none outline-none focus-visible:ring-2 focus-visible:ring-[var(--primary)]/50 disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none {variantClasses[
		variant
	]} {sizeClasses[size]} {className}"
	{title}
	aria-label={ariaLabel || title}
	{onclick}
>
	{#if loading}
		<Spinner size={size === 'sm' ? 12 : 14} class="shrink-0" />
	{/if}
	{@render children?.()}
</button>

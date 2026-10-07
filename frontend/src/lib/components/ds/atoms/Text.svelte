<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		variant = 'primary',
		size = 'sm',
		as = 'p',
		class: className = '',
		children
	}: {
		variant?: 'primary' | 'secondary' | 'muted' | 'error' | 'success';
		size?: 'xs' | 'sm' | 'md' | 'lg';
		as?: 'p' | 'span' | 'div';
		class?: string;
		children?: Snippet;
	} = $props();

	const variantClasses = {
		primary: 'text-[var(--text-primary)]',
		secondary: 'text-[var(--text-secondary)]',
		muted: 'text-[var(--text-muted)]',
		error: 'text-red-400',
		success: 'text-emerald-400'
	};

	const sizeClasses = {
		xs: 'text-[11px] leading-tight',
		sm: 'text-xs leading-normal',
		md: 'text-sm leading-relaxed',
		lg: 'text-base leading-relaxed'
	};
</script>

{#if as === 'span'}
	<span class="{variantClasses[variant]} {sizeClasses[size]} {className}">
		{@render children?.()}
	</span>
{:else if as === 'div'}
	<div class="{variantClasses[variant]} {sizeClasses[size]} {className}">
		{@render children?.()}
	</div>
{:else}
	<p class="{variantClasses[variant]} {sizeClasses[size]} {className}">
		{@render children?.()}
	</p>
{/if}

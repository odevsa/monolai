<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title = '',
		description = '',
		icon: IconComponent,
		size = 'md',
		class: className = '',
		actions
	}: {
		title: string;
		description?: string;
		icon?: any;
		size?: 'sm' | 'md' | 'lg';
		class?: string;
		actions?: Snippet;
	} = $props();

	const sizeClasses = {
		sm: 'p-3 rounded-xl gap-1.5',
		md: 'p-8 sm:p-12 rounded-2xl gap-4',
		lg: 'p-12 sm:p-16 rounded-3xl gap-6'
	};

	const iconSizeClasses = {
		sm: 'w-7 h-7 rounded-lg',
		md: 'w-12 h-12 rounded-2xl',
		lg: 'w-16 h-16 rounded-2xl'
	};
</script>

<div
	class="flex flex-col items-center justify-center text-center border border-dashed border-[var(--border-color)] bg-[var(--bg-surface)]/40 {sizeClasses[
		size
	]} {className}"
>
	{#if IconComponent}
		<div
			class="{iconSizeClasses[
				size
			]} bg-[var(--bg-surface)] text-[var(--text-muted)] flex items-center justify-center border border-[var(--border-color)]"
		>
			<IconComponent size={size === 'sm' ? 14 : size === 'lg' ? 32 : 24} strokeWidth={1.5} />
		</div>
	{/if}

	<div class="flex flex-col gap-0.5">
		{#if size === 'sm'}
			<p class="text-xs text-[var(--text-muted)] m-0 leading-relaxed font-normal">{title}</p>
		{:else}
			<h4 class="text-sm sm:text-base font-bold text-[var(--text-primary)]">{title}</h4>
		{/if}

		{#if description}
			<p class="text-xs text-[var(--text-muted)] max-w-sm leading-relaxed">
				{description}
			</p>
		{/if}
	</div>

	{#if actions}
		<div class="mt-2 flex items-center gap-3">
			{@render actions()}
		</div>
	{/if}
</div>

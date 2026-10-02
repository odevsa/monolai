<script lang="ts">
	import type { Snippet } from 'svelte';
	import { AlertCircle, CheckCircle2, Info } from '@lucide/svelte';

	let {
		variant = 'surface',
		title,
		message,
		showIcon = true,
		class: className = '',
		children,
		action
	}: {
		variant?: 'error' | 'success' | 'surface';
		title?: string;
		message?: string;
		showIcon?: boolean;
		class?: string;
		children?: Snippet;
		action?: Snippet;
	} = $props();

	let variantClass = $derived.by(() => {
		switch (variant) {
			case 'error':
				return 'app-alert-error';
			case 'success':
				return 'app-alert-success';
			default:
				return 'app-alert-surface';
		}
	});
</script>

<div class="app-alert {variantClass} {className}">
	<div class="flex items-center gap-2.5">
		{#if showIcon}
			{#if variant === 'error'}
				<AlertCircle size={16} class="shrink-0 text-red-500" />
			{:else if variant === 'success'}
				<CheckCircle2 size={16} class="shrink-0 text-emerald-400" />
			{:else}
				<Info size={16} class="shrink-0 text-[var(--text-muted)]" />
			{/if}
		{/if}
		<div>
			{#if title}
				<strong class="font-semibold mr-1">{title}</strong>
			{/if}
			{#if message}
				<span>{message}</span>
			{/if}
			{@render children?.()}
		</div>
	</div>

	{#if action}
		<div>
			{@render action()}
		</div>
	{/if}
</div>

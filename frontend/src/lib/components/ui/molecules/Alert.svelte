<script lang="ts">
	import { CircleAlert, CircleCheck, Info, TriangleAlert, X } from '@lucide/svelte';
	import type { Snippet } from 'svelte';

	let {
		variant = 'info',
		title = '',
		message = '',
		dismissible = false,
		class: className = '',
		ondismiss,
		children
	}: {
		variant?: 'error' | 'success' | 'warning' | 'info';
		title?: string;
		message?: string;
		dismissible?: boolean;
		class?: string;
		ondismiss?: () => void;
		children?: Snippet;
	} = $props();

	let dismissed = $state(false);

	const variantStyles = {
		error: 'bg-[var(--danger-light)] border-[var(--danger-border)] text-[var(--danger-text)]',
		success: 'bg-[var(--success-light)] border-[var(--success-border)] text-[var(--success-text)]',
		warning: 'bg-[var(--warning-light)] border-[var(--warning-border)] text-[var(--warning-text)]',
		info: 'bg-[var(--info-light)] border-[var(--info-border)] text-[var(--info-text)]'
	};

	function handleDismiss() {
		dismissed = true;
		if (ondismiss) ondismiss();
	}
</script>

{#if !dismissed}
	<div
		role="alert"
		class="flex items-start gap-3 p-3.5 rounded-xl border text-xs leading-relaxed {variantStyles[
			variant
		]} {className}"
	>
		<div class="shrink-0 mt-0.5">
			{#if variant === 'error'}
				<CircleAlert size={16} />
			{:else if variant === 'warning'}
				<TriangleAlert size={16} />
			{:else if variant === 'success'}
				<CircleCheck size={16} />
			{:else}
				<Info size={16} />
			{/if}
		</div>

		<div class="flex-1 min-w-0">
			{#if title}
				<div class="font-bold mb-0.5">{title}</div>
			{/if}
			<div>
				{#if message}
					{message}
				{:else}
					{@render children?.()}
				{/if}
			</div>
		</div>

		{#if dismissible}
			<button
				type="button"
				class="shrink-0 p-1 opacity-70 hover:opacity-100 transition-opacity cursor-pointer border-0 bg-transparent text-current"
				onclick={handleDismiss}
				aria-label="Dismiss alert"
			>
				<X size={14} />
			</button>
		{/if}
	</div>
{/if}

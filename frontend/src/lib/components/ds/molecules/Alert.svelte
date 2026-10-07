<script lang="ts">
	import type { Snippet } from 'svelte';
	import { AlertTriangle, AlertCircle, CheckCircle2, Info, X } from '@lucide/svelte';

	let {
		variant = 'info',
		title = '',
		dismissible = false,
		class: className = '',
		ondismiss,
		children
	}: {
		variant?: 'error' | 'success' | 'warning' | 'info';
		title?: string;
		dismissible?: boolean;
		class?: string;
		ondismiss?: () => void;
		children?: Snippet;
	} = $props();

	let dismissed = $state(false);

	const variantStyles = {
		error: 'bg-red-500/10 border-red-500/25 text-red-400',
		success: 'bg-emerald-500/10 border-emerald-500/25 text-emerald-400',
		warning: 'bg-amber-500/10 border-amber-500/25 text-amber-400',
		info: 'bg-[var(--primary-light)] border-[var(--primary-focus)] text-[var(--primary)]'
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
				<AlertCircle size={16} />
			{:else if variant === 'warning'}
				<AlertTriangle size={16} />
			{:else if variant === 'success'}
				<CheckCircle2 size={16} />
			{:else}
				<Info size={16} />
			{/if}
		</div>

		<div class="flex-1 min-w-0">
			{#if title}
				<div class="font-bold mb-0.5">{title}</div>
			{/if}
			<div>
				{@render children?.()}
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

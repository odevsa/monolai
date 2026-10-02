<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		variant = 'muted',
		dot = false,
		class: className = '',
		children
	}: {
		variant?: 'success' | 'warning' | 'danger' | 'primary' | 'muted' | 'pill';
		dot?: boolean;
		class?: string;
		children?: Snippet;
	} = $props();

	let variantClass = $derived.by(() => {
		switch (variant) {
			case 'success':
				return 'app-badge-success';
			case 'warning':
				return 'app-badge-warning';
			case 'danger':
				return 'app-badge-danger';
			case 'primary':
				return 'app-badge-primary';
			case 'pill':
				return 'app-badge-pill';
			default:
				return 'app-badge-muted';
		}
	});

	let dotClass = $derived.by(() => {
		switch (variant) {
			case 'success':
				return 'bg-emerald-400';
			case 'warning':
				return 'bg-amber-400';
			case 'danger':
				return 'bg-red-400';
			case 'primary':
				return 'bg-[var(--primary)]';
			default:
				return 'bg-[var(--text-muted)]';
		}
	});
</script>

<span class="{variant === 'pill' ? 'app-badge-pill' : 'app-badge'} {variantClass} {className}">
	{#if dot}
		<span class="app-badge-dot {dotClass}"></span>
	{/if}
	{@render children?.()}
</span>

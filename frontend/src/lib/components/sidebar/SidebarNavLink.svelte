<script lang="ts">
	import type { Component } from 'svelte';

	interface SidebarNavLinkProps {
		href?: string;
		label: string;
		icon: Component<{ size?: number; class?: string }>;
		isActive?: boolean;
		isExpanded?: boolean;
		title?: string;
		onclick?: (e: MouseEvent) => void;
	}

	let {
		href,
		label,
		icon: Icon,
		isActive = false,
		isExpanded = false,
		title,
		onclick
	}: SidebarNavLinkProps = $props();

	let linkTitle = $derived(title ?? label);
	let baseClasses = $derived(
		`flex items-center rounded-xl transition-all duration-150 box-border border-0 cursor-pointer ${
			!isExpanded
				? 'w-10 h-10 mx-auto justify-center'
				: 'gap-3 w-full px-3.5 py-2.5 text-sm font-semibold'
		} ${
			isActive
				? 'text-[var(--primary)] bg-[var(--primary-light)] font-semibold'
				: 'text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'
		}`
	);
</script>

{#if href}
	<a {href} class="{baseClasses} no-underline" {onclick} title={linkTitle}>
		<Icon size={20} class="shrink-0 {isActive ? 'text-[var(--primary)]' : ''}" />
		{#if isExpanded}
			<span>{label}</span>
		{/if}
	</a>
{:else}
	<button type="button" class={baseClasses} {onclick} title={linkTitle}>
		<Icon size={20} class="shrink-0 {isActive ? 'text-[var(--primary)]' : ''}" />
		{#if isExpanded}
			<span>{label}</span>
		{/if}
	</button>
{/if}

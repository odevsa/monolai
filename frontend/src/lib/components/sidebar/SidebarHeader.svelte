<script lang="ts">
	import { Logo } from '$lib/components/ui';
	import { PanelLeftClose, PanelLeftOpen, X } from '@lucide/svelte';

	interface SidebarHeaderProps {
		isExpanded: boolean;
		onToggleCollapse: () => void;
		onCloseMobile: () => void;
	}

	let { isExpanded, onToggleCollapse, onCloseMobile }: SidebarHeaderProps = $props();

	let isLogoHovered = $state(false);
</script>

<div class="flex items-center justify-between px-3 pt-3.5 pb-2 h-[52px] shrink-0 box-border">
	{#if isExpanded}
		<a href="/chat" class="flex items-center no-underline" onclick={onCloseMobile}>
			<Logo size={26} />
		</a>

		<button
			type="button"
			class="hidden md:flex items-center justify-center w-8 h-8 bg-transparent border-0 rounded-lg text-[var(--text-muted)] cursor-pointer transition-all duration-150 hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
			onclick={onToggleCollapse}
			aria-label="Collapse sidebar"
			title="Collapse sidebar"
		>
			<PanelLeftClose size={19} />
		</button>
	{:else}
		<button
			type="button"
			class="hidden md:flex items-center justify-center w-10 h-10 mx-auto bg-[var(--bg-surface)]/50 backdrop-blur-md border border-[var(--border-color)]/70 rounded-xl text-[var(--text-secondary)] cursor-pointer transition-all duration-150 hover:text-[var(--text-primary)] hover:bg-[var(--bg-surface)]/80 hover:border-[var(--border-hover)] shadow-xs"
			onclick={onToggleCollapse}
			onmouseenter={() => (isLogoHovered = true)}
			onmouseleave={() => (isLogoHovered = false)}
			aria-label="Open Sidebar"
			title="Open Sidebar"
		>
			{#if isLogoHovered}
				<PanelLeftOpen size={21} class="text-[var(--text-primary)]" />
			{:else}
				<Logo size={24} />
			{/if}
		</button>
	{/if}

	<!-- Mobile Close Button -->
	<button
		type="button"
		class="flex md:hidden items-center justify-center w-8 h-8 bg-transparent border-0 rounded-md text-[var(--text-muted)] cursor-pointer hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
		onclick={onCloseMobile}
		aria-label="Close menu"
	>
		<X size={18} />
	</button>
</div>

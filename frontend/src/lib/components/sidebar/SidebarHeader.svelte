<script lang="ts">
	import { Button, Logo } from '$lib/components/ui';
	import { t } from '$lib/i18n';
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

		<Button
			variant="ghost"
			size="icon"
			class="hidden md:flex text-[var(--text-muted)]"
			onclick={onToggleCollapse}
			ariaLabel={t('navigation.collapseSidebar')}
			title={t('navigation.collapseSidebar')}
		>
			<PanelLeftClose size={19} />
		</Button>
	{:else}
		<button
			type="button"
			class="hidden md:flex items-center justify-center w-10 h-10 mx-auto bg-[var(--bg-surface)]/50 backdrop-blur-md border border-[var(--border-color)]/70 rounded-xl text-[var(--text-secondary)] cursor-pointer transition-all duration-150 hover:text-[var(--text-primary)] hover:bg-[var(--bg-surface)]/80 hover:border-[var(--border-hover)] shadow-xs"
			onclick={onToggleCollapse}
			onmouseenter={() => (isLogoHovered = true)}
			onmouseleave={() => (isLogoHovered = false)}
			aria-label={t('navigation.openSidebar')}
			title={t('navigation.openSidebar')}
		>
			{#if isLogoHovered}
				<PanelLeftOpen size={21} class="text-[var(--text-primary)]" />
			{:else}
				<Logo size={24} />
			{/if}
		</button>
	{/if}

	<!-- Mobile Close Button -->
	<Button
		variant="ghost"
		size="icon"
		class="flex md:hidden text-[var(--text-muted)]"
		onclick={onCloseMobile}
		ariaLabel={t('navigation.closeMenu')}
	>
		<X size={18} />
	</Button>
</div>

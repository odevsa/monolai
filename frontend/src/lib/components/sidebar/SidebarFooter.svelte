<script lang="ts">
	import { Button } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import { APP_VERSION } from '$lib/version';
	import { Settings } from '@lucide/svelte';

	interface SidebarFooterProps {
		isExpanded: boolean;
		onOpenSettings?: () => void;
		onCloseMobile?: () => void;
	}

	let { isExpanded, onOpenSettings, onCloseMobile }: SidebarFooterProps = $props();

	function handleSettingsClick() {
		onOpenSettings?.();
		onCloseMobile?.();
	}
</script>

<div
	class="mt-auto shrink-0 {isExpanded
		? 'p-3 pb-[max(0.75rem,env(safe-area-inset-bottom,0.75rem))] flex items-center justify-between'
		: 'p-2.5 pb-[max(0.75rem,env(safe-area-inset-bottom,0.75rem))] flex items-center justify-center'}"
>
	{#if isExpanded}
		<span class="text-[0.8rem] text-[var(--text-muted)] select-none">v{APP_VERSION}</span>
		<Button
			variant="ghost"
			size="icon-sm"
			class="text-[var(--text-muted)] hover:text-[var(--text-primary)]"
			onclick={handleSettingsClick}
			title={t('navigation.settings')}
			ariaLabel={t('navigation.settings')}
		>
			<Settings size={18} />
		</Button>
	{:else}
		<button
			type="button"
			class="flex items-center justify-center w-10 h-10 mx-auto rounded-xl text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] transition-all duration-150 cursor-pointer border-0 outline-none focus-visible:ring-2 focus-visible:ring-[var(--primary)]/50"
			onclick={handleSettingsClick}
			title={t('navigation.settings')}
			aria-label={t('navigation.settings')}
		>
			<Settings size={20} class="shrink-0" />
		</button>
	{/if}
</div>

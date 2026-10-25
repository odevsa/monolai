<script lang="ts">
	import {
		SettingsAboutTab,
		SettingsChatTab,
		SettingsGeneralTab,
		SettingsRepositoriesTab
	} from '$lib/components/settings';
	import { Button } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ThemeId } from '$lib/themes';
	import { Database, Info, MessageSquare, SlidersVertical, X } from '@lucide/svelte';

	let {
		open = $bindable(false),
		themeMode = $bindable<ThemeId>('system'),
		effectiveTheme = 'dark'
	}: {
		open: boolean;
		themeMode: ThemeId;
		effectiveTheme: 'dark' | 'light';
	} = $props();

	let activeTab = $state<'general' | 'repositories' | 'chat' | 'about'>('general');

	function close() {
		open = false;
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			close();
		}
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			close();
		}
	}
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if open}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 bg-black/75 backdrop-blur-md flex items-end sm:items-center justify-center z-[200] p-0 sm:p-4 animate-in fade-in duration-200"
		onclick={handleBackdropClick}
	>
		<div
			class="w-full max-w-[760px] h-[92vh] sm:h-[82vh] sm:max-h-[680px] bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-t-2xl sm:rounded-2xl overflow-hidden flex flex-col shadow-2xl transition-all duration-200"
		>
			<!-- Dialog Header -->
			<div
				class="flex items-center justify-between px-4 sm:px-6 py-3.5 border-b border-[var(--border-color)] shrink-0 bg-[var(--bg-sidebar)]/60 backdrop-blur-md"
			>
				<div class="flex items-center gap-2.5">
					<SlidersVertical size={20} class="text-[var(--primary)] shrink-0" />
					<div>
						<h3 class="m-0 text-base sm:text-lg font-bold text-[var(--text-primary)] leading-none">
							{t('navigation.settings')}
						</h3>
					</div>
				</div>

				<Button
					variant="ghost"
					size="icon-sm"
					onclick={close}
					title={t('common.close')}
					ariaLabel={t('common.close')}
				>
					<X size={18} />
				</Button>
			</div>

			<!-- Dialog Body with Left Sidebar / Navigation Bar -->
			<div class="flex flex-col sm:flex-row flex-1 overflow-hidden min-h-0">
				<!-- Navigation Bar: Left Sidebar on Desktop (sm:), Horizontal Scroll Bar on Mobile -->
				<div
					class="w-full sm:w-[190px] bg-[var(--bg-sidebar)]/50 border-b sm:border-b-0 sm:border-r border-[var(--border-color)] p-2 sm:p-3 flex sm:flex-col gap-1.5 shrink-0 overflow-x-auto sm:overflow-x-visible no-scrollbar"
				>
					<button
						type="button"
						class="flex items-center justify-center sm:justify-start gap-2.5 px-3.5 py-2.5 text-xs font-medium border-0 rounded-xl cursor-pointer transition-all duration-150 text-left shrink-0 sm:shrink flex-1 sm:flex-none text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] {activeTab ===
						'general'
							? '!text-[var(--primary)] !bg-[var(--primary-light)] font-semibold shadow-xs'
							: ''}"
						onclick={() => (activeTab = 'general')}
					>
						<SlidersVertical size={16} />
						<span>{t('settings.generalTab')}</span>
					</button>

					<button
						type="button"
						class="flex items-center justify-center sm:justify-start gap-2.5 px-3.5 py-2.5 text-xs font-medium border-0 rounded-xl cursor-pointer transition-all duration-150 text-left shrink-0 sm:shrink flex-1 sm:flex-none text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] {activeTab ===
						'repositories'
							? '!text-[var(--primary)] !bg-[var(--primary-light)] font-semibold shadow-xs'
							: ''}"
						onclick={() => (activeTab = 'repositories')}
					>
						<Database size={16} />
						<span>{t('settings.repositoriesTab')}</span>
					</button>

					<button
						type="button"
						class="flex items-center justify-center sm:justify-start gap-2.5 px-3.5 py-2.5 text-xs font-medium border-0 rounded-xl cursor-pointer transition-all duration-150 text-left shrink-0 sm:shrink flex-1 sm:flex-none text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] {activeTab ===
						'chat'
							? '!text-[var(--primary)] !bg-[var(--primary-light)] font-semibold shadow-xs'
							: ''}"
						onclick={() => (activeTab = 'chat')}
					>
						<MessageSquare size={16} />
						<span>{t('navigation.chat')}</span>
					</button>

					<button
						type="button"
						class="flex items-center justify-center sm:justify-start gap-2.5 px-3.5 py-2.5 text-xs font-medium border-0 rounded-xl cursor-pointer transition-all duration-150 text-left shrink-0 sm:shrink flex-1 sm:flex-none text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] {activeTab ===
						'about'
							? '!text-[var(--primary)] !bg-[var(--primary-light)] font-semibold shadow-xs'
							: ''}"
						onclick={() => (activeTab = 'about')}
					>
						<Info size={16} />
						<span>{t('settings.aboutTab')}</span>
					</button>
				</div>

				<!-- Content View Area -->
				<div class="flex-1 overflow-hidden min-h-0 flex flex-col bg-[var(--bg-surface)]">
					{#if activeTab === 'general'}
						<SettingsGeneralTab bind:themeMode />
					{:else if activeTab === 'repositories'}
						<SettingsRepositoriesTab />
					{:else if activeTab === 'chat'}
						<SettingsChatTab />
					{:else if activeTab === 'about'}
						<SettingsAboutTab />
					{/if}
				</div>
			</div>
		</div>
	</div>
{/if}

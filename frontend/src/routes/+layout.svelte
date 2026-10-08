<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { chatsApi, configApi } from '$lib/api';
	import { Badge, ConfirmDialog, FeatureGate, Logo } from '$lib/components/ds';
	import SettingsDialog from '$lib/components/SettingsDialog.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { askConfirm } from '$lib/confirmStore';
	import { refreshFeatures } from '$lib/featuresStore';
	import { t } from '$lib/i18n';
	import { addChatTab, closeChatTab, headerState, selectChatTab } from '$lib/state/header.svelte';
	import { getThemeType, resolveEffectiveTheme, type ThemeId } from '$lib/themes';
	import type { ConfigStatus } from '$lib/types/config';
	import { PanelLeftOpen, Plus, RefreshCw, Trash2, X } from '@lucide/svelte';
	import { onMount, setContext } from 'svelte';
	import '../app.css';

	let { children } = $props();

	let themeMode = $state<ThemeId>('system');
	let systemPrefersDark = $state(false);
	let sidebarCollapsed = $state(false);
	let mobileSidebarOpen = $state(false);
	let settingsOpen = $state(false);
	let configStatus = $state<ConfigStatus | null>(null);

	let effectiveTheme = $derived(resolveEffectiveTheme(themeMode, systemPrefersDark));
	let themeType = $derived(getThemeType(effectiveTheme));

	let currentPath = $derived(page.url.pathname);
	let isErrorPage = $derived((page.status && page.status >= 400) || !!page.error);
	let isSetupPage = $derived(currentPath.startsWith('/setup'));
	let isChatPage = $derived(
		currentPath === '/chat' || currentPath === '/' || currentPath.startsWith('/chat')
	);
	let activeChatIdFromUrl = $derived(
		currentPath.startsWith('/chat/') && currentPath !== '/chat'
			? currentPath.replace('/chat/', '')
			: null
	);
	let isSysinfoPage = $derived(currentPath.startsWith('/sysinfo'));
	let isRuntimesPage = $derived(currentPath.startsWith('/runtimes'));
	let isImagePage = $derived(currentPath.startsWith('/image'));

	async function deleteActiveChat(id: string) {
		const confirmed = await askConfirm(
			t('chat.deleteChatConfirmMsg'),
			t('chat.deleteChatConfirmTitle'),
			'danger',
			t('common.delete'),
			t('common.cancel')
		);
		if (!confirmed) return;

		try {
			await chatsApi.delete(id);
		} catch (err) {
			console.error('Failed to delete active chat:', err);
		}
		headerState.chatTabs = headerState.chatTabs.filter((t) => t.id !== id);
		if (typeof window !== 'undefined') {
			goto('/chat');
		}
	}

	async function checkConfigStatus() {
		try {
			const status = await configApi.getStatus();
			configStatus = status;
		} catch (err) {
			console.error('Failed to fetch config status:', err);
		}
	}

	setContext('checkConfigStatus', checkConfigStatus);

	onMount(() => {
		checkConfigStatus();
	});

	$effect(() => {
		if (configStatus && !isErrorPage) {
			if (!configStatus.is_valid && !currentPath.startsWith('/setup')) {
				goto('/setup');
			} else if (configStatus.is_valid && currentPath.startsWith('/setup')) {
				goto('/chat');
			}
		}
	});

	$effect(() => {
		if (typeof window !== 'undefined') {
			const stored = localStorage.getItem('monolai:theme') as ThemeId;
			if (stored) {
				themeMode = stored;
			}

			const media = window.matchMedia('(prefers-color-scheme: dark)');
			systemPrefersDark = media.matches;

			const listener = (e: MediaQueryListEvent) => {
				systemPrefersDark = e.matches;
			};
			media.addEventListener('change', listener);
			return () => media.removeEventListener('change', listener);
		}
	});

	onMount(() => {
		refreshFeatures();
		if (typeof window !== 'undefined' && window.visualViewport) {
			const resetScroll = () => {
				if (window.scrollY !== 0) {
					window.scrollTo(0, 0);
				}
			};
			window.visualViewport.addEventListener('resize', resetScroll);
			window.visualViewport.addEventListener('scroll', resetScroll);
			return () => {
				window.visualViewport?.removeEventListener('resize', resetScroll);
				window.visualViewport?.removeEventListener('scroll', resetScroll);
			};
		}
	});

	$effect(() => {
		if (typeof document !== 'undefined') {
			localStorage.setItem('monolai:theme', themeMode);
			document.documentElement.setAttribute('data-theme', effectiveTheme);

			const faviconEl = document.getElementById('app-favicon') as HTMLLinkElement | null;
			if (faviconEl) {
				faviconEl.href = effectiveTheme === 'light' ? '/favicon-light.svg' : '/favicon-dark.svg';
			}
		}
	});
	let isMobileLogoHovered = $state(false);
	let isConnection = !headerState.sysinfoConnected && headerState.sysinfoReconnectFn;
</script>

<div
	class="fixed inset-0 w-full h-full h-dvh flex flex-col overflow-hidden bg-[var(--bg-primary)] text-[var(--text-primary)]"
>
	{#if isErrorPage}
		<main class="flex-1 w-full h-full relative overflow-auto bg-[var(--bg-primary)] p-0 box-border">
			{@render children()}
		</main>
	{:else if isSetupPage || (configStatus && !configStatus.is_valid)}
		<main class="flex-1 w-full h-full relative overflow-auto bg-[var(--bg-primary)] p-0 box-border">
			{@render children()}
		</main>
	{:else}
		<!-- Mobile Floating Sidebar Open Button -->
		<button
			type="button"
			class="md:hidden fixed top-3.5 left-3.5 w-10 h-10 flex items-center justify-center bg-[var(--bg-surface)]/50 backdrop-blur-md border border-[var(--border-color)]/70 rounded-xl text-[var(--text-primary)] cursor-pointer z-[90] transition-all duration-150 hover:bg-[var(--bg-surface)]/80 hover:border-[var(--border-hover)] shadow-xs"
			onclick={() => (mobileSidebarOpen = true)}
			onmouseenter={() => (isMobileLogoHovered = true)}
			onmouseleave={() => (isMobileLogoHovered = false)}
			aria-label="Open Sidebar"
			title="Open Sidebar"
		>
			{#if isMobileLogoHovered}
				<PanelLeftOpen size={21} />
			{:else}
				<Logo size={22} />
			{/if}
		</button>

		<div class="flex flex-1 w-full h-full min-h-0 overflow-hidden">
			<Sidebar
				effectiveTheme={themeType}
				bind:collapsed={sidebarCollapsed}
				bind:mobileOpen={mobileSidebarOpen}
				onOpenSettings={() => (settingsOpen = true)}
			/>

			<div
				class="flex-1 h-full min-h-0 relative flex flex-col bg-[var(--bg-primary)] overflow-hidden"
			>
				<!-- Route-dependent Floating Header Container -->
				<header
					class="absolute top-3.5 md:top-[21px] left-3.5 right-3.5 flex items-center justify-between z-50 pointer-events-none md:ml-0 ml-14"
				>
					<div class="flex items-center gap-2 pointer-events-auto">
						{#if isChatPage}
							<FeatureGate features={['chat']} showCard={false}>
								<div class="flex items-center gap-1.5 pointer-events-auto max-w-full flex-nowrap">
									{#each headerState.chatTabs as tab (tab.id)}
										<a
											href="/chat/{tab.id}"
											class="hidden md:flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-[var(--text-primary)] bg-[var(--bg-surface)]/50 backdrop-blur-md border border-[var(--border-color)]/70 rounded-lg transition-all duration-150 select-none max-w-[130px] whitespace-nowrap shrink min-w-0 no-underline hover:bg-[var(--bg-surface)]/80 hover:border-[var(--border-hover)] {tab.active
												? 'bg-[var(--bg-surface)]/90 border-[var(--border-hover)]'
												: ''}"
											onclick={() => selectChatTab(tab.id)}
											title={tab.title}
										>
											<span class="truncate block min-w-0">{tab.title}</span>
											{#if headerState.chatTabs.length > 1}
												<button
													type="button"
													class="flex items-center justify-center p-0 border-0 bg-transparent text-[var(--text-muted)] cursor-pointer rounded hover:text-[var(--text-primary)]"
													onclick={(e) => {
														e.stopPropagation();
														e.preventDefault();
														closeChatTab(tab.id);
													}}
													title={t('chat.closeTab')}
												>
													<X size={13} />
												</button>
											{/if}
										</a>
									{/each}

									<button
										type="button"
										class="flex items-center justify-center w-7 h-7 bg-[var(--bg-surface)]/50 backdrop-blur-md border border-[var(--border-color)]/70 text-[var(--text-muted)] cursor-pointer rounded-lg transition-all duration-150 pointer-events-auto hover:text-[var(--text-primary)] hover:bg-[var(--bg-surface)]/80 hover:border-[var(--border-hover)] shrink-0"
										onclick={addChatTab}
										title={t('chat.newTab')}
									>
										<Plus size={16} />
									</button>
								</div>
							</FeatureGate>
						{/if}
					</div>

					<div class="flex items-center gap-2 pointer-events-auto">
						{#if isChatPage && activeChatIdFromUrl}
							<FeatureGate features={['chat']} showCard={false}>
								<button
									type="button"
									class="flex items-center justify-center w-7 h-7 bg-[var(--bg-surface)]/50 backdrop-blur-md border border-[var(--border-color)]/70 text-[var(--text-muted)] cursor-pointer rounded-lg transition-all duration-150 shrink-0 hover:text-red-500 hover:bg-red-500/15 hover:border-red-500/30"
									onclick={() => deleteActiveChat(activeChatIdFromUrl)}
									title={t('chat.deleteChat')}
									aria-label={t('chat.deleteChat')}
								>
									<Trash2 size={15} />
								</button>
							</FeatureGate>
						{/if}
						{#if isSysinfoPage}
							<Badge
								variant={!headerState.sysinfoConnected && headerState.sysinfoReconnectFn
									? 'warning'
									: 'success'}
								pulse
								blur
							>
								<span class="truncate">
									{headerState.sysinfoConnected
										? t('sysinfo.streamActive')
										: t('sysinfo.connecting')}
								</span>
								{#if !headerState.sysinfoConnected && headerState.sysinfoReconnectFn}
									<button
										type="button"
										class="flex items-center justify-center p-0.5 border-0 bg-transparent {isConnection
											? 'text-amber-400'
											: 'text-emerald-400'} cursor-pointer rounded"
										onclick={() => headerState.sysinfoReconnectFn?.()}
										title={t('sysinfo.reconnectStream')}
									>
										<RefreshCw size={13} class="animate-spin" />
									</button>
								{/if}
							</Badge>
						{/if}
						{#if isRuntimesPage}
							<button
								type="button"
								class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-[var(--text-secondary)] bg-[var(--bg-surface)]/50 backdrop-blur-md border border-[var(--border-color)]/70 rounded-lg pointer-events-auto transition-all duration-150 hover:text-[var(--text-primary)] hover:bg-[var(--bg-surface)]/80 hover:border-[var(--border-hover)] cursor-pointer"
								onclick={() => headerState.runtimesRefreshFn?.()}
								disabled={headerState.runtimesRefreshing}
								title={t('runtimes.refreshList')}
							>
								<RefreshCw
									size={13}
									class={headerState.runtimesRefreshing ? 'animate-spin text-[var(--primary)]' : ''}
								/>
								<span>{t('common.refresh')}</span>
							</button>
						{/if}
					</div>
				</header>

				<main
					class="flex-1 w-full h-full min-h-0 relative {isChatPage
						? 'overflow-hidden'
						: 'overflow-y-auto'} bg-[var(--bg-primary)]"
				>
					{@render children()}
				</main>
			</div>
		</div>

		<!-- Settings Dialog Modal & Global Confirm Dialog -->
		<SettingsDialog bind:open={settingsOpen} bind:themeMode effectiveTheme={themeType} />
		<ConfirmDialog />
	{/if}
</div>

<script lang="ts">
	import '../app.css';
	import { onMount, setContext } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
	import { askConfirm } from '$lib/confirmStore';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import SettingsDialog from '$lib/components/SettingsDialog.svelte';
	import Logo from '$lib/components/Logo.svelte';
	import { PanelLeftOpen, Plus, X, RefreshCw, Trash2 } from '@lucide/svelte';
	import {
		chatTabs,
		sysinfoConnected,
		sysinfoReconnectFn,
		runtimesRefreshing,
		runtimesRefreshFn,
		addChatTab,
		selectChatTab,
		closeChatTab
	} from '$lib/headerStore';
	import { resolveEffectiveTheme, getThemeType, type ThemeId } from '$lib/themes';

	let { children } = $props();

	interface ConfigStatus {
		is_valid: boolean;
		has_models: boolean;
		has_runtimes: boolean;
		created_auto_file: boolean;
		loaded_path: string | null;
		expected_path: string;
		models_dir: string | null;
		host?: string;
		port?: number;
		error_message: string | null;
		example_yaml: string;
	}

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

	async function deleteActiveChat(id: string) {
		const confirmed = await askConfirm(
			'Are you sure you want to delete this conversation?',
			'Delete Conversation',
			'danger',
			'Delete',
			'Cancel'
		);
		if (!confirmed) return;

		try {
			await fetch(`/api/chats/${encodeURIComponent(id)}`, { method: 'DELETE' });
		} catch (err) {
			console.error('Failed to delete active chat:', err);
		}
		chatTabs.update((tabs) => tabs.filter((t) => t.id !== id));
		if (typeof window !== 'undefined') {
			goto('/chat');
		}
	}

	async function checkConfigStatus() {
		try {
			const res = await fetch('/api/config/status');
			if (res.ok) {
				const status: ConfigStatus = await res.json();
				configStatus = status;
			}
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
			class="md:hidden fixed top-3.5 left-3.5 w-9 h-9 flex items-center justify-center bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-lg text-[var(--text-primary)] cursor-pointer z-[90] transition-all duration-150 hover:bg-[var(--bg-surface-hover)] hover:border-[var(--border-hover)]"
			onclick={() => (mobileSidebarOpen = true)}
			onmouseenter={() => (isMobileLogoHovered = true)}
			onmouseleave={() => (isMobileLogoHovered = false)}
			aria-label="Open Sidebar"
			title="Open Sidebar"
		>
			{#if isMobileLogoHovered}
				<PanelLeftOpen size={20} />
			{:else}
				<Logo size={20} />
			{/if}
		</button>

		<div class="flex flex-1 w-full h-full min-h-0 overflow-hidden">
			<Sidebar
				effectiveTheme={themeType}
				bind:collapsed={sidebarCollapsed}
				bind:mobileOpen={mobileSidebarOpen}
				onOpenSettings={() => (settingsOpen = true)}
			/>

			<main
				class="flex-1 h-full min-h-0 relative {isChatPage
					? 'overflow-hidden'
					: 'overflow-auto'} bg-[var(--bg-primary)]"
			>
				<!-- Route-dependent Floating Header Container -->
				<header
					class="absolute top-3.5 md:top-[21px] left-3.5 right-3.5 flex items-center justify-between z-50 pointer-events-none md:ml-0 ml-13"
				>
					<div class="flex items-center gap-2 pointer-events-auto">
						{#if isChatPage}
							<div class="flex items-center gap-1.5 pointer-events-auto max-w-full flex-nowrap">
								{#each $chatTabs as tab (tab.id)}
									<a
										href="/chat/{tab.id}"
										class="hidden md:flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-[var(--text-primary)] bg-[var(--bg-surface)]/50 backdrop-blur-md border border-[var(--border-color)]/70 rounded-lg transition-all duration-150 select-none max-w-[130px] whitespace-nowrap shrink min-w-0 no-underline hover:bg-[var(--bg-surface)]/80 hover:border-[var(--border-hover)] {tab.active
											? 'bg-[var(--bg-surface)]/90 border-[var(--border-hover)]'
											: ''}"
										onclick={() => selectChatTab(tab.id)}
										title={tab.title}
									>
										<span class="truncate block min-w-0">{tab.title}</span>
										{#if $chatTabs.length > 1}
											<button
												type="button"
												class="flex items-center justify-center p-0 border-0 bg-transparent text-[var(--text-muted)] cursor-pointer rounded hover:text-[var(--text-primary)]"
												onclick={(e) => {
													e.stopPropagation();
													e.preventDefault();
													closeChatTab(tab.id);
												}}
												title="Close tab"
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
									title="New Tab"
								>
									<Plus size={16} />
								</button>
							</div>
						{/if}
					</div>

					<div class="flex items-center gap-2 pointer-events-auto">
						{#if isChatPage && activeChatIdFromUrl}
							<button
								type="button"
								class="flex items-center justify-center w-7 h-7 bg-[var(--bg-surface)]/50 backdrop-blur-md border border-[var(--border-color)]/70 text-[var(--text-muted)] cursor-pointer rounded-lg transition-all duration-150 shrink-0 hover:text-red-500 hover:bg-red-500/15 hover:border-red-500/30"
								onclick={() => deleteActiveChat(activeChatIdFromUrl)}
								title="Delete chat"
								aria-label="Delete chat"
							>
								<Trash2 size={15} />
							</button>
						{/if}
						{#if isSysinfoPage}
							<div
								class="flex items-center gap-2 px-3 py-1.5 text-xs font-medium text-[var(--text-secondary)] bg-[var(--bg-surface)]/50 backdrop-blur-md border border-[var(--border-color)]/70 rounded-lg pointer-events-auto"
							>
								<span
									class="w-2 h-2 rounded-full shrink-0 transition-all duration-200 {$sysinfoConnected
										? 'bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.4)] animate-pulse'
										: 'bg-red-500'}"
								></span>
								<span class="truncate">
									{$sysinfoConnected ? 'Stream Active' : 'Connecting...'}
								</span>
								{#if !$sysinfoConnected && $sysinfoReconnectFn}
									<button
										type="button"
										class="flex items-center justify-center p-0.5 border-0 bg-transparent text-[var(--text-muted)] cursor-pointer rounded hover:text-[var(--text-primary)]"
										onclick={() => $sysinfoReconnectFn?.()}
										title="Reconnect Stream"
									>
										<RefreshCw size={13} class="animate-spin" />
									</button>
								{/if}
							</div>
						{/if}
						{#if isRuntimesPage}
							<button
								type="button"
								class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-[var(--text-secondary)] bg-[var(--bg-surface)]/50 backdrop-blur-md border border-[var(--border-color)]/70 rounded-lg pointer-events-auto transition-all duration-150 hover:text-[var(--text-primary)] hover:bg-[var(--bg-surface)]/80 hover:border-[var(--border-hover)] cursor-pointer"
								onclick={() => $runtimesRefreshFn?.()}
								disabled={$runtimesRefreshing}
								title="Refresh runtimes list"
							>
								<RefreshCw
									size={13}
									class={$runtimesRefreshing ? 'animate-spin text-[var(--primary)]' : ''}
								/>
								<span>Refresh</span>
							</button>
						{/if}
					</div>
				</header>

				<div class="w-full h-full relative">
					{@render children()}
				</div>
			</main>
		</div>

		<!-- Settings Dialog Modal & Global Confirm Dialog -->
		<SettingsDialog bind:open={settingsOpen} bind:themeMode effectiveTheme={themeType} />
		<ConfirmDialog />
	{/if}
</div>

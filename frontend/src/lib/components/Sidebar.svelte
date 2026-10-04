<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { APP_VERSION } from '$lib/version';
	import Logo from '$lib/components/Logo.svelte';
	import {
		MessageSquare,
		Activity,
		Settings,
		Boxes,
		Box,
		PanelLeftClose,
		PanelLeftOpen,
		Search,
		Cpu,
		HardDrive,
		X,
		Power,
		Plus,
		Trash2
	} from '@lucide/svelte';
	import { chatTabs, syncRecentChats, runtimesRefreshFn } from '$lib/headerStore';
	import { askConfirm } from '$lib/confirmStore';
	import ModelBadge from '$lib/components/ModelBadge.svelte';
	import {
		runningModels,
		unloadingModelIds,
		getStatusType,
		startRunningStatePolling,
		unloadModel,
		unloadAllModels
	} from '$lib/runningModelsStore';

	let {
		effectiveTheme = 'dark',
		collapsed = $bindable(false),
		mobileOpen = $bindable(false),
		activeChatId = $bindable('oi-1'),
		onOpenSettings = () => {}
	}: {
		effectiveTheme: 'dark' | 'light';
		collapsed: boolean;
		mobileOpen: boolean;
		activeChatId?: string;
		onOpenSettings?: () => void;
	} = $props();

	let searchQuery = $state('');
	let isLogoHovered = $state(false);

	let loadedModels = $derived($runningModels);
	let unloadingIds = $derived($unloadingModelIds);

	let conversations = $state<{ id: string; title: string }[]>([]);

	async function fetchChats() {
		try {
			const res = await fetch('/api/chats');
			if (res.ok) {
				const data = await res.json();
				conversations = data;
				syncRecentChats(data);
			}
		} catch {
			// ignore polling errors
		}
	}

	async function deleteChat(id: string, e: MouseEvent) {
		e.stopPropagation();
		e.preventDefault();
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
			chatTabs.update((tabs) => tabs.filter((t) => t.id !== id));
			if (currentPath === `/chat/${id}`) {
				goto('/chat');
			}
			await fetchChats();
		} catch (err) {
			console.error('Error deleting chat:', err);
		}
	}

	function createNewChat() {
		closeMobile();
		goto('/chat');
	}

	$effect(() => {
		const stopPolling = startRunningStatePolling(3000);
		fetchChats();
		const interval = setInterval(() => {
			fetchChats();
		}, 3000);
		return () => {
			stopPolling();
			clearInterval(interval);
		};
	});

	function toggleCollapse() {
		collapsed = !collapsed;
	}

	function closeMobile() {
		mobileOpen = false;
	}

	let filteredConversations = $derived(
		conversations.filter((c) => c.title.toLowerCase().includes(searchQuery.toLowerCase().trim()))
	);

	function handleChatSelect(id: string) {
		activeChatId = id;
		closeMobile();
	}

	let currentPath = $derived(page.url.pathname);
	let isChatPage = $derived(
		currentPath === '/chat' || currentPath === '/' || currentPath.startsWith('/chat')
	);
	let isSysInfoPage = $derived(currentPath === '/sysinfo');
	let isRuntimesPage = $derived(currentPath.startsWith('/runtimes'));
	let isModelsPage = $derived(currentPath.startsWith('/models'));
	let currentActiveChatId = $derived(
		currentPath.startsWith('/chat/') ? currentPath.replace('/chat/', '') : ''
	);
	let isExpanded = $derived(!collapsed || mobileOpen);
</script>

<!-- Mobile Overlay Backdrop -->
{#if mobileOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="fixed inset-0 bg-black/50 backdrop-blur-xs z-99 md:hidden"
		onclick={closeMobile}
	></div>
{/if}

<aside
	class="sidebar-card flex flex-col shrink-0 z-100 overflow-hidden box-border transition-all duration-200 ease-in-out my-3 ml-3 rounded-2xl {!isExpanded
		? 'w-[64px] bg-transparent'
		: 'w-[300px] bg-[var(--bg-sidebar)]'} {mobileOpen ? 'mobile-open' : ''}"
>
	<!-- Sidebar Header -->
	<div class="flex items-center justify-between px-3 pt-3.5 pb-2 h-[52px] shrink-0 box-border">
		{#if isExpanded}
			<a href="/chat" class="flex items-center no-underline" onclick={closeMobile}>
				<Logo size={26} />
			</a>

			<button
				type="button"
				class="hidden md:flex items-center justify-center w-8 h-8 bg-transparent border-0 rounded-lg text-[var(--text-muted)] cursor-pointer transition-all duration-150 hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
				onclick={toggleCollapse}
				aria-label="Collapse sidebar"
				title="Collapse sidebar"
			>
				<PanelLeftClose size={19} />
			</button>
		{:else}
			<button
				type="button"
				class="hidden md:flex items-center justify-center w-10 h-10 mx-auto bg-transparent border-0 rounded-xl text-[var(--text-secondary)] cursor-pointer transition-all duration-150 hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
				onclick={toggleCollapse}
				onmouseenter={() => (isLogoHovered = true)}
				onmouseleave={() => (isLogoHovered = false)}
				aria-label="Open Sidebar"
				title="Open Sidebar"
			>
				{#if isLogoHovered}
					<PanelLeftOpen size={20} class="text-[var(--text-primary)]" />
				{:else}
					<Logo size={24} />
				{/if}
			</button>
		{/if}

		<!-- Mobile Close Button -->
		<button
			type="button"
			class="flex md:hidden items-center justify-center w-8 h-8 bg-transparent border-0 rounded-md text-[var(--text-muted)] cursor-pointer hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
			onclick={closeMobile}
			aria-label="Close menu"
		>
			<X size={18} />
		</button>
	</div>

	<!-- Top Action Navigation Items -->
	<div class="flex flex-col gap-1.5 p-2.5">
		<!-- Chat Action -->
		<a
			href="/chat"
			class="flex items-center rounded-xl transition-all duration-150 box-border no-underline border-0 cursor-pointer {!isExpanded
				? 'w-10 h-10 mx-auto justify-center'
				: 'gap-3 w-full px-3.5 py-2.5 text-sm font-medium'} {isChatPage
				? 'text-[var(--primary)] bg-[var(--primary-light)] font-semibold'
				: 'text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'}"
			onclick={closeMobile}
			title="Chat"
		>
			<MessageSquare size={20} class="shrink-0 {isChatPage ? 'text-[var(--primary)]' : ''}" />
			{#if isExpanded}
				<span>Chat</span>
			{/if}
		</a>

		<!-- System Info Action -->
		<a
			href="/sysinfo"
			class="flex items-center rounded-xl transition-all duration-150 box-border no-underline border-0 cursor-pointer {!isExpanded
				? 'w-10 h-10 mx-auto justify-center'
				: 'gap-3 w-full px-3.5 py-2.5 text-sm font-medium'} {isSysInfoPage
				? 'text-[var(--primary)] bg-[var(--primary-light)] font-semibold'
				: 'text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'}"
			onclick={closeMobile}
			title="System Info"
		>
			<Activity size={20} class="shrink-0 {isSysInfoPage ? 'text-[var(--primary)]' : ''}" />
			{#if isExpanded}
				<span>System Info</span>
			{/if}
		</a>
	</div>

	<div class="px-3.5 py-3 border-t border-[var(--border-color)] flex flex-col gap-2">
		<!-- Runtimes Action -->
		<a
			href="/runtimes"
			class="flex items-center rounded-xl transition-all duration-150 box-border no-underline border-0 cursor-pointer {!isExpanded
				? 'w-10 h-10 mx-auto justify-center'
				: 'gap-3 w-full px-3.5 py-2.5 text-sm font-medium'} {isRuntimesPage
				? 'text-[var(--primary)] bg-[var(--primary-light)] font-semibold'
				: 'text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'}"
			onclick={() => {
				closeMobile();
				if (typeof window !== 'undefined') {
					window.dispatchEvent(new CustomEvent('monolai:nav-runtimes'));
				}
				$runtimesRefreshFn?.();
			}}
			title="Runtimes"
		>
			<Boxes size={20} class="shrink-0 {isRuntimesPage ? 'text-[var(--primary)]' : ''}" />
			{#if isExpanded}
				<span>Runtimes</span>
			{/if}
		</a>

		<!-- Models Action -->
		<a
			href="/models"
			class="flex items-center rounded-xl transition-all duration-150 box-border no-underline border-0 cursor-pointer {!isExpanded
				? 'w-10 h-10 mx-auto justify-center'
				: 'gap-3 w-full px-3.5 py-2.5 text-sm font-medium'} {isModelsPage
				? 'text-[var(--primary)] bg-[var(--primary-light)] font-semibold'
				: 'text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'}"
			onclick={() => {
				closeMobile();
				if (typeof window !== 'undefined') {
					window.dispatchEvent(new CustomEvent('monolai:nav-models'));
				}
			}}
			title="Models"
		>
			<Box size={20} class="shrink-0 {isModelsPage ? 'text-[var(--primary)]' : ''}" />
			{#if isExpanded}
				<span>Models</span>
			{/if}
		</a>

		<!-- Settings Action -->
		<button
			type="button"
			class="flex items-center rounded-xl transition-all duration-150 box-border border-0 cursor-pointer text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] {!isExpanded
				? 'w-10 h-10 mx-auto justify-center'
				: 'gap-3 w-full px-3.5 py-2.5 text-sm font-medium'}"
			onclick={() => {
				onOpenSettings();
				closeMobile();
			}}
			title="Settings"
		>
			<Settings size={20} class="shrink-0" />
			{#if isExpanded}
				<span>Settings</span>
			{/if}
		</button>
	</div>

	<!-- Loaded Models Section -->
	{#if isExpanded}
		<div class="px-3.5 py-3 border-t border-[var(--border-color)] flex flex-col gap-2">
			<div class="flex items-center justify-between">
				<span class="text-[0.7rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]">
					Loaded Models
				</span>
				{#if loadedModels.length > 0}
					<button
						type="button"
						class="bg-transparent border-0 text-[var(--text-muted)] text-[0.7rem] cursor-pointer px-1 py-0.5 rounded transition-colors hover:text-red-500"
						onclick={unloadAllModels}
						title="Unload all models"
					>
						Unload All
					</button>
				{/if}
			</div>

			{#if loadedModels.length === 0}
				<div
					class="bg-[var(--bg-surface)] border border-dashed border-[var(--border-color)] rounded-xl p-2.5"
				>
					<p class="text-[0.75rem] text-[var(--text-muted)] leading-snug m-0">
						No models loaded. Models will be loaded on demand when using a feature.
					</p>
				</div>
			{:else}
				<div class="flex flex-col gap-1.5 max-h-[180px] overflow-y-auto">
					{#each loadedModels as model (model.model_id)}
						{@const isUnloading = unloadingIds.has(model.model_id)}
						{@const rawStatus = getStatusType(model.state)}
						{@const statusType = isUnloading ? 'loading' : rawStatus}
						<div
							class="flex items-center justify-between bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-lg px-2.5 py-1.5 gap-2"
						>
							<div class="flex items-center gap-2 min-w-0">
								<span
									class="w-2 h-2 rounded-full shrink-0 {statusType === 'ready'
										? 'bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.4)] animate-pulse'
										: statusType === 'loading'
											? 'bg-amber-500 animate-pulse'
											: 'bg-red-500'}"
									title={statusType === 'ready'
										? 'Active'
										: statusType === 'loading'
											? 'Loading / Unloading...'
											: 'Error'}
								></span>
								<ModelBadge
									model={model.model_id}
									variant="inline"
									showIcon={false}
									class="truncate text-xs font-medium"
								/>
							</div>

							<button
								type="button"
								class="bg-transparent border-0 text-[var(--text-muted)] cursor-pointer p-1 rounded-md flex items-center justify-center transition-colors shrink-0 hover:bg-red-500/15 hover:text-red-500 disabled:opacity-50 disabled:cursor-not-allowed"
								disabled={isUnloading}
								onclick={() => unloadModel(model.model_id)}
								title="Unload model"
								aria-label="Unload model"
							>
								<Power size={13} />
							</button>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{:else}
		{#if loadedModels.length > 0}
			<div
				class="flex items-center justify-center py-2"
				title="{loadedModels.length} model(s) loaded"
			>
				<span
					class="w-2.5 h-2.5 rounded-full bg-emerald-500 shadow-[0_0_6px_rgba(16,185,129,0.4)] animate-pulse"
				></span>
			</div>
		{/if}
	{/if}

	<!-- Contextual Section (Middle area) -->
	{#if isExpanded}
		<div class="flex-1 min-h-0 p-3 overflow-y-auto flex flex-col">
			{#if isChatPage}
				<div class="flex flex-col gap-3">
					<div class="flex items-center justify-between px-1">
						<span
							class="text-[0.75rem] font-bold text-[var(--text-muted)] uppercase tracking-wider"
						>
							Recent conversations
						</span>
						<button
							type="button"
							class="bg-transparent border-0 text-[var(--text-muted)] cursor-pointer p-0.5 rounded flex items-center justify-center transition-colors hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]"
							onclick={createNewChat}
							title="New Chat"
							aria-label="New Chat"
						>
							<Plus size={15} />
						</button>
					</div>

					<div
						class="flex items-center gap-2 bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-lg px-2.5 py-1.5"
					>
						<Search size={14} class="text-[var(--text-muted)] shrink-0" />
						<input
							type="text"
							bind:value={searchQuery}
							placeholder="Search conversations..."
							class="w-full bg-transparent border-0 outline-none text-xs text-[var(--text-primary)] placeholder:[var(--text-muted)]"
						/>
					</div>

					<div class="flex flex-col gap-1">
						{#if filteredConversations.length === 0}
							<div class="text-[0.775rem] text-[var(--text-muted)] p-2 text-center">
								No conversations found
							</div>
						{:else}
							{#each filteredConversations as chat (chat.id)}
								<div
									class="group flex items-center relative rounded-lg transition-all duration-150 hover:bg-[var(--bg-hover)] {currentActiveChatId ===
									chat.id
										? 'bg-[var(--bg-surface-hover)]'
										: ''}"
								>
									<a
										href="/chat/{chat.id}"
										class="flex items-center px-3 py-2 text-xs text-[var(--text-secondary)] no-underline rounded-lg transition-colors flex-1 min-w-0 {currentActiveChatId ===
										chat.id
											? 'text-[var(--text-primary)] font-medium'
											: ''}"
										onclick={() => handleChatSelect(chat.id)}
									>
										<span class="truncate block w-full">{chat.title}</span>
									</a>
									<button
										type="button"
										class="bg-transparent border-0 text-[var(--text-muted)] p-1 mr-1 rounded-md cursor-pointer flex items-center justify-center opacity-100 md:opacity-0 group-hover:opacity-100 transition-opacity hover:text-red-500"
										onclick={(e) => deleteChat(chat.id, e)}
										title="Delete chat"
										aria-label="Delete chat"
									>
										<Trash2 size={13} />
									</button>
								</div>
							{/each}
						{/if}
					</div>
				</div>
			{:else if isSysInfoPage}
				<div class="flex flex-col gap-3">
					<div
						class="text-[0.75rem] font-bold text-[var(--text-muted)] uppercase tracking-wider px-1"
					>
						System Status
					</div>

					<div
						class="bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-xl p-3 flex flex-col gap-3"
					>
						<div class="flex items-center gap-2.5">
							<Cpu size={15} class="text-[var(--text-muted)] shrink-0" />
							<div class="flex flex-col">
								<span class="text-[0.7rem] text-[var(--text-muted)]">CPU Cores</span>
								<span class="text-xs font-semibold text-[var(--text-primary)]">8 Cores Active</span>
							</div>
						</div>

						<div class="flex items-center gap-2.5">
							<HardDrive size={15} class="text-[var(--text-muted)] shrink-0" />
							<div class="flex flex-col">
								<span class="text-[0.7rem] text-[var(--text-muted)]">RAM Memory</span>
								<span class="text-xs font-semibold text-[var(--text-primary)]">16.0 GB Total</span>
							</div>
						</div>
					</div>
				</div>
			{/if}
		</div>
	{/if}

	<!-- Sidebar Footer -->
	{#if isExpanded}
		<div
			class="sidebar-footer p-3 flex items-center justify-between border-t border-[var(--border-color)] shrink-0"
		>
			<span class="text-[0.725rem] text-[var(--text-muted)]">v{APP_VERSION}</span>
		</div>
	{/if}
</aside>

<style>
	.sidebar-card {
		height: calc(100vh - 1.5rem);
	}

	.sidebar-footer {
		padding-bottom: max(0.75rem, env(safe-area-inset-bottom, 0.75rem));
	}

	@media (max-width: 768px) {
		.sidebar-card {
			position: fixed;
			top: 0;
			bottom: 0;
			left: 0;
			margin: 0;
			height: 100vh;
			height: 100dvh;
			border-radius: 0;
			width: 300px !important;
			max-width: 85vw;
			transform: translateX(-100%);
		}

		.sidebar-card.mobile-open {
			transform: translateX(0);
		}
	}
</style>

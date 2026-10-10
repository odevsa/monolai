<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { chatsApi } from '$lib/api/chats';
	import { FeatureGate, Logo, ModelBadge } from '$lib/components/ds';
	import { askConfirm } from '$lib/confirmStore';
	import { checkFeature, featuresStore } from '$lib/featuresStore';
	import { chatTabs, runtimesRefreshFn, syncRecentChats } from '$lib/headerStore';
	import { t } from '$lib/i18n';
	import {
		getStatusType,
		runningModelsState,
		startRunningStatePolling,
		unloadAllModels,
		unloadModel
	} from '$lib/runningModelsStore';
	import { chatsState, imageGalleryState } from '$lib/state';
	import type { ChatConversation, GeneratedImageItem } from '$lib/types/chat';
	import { APP_VERSION } from '$lib/version';
	import {
		Activity,
		Box,
		Boxes,
		Image as ImageIcon,
		MessageSquare,
		PanelLeftClose,
		PanelLeftOpen,
		Plus,
		Power,
		Search,
		Settings,
		Trash,
		X
	} from '@lucide/svelte';
	import { onMount } from 'svelte';

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

	let loadedModels = $derived(runningModelsState.runningModels);
	let unloadingIds = $derived(runningModelsState.unloadingModelIds);
	let isChatAvailable = $derived(
		$featuresStore.isInitialized && checkFeature($featuresStore, 'chat').isSatisfied
	);

	let conversations = $derived(chatsState.conversations);

	async function deleteChat(id: string, e: MouseEvent) {
		e.stopPropagation();
		e.preventDefault();
		const confirmed = await askConfirm(
			t('chat.deleteChatConfirmMsg'),
			t('chat.deleteChatConfirmTitle'),
			'danger',
			t('common.delete'),
			t('common.cancel')
		);
		if (!confirmed) return;

		try {
			await chatsState.deleteChat(id);
			if (currentPath === `/chat/${id}`) {
				goto('/chat');
			}
		} catch (err) {
			console.error('Error deleting chat:', err);
		}
	}

	function createNewChat() {
		closeMobile();
		goto('/chat');
	}

	onMount(() => {
		chatsState.ensureLoaded();
		const stopPolling = startRunningStatePolling(2500);
		return () => {
			stopPolling();
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

	let imageSearchQuery = $state('');

	let filteredImages = $derived.by(() => {
		const q = imageSearchQuery.trim().toLowerCase();
		if (!q) return imageGalleryState.history;
		return imageGalleryState.history.filter(
			(img) =>
				img.prompt.toLowerCase().includes(q) || (img.model && img.model.toLowerCase().includes(q))
		);
	});

	function handleImageSelect(item: GeneratedImageItem) {
		imageGalleryState.selectImage(item);
		closeMobile();
	}

	async function deleteSingleImage(id: string, e: MouseEvent) {
		e.stopPropagation();
		e.preventDefault();
		const confirmed = await askConfirm(
			t('image.deleteImage') + '?',
			t('image.deleteImage'),
			'danger',
			t('common.delete'),
			t('common.cancel')
		);
		if (confirmed) {
			imageGalleryState.deleteImage(id);
		}
	}

	async function clearImageHistory() {
		const confirmed = await askConfirm(
			t('image.clearHistoryConfirmMsg'),
			t('image.clearHistoryConfirmTitle'),
			'danger',
			t('common.delete'),
			t('common.cancel')
		);
		if (confirmed) {
			imageGalleryState.clearHistory();
		}
	}

	let currentPath = $derived(page.url.pathname);
	let isChatPage = $derived(
		currentPath === '/chat' || currentPath === '/' || currentPath.startsWith('/chat')
	);
	let isImagePage = $derived(currentPath.startsWith('/image'));
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
				class="hidden md:flex items-center justify-center w-10 h-10 mx-auto bg-[var(--bg-surface)]/50 backdrop-blur-md border border-[var(--border-color)]/70 rounded-xl text-[var(--text-secondary)] cursor-pointer transition-all duration-150 hover:text-[var(--text-primary)] hover:bg-[var(--bg-surface)]/80 hover:border-[var(--border-hover)] shadow-xs"
				onclick={toggleCollapse}
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
			title={t('navigation.chat')}
		>
			<MessageSquare size={20} class="shrink-0 {isChatPage ? 'text-[var(--primary)]' : ''}" />
			{#if isExpanded}
				<span>{t('navigation.chat')}</span>
			{/if}
		</a>

		<!-- Image Action (below Chat) -->
		<a
			href="/image"
			class="flex items-center rounded-xl transition-all duration-150 box-border no-underline border-0 cursor-pointer {!isExpanded
				? 'w-10 h-10 mx-auto justify-center'
				: 'gap-3 w-full px-3.5 py-2.5 text-sm font-medium'} {isImagePage
				? 'text-[var(--primary)] bg-[var(--primary-light)] font-semibold'
				: 'text-[var(--text-secondary)] bg-transparent hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)]'}"
			onclick={closeMobile}
			title={t('navigation.image')}
		>
			<ImageIcon size={20} class="shrink-0 {isImagePage ? 'text-[var(--primary)]' : ''}" />
			{#if isExpanded}
				<span>{t('navigation.image')}</span>
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
			title={t('navigation.sysinfo')}
		>
			<Activity size={20} class="shrink-0 {isSysInfoPage ? 'text-[var(--primary)]' : ''}" />
			{#if isExpanded}
				<span>{t('navigation.sysinfo')}</span>
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
			title={t('navigation.runtimes')}
		>
			<Boxes size={20} class="shrink-0 {isRuntimesPage ? 'text-[var(--primary)]' : ''}" />
			{#if isExpanded}
				<span>{t('navigation.runtimes')}</span>
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
			title={t('navigation.models')}
		>
			<Box size={20} class="shrink-0 {isModelsPage ? 'text-[var(--primary)]' : ''}" />
			{#if isExpanded}
				<span>{t('navigation.models')}</span>
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
			title={t('navigation.settings')}
		>
			<Settings size={20} class="shrink-0" />
			{#if isExpanded}
				<span>{t('navigation.settings')}</span>
			{/if}
		</button>
	</div>

	<!-- Loaded Models Section -->
	{#if isExpanded}
		<div class="px-3.5 py-3 border-t border-[var(--border-color)] flex flex-col gap-2">
			<div class="flex items-center justify-between">
				<span class="text-[0.7rem] font-semibold uppercase tracking-wider text-[var(--text-muted)]">
					{t('navigation.runningModels')}
				</span>
				{#if loadedModels.length > 0}
					<button
						type="button"
						class="bg-transparent border-0 text-[var(--text-muted)] text-[0.7rem] cursor-pointer px-1 py-0.5 rounded transition-colors hover:text-red-500"
						onclick={unloadAllModels}
						title={t('navigation.unloadAll')}
					>
						{t('navigation.unloadAll')}
					</button>
				{/if}
			</div>

			{#if loadedModels.length === 0}
				<div
					class="bg-[var(--bg-surface)] border border-dashed border-[var(--border-color)] rounded-xl p-2.5"
				>
					<p class="text-[0.75rem] text-[var(--text-muted)] leading-snug m-0">
						{t('navigation.noRunningModels')}
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
									variant="clean"
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
				<FeatureGate features={['chat']} showCard={false}>
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
											<Trash size={13} />
										</button>
									</div>
								{/each}
							{/if}
						</div>
					</div>
				</FeatureGate>
			{:else if isImagePage}
				<FeatureGate features={['image-generation']} showCard={false}>
					<div class="flex flex-col gap-3">
						<div class="flex items-center justify-between px-1">
							<span
								class="text-[0.75rem] font-bold text-[var(--text-muted)] uppercase tracking-wider"
							>
								{t('image.sessionHistory')}
							</span>
							{#if imageGalleryState.history.length > 0}
								<button
									type="button"
									class="bg-transparent border-0 text-[var(--text-muted)] cursor-pointer p-0.5 rounded flex items-center justify-center transition-colors hover:text-red-500 hover:bg-[var(--bg-hover)]"
									onclick={clearImageHistory}
									title={t('image.clearHistory')}
									aria-label={t('image.clearHistory')}
								>
									<Trash size={13} />
								</button>
							{/if}
						</div>

						<div
							class="flex items-center gap-2 bg-[var(--bg-surface)] border border-[var(--border-color)] rounded-lg px-2.5 py-1.5"
						>
							<Search size={14} class="text-[var(--text-muted)] shrink-0" />
							<input
								type="text"
								bind:value={imageSearchQuery}
								placeholder={t('image.searchPrompts')}
								class="w-full bg-transparent border-0 outline-none text-xs text-[var(--text-primary)] placeholder:[var(--text-muted)]"
							/>
						</div>

						<div class="flex flex-col gap-1">
							{#if filteredImages.length === 0}
								<div class="text-[0.775rem] text-[var(--text-muted)] p-2 text-center">
									{imageGalleryState.history.length === 0
										? t('image.noImagesFound')
										: 'No images found'}
								</div>
							{:else}
								{#each filteredImages as item (item.id)}
									<div
										class="group flex items-center gap-2 p-1.5 rounded-xl transition-all duration-150 relative hover:bg-[var(--bg-hover)] {imageGalleryState
											.selectedImage?.id === item.id
											? 'bg-[var(--bg-surface-hover)]'
											: ''}"
									>
										<button
											type="button"
											class="flex items-center gap-2.5 flex-1 min-w-0 bg-transparent border-0 p-0 text-left cursor-pointer"
											onclick={() => handleImageSelect(item)}
											title={item.prompt}
										>
											<div
												class="w-10 h-10 rounded-lg overflow-hidden bg-[var(--bg-primary)] border border-[var(--border-color)] shrink-0"
											>
												<img
													src={item.src}
													alt={item.prompt}
													class="w-full h-full object-cover transition-transform duration-200 group-hover:scale-105"
												/>
											</div>

											<div class="flex-1 min-w-0">
												<span class="text-xs font-medium text-[var(--text-primary)] truncate block">
													{item.prompt}
												</span>
												<div
													class="flex items-center gap-1.5 text-[10px] text-[var(--text-muted)] mt-0.5"
												>
													<span>{item.size}</span>
													{#if item.model}
														<span>•</span>
														<span class="truncate max-w-[80px]">{item.model}</span>
													{/if}
												</div>
											</div>
										</button>

										<button
											type="button"
											class="bg-transparent border-0 text-[var(--text-muted)] p-1 rounded-md cursor-pointer flex items-center justify-center opacity-100 md:opacity-0 group-hover:opacity-100 transition-opacity hover:text-red-500 shrink-0"
											onclick={(e) => deleteSingleImage(item.id, e)}
											title={t('image.deleteImage')}
											aria-label={t('image.deleteImage')}
										>
											<Trash size={13} />
										</button>
									</div>
								{/each}
							{/if}
						</div>
					</div>
				</FeatureGate>
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

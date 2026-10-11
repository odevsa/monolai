<script lang="ts">
	import { page } from '$app/state';
	import {
		SidebarChatHistory,
		SidebarFooter,
		SidebarHeader,
		SidebarImageGallery,
		SidebarNav,
		SidebarRunningModels,
		SidebarSysInfo
	} from './sidebar';

	interface SidebarProps {
		collapsed: boolean;
		mobileOpen: boolean;
		activeChatId?: string;
		onOpenSettings?: () => void;
	}

	let {
		collapsed = $bindable(false),
		mobileOpen = $bindable(false),
		activeChatId = $bindable('oi-1'),
		onOpenSettings = () => {}
	}: SidebarProps = $props();

	let currentPath = $derived(page.url.pathname);
	let isChatPage = $derived(
		currentPath === '/chat' || currentPath === '/' || currentPath.startsWith('/chat')
	);
	let isImagePage = $derived(currentPath.startsWith('/image'));
	let isSysInfoPage = $derived(currentPath === '/sysinfo');
	let isExpanded = $derived(!collapsed || mobileOpen);

	function toggleCollapse() {
		collapsed = !collapsed;
	}

	function closeMobile() {
		mobileOpen = false;
	}
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
	<SidebarHeader {isExpanded} onToggleCollapse={toggleCollapse} onCloseMobile={closeMobile} />

	<SidebarNav {isExpanded} {currentPath} {onOpenSettings} onCloseMobile={closeMobile} />

	<SidebarRunningModels {isExpanded} />

	<!-- Contextual Section (Middle area) -->
	{#if isExpanded}
		<div class="flex-1 min-h-0 p-3 overflow-y-auto flex flex-col">
			{#if isChatPage}
				<SidebarChatHistory bind:activeChatId onCloseMobile={closeMobile} />
			{:else if isImagePage}
				<SidebarImageGallery onCloseMobile={closeMobile} />
			{:else if isSysInfoPage}
				<SidebarSysInfo />
			{/if}
		</div>
	{/if}

	{#if isExpanded}
		<SidebarFooter />
	{/if}
</aside>

<style>
	.sidebar-card {
		height: calc(100vh - 1.5rem);
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

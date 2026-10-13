<script lang="ts">
	import { runtimesRefreshFn } from '$lib/headerStore';
	import { t } from '$lib/i18n';
	import {
		Activity,
		Box,
		Boxes,
		Image as ImageIcon,
		MessageSquare,
		ScrollText
	} from '@lucide/svelte';
	import SidebarNavLink from './SidebarNavLink.svelte';
	import Divider from '../ui/atoms/Divider.svelte';

	interface SidebarNavProps {
		isExpanded: boolean;
		currentPath: string;
		onCloseMobile: () => void;
	}

	let { isExpanded, currentPath, onCloseMobile }: SidebarNavProps = $props();

	let isChatPage = $derived(
		currentPath === '/chat' || currentPath === '/' || currentPath.startsWith('/chat')
	);
	let isImagePage = $derived(currentPath.startsWith('/image'));
	let isSysInfoPage = $derived(currentPath === '/sysinfo');
	let isLogsPage = $derived(currentPath === '/logs');
	let isRuntimesPage = $derived(currentPath.startsWith('/runtimes'));
	let isModelsPage = $derived(currentPath.startsWith('/models'));

	function handleRuntimesClick() {
		onCloseMobile();
		if (typeof window !== 'undefined') {
			window.dispatchEvent(new CustomEvent('monolai:nav-runtimes'));
		}
		$runtimesRefreshFn?.();
	}

	function handleModelsClick() {
		onCloseMobile();
		if (typeof window !== 'undefined') {
			window.dispatchEvent(new CustomEvent('monolai:nav-models'));
		}
	}
</script>

<!-- Top Action Navigation Items -->
<div class="flex flex-col gap-1">
	<div class="flex flex-col gap-1 px-2">
		<SidebarNavLink
			href="/chat"
			label={t('navigation.chat')}
			icon={MessageSquare}
			isActive={isChatPage}
			{isExpanded}
			onclick={onCloseMobile}
		/>

		<SidebarNavLink
			href="/image"
			label={t('navigation.image')}
			icon={ImageIcon}
			isActive={isImagePage}
			{isExpanded}
			onclick={onCloseMobile}
		/>

		<SidebarNavLink
			href="/sysinfo"
			label={t('navigation.sysinfo')}
			icon={Activity}
			isActive={isSysInfoPage}
			{isExpanded}
			onclick={onCloseMobile}
		/>

		<SidebarNavLink
			href="/logs"
			label={t('navigation.logs')}
			icon={ScrollText}
			isActive={isLogsPage}
			{isExpanded}
			onclick={onCloseMobile}
		/>
	</div>

	<Divider />

	<div class="flex flex-col gap-1 px-2">
		<SidebarNavLink
			href="/runtimes"
			label={t('navigation.runtimes')}
			icon={Boxes}
			isActive={isRuntimesPage}
			{isExpanded}
			onclick={handleRuntimesClick}
		/>

		<SidebarNavLink
			href="/models"
			label={t('navigation.models')}
			icon={Box}
			isActive={isModelsPage}
			{isExpanded}
			onclick={handleModelsClick}
		/>
	</div>
</div>

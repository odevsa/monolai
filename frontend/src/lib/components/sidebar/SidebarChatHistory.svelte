<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { Button, EmptyState, FeatureGate, Input } from '$lib/components/ui';
	import { askConfirm } from '$lib/confirmStore';
	import { t } from '$lib/i18n';
	import { chatsState } from '$lib/state';
	import { Plus, Search, Trash } from '@lucide/svelte';
	import { onMount } from 'svelte';

	interface SidebarChatHistoryProps {
		activeChatId?: string;
		onCloseMobile?: () => void;
	}

	let { activeChatId = $bindable(''), onCloseMobile = () => {} }: SidebarChatHistoryProps =
		$props();

	let searchQuery = $state('');

	let conversations = $derived(chatsState.conversations);
	let currentPath = $derived(page.url.pathname);
	let currentActiveChatId = $derived(
		currentPath.startsWith('/chat/') ? currentPath.replace('/chat/', '') : ''
	);

	let filteredConversations = $derived(
		conversations.filter((c) => c.title.toLowerCase().includes(searchQuery.toLowerCase().trim()))
	);

	onMount(() => {
		chatsState.ensureLoaded();
	});

	function createNewChat() {
		onCloseMobile();
		goto('/chat');
	}

	function handleChatSelect(id: string) {
		activeChatId = id;
		onCloseMobile();
	}

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
</script>

{#snippet searchPrefix()}
	<Search size={14} />
{/snippet}

<FeatureGate features={['chat']} showCard={false}>
	<div class="flex flex-col gap-3">
		<div class="flex items-center justify-between px-1">
			<span class="text-[0.75rem] font-bold text-[var(--text-muted)] uppercase tracking-wider">
				{t('navigation.recentConversations')}
			</span>
			<Button
				variant="ghost"
				size="icon-sm"
				class="text-[var(--text-muted)] hover:text-[var(--text-primary)]"
				onclick={createNewChat}
				title={t('common.new')}
				ariaLabel={t('common.new')}
			>
				<Plus size={15} />
			</Button>
		</div>

		<Input
			size="sm"
			variant="surface"
			bind:value={searchQuery}
			placeholder={t('navigation.searchConversations')}
			prefix={searchPrefix}
		/>

		<div class="flex flex-col gap-1">
			{#if filteredConversations.length === 0}
				<EmptyState size="sm" title={t('navigation.noConversationsFound')} />
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
						<Button
							variant="ghost"
							size="icon-sm"
							class="mr-1 text-[var(--text-muted)] opacity-100 md:opacity-0 group-hover:opacity-100 hover:text-[var(--danger-text)]"
							onclick={(e) => deleteChat(chat.id, e)}
							title={t('common.delete')}
							ariaLabel={t('common.delete')}
						>
							<Trash size={13} />
						</Button>
					</div>
				{/each}
			{/if}
		</div>
	</div>
</FeatureGate>

<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { FeatureGate } from '$lib/components/ui';
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

<FeatureGate features={['chat']} showCard={false}>
	<div class="flex flex-col gap-3">
		<div class="flex items-center justify-between px-1">
			<span class="text-[0.75rem] font-bold text-[var(--text-muted)] uppercase tracking-wider">
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

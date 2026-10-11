import { chatsApi } from '$lib/api/chats';
import { headerState } from './header.svelte';
import type { ChatConversation } from '$lib/types/chat';

class ChatsStateManager {
	conversations = $state<ChatConversation[]>([]);
	isLoading = $state<boolean>(false);

	private channel: BroadcastChannel | null = null;
	private initialized = false;

	constructor() {
		if (typeof window !== 'undefined' && 'BroadcastChannel' in window) {
			this.channel = new BroadcastChannel('monolai:chats');
			this.channel.onmessage = (event) => {
				if (event.data?.type === 'chats-updated') {
					this.fetchConversations(event.data?.activeId);
				}
			};
		}
	}

	async fetchConversations(activeId?: string): Promise<ChatConversation[]> {
		this.isLoading = true;
		try {
			const data = await chatsApi.list();
			this.conversations = data || [];
			headerState.syncRecentChats(this.conversations, activeId);
			this.initialized = true;
			return this.conversations;
		} catch (e) {
			console.error('Error fetching chats:', e);
			return this.conversations;
		} finally {
			this.isLoading = false;
		}
	}

	async ensureLoaded(activeId?: string): Promise<void> {
		if (!this.initialized) {
			await this.fetchConversations(activeId);
		}
	}

	notifyUpdated(activeId?: string) {
		this.fetchConversations(activeId);
		if (this.channel) {
			this.channel.postMessage({ type: 'chats-updated', activeId });
		}
		if (typeof window !== 'undefined') {
			window.dispatchEvent(new CustomEvent('monolai:chats-updated', { detail: { activeId } }));
		}
	}

	async deleteChat(id: string): Promise<void> {
		await chatsApi.delete(id);
		this.conversations = this.conversations.filter((c) => c.id !== id);
		headerState.chatTabs = headerState.chatTabs.filter((t) => t.id !== id);
		this.notifyUpdated();
	}
}

export const chatsState = new ChatsStateManager();

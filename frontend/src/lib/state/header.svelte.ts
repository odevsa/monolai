import { goto } from '$app/navigation';

export interface ChatTab {
	id: string;
	title: string;
	active: boolean;
}

class HeaderStateManager {
	chatTabs = $state<ChatTab[]>([]);
	sysinfoConnected = $state<boolean>(false);
	sysinfoReconnectFn = $state<(() => void) | null>(null);
	runtimesRefreshing = $state<boolean>(false);
	runtimesRefreshFn = $state<(() => void) | null>(null);
	modelsRefreshing = $state<boolean>(false);
	modelsRefreshFn = $state<(() => void) | null>(null);

	syncRecentChats(chats: { id: string; title: string }[], activeId?: string) {
		const currentActiveId = activeId || this.chatTabs.find((t) => t.active)?.id;
		const top3 = chats.slice(0, 3);
		let updated: ChatTab[] = top3.map((c) => ({
			id: c.id,
			title: c.title,
			active: c.id === currentActiveId
		}));

		if (currentActiveId && !updated.some((t) => t.id === currentActiveId)) {
			const found = chats.find((c) => c.id === currentActiveId);
			if (found) {
				updated = [{ id: found.id, title: found.title, active: true }, ...updated].slice(0, 3);
			}
		}

		this.chatTabs = updated;
	}

	addChatTab() {
		if (typeof window !== 'undefined') {
			goto('/chat');
		}
	}

	selectChatTab(id: string) {
		this.chatTabs = this.chatTabs.map((t) => ({ ...t, active: t.id === id }));
		if (typeof window !== 'undefined') {
			goto(`/chat/${id}`);
		}
	}

	closeChatTab(id: string) {
		const idx = this.chatTabs.findIndex((t) => t.id === id);
		const wasActive = this.chatTabs[idx]?.active;
		const newTabs = this.chatTabs.filter((t) => t.id !== id);

		if (wasActive) {
			if (newTabs.length > 0) {
				const targetIdx = Math.max(0, idx - 1);
				newTabs[targetIdx].active = true;
				if (typeof window !== 'undefined') {
					goto(`/chat/${newTabs[targetIdx].id}`);
				}
			} else if (typeof window !== 'undefined') {
				goto('/chat');
			}
		}
		this.chatTabs = newTabs;
	}

	setSysinfoStatus(connected: boolean, reconnectFn?: () => void) {
		this.sysinfoConnected = connected;
		if (reconnectFn) {
			this.sysinfoReconnectFn = reconnectFn;
		}
	}

	setRuntimesRefreshing(refreshing: boolean, refreshFn?: () => void) {
		this.runtimesRefreshing = refreshing;
		if (refreshFn) {
			this.runtimesRefreshFn = refreshFn;
		}
	}

	setModelsRefreshing(refreshing: boolean, refreshFn?: () => void) {
		this.modelsRefreshing = refreshing;
		if (refreshFn) {
			this.modelsRefreshFn = refreshFn;
		}
	}
}

export const headerState = new HeaderStateManager();
export const selectChatTab = (id: string) => headerState.selectChatTab(id);
export const closeChatTab = (id: string) => headerState.closeChatTab(id);
export const addChatTab = () => headerState.addChatTab();

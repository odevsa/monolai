import { writable } from 'svelte/store';
import { goto } from '$app/navigation';

export interface ChatTab {
	id: string;
	title: string;
	active: boolean;
}

export const chatTabs = writable<ChatTab[]>([]);

export const sysinfoConnected = writable<boolean>(false);
export const sysinfoReconnectFn = writable<(() => void) | null>(null);

export const runtimesRefreshing = writable<boolean>(false);
export const runtimesRefreshFn = writable<(() => void) | null>(null);

export function syncRecentChats(chats: { id: string; title: string }[], activeId?: string) {
	chatTabs.update((prev) => {
		const currentActiveId = activeId || prev.find((t) => t.active)?.id;
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

		return updated;
	});
}

export function addChatTab() {
	if (typeof window !== 'undefined') {
		goto('/chat');
	}
}

export function selectChatTab(id: string) {
	chatTabs.update((tabs) => tabs.map((t) => ({ ...t, active: t.id === id })));
	if (typeof window !== 'undefined') {
		goto(`/chat/${id}`);
	}
}

export function closeChatTab(id: string) {
	chatTabs.update((tabs) => {
		const idx = tabs.findIndex((t) => t.id === id);
		const wasActive = tabs[idx]?.active;
		const newTabs = tabs.filter((t) => t.id !== id);
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
		return newTabs;
	});
}

export function setSysinfoStatus(connected: boolean, reconnectFn?: () => void) {
	sysinfoConnected.set(connected);
	if (reconnectFn) {
		sysinfoReconnectFn.set(reconnectFn);
	}
}

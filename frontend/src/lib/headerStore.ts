import { headerState, type ChatTab } from '$lib/state/header.svelte';

export type { ChatTab };

export const chatTabs = {
	subscribe(fn: (val: ChatTab[]) => void) {
		fn(headerState.chatTabs);
		return () => {};
	},
	update(fn: (prev: ChatTab[]) => ChatTab[]) {
		headerState.chatTabs = fn(headerState.chatTabs);
	},
	set(val: ChatTab[]) {
		headerState.chatTabs = val;
	}
};

export const sysinfoConnected = {
	subscribe(fn: (val: boolean) => void) {
		fn(headerState.sysinfoConnected);
		return () => {};
	},
	set(val: boolean) {
		headerState.sysinfoConnected = val;
	}
};

export const sysinfoReconnectFn = {
	subscribe(fn: (val: (() => void) | null) => void) {
		fn(headerState.sysinfoReconnectFn);
		return () => {};
	},
	set(val: (() => void) | null) {
		headerState.sysinfoReconnectFn = val;
	}
};

export const runtimesRefreshing = {
	subscribe(fn: (val: boolean) => void) {
		fn(headerState.runtimesRefreshing);
		return () => {};
	},
	set(val: boolean) {
		headerState.runtimesRefreshing = val;
	}
};

export const runtimesRefreshFn = {
	subscribe(fn: (val: (() => void) | null) => void) {
		fn(headerState.runtimesRefreshFn);
		return () => {};
	},
	set(val: (() => void) | null) {
		headerState.runtimesRefreshFn = val;
	}
};

export const modelsRefreshing = {
	subscribe(fn: (val: boolean) => void) {
		fn(headerState.modelsRefreshing);
		return () => {};
	},
	set(val: boolean) {
		headerState.modelsRefreshing = val;
	}
};

export const modelsRefreshFn = {
	subscribe(fn: (val: (() => void) | null) => void) {
		fn(headerState.modelsRefreshFn);
		return () => {};
	},
	set(val: (() => void) | null) {
		headerState.modelsRefreshFn = val;
	}
};

export const syncRecentChats = (chats: { id: string; title: string }[], activeId?: string) =>
	headerState.syncRecentChats(chats, activeId);

export const addChatTab = () => headerState.addChatTab();
export const selectChatTab = (id: string) => headerState.selectChatTab(id);
export const closeChatTab = (id: string) => headerState.closeChatTab(id);
export const setSysinfoStatus = (connected: boolean, reconnectFn?: () => void) =>
	headerState.setSysinfoStatus(connected, reconnectFn);

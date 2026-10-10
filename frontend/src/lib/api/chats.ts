import { api } from './client';
import type { ChatConversation, ChatMessage } from '$lib/types/chat';

export const chatsApi = {
	list: () => api.get<ChatConversation[]>('/api/chats'),

	create: (titleOrPayload?: string | { id?: string; title?: string }) => {
		const payload =
			typeof titleOrPayload === 'string' ? { title: titleOrPayload } : titleOrPayload || {};
		return api.post<ChatConversation>('/api/chats', payload);
	},

	update: (id: string, title: string) =>
		api.put<ChatConversation>(`/api/chats/${encodeURIComponent(id)}`, { title }),

	delete: (id: string) => api.delete<void>(`/api/chats/${encodeURIComponent(id)}`),

	getMessages: (chatId: string) =>
		api.get<ChatMessage[]>(`/api/chats/${encodeURIComponent(chatId)}/messages`),

	createMessage: (chatId: string, message: Partial<ChatMessage>) =>
		api.post<ChatMessage>(`/api/chats/${encodeURIComponent(chatId)}/messages`, message),

	clearMessages: (chatId: string) =>
		api.delete<void>(`/api/chats/${encodeURIComponent(chatId)}/messages`),

	updateMessage: (chatId: string, msgId: string, message: Partial<ChatMessage>) =>
		api.put<ChatMessage>(
			`/api/chats/${encodeURIComponent(chatId)}/messages/${encodeURIComponent(msgId)}`,
			message
		),

	deleteMessage: (chatId: string, msgId: string) =>
		api.delete<void>(
			`/api/chats/${encodeURIComponent(chatId)}/messages/${encodeURIComponent(msgId)}`
		)
};

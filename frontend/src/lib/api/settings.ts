import { api } from './client';

export const settingsApi = {
	getAll: () => api.get<Record<string, string>>('/api/settings'),

	updateAll: (settings: Record<string, string>) =>
		api.post<{ success: boolean }>('/api/settings', settings),

	get: (key: string) =>
		api.get<{ key: string; value: string }>(`/api/settings/${encodeURIComponent(key)}`),

	delete: (key: string) => api.delete<void>(`/api/settings/${encodeURIComponent(key)}`)
};

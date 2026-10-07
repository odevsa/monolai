import { api } from './client';
import type { ConfigStatus, SetupConfigPayload } from '$lib/types/config';

export const configApi = {
	getStatus: () => api.get<ConfigStatus>('/api/config/status'),

	setup: (payload: SetupConfigPayload) =>
		api.post<{ success: boolean; message?: string }>('/api/config/setup', payload)
};

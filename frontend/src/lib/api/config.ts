import { api } from './client';
import type { ConfigStatus, SetupConfigPayload } from '$lib/types/config';

let cachedStatus: ConfigStatus | null = null;

export const configApi = {
	getStatus: async (force = false): Promise<ConfigStatus> => {
		if (!force && cachedStatus) return cachedStatus;
		const res = await api.get<ConfigStatus>('/api/config/status');
		cachedStatus = res;
		return res;
	},

	getCachedStatus: (): ConfigStatus | null => cachedStatus,

	setup: async (payload: SetupConfigPayload) => {
		const res = await api.post<{ success: boolean; message?: string }>(
			'/api/config/setup',
			payload
		);
		cachedStatus = null;
		return res;
	}
};

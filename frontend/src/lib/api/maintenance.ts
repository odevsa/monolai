import { api } from './client';
import type { MaintenanceResult } from '$lib/types/maintenance';

export const maintenanceApi = {
	cleanDatabase: async (): Promise<MaintenanceResult> => {
		return api.post<MaintenanceResult>('/api/maintenance/clean-database');
	},

	cleanStorage: async (): Promise<MaintenanceResult> => {
		return api.post<MaintenanceResult>('/api/maintenance/clean-storage');
	},

	factoryReset: async (): Promise<MaintenanceResult> => {
		return api.post<MaintenanceResult>('/api/maintenance/factory-reset');
	}
};

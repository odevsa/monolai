import { api } from './client';
import type {
	RepositoryRecord,
	CreateRepositoryPayload,
	UpdateRepositoryPayload,
	HubModelItem
} from '$lib/types/models';

export const hubApi = {
	getRepositories: () => api.get<RepositoryRecord[]>('/api/hub/repositories'),

	createRepository: (payload: CreateRepositoryPayload) =>
		api.post<RepositoryRecord>('/api/hub/repositories', payload),

	updateRepository: (id: string, payload: UpdateRepositoryPayload) =>
		api.put<RepositoryRecord>(`/api/hub/repositories/${encodeURIComponent(id)}`, payload),

	deleteRepository: (id: string) =>
		api.delete<void>(`/api/hub/repositories/${encodeURIComponent(id)}`),

	searchModels: (params: {
		repo_id?: string;
		query?: string;
		modality?: string;
		sort?: string;
		page?: number;
	}) => api.get<HubModelItem[]>('/api/hub/models', params),

	getModelDetails: (repoId: string, modelId: string) =>
		api.get<HubModelItem>(`/api/hub/models/${encodeURIComponent(repoId)}/${modelId}`)
};

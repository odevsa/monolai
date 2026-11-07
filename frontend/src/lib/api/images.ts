import { api } from './client';
import type { CreateGeneratedImagePayload, GeneratedImageItem } from '$lib/types/images';

export const imagesApi = {
	list: (limit = 100, offset = 0) =>
		api.get<GeneratedImageItem[]>(`/api/images?limit=${limit}&offset=${offset}`),

	get: (id: string) => api.get<GeneratedImageItem>(`/api/images/${encodeURIComponent(id)}`),

	create: (payload: CreateGeneratedImagePayload) =>
		api.post<GeneratedImageItem>('/api/images', payload),

	delete: (id: string) => api.delete<void>(`/api/images/${encodeURIComponent(id)}`),

	clear: () => api.delete<void>('/api/images')
};

import { api } from './client';
import type { AudioGenerationItem, CreateAudioGenerationPayload } from '$lib/types/audio';

export const audioApi = {
	list: (type?: 'tts' | 'asr', limit = 100, offset = 0) => {
		const query = new URLSearchParams({ limit: String(limit), offset: String(offset) });
		if (type) query.set('type', type);
		return api.get<AudioGenerationItem[]>(`/api/audio?${query.toString()}`);
	},

	get: (id: string) => api.get<AudioGenerationItem>(`/api/audio/${encodeURIComponent(id)}`),

	create: (payload: CreateAudioGenerationPayload) =>
		api.post<AudioGenerationItem>('/api/audio', payload),

	delete: (id: string) => api.delete<void>(`/api/audio/${encodeURIComponent(id)}`),

	clear: () => api.delete<void>('/api/audio')
};

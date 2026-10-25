import { api } from './client';
import type { DownloadRecord, EnqueueDownloadPayload } from '$lib/types/models';

export const downloadsApi = {
	list: () => api.get<DownloadRecord[]>('/api/downloads'),

	enqueue: (payload: EnqueueDownloadPayload) => api.post<DownloadRecord>('/api/downloads', payload),

	pause: (id: string) => api.post<void>(`/api/downloads/${encodeURIComponent(id)}/pause`),

	resume: (id: string) => api.post<void>(`/api/downloads/${encodeURIComponent(id)}/resume`),

	cancel: (id: string) => api.post<void>(`/api/downloads/${encodeURIComponent(id)}/cancel`),

	delete: (id: string) => api.delete<void>(`/api/downloads/${encodeURIComponent(id)}`),

	clearInactive: () => api.post<void>('/api/downloads/clear'),

	subscribe: (
		onUpdate: (downloads: DownloadRecord[]) => void,
		onError?: (err: Event) => void
	): (() => void) => {
		const es = new EventSource('/api/downloads/stream');

		es.onmessage = (event) => {
			try {
				const data = JSON.parse(event.data);
				onUpdate(data);
			} catch (e) {
				console.error('Error parsing downloads SSE stream:', e);
			}
		};

		if (onError) {
			es.onerror = onError;
		}

		return () => es.close();
	}
};

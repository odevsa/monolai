import { api } from './client';
import type { LogEntry, LogFilterParams } from '$lib/types/logs';

export const logsApi = {
	getLogs: (params?: LogFilterParams) => {
		const searchParams = new URLSearchParams();
		if (params?.service && params.service !== 'all') {
			searchParams.set('service', params.service);
		}
		if (params?.level && params.level !== 'ALL') {
			searchParams.set('level', params.level);
		}
		if (params?.search) {
			searchParams.set('search', params.search);
		}
		if (params?.limit) {
			searchParams.set('limit', params.limit.toString());
		}
		const query = searchParams.toString();
		return api.get<LogEntry[]>(`/api/logs${query ? `?${query}` : ''}`);
	},

	getServices: () => api.get<string[]>('/api/logs/services'),

	clearLogs: (service?: string) => {
		const query = service && service !== 'all' ? `?service=${encodeURIComponent(service)}` : '';
		return api.delete<{ success: boolean; message: string }>(`/api/logs${query}`);
	},

	getDownloadUrl: (service?: string) => {
		const query = service && service !== 'all' ? `?service=${encodeURIComponent(service)}` : '';
		return `/api/logs/download${query}`;
	},

	subscribeStream: (
		onEntry: (entry: LogEntry) => void,
		onError?: (err: Event) => void
	): (() => void) => {
		const es = new EventSource('/api/logs/stream');

		es.onmessage = (event) => {
			try {
				const data = JSON.parse(event.data);
				onEntry(data);
			} catch (e) {
				console.error('Error parsing log entry SSE:', e);
			}
		};

		if (onError) {
			es.onerror = onError;
		}

		return () => es.close();
	}
};

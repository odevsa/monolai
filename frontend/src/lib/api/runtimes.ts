import { api } from './client';
import type { Runtime, RuntimeManifest, InstallProgress } from '$lib/types/runtimes';

export const runtimesApi = {
	list: () => api.get<Runtime[]>('/api/runtimes'),

	getManifests: () => api.get<RuntimeManifest[]>('/api/runtime-manifests'),

	getManifestById: (id: string) =>
		api.get<RuntimeManifest>(`/api/runtime-manifests/${encodeURIComponent(id)}`),

	install: (id: string, acceleration?: string) =>
		api.post<{ status: string }>(`/api/runtimes/${encodeURIComponent(id)}/install`, {
			acceleration
		}),

	uninstall: (id: string) => api.delete<void>(`/api/runtimes/${encodeURIComponent(id)}`),

	connectInstallStream: (
		id: string,
		onProgress: (progress: InstallProgress) => void,
		onError?: (err: Event) => void
	): (() => void) => {
		const es = new EventSource(`/api/runtimes/${encodeURIComponent(id)}/install/stream`);

		es.onmessage = (event) => {
			try {
				const data = JSON.parse(event.data);
				onProgress(data);
				if (data.status === 'completed' || data.status === 'error') {
					es.close();
				}
			} catch (e) {
				console.error('Error parsing runtime install SSE stream:', e);
			}
		};

		es.onerror = (err) => {
			if (onError) onError(err);
			es.close();
		};

		return () => es.close();
	}
};

import { api } from './client';
import { listModels } from './openai';
import { runtimesApi } from './runtimes';
import type {
	ModelRecord,
	ModelFileItem,
	RunningModelStatus,
	CreateModelPayload,
	UpdateModelPayload
} from '$lib/types/models';

export interface FeatureModelItem {
	id: string;
	runtime: string;
	flags: string;
}

// Backward-compatibility aliases
export type OpenAIChatModelItem = FeatureModelItem;
export type OpenAIImageModelItem = FeatureModelItem;

/**
 * Retrieves models available via the official OpenAI SDK (GET /v1/models),
 * optionally filtering by required runtime features (e.g. ['chat'], ['image-generation']).
 * If features is empty or undefined, returns all models.
 */
export async function getFeatureModels(features: string[] = []): Promise<FeatureModelItem[]> {
	const [openaiModels, backendModels, manifests] = await Promise.all([
		listModels().catch(() => []),
		modelsApi.list(false).catch(() => []),
		runtimesApi.getManifests().catch(() => [])
	]);

	const candidates: FeatureModelItem[] = (openaiModels || []).map((om) => {
		const match = (backendModels || []).find((bm) => bm.id === om.id);
		return {
			id: om.id,
			runtime: match?.runtime || 'llama-cpp',
			flags: match?.flags || ''
		};
	});

	if (!features || features.length === 0) {
		return candidates;
	}

	const normalizedFeatures = features.map((f) => f.toLowerCase().trim());

	return candidates.filter((m) => {
		const manifest = (manifests || []).find(
			(man) => man.id.toLowerCase() === m.runtime.toLowerCase()
		);
		const manifestFeatures = (manifest?.features || []).map((f) => f.toLowerCase().trim());

		// Check if any requested feature is present in the runtime manifest
		const hasManifestFeature = normalizedFeatures.some((target) =>
			manifestFeatures.includes(target)
		);
		if (hasManifestFeature) return true;

		// Fallbacks for known runtime identifiers
		const rtLower = m.runtime.toLowerCase();
		if (
			normalizedFeatures.includes('chat') &&
			(rtLower.includes('chat') || rtLower === 'llama-cpp')
		) {
			return true;
		}
		if (
			(normalizedFeatures.includes('image-generation') || normalizedFeatures.includes('image')) &&
			(rtLower === 'sd-cpp' || rtLower.includes('diffusion'))
		) {
			return true;
		}

		return false;
	});
}

export const modelsApi = {
	list: (all: boolean = true) => api.get<ModelRecord[]>('/api/models', { all }),

	getAvailableFiles: () => api.get<ModelFileItem[]>('/api/models/available'),

	getRunningState: () => api.get<RunningModelStatus[]>('/api/state'),

	subscribeRunningState: (
		onUpdate: (models: RunningModelStatus[]) => void,
		onError?: (err: Event) => void
	): (() => void) => {
		const es = new EventSource('/api/state/stream');

		es.onmessage = (event) => {
			try {
				const data = JSON.parse(event.data);
				onUpdate(data);
			} catch (e) {
				console.error('Error parsing running models SSE stream:', e);
			}
		};

		if (onError) {
			es.onerror = onError;
		}

		return () => es.close();
	},

	create: (payload: CreateModelPayload) => api.post<ModelRecord>('/api/models', payload),

	update: (id: string, payload: UpdateModelPayload) =>
		api.put<ModelRecord>(`/api/models/${encodeURIComponent(id)}`, payload),

	delete: (id: string) => api.delete<void>(`/api/models/${encodeURIComponent(id)}`),

	load: (id: string) => api.post<{ status: string }>(`/api/models/${encodeURIComponent(id)}/load`),

	unload: (id: string) =>
		api.post<{ status: string }>(`/api/models/${encodeURIComponent(id)}/unload`),

	swap: (id: string) => api.post<{ status: string }>(`/api/models/${encodeURIComponent(id)}/swap`),

	unloadAll: () => api.post<{ status: string }>('/api/models/unload-all'),

	getFeatureModels,

	getChatModels: (features: string[] = ['chat']) => getFeatureModels(features),

	getImageModels: (features: string[] = ['image-generation']) => getFeatureModels(features)
};

import { api } from './client';
import { listModels } from './openai';
import { runtimesApi } from './runtimes';
import { formatCanonicalModelString } from '$lib/utils/model';
import type {
	ModelRecord,
	ModelPreset,
	ModelFileItem,
	RunningModelStatus,
	CreatePresetPayload,
	UpdatePresetPayload
} from '$lib/types/models';
import type { RuntimeManifest } from '$lib/types/runtimes';

export interface FeatureModelItem {
	id: string;
	runtime: string;
	flags: string;
}

export type OpenAIChatModelItem = FeatureModelItem;
export type OpenAIImageModelItem = FeatureModelItem;

/**
 * Retrieves models available via the official OpenAI SDK (GET /v1/models),
 * optionally filtering by required runtime features (e.g. ['chat'], ['image-generation']).
 */
export async function getFeatureModels(features: string[] = []): Promise<FeatureModelItem[]> {
	const [openaiModels, backendModels, manifests] = await Promise.all([
		listModels().catch(() => []),
		modelsApi.list(false).catch(() => []),
		runtimesApi.getManifests().catch(() => [])
	]);

	const candidates: FeatureModelItem[] = (openaiModels || []).map((om) => {
		let matchedRuntime = 'llama-cpp';
		let matchedFlags = '';

		// Match direct model ID, preset ID, or canonical string
		const directMatch = (backendModels || []).find((bm) => {
			if (bm.id === om.id) return true;
			const canonical = formatCanonicalModelString(bm);
			return canonical === om.id || bm.relative_path === om.id;
		});

		if (directMatch) {
			const def = directMatch.presets?.find((p) => p.is_default) || directMatch.presets?.[0];
			matchedRuntime = def?.runtime_id || directMatch.runtime || 'llama-cpp';
			const rawFlags = def?.flags ?? directMatch.flags;
			matchedFlags = typeof rawFlags === 'string' ? rawFlags : JSON.stringify(rawFlags || {});
		} else {
			// Check if om.id is in format model_id:preset_name
			for (const bm of backendModels || []) {
				const presetMatch = bm.presets?.find(
					(p) => p.id === om.id || `${bm.id}:${p.name.toLowerCase().replace(/\s+/g, '-')}` === om.id
				);
				if (presetMatch) {
					matchedRuntime = presetMatch.runtime_id;
					const rawFlags = presetMatch.flags;
					matchedFlags = typeof rawFlags === 'string' ? rawFlags : JSON.stringify(rawFlags || {});
					break;
				}
			}
		}

		const canonicalId = directMatch
			? formatCanonicalModelString(directMatch)
			: formatCanonicalModelString(om.id);

		return {
			id: canonicalId || om.id,
			runtime: matchedRuntime,
			flags: matchedFlags
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

		const hasManifestFeature = normalizedFeatures.some((target) =>
			manifestFeatures.includes(target)
		);
		if (hasManifestFeature) return true;

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
	list: async (all: boolean = true, sync: boolean = false) => {
		const models = await api.get<ModelRecord[]>('/api/models', { all, sync });
		return models.map((m) => {
			const defaultPreset = m.presets?.find((p) => p.is_default) || m.presets?.[0];
			const rawFlags = defaultPreset?.flags ?? m.flags;
			return {
				...m,
				runtime: defaultPreset?.runtime_id || m.runtime || 'llama-cpp',
				flags: typeof rawFlags === 'string' ? rawFlags : JSON.stringify(rawFlags || {})
			};
		});
	},

	getById: (id: string) => api.get<ModelRecord>(`/api/models/${encodeURIComponent(id)}`),

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

	delete: (id: string, deleteFile: boolean = false) =>
		api.delete<void>(`/api/models/${encodeURIComponent(id)}`, { delete_file: deleteFile }),

	// Presets Management
	getPresets: (modelId: string) =>
		api.get<ModelPreset[]>(`/api/models/${encodeURIComponent(modelId)}/presets`),

	createPreset: (modelId: string, payload: CreatePresetPayload) =>
		api.post<ModelPreset>(`/api/models/${encodeURIComponent(modelId)}/presets`, payload),

	updatePreset: (presetId: string, payload: UpdatePresetPayload) =>
		api.put<ModelPreset>(`/api/models/presets/${encodeURIComponent(presetId)}`, payload),

	deletePreset: (presetId: string) =>
		api.delete<void>(`/api/models/presets/${encodeURIComponent(presetId)}`),

	setDefaultPreset: (modelId: string, presetId: string) =>
		api.post<void>(
			`/api/models/${encodeURIComponent(modelId)}/presets/${encodeURIComponent(presetId)}/set-default`
		),

	// Compatible runtimes query
	getCompatibleRuntimes: (
		modalityOrParams?: string | { modality?: string; format?: string },
		format?: string
	) => {
		const params =
			typeof modalityOrParams === 'string'
				? { modality: modalityOrParams, format }
				: modalityOrParams || {};
		return api.get<RuntimeManifest[]>('/api/runtimes/compatible', params);
	},

	load: (id: string) => api.post<{ status: string }>(`/api/models/${encodeURIComponent(id)}/load`),

	unload: (id: string) =>
		api.post<{ status: string }>(`/api/models/${encodeURIComponent(id)}/unload`),

	swap: (id: string) => api.post<{ status: string }>(`/api/models/${encodeURIComponent(id)}/swap`),

	unloadAll: () => api.post<{ status: string }>('/api/models/unload-all'),

	getFeatureModels,

	getChatModels: (features: string[] = ['chat']) => getFeatureModels(features),

	getImageModels: (features: string[] = ['image-generation']) => getFeatureModels(features)
};

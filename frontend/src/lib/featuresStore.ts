import { writable, get } from 'svelte/store';

export interface RuntimeItem {
	id: string;
	name: string;
	features: string[];
	is_installed: boolean;
	binary_path?: string | null;
}

export interface ModelRecord {
	id: string;
	runtime: string;
	flags?: string;
	file_exists?: boolean;
}

export interface FeaturesState {
	isLoading: boolean;
	isInitialized: boolean;
	runtimes: RuntimeItem[];
	models: ModelRecord[];
	lastUpdated: number;
}

const initialState: FeaturesState = {
	isLoading: false,
	isInitialized: false,
	runtimes: [],
	models: [],
	lastUpdated: 0
};

export const featuresStore = writable<FeaturesState>(initialState);

let activeFetchPromise: Promise<FeaturesState> | null = null;

/**
 * Refreshes the global features state (runtimes and models).
 * Deduplicates concurrent calls and caches results to avoid server load.
 */
export async function refreshFeatures(force = false): Promise<FeaturesState> {
	const current = get(featuresStore);
	const now = Date.now();

	// Return cached data if initialized and recent (< 3s)
	if (!force && current.isInitialized && now - current.lastUpdated < 3000) {
		return current;
	}

	if (activeFetchPromise) {
		return activeFetchPromise;
	}

	featuresStore.update((s) => ({ ...s, isLoading: true }));

	activeFetchPromise = Promise.all([
		fetch('/api/runtimes').then((r) => (r.ok ? r.json() : [])),
		fetch('/api/models?all=true').then((r) => (r.ok ? r.json() : []))
	])
		.then(([runtimes, models]: [RuntimeItem[], ModelRecord[]]) => {
			const updated: FeaturesState = {
				isLoading: false,
				isInitialized: true,
				runtimes: Array.isArray(runtimes) ? runtimes : [],
				models: Array.isArray(models) ? models : [],
				lastUpdated: Date.now()
			};
			featuresStore.set(updated);
			return updated;
		})
		.catch((err) => {
			console.error('Failed to load system features:', err);
			featuresStore.update((s) => ({ ...s, isLoading: false }));
			return get(featuresStore);
		})
		.finally(() => {
			activeFetchPromise = null;
		});

	return activeFetchPromise;
}

export interface FeatureCheckResult {
	isSatisfied: boolean;
	missingRuntime: boolean;
	missingModel: boolean;
	candidateNames: string;
	label: string;
}

/**
 * Pure evaluation function to check if a feature has both installed runtime and registered model.
 */
export function checkFeature(
	state: FeaturesState,
	feature: string
): FeatureCheckResult {
	const featLower = feature.toLowerCase().trim();
	const candidateIds = new Set<string>();
	const candidateTitles: string[] = [];

	for (const r of state.runtimes) {
		if (
			r.features &&
			Array.isArray(r.features) &&
			r.features.some((f) => f.toLowerCase() === featLower)
		) {
			candidateIds.add(r.id.toLowerCase());
			if (!candidateTitles.includes(r.name || r.id)) {
				candidateTitles.push(r.name || r.id);
			}
		}
	}

	// Fallback mappings for standard monolai features
	if (candidateIds.size === 0) {
		if (featLower === 'chat') {
			candidateIds.add('llama-cpp');
			candidateTitles.push('LLaMA C++ Server');
		} else if (featLower === 'image-generation' || featLower === 'image') {
			candidateIds.add('sd-cpp');
			candidateTitles.push('Stable Diffusion C++');
		}
	}

	const hasInstalledRuntime = state.runtimes.some(
		(r) => r.is_installed && candidateIds.has(r.id.toLowerCase())
	);

	const hasRegisteredModel = state.models.some((m) =>
		candidateIds.has((m.runtime || '').toLowerCase())
	);

	return {
		isSatisfied: hasInstalledRuntime && hasRegisteredModel,
		missingRuntime: !hasInstalledRuntime,
		missingModel: !hasRegisteredModel,
		candidateNames: candidateTitles.join(' / '),
		label: feature
	};
}

/**
 * Synchronous check against the current store state.
 */
export function isFeatureAvailable(feature: string): boolean {
	const state = get(featuresStore);
	if (!state.isInitialized) return false;
	return checkFeature(state, feature).isSatisfied;
}

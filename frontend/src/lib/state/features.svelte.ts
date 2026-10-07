import { runtimesApi } from '$lib/api/runtimes';
import { modelsApi } from '$lib/api/models';
import type { Runtime } from '$lib/types/runtimes';
import type { ModelRecord } from '$lib/types/models';

export interface FeaturesState {
	isLoading: boolean;
	isInitialized: boolean;
	runtimes: Runtime[];
	models: ModelRecord[];
	lastUpdated: number;
}

export interface FeatureCheckResult {
	isSatisfied: boolean;
	missingRuntime: boolean;
	missingModel: boolean;
	candidateNames: string;
	label: string;
}

const listeners = new Set<() => void>();

export function onFeaturesChange(cb: () => void): () => void {
	listeners.add(cb);
	return () => listeners.delete(cb);
}

class FeaturesStateManager {
	isLoading = $state(false);
	isInitialized = $state(false);
	runtimes = $state<Runtime[]>([]);
	models = $state<ModelRecord[]>([]);
	lastUpdated = $state(0);

	private activeFetchPromise: Promise<void> | null = null;

	private notify() {
		for (const cb of listeners) {
			try {
				cb();
			} catch (e) {
				console.error('Error notifying features listener:', e);
			}
		}
	}

	async refresh(force = false): Promise<void> {
		const now = Date.now();
		if (!force && this.isInitialized && now - this.lastUpdated < 3000) {
			return;
		}

		if (this.activeFetchPromise) {
			return this.activeFetchPromise;
		}

		this.isLoading = true;
		this.notify();

		this.activeFetchPromise = Promise.all([
			runtimesApi.list().catch(() => []),
			modelsApi.list(true).catch(() => [])
		])
			.then(([runtimes, models]) => {
				this.runtimes = Array.isArray(runtimes) ? runtimes : [];
				this.models = Array.isArray(models) ? models : [];
				this.isInitialized = true;
				this.lastUpdated = Date.now();
			})
			.finally(() => {
				this.isLoading = false;
				this.activeFetchPromise = null;
				this.notify();
			});

		return this.activeFetchPromise;
	}

	checkFeature(feature: string): FeatureCheckResult {
		const featLower = feature.toLowerCase().trim();
		const candidateIds = new Set<string>();
		const candidateTitles: string[] = [];

		for (const r of this.runtimes) {
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

		const hasInstalledRuntime = this.runtimes.some(
			(r) => r.is_installed && candidateIds.has(r.id.toLowerCase())
		);

		const hasRegisteredModel = this.models.some((m) =>
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

	isFeatureAvailable(feature: string): boolean {
		if (!this.isInitialized) return false;
		return this.checkFeature(feature).isSatisfied;
	}
}

export const featuresState = new FeaturesStateManager();
export const refreshFeatures = (force?: boolean) => featuresState.refresh(force);
export const checkFeature = (
	_state: { runtimes: Runtime[]; models: ModelRecord[] } | null | undefined,
	feature: string
) => {
	return featuresState.checkFeature(feature);
};
export const isFeatureAvailable = (feature: string) => featuresState.isFeatureAvailable(feature);

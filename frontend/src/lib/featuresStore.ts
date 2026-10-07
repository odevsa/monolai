import {
	featuresState,
	refreshFeatures,
	checkFeature,
	isFeatureAvailable,
	onFeaturesChange,
	type FeaturesState,
	type FeatureCheckResult
} from '$lib/state/features.svelte';
import type { Runtime } from '$lib/types/runtimes';
import type { ModelRecord } from '$lib/types/models';

export type { FeaturesState, FeatureCheckResult };
export type RuntimeItem = Runtime;
export type { ModelRecord };

const subscribers = new Set<(val: FeaturesState) => void>();

function getCurrentState(): FeaturesState {
	return {
		isLoading: featuresState.isLoading,
		isInitialized: featuresState.isInitialized,
		runtimes: featuresState.runtimes,
		models: featuresState.models,
		lastUpdated: featuresState.lastUpdated
	};
}

onFeaturesChange(() => {
	const current = getCurrentState();
	for (const fn of subscribers) {
		fn(current);
	}
});

export const featuresStore = {
	subscribe(fn: (val: FeaturesState) => void) {
		subscribers.add(fn);
		fn(getCurrentState());
		return () => {
			subscribers.delete(fn);
		};
	}
};

export { refreshFeatures, checkFeature, isFeatureAvailable };

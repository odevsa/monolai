import { runningModelsState, getStatusType, onRunningModelsChange } from '$lib/state/models.svelte';
import type { RunningModelStatus, RunningModelStateType } from '$lib/types/models';

export type { RunningModelStatus, RunningModelStateType };
export { getStatusType, runningModelsState };

const runningSubscribers = new Set<(val: RunningModelStatus[]) => void>();
const unloadingSubscribers = new Set<(val: Set<string>) => void>();

onRunningModelsChange(() => {
	const currentModels = runningModelsState.runningModels;
	for (const fn of runningSubscribers) {
		try {
			fn(currentModels);
		} catch (e) {
			console.error('Error in runningModels subscriber:', e);
		}
	}

	const currentUnloading = runningModelsState.unloadingModelIds;
	for (const fn of unloadingSubscribers) {
		try {
			fn(currentUnloading);
		} catch (e) {
			console.error('Error in unloadingModelIds subscriber:', e);
		}
	}
});

export const runningModels = {
	subscribe(fn: (val: RunningModelStatus[]) => void) {
		runningSubscribers.add(fn);
		fn(runningModelsState.runningModels);
		return () => {
			runningSubscribers.delete(fn);
		};
	}
};

export const unloadingModelIds = {
	subscribe(fn: (val: Set<string>) => void) {
		unloadingSubscribers.add(fn);
		fn(runningModelsState.unloadingModelIds);
		return () => {
			unloadingSubscribers.delete(fn);
		};
	}
};

export const fetchRunningState = () => runningModelsState.fetchRunningState();
export const startRunningStatePolling = (intervalMs?: number) =>
	runningModelsState.startPolling(intervalMs);
export const startRunningStateStream = () => runningModelsState.startStreaming();
export const unloadModel = (modelId: string) => runningModelsState.unloadModel(modelId);
export const unloadAllModels = () => runningModelsState.unloadAllModels();

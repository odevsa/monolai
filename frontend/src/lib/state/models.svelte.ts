import { modelsApi } from '$lib/api/models';
import type { RunningModelStatus, RunningModelStateType } from '$lib/types/models';

export function getStatusType(stateObj: any): RunningModelStateType {
	if (!stateObj) return 'idle';
	if (typeof stateObj === 'string') {
		const lower = stateObj.toLowerCase();
		if (lower === 'ready') return 'ready';
		if (lower === 'loading') return 'loading';
		if (lower === 'error') return 'error';
		return 'idle';
	}
	if (typeof stateObj === 'object' && stateObj.status) {
		const lower = String(stateObj.status).toLowerCase();
		if (lower === 'ready') return 'ready';
		if (lower === 'loading') return 'loading';
		if (lower === 'error') return 'error';
	}
	return 'idle';
}

const listeners = new Set<() => void>();

export function onRunningModelsChange(cb: () => void): () => void {
	listeners.add(cb);
	return () => listeners.delete(cb);
}

class RunningModelsStateManager {
	runningModels = $state<RunningModelStatus[]>([]);
	unloadingModelIds = $state<Set<string>>(new Set());

	private closeStream: (() => void) | null = null;
	private streamRefCount = 0;

	private notify() {
		for (const cb of listeners) {
			try {
				cb();
			} catch (e) {
				console.error('Error notifying running models listener:', e);
			}
		}
	}

	private updateState(data: RunningModelStatus[] | undefined | null) {
		this.runningModels = data || [];
		const currentIds = new Set((data || []).map((m) => m.model_id));
		const updatedUnloading = new Set<string>();
		for (const id of this.unloadingModelIds) {
			if (currentIds.has(id)) {
				updatedUnloading.add(id);
			}
		}
		this.unloadingModelIds = updatedUnloading;
		this.notify();
	}

	async fetchRunningState() {
		try {
			const data = await modelsApi.getRunningState();
			this.updateState(data);
		} catch {
			// ignore fetch errors
		}
	}

	startStreaming(): () => void {
		if (typeof window === 'undefined') return () => {};

		this.streamRefCount++;
		if (this.streamRefCount === 1) {
			this.closeStream = modelsApi.subscribeRunningState(
				(models) => {
					this.updateState(models);
				},
				() => {
					// Fallback to fetch on SSE errors or reconnect
					this.fetchRunningState();
				}
			);
		}

		return () => {
			this.streamRefCount = Math.max(0, this.streamRefCount - 1);
			if (this.streamRefCount === 0 && this.closeStream) {
				this.closeStream();
				this.closeStream = null;
			}
		};
	}

	/**
	 * Maintained for backward compatibility. Connects to the real-time SSE stream.
	 */
	startPolling(_intervalMs = 2500): () => void {
		return this.startStreaming();
	}

	async unloadModel(modelId: string) {
		const nextSet = new Set(this.unloadingModelIds);
		nextSet.add(modelId);
		this.unloadingModelIds = nextSet;
		this.notify();

		if (typeof window !== 'undefined') {
			window.dispatchEvent(new CustomEvent('monolai:model-unload', { detail: { modelId } }));
		}

		try {
			await modelsApi.unload(modelId);
			this.runningModels = this.runningModels.filter((m) => m.model_id !== modelId);
			const remainingUnloading = new Set(this.unloadingModelIds);
			remainingUnloading.delete(modelId);
			this.unloadingModelIds = remainingUnloading;
			this.notify();
		} catch (e) {
			console.error('Failed to unload model:', e);
		} finally {
			await this.fetchRunningState();
		}
	}

	async unloadAllModels() {
		if (typeof window !== 'undefined') {
			window.dispatchEvent(new CustomEvent('monolai:model-unload', { detail: { modelId: '*' } }));
		}

		try {
			await modelsApi.unloadAll();
			this.runningModels = [];
			this.unloadingModelIds = new Set();
			this.notify();
		} catch (e) {
			console.error('Failed to unload all models:', e);
		} finally {
			await this.fetchRunningState();
		}
	}
}

export const runningModelsState = new RunningModelsStateManager();

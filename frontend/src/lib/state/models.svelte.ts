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

	private activePollTimer: ReturnType<typeof setInterval> | null = null;
	private pollRefCount = 0;

	private notify() {
		for (const cb of listeners) {
			try {
				cb();
			} catch (e) {
				console.error('Error notifying running models listener:', e);
			}
		}
	}

	async fetchRunningState() {
		try {
			const data = await modelsApi.getRunningState();
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
		} catch {
			// ignore polling errors
		}
	}

	startPolling(intervalMs = 2500): () => void {
		if (typeof window === 'undefined') return () => {};

		this.pollRefCount++;
		if (this.pollRefCount === 1) {
			this.fetchRunningState();
			this.activePollTimer = setInterval(() => this.fetchRunningState(), intervalMs);
		}

		return () => {
			this.pollRefCount = Math.max(0, this.pollRefCount - 1);
			if (this.pollRefCount === 0 && this.activePollTimer) {
				clearInterval(this.activePollTimer);
				this.activePollTimer = null;
			}
		};
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

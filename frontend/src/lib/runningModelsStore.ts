import { writable } from 'svelte/store';

export interface RunningModelStatus {
	model_id: string;
	runtime_id: string;
	pid: number;
	port: number;
	state: { status: string; message?: string } | string;
	idle_seconds: number;
}

export const runningModels = writable<RunningModelStatus[]>([]);
export const unloadingModelIds = writable<Set<string>>(new Set());

export function getStatusType(stateObj: any): 'ready' | 'loading' | 'error' | 'idle' {
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

let activePollTimer: ReturnType<typeof setInterval> | null = null;
let pollRefCount = 0;

export async function fetchRunningState() {
	try {
		const res = await fetch('/api/state');
		if (res.ok) {
			const data: RunningModelStatus[] = await res.json();
			runningModels.set(data);
			const currentIds = new Set(data.map((m) => m.model_id));
			unloadingModelIds.update((prev) => new Set([...prev].filter((id) => currentIds.has(id))));
		}
	} catch {
		// ignore polling errors
	}
}

/**
 * Starts periodic polling of /api/state (reused across components).
 * Uses ref-counting so only one timer runs at any time.
 */
export function startRunningStatePolling(intervalMs = 3000): () => void {
	if (typeof window === 'undefined') return () => {};

	pollRefCount++;
	if (pollRefCount === 1) {
		fetchRunningState();
		activePollTimer = setInterval(fetchRunningState, intervalMs);
	}

	return () => {
		pollRefCount = Math.max(0, pollRefCount - 1);
		if (pollRefCount === 0 && activePollTimer) {
			clearInterval(activePollTimer);
			activePollTimer = null;
		}
	};
}

export async function unloadModel(modelId: string) {
	unloadingModelIds.update((prev) => new Set(prev).add(modelId));
	try {
		await fetch(`/api/models/${encodeURIComponent(modelId)}/unload`, { method: 'POST' });
	} catch (e) {
		console.error('Failed to unload model:', e);
	} finally {
		await fetchRunningState();
	}
}

export async function unloadAllModels() {
	try {
		await fetch('/api/models/unload-all', { method: 'POST' });
	} catch (e) {
		console.error('Failed to unload all models:', e);
	} finally {
		await fetchRunningState();
	}
}

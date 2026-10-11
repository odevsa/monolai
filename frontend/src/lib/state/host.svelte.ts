import { hostApi } from '$lib/api/host';
import type { HostMetricsTick, SysInfo } from '$lib/types/host';

export type HostTickListener = (tick: HostMetricsTick) => void;

class HostState {
	sysinfo = $state<SysInfo | null>(null);
	latestTick = $state<HostMetricsTick | null>(null);
	connected = $state(false);
	error = $state<string | null>(null);

	private unsubscribe: (() => void) | null = null;
	private subscriberCount = 0;
	private listeners = new Set<HostTickListener>();

	async fetchStaticInfo() {
		try {
			const data = await hostApi.getInfo();
			this.sysinfo = data;
			this.error = null;
		} catch (err: unknown) {
			const message = err instanceof Error ? err.message : 'Failed to fetch host system info';
			this.error = message;
		}
	}

	subscribe(listener?: HostTickListener): () => void {
		this.subscriberCount++;
		if (listener) {
			this.listeners.add(listener);
		}
		if (this.subscriberCount === 1) {
			this.startStream();
		}

		return () => {
			if (listener) {
				this.listeners.delete(listener);
			}
			this.subscriberCount = Math.max(0, this.subscriberCount - 1);
			if (this.subscriberCount === 0) {
				this.stopStream();
			}
		};
	}

	startStream() {
		if (this.unsubscribe) return;
		this.fetchStaticInfo();

		this.unsubscribe = hostApi.subscribeMetrics(
			(tick) => {
				this.latestTick = tick;
				this.connected = true;
				this.error = null;
				for (const listener of this.listeners) {
					try {
						listener(tick);
					} catch (e) {
						console.error('[HostState] listener error:', e);
					}
				}
			},
			() => {
				this.connected = false;
			}
		);
	}

	stopStream() {
		if (this.unsubscribe) {
			this.unsubscribe();
			this.unsubscribe = null;
		}
		this.connected = false;
	}

	reconnect() {
		this.stopStream();
		this.startStream();
	}
}

export const hostState = new HostState();

import { api } from './client';
import type { SysInfo, HostMetricsTick } from '$lib/types/host';
import type { HardwareReport } from '$lib/types/config';

export const hostApi = {
	getInfo: () => api.get<SysInfo>('/api/host'),

	detectHardware: () => api.get<HardwareReport>('/api/hardware/detect'),

	subscribeMetrics: (
		onTick: (tick: HostMetricsTick) => void,
		onError?: (err: Event) => void
	): (() => void) => {
		const es = new EventSource('/api/host/usage');

		es.onmessage = (event) => {
			try {
				const data = JSON.parse(event.data);
				onTick(data);
			} catch (e) {
				console.error('Error parsing host metrics SSE tick:', e);
			}
		};

		if (onError) {
			es.onerror = onError;
		}

		return () => es.close();
	}
};

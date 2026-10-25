import { downloadsApi } from '$lib/api/downloads';
import type { DownloadRecord, EnqueueDownloadPayload } from '$lib/types/models';

class DownloadStore {
	downloads = $state<DownloadRecord[]>([]);
	isSubscribed = $state<boolean>(false);
	private unsubscribe: (() => void) | null = null;

	activeDownloads = $derived(this.downloads.filter((d) => d.status === 'downloading'));
	hasActive = $derived(this.activeDownloads.length > 0);
	activeCount = $derived(this.activeDownloads.length);
	totalSpeed = $derived(
		this.activeDownloads.reduce((acc, curr) => acc + (curr.speed_bytes_sec || 0), 0)
	);
	totalSpeedBytesSec = $derived(this.totalSpeed);

	async init() {
		if (this.isSubscribed) return;

		try {
			const initial = await downloadsApi.list();
			this.downloads = initial;
		} catch (e) {
			console.error('Failed to load initial downloads:', e);
		}

		this.unsubscribe = downloadsApi.subscribe(
			(updated) => {
				this.downloads = updated;
			},
			(err) => {
				console.error('Downloads SSE connection error:', err);
			}
		);
		this.isSubscribed = true;
	}

	destroy() {
		if (this.unsubscribe) {
			this.unsubscribe();
			this.unsubscribe = null;
			this.isSubscribed = false;
		}
	}

	async enqueue(payload: EnqueueDownloadPayload) {
		const created = await downloadsApi.enqueue(payload);
		await this.refresh();
		return created;
	}

	async pause(id: string) {
		await downloadsApi.pause(id);
		await this.refresh();
	}

	async resume(id: string) {
		await downloadsApi.resume(id);
		await this.refresh();
	}

	async cancel(id: string) {
		await downloadsApi.cancel(id);
		await this.refresh();
	}

	async delete(id: string) {
		await downloadsApi.delete(id);
		await this.refresh();
	}

	async refresh() {
		try {
			this.downloads = await downloadsApi.list();
		} catch (e) {
			console.error('Failed to refresh downloads:', e);
		}
	}
}

export const downloadStore = new DownloadStore();

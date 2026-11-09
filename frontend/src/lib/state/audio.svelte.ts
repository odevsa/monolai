import { audioApi } from '$lib/api/audio';
import type { AudioGenerationItem, CreateAudioGenerationPayload } from '$lib/types/audio';

class AudioStateManager {
	history = $state<AudioGenerationItem[]>([]);
	selectedItem = $state<AudioGenerationItem | null>(null);
	selectionVersion = $state<number>(0);
	newSessionVersion = $state<number>(0);
	isLoading = $state(false);
	isInitialized = $state(false);

	constructor() {
		if (typeof window !== 'undefined') {
			this.loadHistory();
		}
	}

	async loadHistory(type?: 'tts' | 'asr') {
		if (this.isLoading) return;
		this.isLoading = true;
		try {
			const items = await audioApi.list(type, 200);
			this.history = items;
			this.isInitialized = true;
		} catch (err) {
			console.error('Failed to load audio history from SQLite database:', err);
		} finally {
			this.isLoading = false;
		}
	}

	async addItem(item: CreateAudioGenerationPayload) {
		try {
			const created = await audioApi.create(item);
			this.history = [created, ...this.history];
		} catch (err) {
			console.error('Failed to persist audio generation to database:', err);
			// Optimistic fallback in case of immediate network failure
			const fallback: AudioGenerationItem = {
				id: item.id || `audio-${Date.now()}`,
				...item,
				createdAt: new Date().toISOString()
			};
			this.history = [fallback, ...this.history];
		}
	}

	async deleteItem(id: string) {
		try {
			await audioApi.delete(id);
		} catch (err) {
			console.error('Failed to delete audio item from database:', err);
		}

		const filtered = this.history.filter((i) => i.id !== id);
		this.history = filtered;
		if (this.selectedItem?.id === id) {
			this.selectedItem = null;
			this.selectionVersion++;
		}
	}

	async clearHistory() {
		try {
			await audioApi.clear();
		} catch (err) {
			console.error('Failed to clear audio history from database:', err);
		}

		this.history = [];
		this.selectedItem = null;
		this.selectionVersion++;
	}

	selectItem(item: AudioGenerationItem | null) {
		this.selectedItem = item;
		this.selectionVersion++;
	}

	newSession() {
		this.selectedItem = null;
		this.selectionVersion++;
		this.newSessionVersion++;
	}
}

export const audioState = new AudioStateManager();

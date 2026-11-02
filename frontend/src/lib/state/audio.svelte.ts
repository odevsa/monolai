import type { AudioGenerationItem } from '$lib/types/audio';

const STORAGE_KEY = 'monolai:audio_history';
const MAX_AUDIO_HISTORY = 30;

class AudioStateManager {
	history = $state<AudioGenerationItem[]>([]);
	selectedItem = $state<AudioGenerationItem | null>(null);
	selectionVersion = $state<number>(0);

	constructor() {
		if (typeof window !== 'undefined') {
			this.loadFromStorage();
		}
	}

	loadFromStorage() {
		if (typeof window === 'undefined') return;
		try {
			const saved = localStorage.getItem(STORAGE_KEY);
			if (saved) {
				const items: AudioGenerationItem[] = JSON.parse(saved);
				this.history = items;
				if (items.length > 0 && !this.selectedItem) {
					this.selectedItem = items[0];
				}
			}
		} catch (err) {
			console.error('Failed to load audio history from storage:', err);
		}
	}

	setHistory(newHistory: AudioGenerationItem[]) {
		this.history = newHistory;
		if (typeof window !== 'undefined') {
			try {
				const serializable = newHistory.slice(0, MAX_AUDIO_HISTORY).map((item, idx) => ({
					...item,
					// Persist data URLs in localStorage for the 10 most recent items
					audioUrl: idx < 10 && item.audioUrl?.startsWith('data:') ? item.audioUrl : undefined
				}));
				localStorage.setItem(STORAGE_KEY, JSON.stringify(serializable));
			} catch (err) {
				console.warn('Failed to save audio history with data URLs, saving metadata only:', err);
				try {
					const metadataOnly = newHistory.slice(0, MAX_AUDIO_HISTORY).map((item) => ({
						...item,
						audioUrl: undefined
					}));
					localStorage.setItem(STORAGE_KEY, JSON.stringify(metadataOnly));
				} catch (innerErr) {
					console.error('Failed to save audio metadata to storage:', innerErr);
				}
			}
		}
	}

	addItem(item: AudioGenerationItem) {
		this.setHistory([item, ...this.history]);
		this.selectItem(item);
	}

	deleteItem(id: string) {
		const filtered = this.history.filter((i) => i.id !== id);
		this.setHistory(filtered);
		if (this.selectedItem?.id === id) {
			this.selectedItem = filtered.length > 0 ? filtered[0] : null;
			this.selectionVersion++;
		}
	}

	clearHistory() {
		this.history = [];
		this.selectedItem = null;
		this.selectionVersion++;
		if (typeof window !== 'undefined') {
			localStorage.removeItem(STORAGE_KEY);
		}
	}

	selectItem(item: AudioGenerationItem | null) {
		this.selectedItem = item;
		this.selectionVersion++;
	}
}

export const audioState = new AudioStateManager();

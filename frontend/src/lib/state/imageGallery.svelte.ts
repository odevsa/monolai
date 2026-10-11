import type { GeneratedImageItem } from '$lib/types/chat';

const STORAGE_KEY = 'monolai:image_history';
const MAX_HISTORY_ITEMS = 30;

class ImageGalleryStateManager {
	history = $state<GeneratedImageItem[]>([]);
	selectedImage = $state<GeneratedImageItem | null>(null);
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
				this.history = JSON.parse(saved);
			}
		} catch (err) {
			console.error('Failed to load image history from storage:', err);
		}
	}

	setHistory(newHistory: GeneratedImageItem[]) {
		this.history = newHistory;
		if (typeof window !== 'undefined') {
			try {
				localStorage.setItem(STORAGE_KEY, JSON.stringify(newHistory.slice(0, MAX_HISTORY_ITEMS)));
			} catch (err) {
				console.error('Failed to save image history to storage:', err);
			}
		}
	}

	addImage(item: GeneratedImageItem | GeneratedImageItem[]) {
		const items = Array.isArray(item) ? item : [item];
		this.setHistory([...items, ...this.history]);
		if (items.length > 0) {
			this.selectImage(items[0]);
		}
	}

	deleteImage(id: string) {
		const filtered = this.history.filter((img) => img.id !== id);
		this.setHistory(filtered);
		if (this.selectedImage?.id === id) {
			this.selectedImage = filtered.length > 0 ? filtered[0] : null;
			this.selectionVersion++;
		}
	}

	clearHistory() {
		this.history = [];
		this.selectedImage = null;
		this.selectionVersion++;
		if (typeof window !== 'undefined') {
			localStorage.removeItem(STORAGE_KEY);
		}
	}

	selectImage(item: GeneratedImageItem | null) {
		this.selectedImage = item;
		this.selectionVersion++;
	}
}

export const imageGalleryState = new ImageGalleryStateManager();

import { imagesApi } from '$lib/api/images';
import type { CreateGeneratedImagePayload, GeneratedImageItem } from '$lib/types/images';

class ImageGalleryStateManager {
	history = $state<GeneratedImageItem[]>([]);
	selectedImage = $state<GeneratedImageItem | null>(null);
	selectionVersion = $state<number>(0);
	newSessionVersion = $state<number>(0);
	isLoading = $state(false);
	isInitialized = $state(false);

	constructor() {
		if (typeof window !== 'undefined') {
			this.loadHistory();
		}
	}

	async loadHistory() {
		if (this.isLoading) return;
		this.isLoading = true;
		try {
			const items = await imagesApi.list(200);
			this.history = items;
			this.isInitialized = true;
		} catch (err) {
			console.error('Failed to load image history from SQLite database:', err);
		} finally {
			this.isLoading = false;
		}
	}

	async addImage(item: CreateGeneratedImagePayload | CreateGeneratedImagePayload[]) {
		const items = Array.isArray(item) ? item : [item];
		const createdList: GeneratedImageItem[] = [];

		for (const it of items) {
			try {
				const created = await imagesApi.create(it);
				createdList.push(created);
			} catch (err) {
				console.error('Failed to persist generated image to database:', err);
			}
		}

		if (createdList.length > 0) {
			this.history = [...createdList, ...this.history];
		}
	}

	async deleteImage(id: string) {
		try {
			await imagesApi.delete(id);
		} catch (err) {
			console.error('Failed to delete generated image from database:', err);
		}

		const filtered = this.history.filter((img) => img.id !== id);
		this.history = filtered;
		if (this.selectedImage?.id === id) {
			this.selectedImage = null;
			this.selectionVersion++;
		}
	}

	async clearHistory() {
		try {
			await imagesApi.clear();
		} catch (err) {
			console.error('Failed to clear image history from database:', err);
		}

		this.history = [];
		this.selectedImage = null;
		this.selectionVersion++;
	}

	selectImage(item: GeneratedImageItem | null) {
		this.selectedImage = item;
		this.selectionVersion++;
	}

	newSession() {
		this.selectedImage = null;
		this.selectionVersion++;
		this.newSessionVersion++;
	}
}

export const imageGalleryState = new ImageGalleryStateManager();

export interface GeneratedImageItem {
	id: string;
	prompt: string;
	negativePrompt?: string;
	model: string;
	size: string;
	steps: number;
	cfgScale: number;
	seed?: number;
	samplingMethod?: string;
	scheduleMethod?: string;
	src: string;
	generationTimeSecs?: number;
	createdAt?: string;
}

export interface CreateGeneratedImagePayload {
	id?: string;
	prompt: string;
	negativePrompt?: string;
	model: string;
	size: string;
	steps: number;
	cfgScale: number;
	seed?: number;
	samplingMethod?: string;
	scheduleMethod?: string;
	src: string;
	generationTimeSecs?: number;
}

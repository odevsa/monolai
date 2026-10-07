export interface ChatConversation {
	id: string;
	title: string;
	created_at?: string;
	updated_at?: string;
}

export interface ChatMessage {
	id: string;
	role: 'user' | 'assistant' | 'system';
	content: string;
	reasoning?: string;
	reasoningDuration?: string;
	tokens?: number;
	duration?: string;
	speed?: string;
	modelTags?: string[];
	status?: string;
	created_at?: string;
}

export interface GeneratedImageItem {
	id: string;
	timestamp: number;
	prompt: string;
	negativePrompt?: string;
	model: string;
	size: string;
	steps: number;
	cfgScale: number;
	seed?: number;
	src: string;
	generationTimeSecs?: number;
}

export interface ChatConversation {
	id: string;
	title: string;
	created_at?: string;
	updated_at?: string;
}

export type AttachmentType = 'image' | 'text' | 'audio' | 'video' | 'file';

export interface ChatAttachment {
	id: string;
	type: AttachmentType;
	name: string;
	size: number;
	mimeType: string;
	dataUrl: string;
	thumbnailUrl?: string;
	textContent?: string;
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
	model_tags?: string;
	status?: string;
	attachments?: ChatAttachment[] | string;
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

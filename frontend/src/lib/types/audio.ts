export type AudioMode = 'text-to-speech' | 'speech-to-text';

export interface AudioGenerationItem {
	id: string;
	type: 'tts' | 'stt';
	model: string;
	timestamp: number;
	// TTS fields
	prompt?: string;
	audioUrl?: string;
	voice?: string;
	speed?: number;
	format?: string;
	durationSecs?: number;
	// STT fields
	transcript?: string;
	filename?: string;
	language?: string;
	wordCount?: number;
}

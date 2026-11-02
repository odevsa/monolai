export type AudioMode = 'text-to-speech' | 'automatic-speech-recognition';

export interface AudioGenerationItem {
	id: string;
	type: 'tts' | 'asr';
	model: string;
	timestamp: number;
	// TTS fields
	prompt?: string;
	audioUrl?: string;
	voice?: string;
	speed?: number;
	format?: string;
	durationSecs?: number;
	// ASR fields
	transcript?: string;
	filename?: string;
	language?: string;
	wordCount?: number;
}

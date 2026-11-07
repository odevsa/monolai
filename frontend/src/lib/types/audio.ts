export type AudioMode = 'text-to-speech' | 'automatic-speech-recognition';

export interface AudioGenerationItem {
	id: string;
	type: 'tts' | 'asr';
	model: string;
	prompt?: string;
	audioUrl?: string;
	voice?: string;
	speed?: number;
	format?: string;
	durationSecs?: number;
	transcript?: string;
	filename?: string;
	language?: string;
	wordCount?: number;
	createdAt?: string;
}

export interface CreateAudioGenerationPayload {
	id?: string;
	type: 'tts' | 'asr';
	model: string;
	prompt?: string;
	audioUrl?: string;
	voice?: string;
	speed?: number;
	format?: string;
	durationSecs?: number;
	transcript?: string;
	filename?: string;
	language?: string;
	wordCount?: number;
}

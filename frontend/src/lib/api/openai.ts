import OpenAI from 'openai';
import type { SpeechCreateParams } from 'openai/resources/audio/speech';
import type {
	Transcription,
	TranscriptionCreateParamsNonStreaming
} from 'openai/resources/audio/transcriptions';
import type {
	ChatCompletionChunk,
	ChatCompletionMessageParam
} from 'openai/resources/chat/completions';
import type { ImageGenerateParams, ImagesResponse } from 'openai/resources/images';
import type { Model } from 'openai/resources/models';

function getOpenAIBaseUrl(): string {
	return `${window.location.origin}/v1`;
}

// 1 hour in milliseconds to prevent premature abortion of long local generations (e.g. multi-step CPU diffusion or deep reasoning)
const LOCAL_INFERENCE_TIMEOUT_MS = 60 * 60 * 1000;

export function createOpenAIClient(): OpenAI {
	return new OpenAI({
		baseURL: getOpenAIBaseUrl(),
		apiKey: 'monolai',
		dangerouslyAllowBrowser: true,
		maxRetries: 0,
		timeout: LOCAL_INFERENCE_TIMEOUT_MS
	});
}

// Lazy dynamic proxy to always ensure valid origin URL in browser runtime
export const openai: OpenAI = new Proxy({} as OpenAI, {
	get(_target, prop) {
		const client = createOpenAIClient();
		const val = (client as any)[prop];
		return typeof val === 'function' ? val.bind(client) : val;
	}
});

export interface StreamChatOptions {
	model: string;
	messages: ChatCompletionMessageParam[];
	temperature?: number;
	max_tokens?: number;
	stream?: true;
	signal?: AbortSignal;
	[key: string]: any;
}

/**
 * Lists models available in OpenAI format (GET /v1/models) using the official OpenAI client.
 */
export async function listModels(): Promise<Model[]> {
	const client = createOpenAIClient();
	const page = await client.models.list();
	return page.data || [];
}

/**
 * Creates a streaming chat completion using the official OpenAI client.
 * Returns an async iterable stream of ChatCompletionChunk objects.
 */
export async function streamChatCompletion(
	options: StreamChatOptions
): Promise<AsyncIterable<ChatCompletionChunk>> {
	const { signal, ...params } = options;
	const client = createOpenAIClient();

	const stream = await client.chat.completions.create(
		{
			...params,
			stream: true
		},
		{
			signal,
			timeout: LOCAL_INFERENCE_TIMEOUT_MS
		}
	);

	return stream;
}

/**
 * Generates images using the official OpenAI client.
 */
export async function generateImages(
	params: ImageGenerateParams,
	signal?: AbortSignal
): Promise<ImagesResponse> {
	const client = createOpenAIClient();
	const res = await client.images.generate(params, {
		signal,
		timeout: LOCAL_INFERENCE_TIMEOUT_MS
	});
	return res as ImagesResponse;
}

/**
 * Generates speech audio from text using the official OpenAI client (POST /v1/audio/speech).
 * Returns a Blob containing the synthesized audio data.
 */
export async function generateSpeech(
	params: SpeechCreateParams,
	signal?: AbortSignal
): Promise<Blob> {
	const client = createOpenAIClient();
	const response = await client.audio.speech.create(params, {
		signal,
		timeout: LOCAL_INFERENCE_TIMEOUT_MS
	});
	return await response.blob();
}

/**
 * Transcribes audio file or recording into text using the official OpenAI client (POST /v1/audio/transcriptions).
 * Returns the Transcription object containing the transcribed text.
 */
export async function transcribeAudio(
	params: TranscriptionCreateParamsNonStreaming,
	signal?: AbortSignal
): Promise<Transcription> {
	const client = createOpenAIClient();
	const response = await client.audio.transcriptions.create(params, {
		signal,
		timeout: LOCAL_INFERENCE_TIMEOUT_MS,
		query: { model: params.model }
	});
	return response as Transcription;
}

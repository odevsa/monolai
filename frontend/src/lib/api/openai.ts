import OpenAI from 'openai';
import type {
	ChatCompletionChunk,
	ChatCompletionMessageParam
} from 'openai/resources/chat/completions';
import type { ImageGenerateParams, ImagesResponse } from 'openai/resources/images';
import type { Model } from 'openai/resources/models';

function getOpenAIBaseUrl(): string {
	return `${window.location.origin}/v1`;
}

export function createOpenAIClient(): OpenAI {
	return new OpenAI({
		baseURL: getOpenAIBaseUrl(),
		apiKey: 'monolai',
		dangerouslyAllowBrowser: true
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
			signal
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
	const res = await client.images.generate(params, { signal });
	return res as ImagesResponse;
}

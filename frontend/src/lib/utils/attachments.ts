import { generateUUID } from './common';
import type { ChatAttachment, AttachmentType } from '$lib/types/chat';

export interface ModelSupportedModalities {
	text: boolean;
	image: boolean;
	audio: boolean;
	video: boolean;
}

const TEXT_EXTENSIONS = new Set([
	'txt',
	'md',
	'markdown',
	'rs',
	'py',
	'js',
	'ts',
	'jsx',
	'tsx',
	'json',
	'csv',
	'log',
	'yaml',
	'yml',
	'toml',
	'html',
	'css',
	'scss',
	'c',
	'cpp',
	'h',
	'hpp',
	'go',
	'sh',
	'bash',
	'zsh',
	'sql',
	'xml',
	'ini',
	'env',
	'diff',
	'patch'
]);

/**
 * Inspects a model's flags, architecture, and canonical naming to detect which
 * input modalities (text, image, audio, video) the model can natively interpret.
 */
export function detectModelSupportedModalities(model?: {
	flags?: string;
	architecture?: string;
	modality?: string;
	id?: string;
	modelId?: string;
	presetName?: string;
}): ModelSupportedModalities {
	// Text is universally supported across all chat/completion models
	const result: ModelSupportedModalities = {
		text: true,
		image: false,
		audio: false,
		video: false
	};

	if (!model) return result;

	const arch = (model.architecture || '').toLowerCase();
	const modelId =
		`${model.id || ''} ${model.modelId || ''} ${model.presetName || ''}`.toLowerCase();
	const rawModality = (model.modality || '').toLowerCase();

	// 1. Check for vision/image support
	let hasMmproj = false;
	if (model.flags) {
		try {
			const parsed = typeof model.flags === 'string' ? JSON.parse(model.flags) : model.flags;
			const mmprojVal = parsed['--mmproj'] || parsed['-mmproj'] || parsed['mmproj'];
			if (mmprojVal && typeof mmprojVal === 'string' && mmprojVal.trim().length > 0) {
				hasMmproj = true;
			}
		} catch {
			if (model.flags.includes('--mmproj') || model.flags.includes('mmproj')) {
				hasMmproj = true;
			}
		}
	}

	const isVisionArch =
		arch.includes('clip') ||
		arch.includes('llava') ||
		arch.includes('qwen2_vl') ||
		arch.includes('qwen2.5_vl') ||
		arch.includes('minicpm_v') ||
		arch.includes('internvl') ||
		arch.includes('phi3_vision') ||
		arch.includes('deepseek_vl') ||
		arch.includes('moondream') ||
		arch.includes('paligemma') ||
		arch.includes('smolvlm') ||
		arch.includes('mllama') ||
		arch.includes('vision');

	const isVisionName =
		modelId.includes('vision') ||
		modelId.includes('-vl') ||
		modelId.includes('_vl') ||
		modelId.includes('llava') ||
		modelId.includes('minicpm-v') ||
		modelId.includes('pixtral') ||
		modelId.includes('internvl') ||
		modelId.includes('moondream') ||
		modelId.includes('paligemma') ||
		modelId.includes('smolvlm') ||
		modelId.includes('multimodal');

	if (hasMmproj || isVisionArch || isVisionName || rawModality.includes('image')) {
		result.image = true;
	}

	// 2. Check for audio support
	const isAudioArch =
		arch.includes('whisper') ||
		arch.includes('qwen2_audio') ||
		arch.includes('audio_encoder') ||
		arch.includes('salmonn') ||
		arch.includes('seamless');

	const isAudioName =
		modelId.includes('audio') ||
		modelId.includes('whisper') ||
		modelId.includes('qwen2-audio') ||
		modelId.includes('voice');

	if (
		isAudioArch ||
		isAudioName ||
		rawModality.includes('speech') ||
		rawModality.includes('audio')
	) {
		result.audio = true;
	}

	// 3. Check for video support (models like Qwen2-VL, Video-LLaVA)
	const isVideoArch =
		arch.includes('qwen2_vl') ||
		arch.includes('qwen2.5_vl') ||
		arch.includes('videollava') ||
		arch.includes('llava_video') ||
		arch.includes('cogvlm2_video');

	const isVideoName =
		modelId.includes('videollava') ||
		modelId.includes('llava-video') ||
		modelId.includes('cogvlm2-video') ||
		modelId.includes('qwen2-vl') ||
		modelId.includes('qwen2.5-vl');

	if (isVideoArch || isVideoName || rawModality.includes('video')) {
		result.video = true;
		result.image = true; // Video models inherently support still images
	}

	return result;
}

/**
 * Builds the HTML <input type="file" accept="..."> attribute.
 * Allows all supported media types so users can pick any file.
 */
export function buildAcceptAttribute(_modalities?: ModelSupportedModalities): string {
	return 'image/*,audio/*,video/*,text/*,.txt,.md,.rs,.py,.js,.ts,.jsx,.tsx,.json,.csv,.log,.yaml,.yml,.toml,.html,.css,.scss,.c,.cpp,.h,.hpp,.go,.sh,.bash,.sql,.xml';
}

/**
 * Resolves the attachment category based on MIME type or file extension.
 */
export function determineAttachmentType(file: File): AttachmentType {
	if (file.type.startsWith('image/')) return 'image';
	if (file.type.startsWith('audio/')) return 'audio';
	if (file.type.startsWith('video/')) return 'video';

	const ext = file.name.split('.').pop()?.toLowerCase() || '';
	if (
		file.type.startsWith('text/') ||
		file.type === 'application/json' ||
		file.type === 'application/xml' ||
		file.type === 'application/javascript' ||
		TEXT_EXTENSIONS.has(ext)
	) {
		return 'text';
	}

	return 'file';
}

/**
 * Verifies if a given file can be attached.
 * Accepts any valid file for universal attachments.
 */
export function isFileTypeSupported(_file: File, _modalities?: ModelSupportedModalities): boolean {
	return true;
}

/**
 * Downscales and compresses large camera/desktop images in browser memory using HTML5 Canvas.
 * Prevents bloated multi-megabyte base64 JSON payloads while preserving optimal visual resolution.
 */
export async function optimizeImageFile(
	file: File,
	maxDimension = 1536,
	quality = 0.88
): Promise<{ dataUrl: string; thumbnailUrl: string; width: number; height: number }> {
	return new Promise((resolve, reject) => {
		const img = new Image();
		const objectUrl = URL.createObjectURL(file);

		img.onload = () => {
			URL.revokeObjectURL(objectUrl);
			let { width, height } = img;

			// Check if downscaling is required
			if (width > maxDimension || height > maxDimension) {
				if (width > height) {
					height = Math.round((height * maxDimension) / width);
					width = maxDimension;
				} else {
					width = Math.round((width * maxDimension) / height);
					height = maxDimension;
				}
			}

			// Render main optimized image
			const canvas = document.createElement('canvas');
			canvas.width = width;
			canvas.height = height;
			const ctx = canvas.getContext('2d');
			if (!ctx) {
				return reject(new Error('Failed to acquire canvas 2D context'));
			}
			ctx.drawImage(img, 0, 0, width, height);

			const mimeType =
				file.type === 'image/png' && file.size < 1024 * 1024 ? 'image/png' : 'image/jpeg';
			const dataUrl = canvas.toDataURL(mimeType, quality);

			// Render small fast thumbnail (max 180px)
			const thumbDim = 180;
			let thumbW = width;
			let thumbH = height;
			if (thumbW > thumbDim || thumbH > thumbDim) {
				if (thumbW > thumbH) {
					thumbH = Math.round((thumbH * thumbDim) / thumbW);
					thumbW = thumbDim;
				} else {
					thumbW = Math.round((thumbW * thumbDim) / thumbH);
					thumbH = thumbDim;
				}
			}
			const thumbCanvas = document.createElement('canvas');
			thumbCanvas.width = thumbW;
			thumbCanvas.height = thumbH;
			const thumbCtx = thumbCanvas.getContext('2d');
			if (thumbCtx) {
				thumbCtx.drawImage(img, 0, 0, thumbW, thumbH);
			}
			const thumbnailUrl = thumbCanvas.toDataURL('image/jpeg', 0.8);

			resolve({ dataUrl, thumbnailUrl, width, height });
		};

		img.onerror = () => {
			URL.revokeObjectURL(objectUrl);
			reject(new Error(`Failed to load image: ${file.name}`));
		};

		img.src = objectUrl;
	});
}

/**
 * Reads any File object and converts it into a standardized ChatAttachment.
 */
export async function processAttachmentFile(file: File): Promise<ChatAttachment> {
	const type = determineAttachmentType(file);
	const id = generateUUID();

	if (type === 'image') {
		try {
			const { dataUrl, thumbnailUrl } = await optimizeImageFile(file);
			return {
				id,
				type: 'image',
				name: file.name,
				size: file.size,
				mimeType: file.type || 'image/jpeg',
				dataUrl,
				thumbnailUrl
			};
		} catch {
			// Fallback to raw FileReader if canvas fails
			const rawData = await readAsDataUrl(file);
			return {
				id,
				type: 'image',
				name: file.name,
				size: file.size,
				mimeType: file.type || 'image/jpeg',
				dataUrl: rawData,
				thumbnailUrl: rawData
			};
		}
	}

	if (type === 'text') {
		const textContent = await file.text();
		const rawData = await readAsDataUrl(file);
		return {
			id,
			type: 'text',
			name: file.name,
			size: file.size,
			mimeType: file.type || 'text/plain',
			dataUrl: rawData,
			textContent
		};
	}

	// Audio, video, or generic files
	const rawData = await readAsDataUrl(file);
	return {
		id,
		type,
		name: file.name,
		size: file.size,
		mimeType: file.type || 'application/octet-stream',
		dataUrl: rawData
	};
}

function readAsDataUrl(file: File): Promise<string> {
	return new Promise((resolve, reject) => {
		const reader = new FileReader();
		reader.onload = () => resolve(reader.result as string);
		reader.onerror = () => reject(reader.error);
		reader.readAsDataURL(file);
	});
}

/**
 * Formats byte sizes into human readable strings (e.g. 1.2 MB, 450 KB).
 */
export function formatFileSize(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
	return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

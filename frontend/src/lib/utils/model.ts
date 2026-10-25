/**
 * Model parsing, flag formatting and model inspection utilities
 */

export interface ParsedModelDetails {
	name: string;
	tags: string[];
}

/**
 * Canonical model string formatter (SSoT):
 * Formats any model representation (object, record, path, or raw identifier)
 * into the canonical project standard: "{user}/{model}:{filename}"
 *
 * Examples:
 *   formatCanonicalModelString({ source_repo: "Qwen/Qwen2.5-GGUF", filename: "qwen2.5-1.5b-instruct-q4_k_m.gguf" })
 *   => "Qwen/Qwen2.5-GGUF:qwen2.5-1.5b-instruct-q4_k_m.gguf"
 *
 *   formatCanonicalModelString("text-to-text/Qwen/Qwen2.5-GGUF/qwen2.5-1.5b-instruct-q4_k_m.gguf")
 *   => "Qwen/Qwen2.5-GGUF:qwen2.5-1.5b-instruct-q4_k_m.gguf"
 *
 *   formatCanonicalModelString("Qwen--Qwen2.5-GGUF--qwen2.5-1.5b-instruct")
 *   => "Qwen/Qwen2.5-GGUF:qwen2.5-1.5b-instruct"
 */
export function formatCanonicalModelString(
	input:
		| string
		| string[]
		| {
				id?: string;
				source_repo?: string;
				relative_path?: string;
				filename?: string;
				name?: string;
		  }
		| undefined
		| null
): string {
	if (!input) return '';

	if (Array.isArray(input)) {
		if (input.length === 0) return '';
		return formatCanonicalModelString(input[0]);
	}

	if (typeof input === 'object') {
		// If object has source_repo and filename
		if (input.source_repo && input.filename) {
			const cleanRepo = input.source_repo.trim().replace(/^[:/]+|[:/]+$/g, '');
			const cleanFile = input.filename.trim();
			return `${cleanRepo}:${cleanFile}`;
		}
		// If object has relative_path: e.g. text-to-text/user/model/filename
		if (input.relative_path) {
			const norm = input.relative_path.replace(/\\/g, '/').replace(/^\/+/, '');
			const parts = norm.split('/');
			if (parts.length >= 4) {
				return `${parts[1]}/${parts[2]}:${parts.slice(3).join('/')}`;
			} else if (parts.length === 3) {
				return `${parts[1]}:${parts[2]}`;
			} else if (parts.length === 2 && input.filename) {
				return `${parts[0]}:${parts[1]}`;
			}
		}
		if (input.id) {
			return formatCanonicalModelString(input.id);
		}
		if (input.name && input.filename) {
			return `${input.name}:${input.filename}`;
		}
		return '';
	}

	const str = input.trim();
	if (!str) return '';

	// If already in "{user}/{model}:{filename}" format
	if (str.includes(':')) {
		const [first, ...rest] = str.split(':');
		const file = rest.join(':');
		const normFirst = first.replace(/\\/g, '/').replace(/^\/+/, '');
		const parts = normFirst.split('/');
		if (
			parts.length >= 3 &&
			['text-to-text', 'text-to-image', 'text-to-speech', 'speech-to-text', 'embeddings'].includes(
				parts[0]
			)
		) {
			return `${parts.slice(1).join('/')}:${file}`;
		}
		return `${normFirst}:${file}`;
	}

	// Check for double dash pattern e.g. "Qwen--Qwen2.5-GGUF--qwen2.5-1.5b-instruct"
	if (str.includes('--')) {
		const parts = str.split('--');
		if (parts.length >= 3) {
			return `${parts[0]}/${parts[1]}:${parts.slice(2).join('--')}`;
		} else if (parts.length === 2) {
			return `${parts[0]}:${parts[1]}`;
		}
	}

	// Check if string is a relative path: e.g. "text-to-text/Qwen/Qwen2.5-GGUF/qwen2.5-1.5b-instruct-q4_k_m.gguf"
	const norm = str.replace(/\\/g, '/').replace(/^\/+/, '');
	const parts = norm.split('/');
	if (parts.length >= 4) {
		return `${parts[1]}/${parts[2]}:${parts.slice(3).join('/')}`;
	} else if (parts.length === 3) {
		return `${parts[1]}:${parts[2]}`;
	}

	return str;
}

/**
 * Parses raw model identifiers or canonical "{user}/{model}:{filename}" strings
 * into a clean base model name and tag badges.
 *
 * ModelBadge displays: "{user}/{model} [tags]"
 *
 * Example:
 *   "Qwen/Qwen2.5-GGUF:qwen2.5-1.5b-instruct-q4_k_m.gguf"
 *   => { name: "Qwen/Qwen2.5-GGUF", tags: ["1.5B", "q4_k_m", "instruct"] }
 */
export function parseModelDetails(
	input: string | string[] | any | undefined | null
): ParsedModelDetails {
	if (!input) {
		return { name: '', tags: [] };
	}

	// If explicit array with > 1 element already pre-parsed: [name, ...tags]
	if (Array.isArray(input)) {
		if (input.length === 0) return { name: '', tags: [] };
		if (input.length > 1) {
			return {
				name: input[0],
				tags: input.slice(1)
			};
		}
		input = input[0];
	}

	const canonical = formatCanonicalModelString(input);
	if (!canonical) return { name: '', tags: [] };

	let baseName = '';
	let tagSource = '';

	if (canonical.includes(':')) {
		const colonIdx = canonical.indexOf(':');
		baseName = canonical.slice(0, colonIdx).trim();
		tagSource = canonical.slice(colonIdx + 1).trim();
	} else {
		baseName = canonical;
		tagSource = canonical;
	}

	// Remove file extensions (.gguf, .bin, .safetensors, .pt, .onnx)
	tagSource = tagSource.replace(/\.(gguf|bin|safetensors|pt|onnx)$/i, '');

	let sizeTag = '';
	let quantTag = '';
	let typeTag = '';

	// Size tag match (e.g., 0.5b, 7b, 1.5b, 70b, 1b, 3b, 8b, 14b, 32b, 72b)
	const sizeMatch = tagSource.match(/(?:^|[:\-_])(\d+(?:\.\d+)?[bmktBMKT])(?:[:\-_]|$)/);
	if (sizeMatch) {
		sizeTag = sizeMatch[1].toUpperCase();
		const prefix = sizeMatch[0][0];
		tagSource = tagSource.replace(sizeMatch[0], /[:\-_]/.test(prefix) ? prefix : '');
	}

	// Quantization tag match (e.g., q8_0, q4_k_m, q4_k_s, q5_k_m, q4_0, q5_0, f16, f32, bf16, fp16, fp32, int8, int4, q8_k, q8, q4, q5, q6)
	const quantMatch = tagSource.match(
		/(?:^|[:\-_])(q\d+(?:_[a-z0-9_]+|[kK]_[sSmMlL]|_k)?|f16|f32|bf16|fp16|fp32|int8|int4)(?:[:\-_]|$)/i
	);
	if (quantMatch) {
		quantTag = quantMatch[1];
		const prefix = quantMatch[0][0];
		tagSource = tagSource.replace(quantMatch[0], /[:\-_]/.test(prefix) ? prefix : '');
	}

	// Type/variant tag match (e.g., instruct, chat, coder, code, math, vision, base, it, rlhf, distill, thinking, lite, mtp)
	const typeMatch = tagSource.match(
		/(?:^|[:\-_])(instruct|chat|coder|code|math|vision|base|it|rlhf|distill|thinking|lite|mtp)(?:[:\-_]|$)/i
	);
	if (typeMatch) {
		typeTag = typeMatch[1].toLowerCase();
		const prefix = typeMatch[0][0];
		tagSource = tagSource.replace(typeMatch[0], /[:\-_]/.test(prefix) ? prefix : '');
	}

	// If no colon was provided in the input, baseName might also contain size/quant/type tags that should be cleaned up
	if (!canonical.includes(':')) {
		let cleanedBase = baseName.replace(/\.(gguf|bin|safetensors|pt|onnx)$/i, '');
		if (sizeMatch) cleanedBase = cleanedBase.replace(sizeMatch[0], '');
		if (quantMatch) cleanedBase = cleanedBase.replace(quantMatch[0], '');
		if (typeMatch) cleanedBase = cleanedBase.replace(typeMatch[0], '');
		cleanedBase = cleanedBase.replace(/^[:\-_.]+|[:\-_.]+$|:\-$|:$/g, '').trim();
		if (cleanedBase) baseName = cleanedBase;
	}

	const tags: string[] = [];
	if (sizeTag) tags.push(sizeTag);
	if (quantTag) tags.push(quantTag);
	if (typeTag) tags.push(typeTag);

	return { name: baseName, tags };
}

/**
 * Safely parses flags into a Record<string, string>, handling double or triple JSON encoded strings.
 */
export function parseFlags(flagsInput: any): Record<string, string> {
	if (!flagsInput) return {};
	let val = flagsInput;
	if (typeof val === 'object' && val !== null && 'flags' in val) {
		val = val.flags;
	}
	while (typeof val === 'string') {
		const trimmed = val.trim();
		if (!trimmed) return {};
		try {
			val = JSON.parse(trimmed);
		} catch {
			break;
		}
	}
	if (typeof val === 'object' && val !== null && !Array.isArray(val)) {
		return val as Record<string, string>;
	}
	return {};
}

/**
 * Extracts the primary model file path (-m or --model) from a model record, flags JSON string, or flags object.
 */
export function getMainModelFilePath(flagsOrModel: any): string {
	if (!flagsOrModel) return '';
	const obj = parseFlags(flagsOrModel);
	if (Object.keys(obj).length === 0) return '';

	const primaryKeys = [
		'--model',
		'-m',
		'--weights',
		'-w',
		'--model-path',
		'--checkpoint',
		'model',
		'm'
	];

	for (const key of primaryKeys) {
		const val = obj[key];
		if (typeof val === 'string' && val.trim()) {
			return val.trim();
		}
	}

	for (const val of Object.values(obj)) {
		if (typeof val === 'string') {
			const lower = val.toLowerCase().trim();
			if (
				lower.endsWith('.gguf') ||
				lower.endsWith('.safetensors') ||
				lower.endsWith('.bin') ||
				lower.endsWith('.pt') ||
				lower.endsWith('.onnx')
			) {
				return val.trim();
			}
		}
	}

	return '';
}

/**
 * Normalizes file path from file records
 */
export function getFilePath(f: any): string {
	return f?.absolute_path || f?.path || f?.relative_path || f?.filename || f?.name || '';
}

/**
 * Normalizes relative file path from file records
 */
export function getFileRelativePath(f: any): string {
	return f?.relative_path || f?.filename || f?.name || f?.absolute_path || f?.path || '';
}

/**
 * Resolves a full file path given a name or relative path against a list of available files
 */
export function resolveFullPath(val: string, availableFiles?: any[]): string {
	if (!val || typeof val !== 'string') return val || '';
	if (!availableFiles || !Array.isArray(availableFiles) || availableFiles.length === 0) return val;

	const found = availableFiles.find((f: any) => {
		const abs = f?.absolute_path || f?.path;
		const rel = f?.relative_path || f?.filename || f?.name;
		const stem = f?.name;
		return (
			(abs && (abs === val || abs.endsWith('/' + val))) ||
			(rel && (rel === val || rel.endsWith('/' + val))) ||
			(stem && (stem === val || val.includes(stem)))
		);
	});

	return found ? getFilePath(found) : val;
}

/**
 * Converts a flags dictionary to a CLI custom text string.
 */
export function flagsToCustomText(
	flagsObj: Record<string, string>,
	resolver?: (val: string) => string
): string {
	const parts: string[] = [];
	for (const [key, val] of Object.entries(flagsObj)) {
		if (!key) continue;
		if (val === 'true') {
			parts.push(key);
		} else if (val === 'false') {
			// omitted
		} else if (val !== undefined && val !== '') {
			const fullVal = resolver ? resolver(val) : val;
			if (fullVal.includes(' ')) {
				parts.push(`${key} "${fullVal}"`);
			} else {
				parts.push(`${key} ${fullVal}`);
			}
		}
	}
	return parts.join(' ');
}

/**
 * Converts a CLI custom text string to a flags dictionary.
 */
export function customTextToFlags(
	text: string,
	resolver?: (val: string) => string
): Record<string, string> {
	const flagsObj: Record<string, string> = {};
	if (!text.trim()) return flagsObj;

	const regex = /"([^"\\]*(?:\\.[^"\\]*)*)"|'([^'\\]*(?:\\.[^'\\]*)*)'|(\S+)/g;
	const tokens: string[] = [];
	let match;

	while ((match = regex.exec(text)) !== null) {
		tokens.push(match[1] ?? match[2] ?? match[3]);
	}

	let i = 0;
	while (i < tokens.length) {
		const token = tokens[i];
		if (token.startsWith('-')) {
			const key = token;
			let val = 'true';
			if (i + 1 < tokens.length && !tokens[i + 1].startsWith('-')) {
				val = resolver ? resolver(tokens[i + 1]) : tokens[i + 1];
				i++;
			}
			flagsObj[key] = val;
		}
		i++;
	}

	return flagsObj;
}

/**
 * Extracts the full model name from a file path without directory and without extension.
 * e.g. "/path/to/models/qwen2.5-coder-7b-instruct-q4_k_m.gguf" => "qwen2.5-coder-7b-instruct-q4_k_m"
 */
export function extractCleanModelName(filePath: string): string {
	if (!filePath) return '';
	const fileName = filePath.split(/[/\\]/).pop() || filePath;
	const lastDotIndex = fileName.lastIndexOf('.');
	if (lastDotIndex > 0) {
		return fileName.slice(0, lastDotIndex).trim();
	}
	return fileName.trim();
}

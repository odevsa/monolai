/**
 * Model parsing, flag formatting and model inspection utilities
 */

export interface ParsedModelDetails {
	name: string;
	tags: string[];
}

/**
 * Parses raw model identifiers, paths, or existing tag arrays into a clean base
 * model name and a set of tag badges (e.g. parameter size, quantization, instruction type).
 *
 * Example:
 *   "qwen2.5-0.5b-instruct-q8_0.gguf"
 *   => { name: "qwen2.5", tags: ["0.5B", "q8_0", "instruct"] }
 */
export function parseModelDetails(input: string | string[] | undefined | null): ParsedModelDetails {
	if (!input) {
		return { name: '', tags: [] };
	}

	let str = '';
	if (Array.isArray(input)) {
		if (input.length === 0) return { name: '', tags: [] };
		if (input.length > 1) {
			return {
				name: input[0],
				tags: input.slice(1)
			};
		}
		str = input[0];
	} else {
		str = input;
	}

	if (!str) return { name: '', tags: [] };

	// Remove file extensions (.gguf, .bin, .safetensors, .pt, .onnx)
	let cleaned = str.replace(/\.(gguf|bin|safetensors|pt|onnx)$/i, '');

	// Extract runtime if formatted as "runtime/model"
	if (cleaned.includes('/') && !cleaned.startsWith('/') && !cleaned.startsWith('.')) {
		cleaned = cleaned.split('/')[1] || cleaned;
	}

	let sizeTag = '';
	let quantTag = '';
	let typeTag = '';

	// Size tag match (e.g., 0.5b, 7b, 1.5b, 70b, 1b, 3b, 8b, 14b, 32b, 72b)
	const sizeMatch = cleaned.match(/(?:^|[:\-_])(\d+(?:\.\d+)?[bmktBMKT])(?:[:\-_]|$)/);
	if (sizeMatch) {
		sizeTag = sizeMatch[1].toUpperCase();
		const prefix = sizeMatch[0][0];
		cleaned = cleaned.replace(sizeMatch[0], /[:\-_]/.test(prefix) ? prefix : '');
	}

	// Quantization tag match (e.g., q8_0, q4_k_m, q4_k_s, q5_k_m, q4_0, q5_0, f16, f32, bf16, fp16, fp32, int8, int4, q8_k, q8, q4, q5, q6)
	const quantMatch = cleaned.match(
		/(?:^|[:\-_])(q\d+(?:_[a-z0-9_]+|[kK]_[sSmMlL]|_k)?|f16|f32|bf16|fp16|fp32|int8|int4)(?:[:\-_]|$)/i
	);
	if (quantMatch) {
		quantTag = quantMatch[1];
		const prefix = quantMatch[0][0];
		cleaned = cleaned.replace(quantMatch[0], /[:\-_]/.test(prefix) ? prefix : '');
	}

	// Type/variant tag match (e.g., instruct, chat, coder, code, math, vision, base, it, rlhf, distill, thinking, lite, mtp)
	const typeMatch = cleaned.match(
		/(?:^|[:\-_])(instruct|chat|coder|code|math|vision|base|it|rlhf|distill|thinking|lite|mtp)(?:[:\-_]|$)/i
	);
	if (typeMatch) {
		typeTag = typeMatch[1].toLowerCase();
		const prefix = typeMatch[0][0];
		cleaned = cleaned.replace(typeMatch[0], /[:\-_]/.test(prefix) ? prefix : '');
	}

	// Clean up remaining string for base name
	let name = cleaned.replace(/^[:\-_.]+|[:\-_.]+$|:\-$|:$/g, '').trim();
	name = name.replace(/[:\-_]{2,}/g, '-');
	if (!name) name = str;

	const tags: string[] = [];
	if (sizeTag) tags.push(sizeTag);
	if (quantTag) tags.push(quantTag);
	if (typeTag) tags.push(typeTag);

	return { name, tags };
}

/**
 * Extracts the primary model file path (-m or --model) from a model record or flags JSON string.
 */
export function getMainModelFilePath(flagsOrModel: string | { flags?: string }): string {
	try {
		const rawFlags = typeof flagsOrModel === 'string' ? flagsOrModel : flagsOrModel?.flags || '{}';
		const parsed = JSON.parse(rawFlags);
		return parsed['-m'] || parsed['--model'] || '';
	} catch {
		return '';
	}
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

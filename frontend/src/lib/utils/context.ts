/**
 * Context window, token estimation, and sliding window management utilities
 */
import type { ChatAttachment } from '$lib/types/chat';

export const DEFAULT_SYSTEM_PROMPT =
	'You are a helpful, respectful, and precise AI assistant. Provide clear, accurate, and direct answers while maintaining context across the conversation. Format code, tables, and lists neatly when requested.';

export interface ReasoningPreset {
	id: 'default' | 'off' | 'low' | 'medium' | 'high' | 'max';
	label: string;
	desc: string;
	maxTokens: number | null;
}

export const REASONING_PRESETS: ReasoningPreset[] = [
	{ id: 'default', label: 'Default', desc: 'Model default', maxTokens: null },
	{ id: 'off', label: 'Off', desc: '', maxTokens: 0 },
	{ id: 'low', label: 'Low', desc: 'Max 512 tokens', maxTokens: 512 },
	{ id: 'medium', label: 'Medium', desc: 'Max 2,048 tokens', maxTokens: 2048 },
	{ id: 'high', label: 'High', desc: 'Max 8,192 tokens', maxTokens: 8192 },
	{ id: 'max', label: 'Max', desc: 'Unlimited', maxTokens: null }
];

/**
 * Fast and accurate token estimation (~3.8 chars per token for multilingual text and code)
 */
export function estimateTokens(text: string | null | undefined): number {
	if (!text) return 0;
	return Math.max(1, Math.ceil(text.length / 3.8));
}

/**
 * Formats token counts into human readable numbers (e.g. 431 -> "431", 4100 -> "4.10K")
 */
export function formatTokenCount(tokens: number): string {
	if (tokens < 1000) return String(tokens);
	return (tokens / 1000).toFixed(2).replace(/\.00$/, '') + 'K';
}

/**
 * Extracts context window limit from model flags or defaults to 4096 (4.10K)
 */
export function getModelContextLimit(flagsJson?: string | null): number {
	if (!flagsJson) return 4096;
	try {
		const parsed = JSON.parse(flagsJson);
		const cVal = parsed['-c'] || parsed['--ctx-size'];
		if (cVal) {
			const num = parseInt(cVal, 10);
			if (!isNaN(num) && num > 0) return num;
		}
	} catch {
		// Fallback to default
	}
	return 4096;
}

export interface ChatMessageLike {
	role: 'user' | 'assistant' | 'system';
	content: string;
	status?: string;
	tokens?: number;
	duration?: string;
	speed?: string;
	attachments?: ChatAttachment[];
}

export interface TokenUsageDetails {
	// Across all turns
	allPromptEvaluated: number; // Prompt tokens evaluated (fresh)
	allPromptCached: number; // Reused from KV cache
	allTokensGenerated: number; // Tokens generated

	// This turn · KV cache
	thisTurnPrompt: number; // Prompt tokens (fresh + cached)
	thisTurnFresh: number; // Fresh prompt tokens
	thisTurnCached: number; // Cached prompt tokens
	thisTurnGenerated: number; // Generated tokens in this turn
	kvCacheTotal: number; // KV cache total (prompt + generated)

	// Speed
	avgSpeed: string; // E.g. "33.2t/s"
}

export interface PreparedContext {
	messages: { role: string; content: string | any[] }[];
	usedTokens: number;
	maxTokens: number;
	percentage: number;
	remainingTokens: number;
	systemTokens: number;
	historyTokens: number;
	usageDetails: TokenUsageDetails;
}

/**
 * Builds conversation context using the industry-standard Sliding Window algorithm:
 * 1. Anchors the System Prompt at index 0.
 * 2. Anchors the latest User message at the end.
 * 3. Filters out corrupted/errored/empty messages.
 * 4. Iteratively fills previous conversation turns (User + Assistant pairs) from newest to oldest
 *    until reaching the model's safe input budget.
 * 5. Computes exact turn and session KV cache token usage metrics matching the inspector details.
 */
export function prepareSlidingWindowContext(
	messages: ChatMessageLike[],
	systemPrompt: string = DEFAULT_SYSTEM_PROMPT,
	maxContextLimit: number = 4096,
	reserveOutputTokens: number = 1024,
	liveMetrics?: Partial<TokenUsageDetails>
): PreparedContext {
	const systemTokens = estimateTokens(systemPrompt);
	const safeInputBudget = Math.max(512, maxContextLimit - reserveOutputTokens);

	// Filter valid dialogue messages
	const cleanMessages: {
		role: 'user' | 'assistant';
		content: string | any[];
		tokens: number;
		speed?: string;
		duration?: string;
	}[] = [];
	for (const m of messages) {
		if (m.role !== 'user' && m.role !== 'assistant') continue;
		const rawContent = m.content?.trim();
		const hasAttachments = m.attachments && m.attachments.length > 0;
		if (!rawContent && !hasAttachments) continue;
		if (m.status === 'ERROR' || rawContent?.startsWith('⚠️')) continue;
		if (rawContent === '(Cancelled by user)') continue;

		// Estimate extra tokens from attachments
		let attachmentTokens = 0;
		if (m.attachments) {
			for (const att of m.attachments) {
				if (att.type === 'image') {
					attachmentTokens += 800;
				} else if (att.type === 'audio') {
					attachmentTokens += 500;
				} else if (att.type === 'text' && att.textContent) {
					attachmentTokens += estimateTokens(att.textContent);
				}
			}
		}

		const baseTokens = m.tokens && m.tokens > 0 ? m.tokens : estimateTokens(rawContent) + 4;
		const totalTokens = baseTokens + attachmentTokens;

		// Format content with attachments
		let formattedContent: string | any[] = rawContent || '';
		if (hasAttachments && m.attachments) {
			let textWithDocs = rawContent || '';
			const textFiles = m.attachments.filter((a) => a.type === 'text' && a.textContent);
			if (textFiles.length > 0) {
				const docBlocks = textFiles
					.map(
						(f) =>
							`--- Attached File: ${f.name} ---\n\`\`\`\n${f.textContent}\n\`\`\`\n--- End of ${f.name} ---`
					)
					.join('\n\n');
				textWithDocs = textWithDocs ? `${docBlocks}\n\n${textWithDocs}` : docBlocks;
			}

			const images = m.attachments.filter((a) => a.type === 'image' && a.dataUrl);
			const audios = m.attachments.filter((a) => a.type === 'audio' && a.dataUrl);

			if (images.length > 0 || audios.length > 0) {
				const parts: any[] = [];
				if (textWithDocs) {
					parts.push({ type: 'text', text: textWithDocs });
				}
				for (const img of images) {
					parts.push({
						type: 'image_url',
						image_url: { url: img.dataUrl }
					});
				}
				for (const aud of audios) {
					const base64Data = aud.dataUrl.split(',')[1] || '';
					const format = aud.mimeType?.includes('mp3') ? 'mp3' : 'wav';
					parts.push({
						type: 'input_audio',
						input_audio: {
							data: base64Data,
							format
						}
					});
				}
				formattedContent = parts;
			} else {
				formattedContent = textWithDocs;
			}
		}

		cleanMessages.push({
			role: m.role,
			content: formattedContent,
			tokens: totalTokens,
			speed: m.speed,
			duration: m.duration
		});
	}

	if (cleanMessages.length === 0) {
		const defaultDetails: TokenUsageDetails = {
			allPromptEvaluated: liveMetrics?.allPromptEvaluated ?? 0,
			allPromptCached: liveMetrics?.allPromptCached ?? 0,
			allTokensGenerated: liveMetrics?.allTokensGenerated ?? 0,
			thisTurnPrompt: liveMetrics?.thisTurnPrompt ?? 0,
			thisTurnFresh: liveMetrics?.thisTurnFresh ?? 0,
			thisTurnCached: liveMetrics?.thisTurnCached ?? 0,
			thisTurnGenerated: liveMetrics?.thisTurnGenerated ?? 0,
			kvCacheTotal: liveMetrics?.kvCacheTotal ?? 0,
			avgSpeed: liveMetrics?.avgSpeed ?? '0.0t/s'
		};

		return {
			messages:
				systemPrompt && systemPrompt.trim()
					? [{ role: 'system', content: systemPrompt.trim() }]
					: [],
			usedTokens: 0,
			maxTokens: maxContextLimit,
			percentage: 0,
			remainingTokens: maxContextLimit,
			systemTokens: 0,
			historyTokens: 0,
			usageDetails: defaultDetails
		};
	}

	// Always keep the latest user message
	const latest = cleanMessages[cleanMessages.length - 1];
	const preservedPrevious: typeof cleanMessages = [];
	let currentTokens = systemTokens + latest.tokens;

	// Traverse history backwards (from newest past message to oldest)
	for (let i = cleanMessages.length - 2; i >= 0; i--) {
		const msg = cleanMessages[i];
		if (currentTokens + msg.tokens <= safeInputBudget) {
			preservedPrevious.unshift(msg);
			currentTokens += msg.tokens;
		} else {
			// Budget reached: slide window (drop oldest turns)
			break;
		}
	}

	const finalMessages: { role: string; content: string | any[] }[] = [];
	if (systemPrompt && systemPrompt.trim()) {
		finalMessages.push({ role: 'system', content: systemPrompt.trim() });
	}

	for (const m of preservedPrevious) {
		finalMessages.push({ role: m.role, content: m.content });
	}
	finalMessages.push({ role: latest.role, content: latest.content });

	const totalUsed = currentTokens;

	// Calculate KV cache metrics across turns and this turn
	let computedAllGenerated = 0;
	let computedAllPromptEvaluated = 0;
	let computedAllPromptCached = 0;

	// Sum historical assistant outputs
	for (const m of cleanMessages) {
		if (m.role === 'assistant') {
			computedAllGenerated += m.tokens;
		}
	}

	// Determine this turn's prompt breakdown
	let thisTurnCached = systemTokens;
	let thisTurnFresh = latest.tokens;
	for (const m of preservedPrevious) {
		thisTurnCached += m.tokens;
	}

	// In multi-turn chat with KV cache, earlier turns are cached
	computedAllPromptEvaluated = thisTurnFresh;
	computedAllPromptCached = thisTurnCached;

	// If the latest message was an assistant response, count it in this turn's generated
	let thisTurnGenerated = 0;
	if (latest.role === 'assistant') {
		thisTurnGenerated = latest.tokens;
	}

	const thisTurnPrompt = thisTurnFresh + thisTurnCached;
	const computedKvTotal = thisTurnPrompt + thisTurnGenerated;

	// Compute avgSpeed: prefer active liveMetrics, or fallback to assistant history
	let computedAvgSpeed = liveMetrics?.avgSpeed;
	if (
		!computedAvgSpeed ||
		computedAvgSpeed === '0.0t/s' ||
		computedAvgSpeed === '0 t/s' ||
		computedAvgSpeed === '0.0 t/s'
	) {
		const assistantMsgs = cleanMessages.filter((m) => m.role === 'assistant');
		for (let i = assistantMsgs.length - 1; i >= 0; i--) {
			const m = assistantMsgs[i];
			if (m.speed && m.speed !== '0 t/s' && m.speed !== '0.0t/s' && m.speed !== '0.0 t/s') {
				computedAvgSpeed = m.speed;
				break;
			}
			if (m.tokens && m.duration) {
				const sec = parseFloat(m.duration);
				if (!isNaN(sec) && sec > 0) {
					computedAvgSpeed = `${(m.tokens / sec).toFixed(1)} t/s`;
					break;
				}
			}
		}
	}
	if (!computedAvgSpeed) {
		computedAvgSpeed = '0.0t/s';
	}

	const usageDetails: TokenUsageDetails = {
		allPromptEvaluated: liveMetrics?.allPromptEvaluated ?? computedAllPromptEvaluated,
		allPromptCached: liveMetrics?.allPromptCached ?? computedAllPromptCached,
		allTokensGenerated: liveMetrics?.allTokensGenerated ?? computedAllGenerated,
		thisTurnPrompt: liveMetrics?.thisTurnPrompt ?? thisTurnPrompt,
		thisTurnFresh: liveMetrics?.thisTurnFresh ?? thisTurnFresh,
		thisTurnCached: liveMetrics?.thisTurnCached ?? thisTurnCached,
		thisTurnGenerated: liveMetrics?.thisTurnGenerated ?? thisTurnGenerated,
		kvCacheTotal: liveMetrics?.kvCacheTotal ?? computedKvTotal,
		avgSpeed: computedAvgSpeed
	};

	const effectiveUsed = usageDetails.kvCacheTotal > 0 ? usageDetails.kvCacheTotal : totalUsed;
	const pct = Math.min(100, Math.round((effectiveUsed / maxContextLimit) * 100));

	return {
		messages: finalMessages,
		usedTokens: effectiveUsed,
		maxTokens: maxContextLimit,
		percentage: pct,
		remainingTokens: Math.max(0, maxContextLimit - effectiveUsed),
		systemTokens,
		historyTokens: totalUsed - systemTokens,
		usageDetails
	};
}

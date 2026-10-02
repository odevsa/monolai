import { Marked, type Token } from 'marked';
import DOMPurify from 'dompurify';

export interface MarkdownBlock {
	id: string;
	type: 'code' | 'html';
	code?: string;
	lang?: string;
	html?: string;
}

const markedInstance = new Marked({
	breaks: true,
	gfm: true
});

markedInstance.use({
	renderer: {
		link({ href, title, text }) {
			const titleAttr = title ? ` title="${title}"` : '';
			return `<a href="${href}" target="_blank" rel="noopener noreferrer"${titleAttr}>${text}</a>`;
		}
	}
});

export function sanitizeHtml(rawHtml: string): string {
	if (!rawHtml) return '';
	if (typeof window !== 'undefined' && DOMPurify.sanitize) {
		return DOMPurify.sanitize(rawHtml, {
			ADD_ATTR: ['target', 'rel'],
			ADD_TAGS: ['iframe']
		});
	}
	return rawHtml;
}

export function parseMarkdownToBlocks(content: string): MarkdownBlock[] {
	if (!content || !content.trim()) {
		return [];
	}

	try {
		const tokens = markedInstance.lexer(content);
		const blocks: MarkdownBlock[] = [];
		let currentTokens: Token[] = [];
		let blockIndex = 0;

		const flushHtmlTokens = () => {
			if (currentTokens.length > 0) {
				const rawHtml = markedInstance.parser(currentTokens);
				const cleanHtml = sanitizeHtml(rawHtml);
				blocks.push({
					id: `html-${blockIndex++}`,
					type: 'html',
					html: cleanHtml
				});
				currentTokens = [];
			}
		};

		for (const token of tokens) {
			if (token.type === 'code') {
				flushHtmlTokens();
				blocks.push({
					id: `code-${blockIndex++}`,
					type: 'code',
					code: token.text,
					lang: token.lang || ''
				});
			} else {
				currentTokens.push(token);
			}
		}

		flushHtmlTokens();

		return blocks;
	} catch (err) {
		console.error('Failed to parse markdown blocks:', err);
		return [
			{
				id: 'fallback-0',
				type: 'html',
				html: sanitizeHtml(content)
			}
		];
	}
}

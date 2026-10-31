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

export function stripMarkdownFrontmatter(content: string): string {
	if (!content) return '';
	return content.replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n?/, '');
}

export function resolveMarkdownRelativeUrls(content: string, baseUrl: string): string {
	if (!content || !baseUrl) return content;
	const trimmedBase = baseUrl.replace(/\/+$/, '');

	const isRelative = (url: string) => {
		if (!url) return false;
		const trimmed = url.trim();
		return !/^(?:https?:|\/\/|data:|mailto:|#|javascript:)/i.test(trimmed);
	};

	const resolve = (url: string) => {
		if (!isRelative(url)) return url;
		const clean = url
			.trim()
			.replace(/^(\.\.?\/)+/, '')
			.replace(/^\/+/, '');
		return `${trimmedBase}/${clean}`;
	};

	// 1. Markdown images: ![alt](url "optional title")
	let result = content.replace(
		/!\[([^\]]*)\]\(([^)\s]+)(?:\s+("[^"]*"|'[^']*'))?\)/g,
		(_match, alt, url, title) => {
			const resolved = resolve(url);
			return title ? `![${alt}](${resolved} ${title})` : `![${alt}](${resolved})`;
		}
	);

	// 2. Markdown links: [text](url "optional title") (negative lookbehind for !)
	result = result.replace(
		/(?<!!)\[([^\]]+)\]\(([^)\s]+)(?:\s+("[^"]*"|'[^']*'))?\)/g,
		(_match, text, url, title) => {
			const resolved = resolve(url);
			return title ? `[${text}](${resolved} ${title})` : `[${text}](${resolved})`;
		}
	);

	// 3. HTML tags with src="..." or src='...'
	result = result.replace(
		/(<(?:img|video|audio|source)\b[^>]*?\bsrc=["'])([^"'>]+)(["'])/gi,
		(_match, prefix, url, suffix) => {
			return `${prefix}${resolve(url)}${suffix}`;
		}
	);

	// 4. HTML a tags with href="..." or href='...'
	result = result.replace(
		/(<a\b[^>]*?\bhref=["'])([^"'>]+)(["'])/gi,
		(_match, prefix, url, suffix) => {
			return `${prefix}${resolve(url)}${suffix}`;
		}
	);

	return result;
}

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

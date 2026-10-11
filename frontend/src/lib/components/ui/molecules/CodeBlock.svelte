<script lang="ts">
	import { copyToClipboard } from '$lib/utils/clipboard';
	import { highlightCode, normalizeLanguage } from '$lib/utils/prism';
	import {
		Check,
		Code as CodeIcon,
		Copy,
		ExternalLink,
		Eye,
		Moon,
		RotateCcw,
		Sun
	} from '@lucide/svelte';

	let { code = '', lang = '' }: { code?: string; lang?: string } = $props();

	let containerRef = $state<HTMLDivElement | null>(null);
	let isCopied = $state(false);
	let viewMode = $state<'code' | 'preview'>('code');
	let previewCanvasTheme = $state<'light' | 'dark'>('light');
	let iframeRefreshKey = $state(0);
	let copyTimeout: ReturnType<typeof setTimeout> | null = null;

	const langInfo = $derived(normalizeLanguage(lang));
	const isHtml = $derived(langInfo.isHtml);
	const highlighted = $derived(highlightCode(code, lang));
	const lineCount = $derived(code ? code.split('\n').length : 0);
	const isLongBlock = $derived(lineCount > 8);

	function switchMode(mode: 'code' | 'preview', event?: Event) {
		if (event) {
			event.preventDefault();
			event.stopPropagation();
		}
		viewMode = mode;
	}

	async function handleCopy() {
		if (!code) return;
		const success = await copyToClipboard(code);
		if (success) {
			isCopied = true;
			if (copyTimeout) clearTimeout(copyTimeout);
			copyTimeout = setTimeout(() => {
				isCopied = false;
			}, 2000);
		}
	}

	function handleRefreshPreview() {
		iframeRefreshKey++;
	}

	function handleOpenInNewTab() {
		if (!code) return;
		const fullHtml = buildPreviewDocument(code, previewCanvasTheme);
		const blob = new Blob([fullHtml], { type: 'text/html;charset=utf-8' });
		const url = URL.createObjectURL(blob);
		window.open(url, '_blank');
	}

	function buildPreviewDocument(snippet: string, theme: 'light' | 'dark'): string {
		const isFullDocument = /<!doctype\s+html/i.test(snippet) || /<html[\s>]/i.test(snippet);

		if (isFullDocument) {
			return snippet;
		}

		const bg = theme === 'dark' ? '#18181b' : '#ffffff';
		const text = theme === 'dark' ? '#f4f4f5' : '#09090b';

		return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <style>
    *, *::before, *::after { box-sizing: border-box; }
    body {
      margin: 0;
      padding: 16px;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      color: ${text};
      background-color: ${bg};
      line-height: 1.45;
    }
  </style>
</head>
<body>
  ${snippet}
</body>
</html>`;
	}

	const previewSrcDoc = $derived(buildPreviewDocument(code, previewCanvasTheme));
</script>

<div bind:this={containerRef} class="code-block-container">
	<!-- Static Header on Left (Transparent, No Background) -->
	<div class="code-block-header">
		<span class="code-lang-badge">{langInfo.name}</span>
	</div>

	<!-- Action Icons: Float along top-right during scroll only in code mode on long blocks (No Background) -->
	<div class="code-block-actions {viewMode === 'code' && isLongBlock ? 'sticky-icons' : ''}">
		{#if isHtml}
			<div class="view-mode-tabs" role="tablist">
				<button
					type="button"
					tabindex="-1"
					class="mode-icon-btn {viewMode === 'code' ? 'active' : ''}"
					onmousedown={(e) => switchMode('code', e)}
					onpointerdown={(e) => switchMode('code', e)}
					onclick={(e) => switchMode('code', e)}
					title="Code"
					aria-label="Code"
					role="tab"
					aria-selected={viewMode === 'code'}
				>
					<CodeIcon size={18} />
				</button>
				<button
					type="button"
					tabindex="-1"
					class="mode-icon-btn {viewMode === 'preview' ? 'active' : ''}"
					onmousedown={(e) => switchMode('preview', e)}
					onpointerdown={(e) => switchMode('preview', e)}
					onclick={(e) => switchMode('preview', e)}
					title="Preview"
					aria-label="Preview"
					role="tab"
					aria-selected={viewMode === 'preview'}
				>
					<Eye size={18} />
				</button>
			</div>
		{/if}

		<button
			type="button"
			tabindex="-1"
			class="copy-icon-btn {isCopied ? 'copied' : ''}"
			onmousedown={(e) => {
				e.preventDefault();
			}}
			onpointerdown={(e) => {
				e.preventDefault();
			}}
			onclick={handleCopy}
			title={isCopied ? 'Copied!' : 'Copy code'}
			aria-label={isCopied ? 'Copied!' : 'Copy code'}
		>
			{#if isCopied}
				<Check size={18} class="check-icon" />
			{:else}
				<Copy size={18} />
			{/if}
		</button>
	</div>

	{#if viewMode === 'code' || !isHtml}
		<div class="code-block-body">
			<pre class="code-pre"><code class="language-{langInfo.id}">{@html highlighted}</code></pre>
		</div>
	{:else}
		<div class="preview-container">
			<div class="preview-toolbar">
				<div class="preview-info">
					<span class="preview-dot"></span>
					<span class="preview-label">Interactive Preview</span>
				</div>
				<div class="preview-actions">
					<button
						type="button"
						class="preview-tool-btn"
						title={previewCanvasTheme === 'light'
							? 'Switch canvas to dark'
							: 'Switch canvas to light'}
						aria-label={previewCanvasTheme === 'light'
							? 'Switch canvas to dark'
							: 'Switch canvas to light'}
						onclick={() => (previewCanvasTheme = previewCanvasTheme === 'light' ? 'dark' : 'light')}
					>
						{#if previewCanvasTheme === 'light'}
							<Moon size={15} />
							<span class="tool-label">Dark</span>
						{:else}
							<Sun size={15} />
							<span class="tool-label">Light</span>
						{/if}
					</button>
					<button
						type="button"
						class="preview-tool-btn"
						title="Reload preview"
						aria-label="Reload preview"
						onclick={handleRefreshPreview}
					>
						<RotateCcw size={15} />
					</button>
					<button
						type="button"
						class="preview-tool-btn"
						title="Open preview in new tab"
						aria-label="Open preview in new tab"
						onclick={handleOpenInNewTab}
					>
						<ExternalLink size={15} />
					</button>
				</div>
			</div>

			<div class="preview-frame-wrapper {previewCanvasTheme}">
				{#key iframeRefreshKey}
					<iframe
						title="HTML Preview"
						class="preview-iframe"
						sandbox="allow-scripts allow-modals"
						srcdoc={previewSrcDoc}
					></iframe>
				{/key}
			</div>
		</div>
	{/if}
</div>

<style>
	.code-block-container {
		position: relative;
		display: grid;
		grid-template-columns: 1fr auto;
		grid-template-rows: auto 1fr;
		margin: 0.85rem 0;
		background-color: var(--bg-surface);
		border-radius: 0.625rem;
		border: none;
		box-shadow: none;
	}

	/* Static Header: Just the language badge on the left, inside the card, completely transparent */
	.code-block-header {
		grid-column: 1;
		grid-row: 1;
		display: flex;
		align-items: center;
		min-height: 34px;
		padding: 0.75rem 0.75rem;
		background: transparent !important;
		border: none !important;
	}

	.code-lang-badge {
		font-family: inherit;
		font-size: 0.75rem;
		font-weight: 600;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--text-secondary);
	}

	/* Action Icons: Sticky along top-right, transparent background */
	.code-block-actions {
		grid-column: 2;
		grid-row: 1 / span 2;
		display: flex;
		align-items: center;
		gap: 0.35rem;
		padding: 0.5rem 0.75rem;
		justify-self: end;
		align-self: start;
		z-index: 25;
		background: transparent !important;
	}

	.code-block-actions.sticky-icons {
		position: sticky;
		top: 0.5rem;
		margin-bottom: 3.5rem;
	}

	/* View Mode Tabs & Buttons: No background on any of them */
	.view-mode-tabs {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		background: transparent !important;
		border: none !important;
		padding: 0 !important;
	}

	.mode-icon-btn,
	.copy-icon-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 30px;
		height: 30px;
		padding: 0;
		border-radius: 0.25rem;
		background: transparent !important;
		border: none !important;
		box-shadow: none !important;
		color: var(--text-secondary);
		cursor: pointer;
		user-select: none;
		touch-action: manipulation;
		transition: color 0.15s ease;
	}

	.mode-icon-btn:hover,
	.copy-icon-btn:hover {
		color: var(--text-primary);
		background: transparent !important;
	}

	/* Active state: Only icon color changes, NO background */
	.mode-icon-btn.active {
		color: var(--primary) !important;
		background: transparent !important;
	}

	/* Copied state: Only icon color changes to green, NO background */
	.copy-icon-btn.copied {
		color: #10b981 !important;
		background: transparent !important;
	}

	.mode-icon-btn :global(svg),
	.copy-icon-btn :global(svg) {
		pointer-events: none;
	}

	/* Code Body */
	.code-block-body {
		grid-column: 1 / span 2;
		grid-row: 2;
		position: relative;
		padding: 0 0.5rem 0.65rem 0.5rem;
		background: transparent !important;
		border-radius: 0 0 0.625rem 0.625rem;
	}

	/* Compact font size and line height (no vertical scroll on codeblock) */
	.code-pre {
		margin: 0;
		padding: 0.25rem 0.65rem 0.35rem 0.65rem;
		overflow-x: auto;
		font-family:
			ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', monospace;
		font-size: 0.8125rem;
		line-height: 1.45;
		color: var(--text-primary);
		white-space: pre;
		word-break: normal;
		tab-size: 2;
	}

	.code-pre code {
		display: block;
		font-family: inherit;
		background: transparent !important;
		padding: 0 !important;
		border: none !important;
		border-radius: 0 !important;
	}

	/* Preview Container & Frame (static, does not float) */
	.preview-container {
		grid-column: 1 / span 2;
		grid-row: 2;
		position: relative;
		display: flex;
		flex-direction: column;
		background-color: var(--bg-surface);
		border-radius: 0.625rem;
		overflow: hidden;
	}

	.preview-toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.35rem 0.75rem;
		background-color: var(--bg-surface-hover);
		border-bottom: 1px solid var(--border-color);
		font-size: 0.75rem;
	}

	.preview-info {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		color: var(--text-secondary);
	}

	.preview-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background-color: #10b981;
		box-shadow: 0 0 6px rgba(16, 185, 129, 0.6);
	}

	.preview-label {
		font-size: 0.75rem;
		font-weight: 500;
	}

	.preview-actions {
		display: flex;
		align-items: center;
		gap: 0.35rem;
	}

	.preview-tool-btn {
		display: inline-flex;
		align-items: center;
		gap: 0.3rem;
		padding: 0.25rem 0.45rem;
		border-radius: 0.25rem;
		background: transparent;
		border: 1px solid transparent;
		color: var(--text-secondary);
		cursor: pointer;
		font-size: 0.7rem;
		transition: all 0.15s ease;
	}

	.preview-tool-btn:hover {
		background-color: var(--bg-hover);
		color: var(--text-primary);
		border-color: var(--border-color);
	}

	.tool-label {
		font-size: 0.7rem;
	}

	.preview-frame-wrapper {
		position: relative;
		width: 100%;
		min-height: 240px;
		height: 340px;
		resize: vertical;
		overflow: hidden;
		transition: background-color 0.2s ease;
	}

	.preview-frame-wrapper.light {
		background-color: #ffffff;
	}

	.preview-frame-wrapper.dark {
		background-color: #18181b;
	}

	.preview-iframe {
		width: 100%;
		height: 100%;
		border: none;
		display: block;
	}

	/* Theme-Adaptive Syntax Highlighting */
	:global(.token.comment),
	:global(.token.prolog),
	:global(.token.doctype),
	:global(.token.cdata) {
		color: var(--syntax-comment, #71717a);
		font-style: italic;
	}

	:global(.token.punctuation) {
		color: var(--syntax-punctuation, #a1a1aa);
	}

	:global(.token.property),
	:global(.token.tag),
	:global(.token.boolean),
	:global(.token.number),
	:global(.token.constant),
	:global(.token.symbol),
	:global(.token.deleted) {
		color: var(--syntax-tag, #f43f5e);
	}

	:global(.token.selector),
	:global(.token.attr-name),
	:global(.token.string),
	:global(.token.char),
	:global(.token.builtin),
	:global(.token.inserted) {
		color: var(--syntax-string, #10b981);
	}

	:global(.token.operator),
	:global(.token.entity),
	:global(.token.url),
	:global(.language-css .token.string),
	:global(.style .token.string) {
		color: var(--syntax-operator, #38bdf8);
	}

	:global(.token.atrule),
	:global(.token.attr-value),
	:global(.token.keyword) {
		color: var(--syntax-keyword, #818cf8);
		font-weight: 500;
	}

	:global(.token.function),
	:global(.token.class-name) {
		color: var(--syntax-function, #fbbf24);
	}

	:global(.token.regex),
	:global(.token.important),
	:global(.token.variable) {
		color: var(--syntax-variable, #f97316);
	}

	/* Light Theme Syntax Tokens */
	:global([data-theme='light']) {
		--syntax-comment: #6b7280;
		--syntax-punctuation: #4b5563;
		--syntax-tag: #dc2626;
		--syntax-string: #059669;
		--syntax-operator: #0284c7;
		--syntax-keyword: #4f46e5;
		--syntax-function: #b45309;
		--syntax-variable: #c2410c;
	}

	/* VS Code Dark+ Theme */
	:global([data-theme='code']) {
		--syntax-comment: #6a9955;
		--syntax-punctuation: #d4d4d4;
		--syntax-tag: #569cd6;
		--syntax-string: #ce9178;
		--syntax-operator: #d4d4d4;
		--syntax-keyword: #569cd6;
		--syntax-function: #dcdcaa;
		--syntax-variable: #9cdcfe;
	}

	/* Dracula Theme */
	:global([data-theme='dracula']) {
		--syntax-comment: #6272a4;
		--syntax-punctuation: #f8f8f2;
		--syntax-tag: #ff79c6;
		--syntax-string: #f1fa8c;
		--syntax-operator: #ff79c6;
		--syntax-keyword: #ff79c6;
		--syntax-function: #50fa7b;
		--syntax-variable: #bd93f9;
	}

	/* Catppuccin Theme */
	:global([data-theme='catppuccin']) {
		--syntax-comment: #6c7086;
		--syntax-punctuation: #bac2de;
		--syntax-tag: #f38ba8;
		--syntax-string: #a6e3a1;
		--syntax-operator: #89dceb;
		--syntax-keyword: #cba6f7;
		--syntax-function: #89b4fa;
		--syntax-variable: #fab387;
	}

	/* Nord Theme */
	:global([data-theme='nord']) {
		--syntax-comment: #616e88;
		--syntax-punctuation: #d8dee9;
		--syntax-tag: #81a1c1;
		--syntax-string: #a3be8c;
		--syntax-operator: #81a1c1;
		--syntax-keyword: #81a1c1;
		--syntax-function: #88c0d0;
		--syntax-variable: #b48ead;
	}

	/* Monokai Theme */
	:global([data-theme='monokai']) {
		--syntax-comment: #727072;
		--syntax-punctuation: #fcfcfa;
		--syntax-tag: #ff6188;
		--syntax-string: #ffd866;
		--syntax-operator: #ff6188;
		--syntax-keyword: #ff6188;
		--syntax-function: #a9dc76;
		--syntax-variable: #78dce8;
	}
</style>

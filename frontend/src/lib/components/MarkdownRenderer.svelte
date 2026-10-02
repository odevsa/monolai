<script lang="ts">
	import CodeBlock from '$lib/components/CodeBlock.svelte';
	import { parseMarkdownToBlocks } from '$lib/utils/markdown';
	import { copyToClipboard } from '$lib/utils/clipboard';

	let { content = '' }: { content?: string } = $props();

	const blocks = $derived(parseMarkdownToBlocks(content));

	/**
	 * Action to enhance any nested code blocks inside lists, blockquotes, etc.
	 * giving them the copy icon button and HTML preview button.
	 */
	function enhanceNestedCodeBlocks(node: HTMLElement) {
		function processNode() {
			const preElements = node.querySelectorAll('pre');
			preElements.forEach((pre) => {
				if (pre.dataset.enhanced) return;
				pre.dataset.enhanced = 'true';

				const codeEl = pre.querySelector('code');
				if (!codeEl) return;

				const rawCode = codeEl.textContent || '';
				const classList = Array.from(codeEl.classList);
				const langClass = classList.find((c) => c.startsWith('language-'));
				const lang = langClass ? langClass.replace('language-', '').toLowerCase() : 'text';
				const isHtml = lang === 'html' || lang === 'htm';

				// Wrapper
				const wrapper = document.createElement('div');
				wrapper.className = 'nested-code-block-wrapper';

				// Header (sticky in code mode)
				const header = document.createElement('div');
				header.className = 'nested-code-header';

				const langBadge = document.createElement('span');
				langBadge.className = 'nested-code-lang';
				langBadge.textContent = lang.toUpperCase();
				header.appendChild(langBadge);

				const actionsDiv = document.createElement('div');
				actionsDiv.className = 'nested-code-actions';

				let previewPane: HTMLDivElement | null = null;
				let isPreviewActive = false;

				if (isHtml) {
					const previewBtn = document.createElement('button');
					previewBtn.type = 'button';
					previewBtn.className = 'nested-btn nested-preview-btn';
					previewBtn.innerHTML = `
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0"/><circle cx="12" cy="12" r="3"/></svg>
					`;
					previewBtn.title = 'View HTML preview';
					previewBtn.setAttribute('aria-label', 'View HTML preview');

					const togglePreview = () => {
						isPreviewActive = !isPreviewActive;
						if (isPreviewActive) {
							wrapper.classList.add('is-preview');
							if (!previewPane) {
								previewPane = document.createElement('div');
								previewPane.className = 'nested-preview-pane';
								const iframe = document.createElement('iframe');
								iframe.className = 'nested-preview-iframe';
								iframe.sandbox.add('allow-scripts', 'allow-modals');
								iframe.srcdoc = `<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><style>body{margin:0;padding:12px;font-family:sans-serif;color:#18181b;background:#fff;line-height:1.45;}</style></head><body>${rawCode}</body></html>`;
								previewPane.appendChild(iframe);
								wrapper.appendChild(previewPane);
							}
							previewPane.style.display = 'block';
							pre.style.display = 'none';
							previewBtn.classList.add('active');
						} else {
							wrapper.classList.remove('is-preview');
							if (previewPane) previewPane.style.display = 'none';
							pre.style.display = 'block';
							previewBtn.classList.remove('active');
						}
					};

					previewBtn.tabIndex = -1;
					previewBtn.onmousedown = (e) => {
						e.preventDefault();
						togglePreview();
					};
					previewBtn.onpointerdown = (e) => {
						e.preventDefault();
						togglePreview();
					};
					previewBtn.onclick = (e) => {
						e.preventDefault();
						togglePreview();
					};

					actionsDiv.appendChild(previewBtn);
				}

				// Copy Button (icon only, size 18)
				const copyBtn = document.createElement('button');
				copyBtn.type = 'button';
				copyBtn.tabIndex = -1;
				copyBtn.className = 'nested-btn nested-copy-btn';
				copyBtn.innerHTML = `
					<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
				`;
				copyBtn.title = 'Copy code';
				copyBtn.setAttribute('aria-label', 'Copy code');

				copyBtn.onmousedown = (e) => {
					e.preventDefault();
				};

				copyBtn.onpointerdown = (e) => {
					e.preventDefault();
				};

				copyBtn.onclick = async () => {
					const ok = await copyToClipboard(rawCode);
					if (ok) {
						copyBtn.classList.add('copied');
						copyBtn.title = 'Copied!';
						copyBtn.setAttribute('aria-label', 'Copied!');
						copyBtn.innerHTML = `
							<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="#10b981" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"/></svg>
						`;
						setTimeout(() => {
							copyBtn.classList.remove('copied');
							copyBtn.title = 'Copy code';
							copyBtn.setAttribute('aria-label', 'Copy code');
							copyBtn.innerHTML = `
								<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
							`;
						}, 2000);
					}
				};

				actionsDiv.appendChild(copyBtn);
				header.appendChild(actionsDiv);

				const lineCount = rawCode.split('\n').length;
				if (lineCount > 8) {
					wrapper.classList.add('early-stop-header');
				}

				pre.parentNode?.insertBefore(wrapper, pre);
				wrapper.appendChild(header);
				wrapper.appendChild(pre);
			});
		}

		processNode();

		const observer = new MutationObserver(() => {
			processNode();
		});

		observer.observe(node, { childList: true, subtree: true });

		return {
			destroy() {
				observer.disconnect();
			}
		};
	}
</script>

<div class="markdown-container">
	{#each blocks as block (block.id)}
		{#if block.type === 'code'}
			<CodeBlock code={block.code || ''} lang={block.lang || ''} />
		{:else if block.html}
			<div class="markdown-prose" use:enhanceNestedCodeBlocks>
				{@html block.html}
			</div>
		{/if}
	{/each}
</div>

<style>
	.markdown-container {
		display: flex;
		flex-direction: column;
		width: 100%;
		font-size: 1rem;
		line-height: 1.65;
		color: var(--text-primary);
		word-break: break-word;
	}

	.markdown-prose {
		width: 100%;
	}

	/* Paragraphs */
	:global(.markdown-prose > p) {
		margin-top: 0.65em;
		margin-bottom: 0.65em;
	}

	:global(.markdown-prose > p:first-child) {
		margin-top: 0;
	}

	:global(.markdown-prose > p:last-child) {
		margin-bottom: 0;
	}

	/* Headings */
	:global(.markdown-prose h1),
	:global(.markdown-prose h2),
	:global(.markdown-prose h3),
	:global(.markdown-prose h4),
	:global(.markdown-prose h5),
	:global(.markdown-prose h6) {
		color: var(--text-primary);
		font-weight: 600;
		line-height: 1.35;
		margin-top: 1.25em;
		margin-bottom: 0.5em;
	}

	:global(.markdown-prose h1:first-child),
	:global(.markdown-prose h2:first-child),
	:global(.markdown-prose h3:first-child) {
		margin-top: 0;
	}

	:global(.markdown-prose h1) {
		font-size: 1.5rem;
		border-bottom: 1px solid var(--border-color);
		padding-bottom: 0.3em;
	}

	:global(.markdown-prose h2) {
		font-size: 1.3rem;
		border-bottom: 1px solid var(--border-color);
		padding-bottom: 0.25em;
	}

	:global(.markdown-prose h3) {
		font-size: 1.15rem;
	}

	:global(.markdown-prose h4) {
		font-size: 1.05rem;
	}

	:global(.markdown-prose h5),
	:global(.markdown-prose h6) {
		font-size: 0.95rem;
		color: var(--text-secondary);
	}

	/* Lists */
	:global(.markdown-prose ul),
	:global(.markdown-prose ol) {
		padding-left: 1.5rem;
		margin-top: 0.5em;
		margin-bottom: 0.5em;
	}

	:global(.markdown-prose ul) {
		list-style-type: disc;
	}

	:global(.markdown-prose ol) {
		list-style-type: decimal;
	}

	:global(.markdown-prose li) {
		margin-top: 0.25em;
		margin-bottom: 0.25em;
	}

	:global(.markdown-prose li > p) {
		margin-top: 0.25em;
		margin-bottom: 0.25em;
	}

	:global(.markdown-prose input[type='checkbox']) {
		margin-right: 0.5rem;
		accent-color: var(--primary);
	}

	/* Blockquotes */
	:global(.markdown-prose blockquote) {
		margin: 0.85em 0;
		padding: 0.5em 1em;
		border-left: 3px solid var(--primary);
		background-color: var(--primary-light);
		border-radius: 0 0.375rem 0.375rem 0;
		color: var(--text-secondary);
	}

	:global(.markdown-prose blockquote > p) {
		margin: 0.3em 0;
	}

	/* Inline Code */
	:global(.markdown-prose :not(pre) > code) {
		font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
		font-size: 0.88em;
		padding: 0.15rem 0.35rem;
		border-radius: 0.3rem;
		background-color: var(--bg-hover);
		border: 1px solid var(--border-color);
		color: var(--primary);
	}

	/* Links */
	:global(.markdown-prose a) {
		color: var(--primary);
		text-decoration: underline;
		text-underline-offset: 2px;
		transition: color 0.15s ease;
	}

	:global(.markdown-prose a:hover) {
		color: var(--primary-hover);
	}

	/* Tables */
	:global(.markdown-prose table) {
		width: 100%;
		border-collapse: collapse;
		margin: 1em 0;
		font-size: 0.9rem;
		overflow-x: auto;
		display: block;
	}

	:global(.markdown-prose th),
	:global(.markdown-prose td) {
		padding: 0.5rem 0.75rem;
		border: 1px solid var(--border-color);
		text-align: left;
	}

	:global(.markdown-prose th) {
		background-color: var(--bg-hover);
		font-weight: 600;
		color: var(--text-primary);
	}

	:global(.markdown-prose tr:nth-child(even)) {
		background-color: var(--bg-surface-hover);
	}

	/* Horizontal Rule */
	:global(.markdown-prose hr) {
		border: none;
		height: 1px;
		background-color: var(--border-color);
		margin: 1.25em 0;
	}

	/* Strong & Emphasis */
	:global(.markdown-prose strong),
	:global(.markdown-prose b) {
		font-weight: 600;
		color: var(--text-primary);
	}

	/* Nested Code Block Wrapper Styles */
	:global(.nested-code-block-wrapper) {
		position: relative;
		margin: 0.75rem 0;
		border-radius: 0.625rem;
		border: none;
		background-color: var(--bg-surface);
		box-shadow: none;
	}

	/* Static Header: Transparent background, no border */
	:global(.nested-code-header) {
		display: flex;
		align-items: center;
		justify-content: space-between;
		min-height: 34px;
		padding: 0.4rem 0.75rem 0.15rem 0.75rem;
		background: transparent !important;
		border: none !important;
		box-shadow: none !important;
	}

	:global(.nested-code-lang) {
		font-family: inherit;
		font-weight: 600;
		color: var(--text-secondary);
	}

	:global(.nested-code-actions) {
		display: flex;
		align-items: center;
		gap: 0.35rem;
		background: transparent !important;
	}

	/* Sticky icons in code mode: stops before bottom */
	:global(.nested-code-block-wrapper:not(.is-preview) .nested-code-actions) {
		position: sticky;
		top: 0.5rem;
		margin-bottom: 3.5rem;
		z-index: 25;
	}

	:global(.nested-btn) {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 30px;
		height: 30px;
		border-radius: 0.25rem;
		border: none !important;
		background: transparent !important;
		box-shadow: none !important;
		color: var(--text-secondary);
		cursor: pointer;
		user-select: none;
		touch-action: manipulation;
		transition: color 0.15s ease;
	}

	:global(.nested-btn svg) {
		pointer-events: none;
	}

	:global(.nested-btn:hover) {
		color: var(--text-primary);
		background: transparent !important;
	}

	/* Active state: Only icon color changes, NO background */
	:global(.nested-btn.active) {
		color: var(--primary) !important;
		background: transparent !important;
	}

	/* Copied state: Only icon color changes to green, NO background */
	:global(.nested-btn.copied) {
		color: #10b981 !important;
		background: transparent !important;
	}

	:global(.nested-preview-pane) {
		width: 100%;
		height: 240px;
		background: #ffffff;
		border-bottom-left-radius: 0.625rem;
		border-bottom-right-radius: 0.625rem;
		overflow: hidden;
	}

	:global(.nested-preview-iframe) {
		width: 100%;
		height: 100%;
		border: none;
	}

	:global(.nested-code-block-wrapper pre) {
		margin: 0;
		padding: 0.25rem 0.75rem 0.65rem 0.75rem;
		overflow-x: auto;
		font-family:
			ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', monospace;
		font-size: 0.8125rem;
		line-height: 1.45;
		background-color: transparent !important;
		border-bottom-left-radius: 0.625rem;
		border-bottom-right-radius: 0.625rem;
	}
</style>

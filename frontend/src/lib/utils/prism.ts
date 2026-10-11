import Prism from './prism-init';

// Import essential Prism language grammars
import 'prismjs/components/prism-markup.js';
import 'prismjs/components/prism-css.js';
import 'prismjs/components/prism-clike.js';
import 'prismjs/components/prism-javascript.js';
import 'prismjs/components/prism-typescript.js';
import 'prismjs/components/prism-python.js';
import 'prismjs/components/prism-bash.js';
import 'prismjs/components/prism-json.js';
import 'prismjs/components/prism-rust.js';
import 'prismjs/components/prism-sql.js';
import 'prismjs/components/prism-markdown.js';
import 'prismjs/components/prism-yaml.js';
import 'prismjs/components/prism-c.js';
import 'prismjs/components/prism-cpp.js';
import 'prismjs/components/prism-go.js';
import 'prismjs/components/prism-java.js';

export interface LanguageInfo {
	id: string;
	name: string;
	isHtml: boolean;
	prismLang?: string;
}

const LANGUAGE_MAP: Record<string, LanguageInfo> = {
	html: { id: 'html', name: 'HTML', isHtml: true, prismLang: 'markup' },
	htm: { id: 'htm', name: 'HTML', isHtml: true, prismLang: 'markup' },
	xml: { id: 'xml', name: 'XML', isHtml: false, prismLang: 'markup' },
	svg: { id: 'svg', name: 'SVG', isHtml: true, prismLang: 'markup' },
	javascript: { id: 'javascript', name: 'JavaScript', isHtml: false, prismLang: 'javascript' },
	js: { id: 'js', name: 'JavaScript', isHtml: false, prismLang: 'javascript' },
	typescript: { id: 'typescript', name: 'TypeScript', isHtml: false, prismLang: 'typescript' },
	ts: { id: 'ts', name: 'TypeScript', isHtml: false, prismLang: 'typescript' },
	python: { id: 'python', name: 'Python', isHtml: false, prismLang: 'python' },
	py: { id: 'py', name: 'Python', isHtml: false, prismLang: 'python' },
	rust: { id: 'rust', name: 'Rust', isHtml: false, prismLang: 'rust' },
	rs: { id: 'rs', name: 'Rust', isHtml: false, prismLang: 'rust' },
	bash: { id: 'bash', name: 'Bash', isHtml: false, prismLang: 'bash' },
	sh: { id: 'sh', name: 'Shell', isHtml: false, prismLang: 'bash' },
	shell: { id: 'shell', name: 'Shell', isHtml: false, prismLang: 'bash' },
	zsh: { id: 'zsh', name: 'Zsh', isHtml: false, prismLang: 'bash' },
	json: { id: 'json', name: 'JSON', isHtml: false, prismLang: 'json' },
	css: { id: 'css', name: 'CSS', isHtml: false, prismLang: 'css' },
	scss: { id: 'scss', name: 'SCSS', isHtml: false, prismLang: 'css' },
	sql: { id: 'sql', name: 'SQL', isHtml: false, prismLang: 'sql' },
	markdown: { id: 'markdown', name: 'Markdown', isHtml: false, prismLang: 'markdown' },
	md: { id: 'md', name: 'Markdown', isHtml: false, prismLang: 'markdown' },
	yaml: { id: 'yaml', name: 'YAML', isHtml: false, prismLang: 'yaml' },
	yml: { id: 'yml', name: 'YAML', isHtml: false, prismLang: 'yaml' },
	c: { id: 'c', name: 'C', isHtml: false, prismLang: 'c' },
	cpp: { id: 'cpp', name: 'C++', isHtml: false, prismLang: 'cpp' },
	go: { id: 'go', name: 'Go', isHtml: false, prismLang: 'go' },
	java: { id: 'java', name: 'Java', isHtml: false, prismLang: 'java' }
};

export function escapeHtml(unsafe: string): string {
	return unsafe
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;')
		.replace(/'/g, '&#039;');
}

export function normalizeLanguage(lang?: string): LanguageInfo {
	if (!lang) {
		return { id: 'text', name: 'Text', isHtml: false };
	}

	const cleaned = lang.trim().toLowerCase().split(/\s+/)[0];
	if (cleaned in LANGUAGE_MAP) {
		return LANGUAGE_MAP[cleaned];
	}

	return {
		id: cleaned,
		name: cleaned.toUpperCase(),
		isHtml: cleaned === 'html' || cleaned === 'htm'
	};
}

export function highlightCode(code: string, lang?: string): string {
	if (!code) return '';

	const info = normalizeLanguage(lang);
	const targetGrammar = info.prismLang || info.id;

	if (targetGrammar && Prism.languages[targetGrammar]) {
		try {
			return Prism.highlight(code, Prism.languages[targetGrammar], targetGrammar);
		} catch (err) {
			console.warn('Prism highlight failed for lang:', lang, err);
		}
	}

	return escapeHtml(code);
}

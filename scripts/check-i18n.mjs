#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');
const frontendSrc = path.join(rootDir, 'frontend', 'src');
const localesDir = path.join(frontendSrc, 'lib', 'i18n', 'locales');
const enJsonPath = path.join(localesDir, 'en.json');

function getAllSourceFiles(dir) {
	let results = [];
	for (const file of fs.readdirSync(dir)) {
		const full = path.join(dir, file);
		const stat = fs.statSync(full);
		if (stat.isDirectory()) {
			if (file === 'node_modules' || file === '.svelte-kit') continue;
			results = results.concat(getAllSourceFiles(full));
		} else if (file.endsWith('.svelte') || file.endsWith('.ts')) {
			// Exclude locale JSON files themselves from usage scanning
			if (!full.includes(path.join('lib', 'i18n', 'locales'))) {
				results.push(full);
			}
		}
	}
	return results;
}

function flattenKeys(obj, prefix = '') {
	const keys = new Set();
	for (const [key, value] of Object.entries(obj)) {
		const fullKey = prefix ? `${prefix}.${key}` : key;
		if (value && typeof value === 'object' && !Array.isArray(value)) {
			for (const sub of flattenKeys(value, fullKey)) {
				keys.add(sub);
			}
		} else {
			keys.add(fullKey);
		}
	}
	return keys;
}

function escapeRegex(str) {
	return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

if (!fs.existsSync(enJsonPath)) {
	console.error(`Error: Canonical en.json not found at ${enJsonPath}`);
	process.exit(1);
}

const availableFiles = fs.existsSync(localesDir)
	? fs.readdirSync(localesDir).filter((f) => f.endsWith('.json'))
	: [];

const rawArg = process.argv[2]?.trim() || 'en';
const cleanArg = rawArg.endsWith('.json') ? rawArg.slice(0, -5) : rawArg;

// Find matching file (exact match first, then case-insensitive / normalized underscore-to-hyphen match)
let targetFileName = availableFiles.find(
	(f) => f === `${cleanArg}.json` || f.replace(/\.json$/, '') === cleanArg
);

if (!targetFileName) {
	const normalizedArg = cleanArg.toLowerCase().replace(/_/g, '-');
	targetFileName = availableFiles.find(
		(f) => f.replace(/\.json$/, '').toLowerCase().replace(/_/g, '-') === normalizedArg
	);
}

if (!targetFileName) {
	targetFileName = cleanArg.endsWith('.json') ? cleanArg : `${cleanArg}.json`;
}

const targetJsonPath = path.isAbsolute(targetFileName)
	? targetFileName
	: path.join(localesDir, targetFileName);

if (!fs.existsSync(targetJsonPath)) {
	console.error(`Error: Locale file not found at ${targetJsonPath}`);
	const available = availableFiles.map((f) => f.replace(/\.json$/, ''));
	console.error(`Available locales: ${available.join(', ')}`);
	process.exit(1);
}

const enContent = JSON.parse(fs.readFileSync(enJsonPath, 'utf8'));
const enKeys = flattenKeys(enContent);

const targetContent = JSON.parse(fs.readFileSync(targetJsonPath, 'utf8'));
const targetKeys = flattenKeys(targetContent);
const targetLocaleName = path.basename(targetJsonPath);
const isDefaultEn = targetLocaleName === 'en.json';

const sourceFiles = getAllSourceFiles(frontendSrc);

// 1. Scan for direct t('key') and t("key") calls
const tRegex = /\bt\(\s*['"]([a-zA-Z0-9_.-]+)['"]/g;
// 2. Scan for template literal t(`foo.${bar}.baz`) calls
const tTemplateRegex = /\bt\(\s*`([^`]+)`\s*\)/g;

const keyOccurrences = new Map();
const dynamicRegexes = [];
const allSourceContents = [];

for (const file of sourceFiles) {
	const relPath = path.relative(rootDir, file);
	const content = fs.readFileSync(file, 'utf8');
	allSourceContents.push(content);
	const lines = content.split('\n');

	lines.forEach((lineText, lineIdx) => {
		// Static t('key') or t("key")
		let match;
		tRegex.lastIndex = 0;
		while ((match = tRegex.exec(lineText)) !== null) {
			const key = match[1];
			if (!keyOccurrences.has(key)) {
				keyOccurrences.set(key, []);
			}
			keyOccurrences.get(key).push({
				file: relPath,
				line: lineIdx + 1
			});
		}

		// Dynamic t(`...`) calls
		tTemplateRegex.lastIndex = 0;
		while ((match = tTemplateRegex.exec(lineText)) !== null) {
			const template = match[1];
			const parts = template.split(/\$\{[^}]+\}/);
			const patternStr = '^' + parts.map(escapeRegex).join('[^.]+') + '$';
			try {
				dynamicRegexes.push(new RegExp(patternStr));
			} catch (_) {}
		}
	});
}

const joinedContents = allSourceContents.join('\n');

// Find missing keys (called with t() or defined in en.json but absent in target locale)
const missing = [];
for (const [key, locations] of keyOccurrences.entries()) {
	if (!targetKeys.has(key)) {
		missing.push({ key, locations });
	}
}

// When checking a non-English locale, also check if keys defined in canonical en.json are missing
if (!isDefaultEn) {
	for (const key of enKeys) {
		if (!targetKeys.has(key) && !keyOccurrences.has(key)) {
			missing.push({
				key,
				locations: [{ file: path.relative(rootDir, enJsonPath), line: '(canonical key in en.json)' }]
			});
		}
	}
}

// Find unused keys (defined in target file but not used in frontend source)
const unused = [];
for (const key of targetKeys) {
	// 1. Used in static t('key')
	if (keyOccurrences.has(key)) {
		continue;
	}
	// 2. Matches dynamic template literal regex
	if (dynamicRegexes.some((re) => re.test(key))) {
		continue;
	}
	// 3. Used as string literal in source (e.g. 'common.confirm' in config/menus)
	if (
		joinedContents.includes(`'${key}'`) ||
		joinedContents.includes(`"${key}"`) ||
		joinedContents.includes(`\`${key}\``)
	) {
		continue;
	}

	unused.push(key);
}

unused.sort();
missing.sort((a, b) => a.key.localeCompare(b.key));

console.log('='.repeat(70));
console.log(` Monolai i18n Verification & Audit (${targetLocaleName})`);
console.log('='.repeat(70));
console.log(`Scanned ${sourceFiles.length} source files.`);
console.log(`Found ${keyOccurrences.size} unique t() calls.`);
if (isDefaultEn) {
	console.log(`Defined ${targetKeys.size} keys in ${targetLocaleName}.\n`);
} else {
	console.log(`Defined ${targetKeys.size} keys in ${targetLocaleName} (canonical en.json has ${enKeys.size} keys).\n`);
}

// Section 1: Missing Keys
if (missing.length === 0) {
	console.log(' [1] Missing Keys: 0');
	console.log(
		isDefaultEn
			? `     ✓ All t() calls exist in ${targetLocaleName}!\n`
			: `     ✓ All keys exist in ${targetLocaleName}!\n`
	);
} else {
	console.log(` [1] Missing Keys: ${missing.length}`);
	console.log(
		isDefaultEn
			? `     ✗ The following keys are called in views but MISSING from ${targetLocaleName}:`
			: `     ✗ The following keys are MISSING from ${targetLocaleName}:`
	);
	for (const item of missing) {
		console.log(`     - "${item.key}"`);
		for (const loc of item.locations) {
			console.log(`         at ${loc.file}:${loc.line}`);
		}
	}
	console.log();
}

// Section 2: Unused Keys
if (unused.length === 0) {
	console.log(' [2] Unused Keys: 0');
	console.log(`     ✓ All keys defined in ${targetLocaleName} are actively used!\n`);
} else {
	console.log(` [2] Unused Keys: ${unused.length}`);
	console.log(`     ℹ The following keys are defined in ${targetLocaleName} but NOT used in code:`);
	for (const key of unused) {
		console.log(`     - "${key}"`);
	}
	console.log();
}

console.log('-'.repeat(70));
console.log(`Audit Summary: ${missing.length} missing, ${unused.length} unused.`);
console.log('='.repeat(70) + '\n');

if (missing.length > 0) {
	console.error(`ERROR: Missing translation keys detected in ${targetLocaleName}. Please add them.`);
	process.exit(1);
} else {
	process.exit(0);
}

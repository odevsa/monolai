import type { ModelFileItem } from '$lib/types/models';

export type ModelPickerFilter = 'all' | 'primary' | 'auxiliary';

export interface FolderNode {
	name: string;
	path: string;
	folders: FolderNode[];
	files: ModelFileItem[];
	totalFilesCount: number;
}

/**
 * Normalizes a relative path by converting Windows backslashes to forward slashes
 * and stripping leading/trailing slashes.
 */
export function normalizePath(p: string): string {
	return p.replace(/\\/g, '/').replace(/^\/+|\/+$/g, '');
}

/**
 * Builds a pruned directory tree containing only folders that have files
 * matching the given filterMode and search criteria.
 */
export function buildFolderTree(
	allFiles: ModelFileItem[],
	filterMode: ModelPickerFilter = 'all',
	searchQuery: string = ''
): FolderNode {
	const q = searchQuery.trim().toLowerCase();

	// Step 1: Filter files according to filterMode and search
	const matchingFiles = allFiles.filter((f) => {
		// Filter by primary / auxiliary mode
		if (filterMode === 'primary' && f.is_auxiliary) {
			return false;
		}
		if (filterMode === 'auxiliary' && !f.is_auxiliary) {
			return false;
		}

		// Filter by search query if present
		if (q) {
			const name = (f.name || f.filename || '').toLowerCase();
			const rel = (f.relative_path || '').toLowerCase();
			if (!name.includes(q) && !rel.includes(q)) {
				return false;
			}
		}

		return true;
	});

	// Step 2: Build tree structure bottom-up
	const root: FolderNode = {
		name: 'models',
		path: '',
		folders: [],
		files: [],
		totalFilesCount: 0
	};

	// Helper to find or create child folder
	function getOrCreateFolder(parent: FolderNode, name: string, fullPath: string): FolderNode {
		let child = parent.folders.find((f) => f.name === name);
		if (!child) {
			child = {
				name,
				path: fullPath,
				folders: [],
				files: [],
				totalFilesCount: 0
			};
			parent.folders.push(child);
		}
		return child;
	}

	for (const file of matchingFiles) {
		const rawPath = file.relative_path || file.filename || '';
		const normalized = normalizePath(rawPath);
		const segments = normalized.split('/').filter(Boolean);

		if (segments.length <= 1) {
			// File is directly in root
			root.files.push(file);
			root.totalFilesCount++;
		} else {
			// Traverse or create folders up to the parent directory of this file
			let current = root;
			let currentPath = '';

			for (let i = 0; i < segments.length - 1; i++) {
				const seg = segments[i];
				currentPath = currentPath ? `${currentPath}/${seg}` : seg;
				current = getOrCreateFolder(current, seg, currentPath);
			}

			current.files.push(file);
		}
	}

	// Step 3: Compute total counts recursively and prune empty folders (only keep folders with files)
	function pruneAndCount(node: FolderNode): number {
		let total = node.files.length;

		// Prune subfolders that end up with 0 matching files
		node.folders = node.folders.filter((sub) => {
			const subCount = pruneAndCount(sub);
			total += subCount;
			return subCount > 0;
		});

		// Sort subfolders and files alphabetically
		node.folders.sort((a, b) => a.name.localeCompare(b.name));
		node.files.sort((a, b) => {
			const nameA = a.name || a.filename || '';
			const nameB = b.name || b.filename || '';
			return nameA.localeCompare(nameB);
		});

		node.totalFilesCount = total;
		return total;
	}

	pruneAndCount(root);

	return root;
}

/**
 * Resolves a folder node from a path string or array of path segments.
 */
export function getNodeAtPath(
	root: FolderNode,
	pathOrSegments: string | string[]
): FolderNode | null {
	const segments = Array.isArray(pathOrSegments)
		? pathOrSegments.filter(Boolean)
		: normalizePath(pathOrSegments).split('/').filter(Boolean);

	if (segments.length === 0) {
		return root;
	}

	let current: FolderNode = root;
	for (const seg of segments) {
		const next = current.folders.find((f) => f.name === seg);
		if (!next) return null;
		current = next;
	}

	return current;
}

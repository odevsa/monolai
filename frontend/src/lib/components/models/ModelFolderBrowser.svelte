<script lang="ts">
	import { Badge, Button, EmptyState, Input } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ModelFileItem } from '$lib/types/models';
	import { formatBytes } from '$lib/utils/format';
	import {
		buildFolderTree,
		getNodeAtPath,
		type FolderNode,
		type ModelPickerFilter
	} from '$lib/utils/modelTree';
	import {
		Check,
		ChevronRight,
		FileBox,
		Folder,
		FolderOpen,
		HardDrive,
		Search,
		Undo2
	} from '@lucide/svelte';

	let {
		files = [],
		filterMode = 'all',
		showFilterControls = true,
		returnType = 'relative',
		selectedPath = $bindable(''),
		onSelect
	}: {
		files: ModelFileItem[];
		filterMode?: ModelPickerFilter;
		showFilterControls?: boolean;
		returnType?: 'relative' | 'absolute';
		selectedPath?: string;
		onSelect?: (path: string, file: ModelFileItem) => void;
	} = $props();

	let activeFilter = $state<ModelPickerFilter>('all');

	$effect(() => {
		activeFilter = filterMode;
	});
	let searchQuery = $state('');
	let currentSegments = $state<string[]>([]);
	let highlightedFile = $state<ModelFileItem | null>(null);

	// If selectedPath is already passed, initialize highlightedFile
	$effect(() => {
		if (selectedPath && !highlightedFile) {
			const found = files.find(
				(f) => f.relative_path === selectedPath || f.absolute_path === selectedPath
			);
			if (found) {
				highlightedFile = found;
			}
		}
	});

	let rootNode = $derived(buildFolderTree(files, activeFilter, searchQuery));

	let currentNode = $derived.by(() => {
		const node = getNodeAtPath(rootNode, currentSegments);
		if (!node && currentSegments.length > 0) {
			// If current path became invalid due to filtering/search, fallback safely
			return rootNode;
		}
		return node || rootNode;
	});

	let isAtRoot = $derived(currentSegments.length === 0);

	function navigateToSegment(index: number) {
		if (index < 0) {
			currentSegments = [];
		} else {
			currentSegments = currentSegments.slice(0, index + 1);
		}
	}

	function navigateUp() {
		if (currentSegments.length > 0) {
			currentSegments = currentSegments.slice(0, -1);
		}
	}

	function enterFolder(folderName: string) {
		currentSegments = [...currentSegments, folderName];
	}

	function getFilePath(file: ModelFileItem): string {
		if (returnType === 'absolute' && file.absolute_path) {
			return file.absolute_path;
		}
		return file.relative_path || file.filename || '';
	}

	function selectFile(file: ModelFileItem) {
		highlightedFile = file;
		selectedPath = getFilePath(file);
	}

	function confirmSelection() {
		if (!highlightedFile) return;
		const path = getFilePath(highlightedFile);
		selectedPath = path;
		if (onSelect) {
			onSelect(path, highlightedFile);
		}
	}
</script>

<div class="flex flex-col gap-3 w-full">
	<!-- Top Controls: Search Bar & Optional Filter Pills -->
	<div class="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-2">
		<!-- Search Input -->
		<div class="relative grow-1">
			<Input
				type="text"
				bind:value={searchQuery}
				placeholder={t('models.picker.searchPlaceholder')}
				size="sm"
				class="w-full"
			>
				{#snippet prefix()}
					<Search size={14} class="text-[var(--text-muted)]" />
				{/snippet}
			</Input>
		</div>

		<!-- Filter Mode Pills -->
		{#if showFilterControls}
			<div
				class="flex items-center gap-1 p-0.5 rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] shrink-0 self-start sm:self-auto"
			>
				<button
					type="button"
					class="px-2.5 py-1 text-[11px] font-medium rounded-md transition-colors cursor-pointer border-0 {activeFilter ===
					'all'
						? 'bg-[var(--primary)] text-[var(--primary-text,#fff)] font-semibold shadow-xs'
						: 'bg-transparent text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
					onclick={() => (activeFilter = 'all')}
				>
					{t('models.picker.filterAll')}
				</button>
				<button
					type="button"
					class="px-2.5 py-1 text-[11px] font-medium rounded-md transition-colors cursor-pointer border-0 {activeFilter ===
					'primary'
						? 'bg-[var(--primary)] text-[var(--primary-text,#fff)] font-semibold shadow-xs'
						: 'bg-transparent text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
					onclick={() => (activeFilter = 'primary')}
				>
					{t('models.picker.filterPrimary')}
				</button>
				<button
					type="button"
					class="px-2.5 py-1 text-[11px] font-medium rounded-md transition-colors cursor-pointer border-0 {activeFilter ===
					'auxiliary'
						? 'bg-[var(--primary)] text-[var(--primary-text,#fff)] font-semibold shadow-xs'
						: 'bg-transparent text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
					onclick={() => (activeFilter = 'auxiliary')}
				>
					{t('models.picker.filterAuxiliary')}
				</button>
			</div>
		{/if}
	</div>

	<!-- Breadcrumbs Bar -->
	<div
		class="flex items-center gap-1.5 px-3 py-2 rounded-lg bg-[var(--bg-primary)] border border-[var(--border-color)] text-xs text-[var(--text-secondary)] overflow-x-auto"
	>
		<!-- Up Button -->
		<Button
			variant="ghost"
			size="icon-xs"
			onclick={navigateUp}
			disabled={isAtRoot}
			title={t('models.picker.upOneLevel')}
			ariaLabel={t('models.picker.upOneLevel')}
		>
			<Undo2 size={13} class={isAtRoot ? 'opacity-30' : 'text-[var(--text-primary)]'} />
		</Button>

		<div class="h-4 w-px bg-[var(--border-color)] mx-0.5"></div>

		<!-- Root segment -->
		<button
			type="button"
			class="flex items-center gap-1 hover:text-[var(--primary)] font-medium transition-colors bg-transparent border-0 cursor-pointer p-0.5 rounded text-xs {isAtRoot
				? 'text-[var(--primary)] font-semibold'
				: 'text-[var(--text-secondary)]'}"
			onclick={() => navigateToSegment(-1)}
		>
			<HardDrive size={13} />
			<span>{t('models.picker.root')}</span>
		</button>

		<!-- Subfolder segments -->
		{#each currentSegments as seg, index}
			<ChevronRight size={12} class="text-[var(--text-muted)] shrink-0" />
			<button
				type="button"
				class="hover:text-[var(--primary)] transition-colors bg-transparent border-0 cursor-pointer p-0.5 rounded text-xs truncate max-w-[150px] {index ===
				currentSegments.length - 1
					? 'text-[var(--primary)] font-semibold'
					: 'text-[var(--text-secondary)]'}"
				onclick={() => navigateToSegment(index)}
				title={seg}
			>
				{seg}
			</button>
		{/each}
	</div>

	<!-- File & Folder Browser List View -->
	<div
		class="min-h-[280px] max-h-[380px] overflow-y-auto rounded-xl border border-[var(--border-color)] bg-[var(--bg-surface)] p-2 space-y-1"
	>
		{#if currentNode.folders.length === 0 && currentNode.files.length === 0}
			<div class="py-12">
				<EmptyState
					title={t('models.picker.noFiles')}
					description={t('models.noMatchingModelsDesc')}
					icon={FileBox}
				/>
			</div>
		{:else}
			<!-- Folders List -->
			{#each currentNode.folders as folder (folder.name)}
				<button
					type="button"
					class="w-full flex items-center justify-between p-2.5 rounded-lg text-left transition-colors cursor-pointer border-0 bg-transparent hover:bg-[var(--bg-surface-hover)] text-[var(--text-primary)] group"
					onclick={() => enterFolder(folder.name)}
				>
					<div class="flex items-center gap-2.5 min-w-0">
						<Folder
							size={16}
							class="text-[var(--primary)] shrink-0 group-hover:scale-105 transition-transform"
						/>
						<span class="text-xs font-semibold truncate">{folder.name}</span>
					</div>
					<div class="flex items-center gap-1.5 shrink-0">
						<span
							class="text-[10px] px-2 py-0.5 rounded-full bg-[var(--bg-primary)] border border-[var(--border-color)] text-[var(--text-muted)] font-mono"
						>
							{folder.totalFilesCount}
						</span>
						<ChevronRight size={13} class="text-[var(--text-muted)]" />
					</div>
				</button>
			{/each}

			<!-- Files List -->
			{#each currentNode.files as file (file.relative_path || file.filename)}
				{@const isSelected =
					highlightedFile?.relative_path === file.relative_path ||
					selectedPath === file.relative_path ||
					selectedPath === file.absolute_path}
				<button
					type="button"
					class="w-full flex items-center justify-between p-2.5 rounded-lg text-left transition-colors cursor-pointer border text-[var(--text-primary)] {isSelected
						? 'bg-[var(--primary-light)] border-[var(--primary)]'
						: 'bg-transparent border-transparent hover:bg-[var(--bg-surface-hover)]'}"
					onclick={() => selectFile(file)}
					ondblclick={confirmSelection}
				>
					<div class="flex items-center gap-2.5 min-w-0 pr-2">
						<FileBox
							size={16}
							class="shrink-0 {isSelected ? 'text-[var(--primary)]' : 'text-[var(--text-muted)]'}"
						/>
						<div class="min-w-0 flex flex-col">
							<span
								class="text-xs font-semibold truncate {isSelected ? 'text-[var(--primary)]' : ''}"
							>
								{file.filename || file.name}
							</span>
							{#if file.relative_path && file.relative_path !== file.filename}
								<span class="text-[10px] font-mono text-[var(--text-muted)] truncate">
									{file.relative_path}
								</span>
							{/if}
						</div>
					</div>

					<div class="flex items-center gap-2 shrink-0">
						<!-- Format or Auxiliary Badge -->
						{#if file.is_auxiliary}
							<Badge variant="warning" size="xs">
								{file.auxiliary_type?.toUpperCase() || 'AUX'}
							</Badge>
						{:else if file.format}
							<Badge variant="muted" size="xs">
								{file.format.toUpperCase()}
							</Badge>
						{/if}

						<!-- File Size -->
						<span class="text-[11px] font-mono text-[var(--text-secondary)] whitespace-nowrap">
							{formatBytes(file.size_bytes || file.size || 0)}
						</span>

						<!-- Selection Indicator -->
						{#if isSelected}
							<Check size={14} class="text-[var(--primary)]" />
						{/if}
					</div>
				</button>
			{/each}
		{/if}
	</div>

	<!-- Bottom Action Bar -->
	<div
		class="flex items-center justify-between pt-2 border-t border-[var(--border-color)] gap-2 text-xs"
	>
		<div class="min-w-0 truncate text-[var(--text-muted)] font-mono text-[11px]">
			{#if highlightedFile}
				<span class="text-[var(--text-primary)] font-medium">
					{getFilePath(highlightedFile)}
				</span>
			{:else}
				<span>{t('models.picker.selectFilePrompt')}</span>
			{/if}
		</div>

		<Button variant="primary" size="sm" disabled={!highlightedFile} onclick={confirmSelection}>
			<Check size={13} />
			<span>{t('models.picker.selectButton')}</span>
		</Button>
	</div>
</div>

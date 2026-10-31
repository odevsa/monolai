<script lang="ts">
	import { modelsApi } from '$lib/api/models';
	import { Modal, Spinner } from '$lib/components/ui';
	import { t } from '$lib/i18n';
	import type { ModelFileItem } from '$lib/types/models';
	import type { ModelPickerFilter } from '$lib/utils/modelTree';
	import { onMount } from 'svelte';
	import ModelFolderBrowser from './ModelFolderBrowser.svelte';

	let {
		open = false,
		title = '',
		filterMode = 'all',
		showFilterControls = true,
		returnType = 'relative',
		selectedPath = $bindable(''),
		onSelect,
		onClose
	}: {
		open: boolean;
		title?: string;
		filterMode?: ModelPickerFilter;
		showFilterControls?: boolean;
		returnType?: 'relative' | 'absolute';
		selectedPath?: string;
		onSelect: (path: string, file: ModelFileItem) => void;
		onClose: () => void;
	} = $props();

	let files = $state<ModelFileItem[]>([]);
	let isLoading = $state(false);

	async function loadFiles() {
		isLoading = true;
		try {
			files = await modelsApi.getAvailableFiles();
		} catch (err: unknown) {
			console.error('Failed to load available model files:', err);
		} finally {
			isLoading = false;
		}
	}

	$effect(() => {
		if (open && files.length === 0 && !isLoading) {
			loadFiles();
		}
	});

	function handleSelection(path: string, file: ModelFileItem) {
		selectedPath = path;
		onSelect(path, file);
		onClose();
	}
</script>

<Modal {open} title={title || t('models.picker.title')} onclose={onClose} maxWidth="2xl">
	{#if isLoading && files.length === 0}
		<div class="flex flex-col items-center justify-center py-16 gap-3 text-[var(--text-muted)]">
			<Spinner size={24} />
			<span class="text-xs">{t('common.loading')}</span>
		</div>
	{:else}
		<ModelFolderBrowser
			{files}
			{filterMode}
			{showFilterControls}
			{returnType}
			bind:selectedPath
			onSelect={handleSelection}
		/>
	{/if}
</Modal>

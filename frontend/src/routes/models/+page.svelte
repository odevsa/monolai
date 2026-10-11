<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import { modelsApi, runtimesApi } from '$lib/api';
	import {
		Alert,
		Badge,
		Button,
		EmptyState,
		ModelBadge,
		PageHeader,
		Select
	} from '$lib/components/ds';
	import ModelForm from '$lib/components/ModelForm.svelte';
	import { askConfirm, showAlert } from '$lib/confirmStore';
	import { refreshFeatures } from '$lib/featuresStore';
	import { t } from '$lib/i18n';
	import { runningModelsState } from '$lib/runningModelsStore';
	import type { ModelFileItem, ModelRecord, RunningModelStatus } from '$lib/types/models';
	import type { Runtime, RuntimeManifest } from '$lib/types/runtimes';
	import { copyToClipboard } from '$lib/utils/clipboard';
	import { formatBytes } from '$lib/utils/format';
	import { getMainModelFilePath } from '$lib/utils/model';
	import {
		AlertTriangle,
		ArrowLeft,
		Box,
		FileText,
		Pencil,
		Plus,
		Search,
		Trash2,
		X
	} from '@lucide/svelte';
	import { onMount } from 'svelte';

	let registeredModels = $state<ModelRecord[]>([]);
	let runtimes = $state<Runtime[]>([]);
	let availableFiles = $state<ModelFileItem[]>([]);
	let runtimeManifests = $state<RuntimeManifest[]>([]);
	let isLoadingData = $state(true);
	let error = $state<string | null>(null);

	// Filter and search states
	let searchQuery = $state('');
	let selectedRuntimeFilter = $state<string>('all');
	let selectedStatusFilter = $state<'all' | 'running' | 'missing'>('all');
	let copiedId = $state<string | null>(null);

	// Form editing state
	let isEditingModel = $state(false);
	let editingModel = $state<ModelRecord | null>(null);

	// Map of running models for quick lookup
	let runningMap = $derived.by(() => {
		const map = new Map<string, RunningModelStatus>();
		for (const rm of runningModelsState.runningModels) {
			map.set(rm.model_id, rm);
		}
		return map;
	});

	// Selectbox options
	let runtimeFilterOptions = $derived([
		{ value: 'all', label: t('models.allRuntimes') },
		...runtimes.map((rt) => ({ value: rt.id, label: rt.name }))
	]);

	let statusFilterOptions = $derived([
		{ value: 'all', label: t('models.allStatuses') },
		{ value: 'running', label: t('models.runningStatus') },
		{ value: 'missing', label: t('models.missingStatus') }
	]);

	// Filtered models list
	let filteredModels = $derived.by(() => {
		let list = registeredModels;

		const q = searchQuery.trim().toLowerCase();
		if (q) {
			list = list.filter((m) => {
				const mainFile = getMainModelFilePath(m).toLowerCase();
				return (
					m.id.toLowerCase().includes(q) ||
					mainFile.includes(q) ||
					m.runtime.toLowerCase().includes(q)
				);
			});
		}

		if (selectedRuntimeFilter !== 'all') {
			list = list.filter((m) => m.runtime.toLowerCase() === selectedRuntimeFilter.toLowerCase());
		}

		if (selectedStatusFilter === 'running') {
			list = list.filter((m) => runningMap.has(m.id));
		} else if (selectedStatusFilter === 'missing') {
			list = list.filter((m) => m.file_exists === false);
		}

		return list;
	});

	function getMainFile(flagsStr: string): string {
		try {
			const flags = JSON.parse(flagsStr);
			return flags.m || flags['--model'] || flags.model || '';
		} catch {
			return '';
		}
	}

	function getFileInfo(
		filePath: string,
		files: ModelFileItem[]
	): { fileName: string; sizeFormatted: string | null; format: string } {
		if (!filePath)
			return { fileName: 'No file configured', sizeFormatted: null, format: 'UNKNOWN' };

		const fileName = filePath.split('/').pop() || filePath;
		const found = files.find(
			(f) =>
				(f.name && f.name === fileName) ||
				(f.path && f.path.endsWith(fileName)) ||
				(f.relative_path && f.relative_path.endsWith(fileName))
		);

		const size = found?.size_bytes ?? found?.size;
		const sizeFormatted = size ? formatBytes(size) : null;

		let format = 'MODEL';
		if (fileName.toLowerCase().endsWith('.gguf')) format = 'GGUF';
		else if (fileName.toLowerCase().endsWith('.safetensors')) format = 'SAFETENSORS';
		else if (fileName.toLowerCase().endsWith('.bin')) format = 'BIN';

		return { fileName, sizeFormatted, format };
	}

	async function loadAllModelsData(isSilent = false) {
		if (!isSilent) isLoadingData = true;
		error = null;

		try {
			const [modelsRes, runtimesRes, filesRes, manifestsRes] = await Promise.all([
				modelsApi.list(true).catch(() => []),
				runtimesApi.list().catch(() => []),
				modelsApi.getAvailableFiles().catch(() => []),
				runtimesApi.getManifests().catch(() => [])
			]);

			registeredModels = modelsRes || [];
			runtimes = runtimesRes || [];
			availableFiles = filesRes || [];
			runtimeManifests = manifestsRes || [];
		} catch (e: any) {
			error = e.message || 'Failed to load models data from server';
			console.error('Error loading models data:', e);
		} finally {
			isLoadingData = false;
		}
	}

	afterNavigate(() => {
		isEditingModel = false;
		editingModel = null;
	});

	onMount(() => {
		loadAllModelsData();

		const handleNavModels = () => {
			if (isEditingModel) {
				isEditingModel = false;
				editingModel = null;
			}
		};

		window.addEventListener('monolai:nav-models', handleNavModels);
		return () => {
			window.removeEventListener('monolai:nav-models', handleNavModels);
		};
	});

	let containerRef = $state<HTMLDivElement | null>(null);

	function startAddModel() {
		editingModel = null;
		isEditingModel = true;
		(containerRef?.closest('main') || containerRef)?.scrollTo({ top: 0, behavior: 'smooth' });
	}

	function startEditModel(model: ModelRecord) {
		editingModel = model;
		isEditingModel = true;
		(containerRef?.closest('main') || containerRef)?.scrollTo({ top: 0, behavior: 'smooth' });
	}

	function cancelEdit() {
		isEditingModel = false;
		editingModel = null;
	}

	async function copyId(id: string) {
		const ok = await copyToClipboard(id);
		if (ok) {
			copiedId = id;
			setTimeout(() => {
				if (copiedId === id) copiedId = null;
			}, 2000);
		}
	}

	async function handleSaveModel(payload: {
		id: string;
		runtime: string;
		flags: Record<string, string>;
	}) {
		const isExisting = !!editingModel;
		if (isExisting) {
			await modelsApi.update(payload.id, {
				runtime: payload.runtime,
				flags: JSON.stringify(payload.flags)
			});
		} else {
			await modelsApi.create({
				id: payload.id,
				runtime: payload.runtime,
				flags: JSON.stringify(payload.flags)
			});
		}

		await loadAllModelsData(true);
		refreshFeatures(true);
		isEditingModel = false;
		editingModel = null;
	}

	async function deleteModel(id: string) {
		const confirmed = await askConfirm(
			t('models.deleteConfirmMsg', { id }),
			t('models.deleteConfirmTitle'),
			'danger',
			t('common.delete'),
			t('common.cancel')
		);
		if (!confirmed) return;

		try {
			await modelsApi.delete(id);
			await loadAllModelsData(true);
			refreshFeatures(true);
		} catch (e: any) {
			await showAlert(e.message || 'Error deleting model', 'Error', 'danger');
		}
	}
</script>

<div
	bind:this={containerRef}
	class="w-full px-4 sm:px-8 py-5 md:py-8 pt-16 md:pt-16 max-w-7xl mx-auto box-border"
>
	<div class="flex flex-col gap-5 sm:gap-6">
		<PageHeader title={t('models.pageTitle')} subtitle={t('models.pageSubtitle')} icon={Box}>
			{#if !isEditingModel}
				<Button variant="primary" size="md" onclick={startAddModel}>
					<Plus size={15} />
					<span>{t('models.registerModel')}</span>
				</Button>
			{:else}
				<Button variant="secondary" size="md" onclick={cancelEdit}>
					<ArrowLeft size={15} />
					<span>{t('common.back')}</span>
				</Button>
			{/if}
		</PageHeader>

		{#if error}
			<Alert variant="error" title="Backend Connection Failure:">
				<div class="flex items-center justify-between gap-3">
					<span>{error}</span>
					<Button variant="secondary" size="sm" onclick={() => loadAllModelsData()}>
						{t('common.refresh')}
					</Button>
				</div>
			</Alert>
		{/if}

		{#if isEditingModel}
			<!-- Dedicated Form Container -->
			<div class="bg-[var(--bg-surface)] rounded-2xl p-5 sm:p-7 shadow-sm">
				<div class="flex items-center gap-2.5 pb-4 mb-5 border-b border-[var(--border-color)]">
					<div
						class="w-8 h-8 rounded-xl bg-[var(--primary-light)] text-[var(--primary)] flex items-center justify-center border-0"
					>
						<Box size={16} />
					</div>
					<div>
						<h3 class="m-0 text-base font-bold text-[var(--text-primary)]">
							{editingModel
								? `${t('models.editModel')}: ${editingModel.id}`
								: t('models.registerModel')}
						</h3>
						<p class="m-0 text-xs text-[var(--text-muted)] mt-0.5">
							Configure weights file, runtime backend, and execution parameters.
						</p>
					</div>
				</div>

				<ModelForm
					{runtimes}
					{availableFiles}
					{runtimeManifests}
					{editingModel}
					onSave={handleSaveModel}
					onCancel={cancelEdit}
				/>
			</div>
		{:else}
			<!-- Search & Filter Controls Bar -->
			<div
				class="flex flex-col sm:flex-row items-stretch sm:items-center gap-2.5 p-2 bg-[var(--bg-surface)] rounded-xl shadow-xs"
			>
				<!-- Search Input -->
				<div class="relative flex-1 min-w-[180px]">
					<Search
						size={14}
						class="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-muted)] pointer-events-none"
					/>
					<input
						type="text"
						bind:value={searchQuery}
						placeholder={t('models.searchPlaceholder')}
						class="w-full pl-8.5 pr-8 py-2 text-xs rounded-lg bg-[var(--bg-primary)] border border-transparent text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-hidden transition-colors box-border"
					/>
					{#if searchQuery}
						<button
							type="button"
							class="absolute right-2.5 top-1/2 -translate-y-1/2 text-[var(--text-muted)] hover:text-[var(--text-primary)] bg-transparent border-0 cursor-pointer p-0"
							onclick={() => (searchQuery = '')}
							title={t('models.clearSearch')}
						>
							<X size={13} />
						</button>
					{/if}
				</div>

				<!-- Runtime Custom Selectbox -->
				<div class="w-full sm:w-52">
					<Select
						bind:value={selectedRuntimeFilter}
						options={runtimeFilterOptions}
						placeholder={t('models.allRuntimes')}
						ariaLabel={t('models.filterByRuntime')}
					/>
				</div>

				<!-- Status Custom Selectbox -->
				<div class="w-full sm:w-44">
					<Select
						bind:value={selectedStatusFilter}
						options={statusFilterOptions}
						placeholder={t('models.allStatuses')}
						ariaLabel={t('models.filterByStatus')}
					/>
				</div>
			</div>

			<!-- Models Catalog Grid -->
			{#if isLoadingData && registeredModels.length === 0}
				<div
					class="flex flex-col items-center justify-center gap-3 p-16 text-[var(--text-muted)] text-sm"
				>
					<div
						class="w-8 h-8 rounded-full border-2 border-[var(--border-color)] border-t-[var(--primary)] animate-spin"
					></div>
					<p class="m-0 text-xs">{t('common.loading')}</p>
				</div>
			{:else if registeredModels.length === 0}
				<EmptyState
					title={t('models.noModelsFound')}
					description={t('models.noModelsFoundDesc')}
					icon={Box}
				>
					{#snippet actions()}
						<Button variant="primary" onclick={startAddModel}>
							<Plus size={15} />
							<span>{t('models.registerModel')}</span>
						</Button>
					{/snippet}
				</EmptyState>
			{:else if filteredModels.length === 0}
				<EmptyState
					title={t('models.noMatchingModels')}
					description={t('models.noMatchingModelsDesc')}
					icon={Search}
				>
					{#snippet actions()}
						<Button
							variant="secondary"
							onclick={() => {
								searchQuery = '';
								selectedRuntimeFilter = 'all';
								selectedStatusFilter = 'all';
							}}
						>
							{t('models.clearFilters')}
						</Button>
					{/snippet}
				</EmptyState>
			{:else}
				<!-- Compact Model Cards Grid -->
				<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3 sm:gap-3.5">
					{#each filteredModels as model (model.id)}
						{@const mainFile = getMainFile(model.flags)}
						{@const fileDetails = getFileInfo(mainFile, availableFiles)}
						{@const isMissing = model.file_exists === false}
						{@const runningInfo = runningMap.get(model.id)}
						{@const isRunning = !!runningInfo}

						<div
							class="bg-[var(--bg-surface)] rounded-xl p-4 flex flex-col justify-between gap-3 shadow-xs transition-opacity {isMissing
								? 'opacity-65 hover:opacity-85'
								: ''}"
						>
							<!-- Header: Model ID + Badges + Status -->
							<div class="flex flex-col gap-2 min-w-0">
								<ModelBadge model={model.id} variant="badge-full" />

								<div class="flex justify-between gap-1.5">
									<!-- Model ID with copy shortcut -->
									<span
										class="flex items-center gap-1.5 text-xs text-[var(--text-muted)] p-1 truncate"
										title={model.id}
									>
										{model.id}
									</span>

									<Badge variant="muted" rounded="rounded-md">{model.runtime}</Badge>

									{#if isRunning}
										<Badge variant="success" pulse={true}>
											Running :{runningInfo?.port}
										</Badge>
									{:else if isMissing}
										<Badge variant="warning">
											<AlertTriangle size={11} class="shrink-0" />
											<span>{t('models.fileMissing')}</span>
										</Badge>
									{/if}
								</div>
							</div>

							<!-- File information -->
							<div
								class="flex items-center justify-between gap-2 text-xs py-2 px-2.5 rounded-lg bg-[var(--bg-hover)] border border-[var(--border-color)]/50"
							>
								<div
									class="flex items-center gap-1.5 min-w-0 font-mono {isMissing
										? 'text-amber-500/80 line-through'
										: 'text-[var(--text-secondary)]'}"
								>
									<FileText size={13} class="shrink-0 text-[var(--text-muted)]" />
									<span class="truncate text-[0.72rem]" title={mainFile || fileDetails.fileName}>
										{fileDetails.fileName}
									</span>
								</div>

								<div class="flex items-center gap-1.5 shrink-0">
									{#if fileDetails.sizeFormatted}
										<span class="text-[0.68rem] text-[var(--text-muted)] font-mono">
											{fileDetails.sizeFormatted}
										</span>
									{/if}
									<span
										class="px-1.5 py-0.2 rounded text-[0.65rem] font-semibold bg-[var(--bg-surface)] text-[var(--text-muted)] border border-[var(--border-color)]/50"
									>
										{fileDetails.format}
									</span>
								</div>
							</div>

							<!-- Action Buttons -->
							<div class="flex items-center justify-end gap-2 pt-1">
								<Button
									variant="secondary"
									size="sm"
									onclick={() => startEditModel(model)}
									title={t('common.edit')}
								>
									<Pencil size={13} />
									<span>{t('common.edit')}</span>
								</Button>
								<Button
									variant="danger"
									size="sm"
									onclick={() => deleteModel(model.id)}
									title={t('common.delete')}
								>
									<Trash2 size={13} />
									<span>{t('common.delete')}</span>
								</Button>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		{/if}
	</div>
</div>

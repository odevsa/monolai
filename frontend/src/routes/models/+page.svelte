<script lang="ts">
	import { afterNavigate } from '$app/navigation';
	import { modelsApi, runtimesApi } from '$lib/api';
	import { ModelCard, ModelEditSection, ModelFilterBar } from '$lib/components/models';
	import { Alert, Button, EmptyState, PageHeader, Spinner } from '$lib/components/ui';
	import { askConfirm, showAlert } from '$lib/confirmStore';
	import { refreshFeatures } from '$lib/featuresStore';
	import { t } from '$lib/i18n';
	import { runningModelsState } from '$lib/runningModelsStore';
	import type { ModelFileItem, ModelRecord, RunningModelStatus } from '$lib/types/models';
	import type { Runtime, RuntimeManifest } from '$lib/types/runtimes';
	import { copyToClipboard } from '$lib/utils/clipboard';
	import { getMainModelFilePath, parseFlags } from '$lib/utils/model';
	import { ArrowLeft, Box, Plus, Search } from '@lucide/svelte';
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

	let statusFilterOptions = $derived<{ value: 'all' | 'running' | 'missing'; label: string }[]>([
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
		const flagsObj = parseFlags(payload.flags);
		const flagsStr = JSON.stringify(flagsObj);

		if (isExisting && editingModel) {
			await modelsApi.update(editingModel.id, {
				id: payload.id,
				runtime: payload.runtime,
				flags: flagsObj
			});
		} else {
			await modelsApi.create({
				id: payload.id,
				runtime: payload.runtime,
				flags: flagsObj
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
					<span>{t('models.addModel')}</span>
				</Button>
			{:else}
				<Button variant="secondary" size="md" onclick={cancelEdit}>
					<ArrowLeft size={15} />
					<span>{t('common.back')}</span>
				</Button>
			{/if}
		</PageHeader>

		{#if error}
			<Alert variant="error" title={t('models.connectionFailure')}>
				<div class="flex items-center justify-between gap-3">
					<span>{error}</span>
					<Button variant="secondary" size="sm" onclick={() => loadAllModelsData()}>
						{t('common.refresh')}
					</Button>
				</div>
			</Alert>
		{/if}

		{#if isEditingModel}
			<ModelEditSection
				{runtimes}
				{availableFiles}
				{runtimeManifests}
				{editingModel}
				onSave={handleSaveModel}
				onCancel={cancelEdit}
			/>
		{:else}
			<ModelFilterBar
				bind:searchQuery
				bind:selectedRuntimeFilter
				bind:selectedStatusFilter
				{runtimeFilterOptions}
				{statusFilterOptions}
			/>

			<!-- Models Catalog Grid -->
			{#if isLoadingData && registeredModels.length === 0}
				<div
					class="flex flex-col items-center justify-center gap-3 p-16 text-[var(--text-muted)] text-sm"
				>
					<Spinner size={32} />
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
							<span>{t('models.addModel')}</span>
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
				<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3 sm:gap-3.5">
					{#each filteredModels as model (model.id)}
						<ModelCard
							{model}
							{availableFiles}
							runningInfo={runningMap.get(model.id)}
							{copiedId}
							onCopyId={copyId}
							onEdit={startEditModel}
							onDelete={deleteModel}
						/>
					{/each}
				</div>
			{/if}
		{/if}
	</div>
</div>

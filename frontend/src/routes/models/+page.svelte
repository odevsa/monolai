<script lang="ts">
	import { onMount } from 'svelte';
	import { afterNavigate } from '$app/navigation';
	import {
		Box,
		Plus,
		Edit2,
		Trash2,
		FileText,
		AlertTriangle,
		ArrowLeft,
		Search,
		Copy,
		Check,
		X
	} from '@lucide/svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import Alert from '$lib/components/Alert.svelte';
	import ModelBadge from '$lib/components/ModelBadge.svelte';
	import ModelForm from '$lib/components/ModelForm.svelte';
	import Select from '$lib/components/Select.svelte';
	import { askConfirm, showAlert } from '$lib/confirmStore';
	import { getMainModelFilePath } from '$lib/utils/model';
	import { formatBytes } from '$lib/utils/format';
	import { copyToClipboard } from '$lib/utils/clipboard';
	import { refreshFeatures } from '$lib/featuresStore';
	import { runningModels, type RunningModelStatus } from '$lib/runningModelsStore';

	interface ModelRecord {
		id: string;
		runtime: string;
		flags: string;
		created_at?: string;
		file_exists?: boolean;
	}

	interface RuntimeItem {
		id: string;
		name: string;
		binary_path: string;
	}

	interface ModelFileItem {
		name?: string;
		filename?: string;
		path?: string;
		relative_path?: string;
		absolute_path?: string;
		size?: number;
		size_bytes?: number;
	}

	interface ManifestFlag {
		flag: string;
		name: string;
		description: string;
		type: string;
		important: boolean;
		default_value: string;
		options?: string[];
	}

	interface RuntimeManifest {
		id: string;
		name: string;
		description: string;
		flags: ManifestFlag[];
	}

	let registeredModels = $state<ModelRecord[]>([]);
	let runtimes = $state<RuntimeItem[]>([]);
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
		for (const rm of $runningModels) {
			map.set(rm.model_id, rm);
		}
		return map;
	});

	// Selectbox options
	let runtimeFilterOptions = $derived([
		{ value: 'all', label: 'All Runtimes' },
		...runtimes.map((rt) => ({ value: rt.id, label: rt.name }))
	]);

	const statusFilterOptions = [
		{ value: 'all', label: 'All Statuses' },
		{ value: 'running', label: 'Running' },
		{ value: 'missing', label: 'Missing Weights' }
	];

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
				fetch('/api/models?all=true').then((r) => (r.ok ? r.json() : [])),
				fetch('/api/runtimes').then((r) => (r.ok ? r.json() : [])),
				fetch('/api/models/available').then((r) => (r.ok ? r.json() : [])),
				fetch('/api/runtime-manifests').then((r) => (r.ok ? r.json() : []))
			]);

			registeredModels = modelsRes;
			runtimes = runtimesRes;
			availableFiles = filesRes;
			runtimeManifests = manifestsRes;
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

	function startAddModel() {
		editingModel = null;
		isEditingModel = true;
	}

	function startEditModel(model: ModelRecord) {
		editingModel = model;
		isEditingModel = true;
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
		const url = isExisting ? `/api/models/${encodeURIComponent(payload.id)}` : '/api/models';
		const method = isExisting ? 'PUT' : 'POST';

		const res = await fetch(url, {
			method,
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(payload)
		});

		if (!res.ok) {
			const errorText = await res.text();
			throw new Error(errorText || 'Failed to save model');
		}

		await loadAllModelsData(true);
		refreshFeatures(true);
		isEditingModel = false;
		editingModel = null;
	}

	async function deleteModel(id: string) {
		const confirmed = await askConfirm(
			`Are you sure you want to delete model '${id}'?`,
			'Delete Model',
			'danger',
			'Delete',
			'Cancel'
		);
		if (!confirmed) return;

		try {
			const res = await fetch(`/api/models/${encodeURIComponent(id)}`, {
				method: 'DELETE'
			});

			if (!res.ok) {
				throw new Error('Failed to delete model');
			}

			await loadAllModelsData(true);
			refreshFeatures(true);
		} catch (e: any) {
			await showAlert(e.message || 'Error deleting model', 'Error', 'danger');
		}
	}

	function getMainFile(flagsJson: string): string {
		try {
			const obj: Record<string, string> = JSON.parse(flagsJson);
			return obj['--model'] || obj['-m'] || '';
		} catch {
			return '';
		}
	}

	function getFileInfo(filePath: string, files: ModelFileItem[]) {
		if (!filePath)
			return { fileName: 'No file configured', format: 'GGUF', sizeFormatted: undefined };
		const fileName = filePath.split(/[/\\]/).pop() || filePath;
		const match = files.find((f) => {
			const p = f.path || f.absolute_path || f.relative_path || '';
			return (
				p === filePath || p.endsWith(fileName) || f.name === fileName || f.filename === fileName
			);
		});

		const ext = fileName.split('.').pop()?.toUpperCase() || 'GGUF';
		const sizeBytes = match?.size_bytes ?? match?.size;
		const sizeFormatted = sizeBytes ? formatBytes(sizeBytes) : undefined;
		return { fileName, sizeFormatted, format: ext };
	}
</script>

<svelte:head>
	<title>Monolai - Models</title>
</svelte:head>

<div class="page-container">
	<div class="page-inner gap-5 sm:gap-6">
		<!-- Page Header matching Monolai design standards -->
		<PageHeader
			title="Models"
			subtitle="Manage local model definitions, weights files, and runtime execution flags"
			icon={Box}
		>
			{#if !isEditingModel}
				<button
					type="button"
					class="app-btn app-btn-primary app-btn-md shadow-xs"
					onclick={startAddModel}
				>
					<Plus size={15} />
					<span>New</span>
				</button>
			{:else}
				<button
					type="button"
					class="app-btn app-btn-secondary app-btn-md shadow-xs"
					onclick={cancelEdit}
				>
					<ArrowLeft size={15} />
					<span>Back</span>
				</button>
			{/if}
		</PageHeader>

		{#if error}
			<Alert variant="error" title="Backend Connection Failure:" message={error}>
				{#snippet action()}
					<button
						type="button"
						class="app-btn app-btn-secondary app-btn-sm"
						onclick={() => loadAllModelsData()}
					>
						Retry Connection
					</button>
				{/snippet}
			</Alert>
		{/if}

		{#if isEditingModel}
			<!-- Dedicated Form Container (Borderless solid panel) -->
			<div class="bg-[var(--bg-surface)] rounded-2xl border-0 shadow-xs p-5 sm:p-7">
				<div class="flex items-center gap-2.5 pb-4 mb-5 border-b border-[var(--border-color)]">
					<div
						class="w-8 h-8 rounded-xl bg-[var(--primary-light)] text-[var(--primary)] flex items-center justify-center border-0"
					>
						<Box size={16} />
					</div>
					<div>
						<h3 class="m-0 text-base font-bold text-[var(--text-primary)]">
							{editingModel ? `Edit Model: ${editingModel.id}` : 'New Model'}
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
			<!-- Search & Filter Controls Bar (Standardized Custom Selectboxes, Borderless) -->
			<div
				class="flex flex-col sm:flex-row items-stretch sm:items-center gap-2.5 p-2 bg-[var(--bg-surface)] border-0 rounded-xl shadow-xs"
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
						placeholder="Search by model ID, filename, or runtime..."
						class="w-full pl-8.5 pr-8 py-2 text-xs rounded-lg bg-[var(--bg-primary)] border-0 text-[var(--text-primary)] placeholder-[var(--text-muted)] focus:outline-hidden transition-colors box-border"
					/>
					{#if searchQuery}
						<button
							type="button"
							class="absolute right-2.5 top-1/2 -translate-y-1/2 text-[var(--text-muted)] hover:text-[var(--text-primary)] bg-transparent border-0 cursor-pointer p-0"
							onclick={() => (searchQuery = '')}
							title="Clear search"
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
						placeholder="All Runtimes"
						ariaLabel="Filter by runtime"
					/>
				</div>

				<!-- Status Custom Selectbox -->
				<div class="w-full sm:w-44">
					<Select
						bind:value={selectedStatusFilter}
						options={statusFilterOptions}
						placeholder="All Statuses"
						ariaLabel="Filter by status"
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
					<p class="m-0 text-xs">Loading registered models...</p>
				</div>
			{:else if registeredModels.length === 0}
				<!-- Zero Models Registered Empty State -->
				<div
					class="bg-[var(--bg-surface)] border-0 rounded-2xl p-10 flex flex-col items-center justify-center gap-3.5 text-center shadow-xs"
				>
					<div
						class="w-14 h-14 rounded-2xl bg-[var(--primary-light)] text-[var(--primary)] flex items-center justify-center border-0 shadow-xs"
					>
						<Box size={28} />
					</div>
					<div class="flex flex-col gap-1 max-w-[420px]">
						<h3 class="m-0 text-base font-bold text-[var(--text-primary)]">No Registered Models</h3>
						<p class="m-0 text-xs text-[var(--text-muted)] leading-relaxed">
							Register your GGUF or SafeTensors weights from disk to start serving local LLM
							inference.
						</p>
					</div>
					<button
						type="button"
						class="app-btn app-btn-primary app-btn-md shadow-xs mt-1"
						onclick={startAddModel}
					>
						<Plus size={15} />
						<span>New Model</span>
					</button>
				</div>
			{:else if filteredModels.length === 0}
				<!-- Search Yielded No Results -->
				<div
					class="bg-[var(--bg-surface)] border-0 rounded-2xl p-8 flex flex-col items-center justify-center gap-2.5 text-center shadow-xs"
				>
					<div
						class="w-10 h-10 rounded-xl bg-[var(--bg-primary)] text-[var(--text-muted)] flex items-center justify-center"
					>
						<Search size={18} />
					</div>
					<h4 class="m-0 text-sm font-bold text-[var(--text-primary)]">
						No models match your search
					</h4>
					<p class="m-0 text-xs text-[var(--text-muted)]">
						Try adjusting your filters or search keywords.
					</p>
					<button
						type="button"
						class="app-btn app-btn-secondary app-btn-sm mt-1"
						onclick={() => {
							searchQuery = '';
							selectedRuntimeFilter = 'all';
							selectedStatusFilter = 'all';
						}}
					>
						Clear Filters
					</button>
				</div>
			{:else}
				<!-- Compact Model Cards Grid (Borderless, Muted when missing) -->
				<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3 sm:gap-3.5">
					{#each filteredModels as model (model.id)}
						{@const mainFile = getMainFile(model.flags)}
						{@const fileDetails = getFileInfo(mainFile, availableFiles)}
						{@const isMissing = model.file_exists === false}
						{@const runningInfo = runningMap.get(model.id)}
						{@const isRunning = !!runningInfo}

						<div
							class="model-card bg-[var(--bg-surface)] border-0 rounded-xl p-4 flex flex-col justify-between gap-3 shadow-xs transition-opacity {isMissing
								? 'opacity-55 hover:opacity-75'
								: ''}"
						>
							<!-- Header: Model ID + Badges + Status -->
							<div class="flex flex-col gap-2 min-w-0">
								<div class="flex items-center justify-between gap-2">
									<div class="flex items-center gap-1.5 min-w-0">
										<ModelBadge model={model.id} variant="inline" />
										<span
											class="px-2 py-0.5 rounded text-[0.68rem] font-medium bg-[var(--bg-primary)] text-[var(--text-muted)] border-0 shrink-0"
										>
											{model.runtime}
										</span>
									</div>

									{#if isRunning}
										<span
											class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[0.68rem] font-semibold bg-emerald-500/15 text-emerald-400 border-0 shrink-0"
										>
											<span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
											<span>Running :{runningInfo?.port}</span>
										</span>
									{:else if isMissing}
										<span
											class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[0.68rem] font-medium bg-amber-500/15 text-amber-500 border-0 shrink-0"
											title="Weights file not found on disk"
										>
											<AlertTriangle size={11} />
											<span>Missing File</span>
										</span>
									{/if}
								</div>

								<!-- Model ID with copy shortcut -->
								<div
									class="flex items-center gap-1.5 text-xs font-mono font-semibold text-[var(--text-primary)] min-w-0"
								>
									<span class="truncate" title={model.id}>{model.id}</span>
									<button
										type="button"
										class="p-0.5 bg-transparent border-0 text-[var(--text-muted)] hover:text-[var(--text-primary)] cursor-pointer rounded transition-colors shrink-0"
										onclick={() => copyId(model.id)}
										title="Copy model ID"
									>
										{#if copiedId === model.id}
											<Check size={12} class="text-emerald-400" />
										{:else}
											<Copy size={12} />
										{/if}
									</button>
								</div>
							</div>

							<!-- File information (Borderless compact pill) -->
							<div
								class="flex items-center justify-between gap-2 text-xs py-2 px-2.5 rounded-lg bg-[var(--bg-primary)] border-0"
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
										class="px-1.5 py-0.2 rounded text-[0.65rem] font-semibold bg-[var(--bg-surface)] text-[var(--text-muted)] border-0"
									>
										{fileDetails.format}
									</span>
								</div>
							</div>

							<!-- Action Buttons: Standardized matching runtimes page -->
							<div class="flex items-center justify-end gap-2 pt-1">
								<button
									type="button"
									class="app-btn app-btn-secondary app-btn-sm"
									onclick={() => startEditModel(model)}
									title="Edit model"
								>
									<Edit2 size={13} />
									<span>Edit</span>
								</button>
								<button
									type="button"
									class="app-btn app-btn-danger app-btn-sm"
									onclick={() => deleteModel(model.id)}
									title="Delete model"
								>
									<Trash2 size={13} />
									<span>Delete</span>
								</button>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		{/if}
	</div>
</div>

<style>
	/* Solid card container without background flash on hover */
	.model-card {
		background-color: var(--bg-surface);
	}
</style>

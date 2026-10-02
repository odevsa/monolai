<script lang="ts">
	import { Box, Plus, Edit2, Trash2, Cpu, RefreshCw, FileText } from '@lucide/svelte';
	import ModelForm from './ModelForm.svelte';
	import ModelBadge from './ModelBadge.svelte';
	import { askConfirm, showAlert } from '$lib/confirmStore';
	import { getMainModelFilePath } from '$lib/utils/model';

	interface ModelRecord {
		id: string;
		runtime: string;
		flags: string;
		created_at?: string;
	}

	interface RuntimeItem {
		id: string;
		name: string;
		binary_path: string;
	}

	interface ModelFileItem {
		name: string;
		path: string;
		size?: number;
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
	let isLoadingData = $state(false);

	let isEditingModel = $state(false);
	let editingModel = $state<ModelRecord | null>(null);

	async function loadAllModelsData() {
		isLoadingData = true;
		try {
			const [modelsRes, runtimesRes, filesRes, manifestsRes] = await Promise.all([
				fetch('/api/models').then((r) => (r.ok ? r.json() : [])),
				fetch('/api/runtimes').then((r) => (r.ok ? r.json() : [])),
				fetch('/api/models/available').then((r) => (r.ok ? r.json() : [])),
				fetch('/api/runtime-manifests').then((r) => (r.ok ? r.json() : []))
			]);

			registeredModels = modelsRes;
			runtimes = runtimesRes;
			availableFiles = filesRes;
			runtimeManifests = manifestsRes;

			if (runtimes.length === 0) {
				runtimes = [
					{ id: 'llama-cpp', name: 'LLaMA C++ Server', binary_path: '/usr/bin/llama-server' },
					{ id: 'sd-cpp', name: 'Stable Diffusion C++ Server', binary_path: '/usr/bin/sd-server' }
				];
			}
		} catch (e) {
			console.error('Error loading models data:', e);
		} finally {
			isLoadingData = false;
		}
	}

	$effect(() => {
		loadAllModelsData();
	});

	function startAddModel() {
		editingModel = null;
		isEditingModel = true;
	}

	function startEditModel(model: ModelRecord) {
		editingModel = model;
		isEditingModel = true;
	}



	async function handleSaveModel(payload: { id: string; runtime: string; flags: Record<string, string> }) {
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

		await loadAllModelsData();
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
		if (!confirmed) {
			return;
		}

		try {
			const res = await fetch(`/api/models/${encodeURIComponent(id)}`, {
				method: 'DELETE'
			});

			if (!res.ok) {
				throw new Error('Failed to delete model');
			}

			await loadAllModelsData();
		} catch (e: any) {
			await showAlert(e.message || 'Error deleting model', 'Error', 'danger');
		}
	}

	export function resetView() {
		isEditingModel = false;
		editingModel = null;
	}
</script>

<div class="flex flex-col gap-5">
	{#if !isEditingModel}
		<!-- List Models Header View -->
		<div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 border-b border-[var(--border-color)] pb-3">
			<div>
				<h4 class="m-0 text-sm sm:text-base font-bold text-[var(--text-primary)] flex items-center gap-2">
					<Box size={18} class="text-[var(--primary)]" />
					Registered Models
					{#if registeredModels.length > 0}
						<span class="text-[0.7rem] px-2 py-0.5 rounded-full font-bold bg-[var(--primary-light)] text-[var(--primary)] border border-[var(--primary)]/20">
							{registeredModels.length}
						</span>
					{/if}
				</h4>
				<p class="m-0 text-xs text-[var(--text-muted)] mt-1 leading-relaxed">
					Manage local LLM definitions, GGUF weights, and CLI execution parameters.
				</p>
			</div>
			<button
				type="button"
				class="flex items-center justify-center gap-1.5 px-3.5 py-2 bg-[var(--primary)] text-white border-0 rounded-xl text-xs font-bold cursor-pointer transition-all duration-150 hover:bg-[var(--primary-hover)] shrink-0 shadow-md shadow-[var(--primary)]/15 active:scale-95"
				onclick={startAddModel}
			>
				<Plus size={16} />
				<span>Register Model</span>
			</button>
		</div>

		{#if isLoadingData}
			<div class="flex flex-col items-center justify-center gap-3 py-12 text-[var(--text-muted)] text-xs">
				<RefreshCw size={22} class="animate-spin text-[var(--primary)]" />
				<span>Loading registered models...</span>
			</div>
		{:else if registeredModels.length === 0}
			<div class="flex flex-col items-center justify-center gap-3 py-12 px-4 bg-[var(--bg-primary)] border-2 border-dashed border-[var(--border-color)] rounded-2xl text-center">
				<div class="w-12 h-12 rounded-2xl bg-[var(--primary-light)] text-[var(--primary)] flex items-center justify-center border border-[var(--primary)]/20 mb-1">
					<Box size={24} />
				</div>
				<div class="flex flex-col gap-1 max-w-[320px]">
					<h5 class="m-0 text-sm font-bold text-[var(--text-primary)]">No Registered Models</h5>
					<p class="m-0 text-xs text-[var(--text-muted)] leading-relaxed">
						Register your GGUF model files to begin running local inference.
					</p>
				</div>
				<button
					type="button"
					class="mt-2 px-4 py-2.5 bg-[var(--primary)] text-white border-0 rounded-xl text-xs font-bold cursor-pointer transition-all hover:bg-[var(--primary-hover)] shadow-sm"
					onclick={startAddModel}
				>
					Register First Model
				</button>
			</div>
		{:else}
			<div class="flex flex-col gap-3">
				{#each registeredModels as model}
					{@const mainFile = getMainModelFilePath(model)}
					<div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-3.5 bg-[var(--bg-primary)] border border-[var(--border-color)] rounded-xl shadow-2xs hover:border-[var(--primary)]/50 transition-all duration-150">
						<div class="flex flex-col gap-1.5 min-w-0">
							<div class="flex items-center gap-2 flex-wrap">
								<ModelBadge model={model.id} variant="inline" />
								<span class="text-[0.675rem] font-semibold bg-[var(--bg-surface)] border border-[var(--border-color)] text-[var(--text-secondary)] px-2 py-0.5 rounded-md uppercase tracking-wider">
									{model.runtime}
								</span>
							</div>

							{#if mainFile}
								<div class="flex items-center gap-1.5 text-[0.725rem] text-[var(--text-muted)] font-mono truncate">
									<FileText size={13} class="shrink-0 text-[var(--text-muted)] opacity-70" />
									<span class="truncate" title={mainFile}>
										{mainFile.split('/').pop()}
									</span>
								</div>
							{/if}
						</div>

						<div class="flex items-center gap-2 shrink-0 self-end sm:self-center pt-2 sm:pt-0 border-t sm:border-t-0 border-[var(--border-color)]/50 w-full sm:w-auto justify-end">
							<button
								type="button"
								class="flex items-center gap-1.5 px-3 py-1.5 bg-[var(--bg-surface)] border border-[var(--border-color)] text-[var(--text-secondary)] rounded-lg text-xs font-medium cursor-pointer transition-all hover:text-[var(--text-primary)] hover:border-[var(--primary)] hover:bg-[var(--bg-hover)]"
								onclick={() => startEditModel(model)}
								title="Edit model"
							>
								<Edit2 size={14} />
								<span>Edit</span>
							</button>
							<button
								type="button"
								class="flex items-center gap-1.5 px-3 py-1.5 bg-[var(--bg-surface)] border border-[var(--border-color)] text-[var(--text-muted)] rounded-lg text-xs font-medium cursor-pointer transition-all hover:text-red-500 hover:border-red-500/40 hover:bg-red-500/10"
								onclick={() => deleteModel(model.id)}
								title="Delete model"
							>
								<Trash2 size={14} />
								<span>Delete</span>
							</button>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	{:else}
		<!-- Embedded Model Form -->
		<ModelForm
			{runtimes}
			{availableFiles}
			{runtimeManifests}
			{editingModel}
			onSave={handleSaveModel}
			onCancel={() => {
				isEditingModel = false;
				editingModel = null;
			}}
		/>
	{/if}
</div>

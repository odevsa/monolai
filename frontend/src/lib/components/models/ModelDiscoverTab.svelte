<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { hubApi } from '$lib/api/hub';
	import { modelsApi } from '$lib/api/models';
	import {
		Alert,
		Badge,
		Button,
		Card,
		DiagonalLabel,
		EmptyState,
		Input,
		Modal,
		Select,
		Spinner,
		Tabs
	} from '$lib/components/ui';
	import { askConfirm, showAlert } from '$lib/confirmStore';
	import { t } from '$lib/i18n';
	import { downloadStore } from '$lib/state/downloads.svelte';
	import { hostState } from '$lib/state/host.svelte';
	import { settingsDialogState } from '$lib/state';
	import type {
		HubModelItem,
		HubModelSibling,
		ModelRecord,
		RepositoryRecord
	} from '$lib/types/models';
	import { formatBytes } from '$lib/utils/format';
	import {
		ArrowDownToLine,
		Box,
		Download,
		FileDown,
		Globe,
		Heart,
		Search,
		Sparkles,
		Trash
	} from '@lucide/svelte';
	import { onMount } from 'svelte';

	let repositories = $state<RepositoryRecord[]>([]);
	let selectedRepoId = $state<string>('');
	let searchQuery = $state('');
	let selectedModality = $state<string>('all');
	let selectedSort = $state<'downloads' | 'likes' | 'lastModified'>('downloads');

	let isLoadingRepos = $state(true);
	let isSearching = $state(false);
	let searchResults = $state<HubModelItem[]>([]);
	let errorMessage = $state<string | null>(null);

	// Registered local models for installed detection
	let registeredModels = $state<ModelRecord[]>([]);

	// Details & Quants state
	let inspectingModel = $state<HubModelItem | null>(null);
	let isLoadingDetails = $state(false);
	let modelFiles = $state<HubModelSibling[]>([]);
	let enqueuedFiles = $state<Record<string, boolean>>({});

	let enabledRepos = $derived(repositories.filter((r) => r.enabled));

	let lastCompletedCount = $state<number | null>(null);

	$effect(() => {
		const current = downloadStore.completedCount;
		if (lastCompletedCount !== null && current > lastCompletedCount) {
			loadRegisteredModels();
		}
		lastCompletedCount = current;
	});

	const modalityTabItems = [
		{ id: 'all', label: 'All' },
		{ id: 'text-to-text', label: 'Text' },
		{ id: 'text-to-image', label: 'Image' },
		{ id: 'text-to-speech', label: 'TTS' },
		{ id: 'speech-to-text', label: 'ASR' }
	];

	const sortOptions = [
		{ value: 'downloads', label: 'Most Downloads' },
		{ value: 'likes', label: 'Most Likes' },
		{ value: 'lastModified', label: 'Recently Updated' }
	];

	async function loadRepos() {
		isLoadingRepos = true;
		errorMessage = null;
		try {
			repositories = await hubApi.getRepositories();
			const enabled = repositories.filter((r) => r.enabled);
			if (enabled.length > 0 && !selectedRepoId) {
				selectedRepoId = enabled[0].id;
			}
			if (selectedRepoId) {
				executeSearch();
			}
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			errorMessage = msg;
		} finally {
			isLoadingRepos = false;
		}
	}

	async function loadRegisteredModels() {
		try {
			registeredModels = await modelsApi.list(true);
		} catch (err) {
			console.error('Failed to load registered models:', err);
		}
	}

	const GENERIC_FILENAMES = new Set([
		'model.safetensors',
		'model.bin',
		'model.gguf',
		'weights.safetensors',
		'weights.bin',
		'pytorch_model.bin',
		'diffusion_pytorch_model.safetensors',
		'config.json',
		'tokenizer.json'
	]);

	function findDownloadedModelForFile(
		file: HubModelSibling,
		model?: HubModelItem | null
	): ModelRecord | undefined {
		const baseName = (file.rfilename.split('/').pop() || file.rfilename).toLowerCase();
		return registeredModels.find((m) => {
			if (m.file_exists === false) return false;
			const mBase = (m.filename || m.relative_path.split('/').pop() || '').toLowerCase();
			if (mBase !== baseName) return false;

			// If we know which Hub model we are inspecting, verify it matches
			if (model) {
				const normModelId = model.id.toLowerCase();
				const baseId = (model.id.split('/').pop() || model.id).toLowerCase();
				if (m.source_repo && m.source_repo.toLowerCase() === normModelId) return true;
				const rel = m.relative_path.toLowerCase();
				if (rel.includes(`/${normModelId}/`) || rel.includes(`/${baseId}/`)) return true;
				// Never match a generic filename against a model unless the directory or source_repo matches
				if (GENERIC_FILENAMES.has(baseName)) return false;
			}

			return !GENERIC_FILENAMES.has(baseName);
		});
	}

	function isModelInstalled(model: HubModelItem): boolean {
		const normId = model.id.toLowerCase();
		const baseId = (model.id.split('/').pop() || model.id).toLowerCase();

		// 1. Direct match on source_repo or path containing the model ID in folder structure
		const directMatch = registeredModels.some((m) => {
			if (m.file_exists === false) return false;
			if (m.source_repo && m.source_repo.toLowerCase() === normId) return true;
			const rel = m.relative_path.toLowerCase();
			if (rel.includes(`/${normId}/`) || rel.includes(`/${baseId}/`)) return true;
			const mBase = (m.filename || rel.split('/').pop() || '').toLowerCase();
			if (!GENERIC_FILENAMES.has(mBase)) {
				return (
					m.id.toLowerCase() === baseId || m.name.toLowerCase() === baseId || rel.includes(baseId)
				);
			}
			return false;
		});

		if (directMatch) return true;

		// 2. Sibling check (only for non-generic files or files belonging to this model)
		if (model.siblings && model.siblings.length > 0) {
			return model.siblings.some((s) => !!findDownloadedModelForFile(s, model));
		}

		return false;
	}

	function getEffectiveFileSize(file: HubModelSibling): number {
		if (file.size && file.size > 0) return file.size;

		// Fallback estimation from filename (e.g. qwen2.5-1.5b-instruct-q4_k_m.gguf)
		const name = (file.rfilename || '').toLowerCase();
		const paramMatch = name.match(/(\d+(?:\.\d+)?)\s*b/);
		if (paramMatch) {
			const billions = parseFloat(paramMatch[1]);
			let bitsPerWeight = 4.5; // default Q4
			if (name.includes('q2')) bitsPerWeight = 2.8;
			else if (name.includes('q3')) bitsPerWeight = 3.5;
			else if (name.includes('q4')) bitsPerWeight = 4.5;
			else if (name.includes('q5')) bitsPerWeight = 5.5;
			else if (name.includes('q6')) bitsPerWeight = 6.6;
			else if (name.includes('q8')) bitsPerWeight = 8.5;
			else if (name.includes('fp16') || name.includes('f16')) bitsPerWeight = 16;

			return Math.round((billions * 1e9 * bitsPerWeight) / 8);
		}

		return 0;
	}

	function getPerformanceFit(fileSizeBytes?: number) {
		if (!fileSizeBytes || fileSizeBytes <= 0) {
			return {
				color: 'bg-[var(--text-muted)]',
				title: t('models.discover.fitUnknown')
			};
		}

		const info = hostState.sysinfo;
		if (!info) {
			return {
				color: 'bg-[var(--text-muted)]',
				title: t('models.discover.fitUnknown')
			};
		}

		// Estimate memory with 15% context / KV cache overhead
		const requiredBytes = fileSizeBytes * 1.15;
		const vramTotal =
			info.vram?.total_bytes ??
			(info.gpus?.reduce((acc, g) => acc + (g.memory_total_bytes ?? 0), 0) ||
				info.gpu?.memory_total_bytes ||
				0);
		const ramTotal = info.ram?.total_bytes ?? 0;

		// 1. Fits in GPU VRAM -> Optimal (Green)
		if (vramTotal > 0 && requiredBytes <= vramTotal) {
			return {
				color: 'bg-[var(--success)]',
				title: t('models.discover.fitOptimal')
			};
		}

		// 2. Fits within system RAM -> Moderate (Yellow)
		if (ramTotal > 0 && requiredBytes <= ramTotal * 0.85) {
			return {
				color: 'bg-[var(--warning)]',
				title: vramTotal > 0 ? t('models.discover.fitPartialGpu') : t('models.discover.fitModerate')
			};
		}

		// 3. Exceeds system RAM -> Risk of OOM or heavy swap (Red)
		return {
			color: 'bg-[var(--danger)]',
			title: t('models.discover.fitLow')
		};
	}

	function getOnlyModelName(model: HubModelItem) {
		return model.id.split('/').pop();
	}

	async function enableHuggingFace() {
		try {
			await hubApi.updateRepository('huggingface', { enabled: true });
			await loadRepos();
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			errorMessage = msg;
		}
	}

	function updateUrlParams() {
		if (page.url.searchParams.get('tab') !== 'discover') return;
		const url = new URL(page.url);
		url.searchParams.set('tab', 'discover');

		const trimmed = searchQuery.trim();
		if (trimmed) {
			url.searchParams.set('q', trimmed);
		} else {
			url.searchParams.delete('q');
		}

		if (selectedModality && selectedModality !== 'all') {
			url.searchParams.set('modality', selectedModality);
		} else {
			url.searchParams.delete('modality');
		}

		if (selectedSort && selectedSort !== 'downloads') {
			url.searchParams.set('sort', selectedSort);
		} else {
			url.searchParams.delete('sort');
		}

		if (selectedRepoId && selectedRepoId !== 'huggingface') {
			url.searchParams.set('repo', selectedRepoId);
		} else {
			url.searchParams.delete('repo');
		}

		const rawSearch = url.searchParams.toString().replace(/%20/g, '+');
		const target = `${url.pathname}${rawSearch ? '?' + rawSearch : ''}`;
		if (target !== `${page.url.pathname}${page.url.search}`) {
			goto(target, { replaceState: true, noScroll: true, keepFocus: true });
		}
	}

	async function executeSearch() {
		if (!selectedRepoId) return;

		updateUrlParams();

		isSearching = true;
		errorMessage = null;
		try {
			searchResults = await hubApi.searchModels({
				repo_id: selectedRepoId,
				query: searchQuery.trim() || undefined,
				modality: selectedModality === 'all' ? undefined : selectedModality,
				sort: selectedSort
			});
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			errorMessage = msg;
		} finally {
			isSearching = false;
		}
	}

	async function inspectFiles(model: HubModelItem) {
		if (model.requires_auth) {
			await showAlert(
				t('models.discover.authRequiredDesc'),
				t('models.discover.authRequiredTitle'),
				'warning'
			);
			settingsDialogState.open('repositories');
			return;
		}

		inspectingModel = model;
		isLoadingDetails = true;
		modelFiles = [];
		try {
			const details = await hubApi.getModelDetails(selectedRepoId, model.id);
			modelFiles = details.siblings || [];
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			if (
				msg.includes('401') ||
				msg.toLowerCase().includes('unauthorized') ||
				msg.toLowerCase().includes('token')
			) {
				await showAlert(
					t('models.discover.authRequiredDesc'),
					t('models.discover.authRequiredTitle'),
					'warning'
				);
				settingsDialogState.open('repositories');
				inspectingModel = null;
				return;
			}
			errorMessage = msg;
		} finally {
			isLoadingDetails = false;
		}
	}

	async function handleDownload(file: HubModelSibling) {
		if (!inspectingModel) return;

		const filename = file.rfilename.split('/').pop() || file.rfilename;
		enqueuedFiles[file.rfilename] = true;

		const resolvedUrl =
			file.download_url ||
			`https://huggingface.co/${inspectingModel.id}/resolve/main/${file.rfilename}`;

		const modelModality =
			inspectingModel.modality ||
			(inspectingModel.pipeline_tag === 'text-to-image' ? 'text-to-image' : 'text-to-text');

		try {
			await downloadStore.enqueue({
				url: resolvedUrl,
				download_url: resolvedUrl,
				model_name: inspectingModel.id,
				filename,
				repo_id: selectedRepoId,
				modality: modelModality,
				size_bytes: file.size || 0
			});
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			errorMessage = msg;
			enqueuedFiles[file.rfilename] = false;
		}
	}

	async function handleDeleteDownloaded(record: ModelRecord) {
		const confirmed = await askConfirm(
			t('models.deleteConfirmMsg', { id: record.id }),
			t('models.deleteConfirmTitle'),
			'danger',
			t('common.delete'),
			t('common.cancel')
		);
		if (!confirmed) return;

		try {
			await modelsApi.delete(record.id, true);
			await loadRegisteredModels();
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			await showAlert(msg || t('models.deleteError'), t('common.error'), 'danger');
		}
	}

	onMount(() => {
		const p = page.url.searchParams;
		const q = p.get('q');
		if (q) searchQuery = q;
		const mod = p.get('modality');
		if (mod) selectedModality = mod;
		const sort = p.get('sort');
		if (sort === 'downloads' || sort === 'likes' || sort === 'lastModified') {
			selectedSort = sort;
		}
		const repo = p.get('repo');
		if (repo) selectedRepoId = repo;

		loadRepos();
		loadRegisteredModels();
		hostState.fetchStaticInfo();
	});
</script>

<div class="space-y-5">
	{#if errorMessage}
		<Alert variant="error" title={t('common.error')}>
			{errorMessage}
		</Alert>
	{/if}

	{#if isLoadingRepos}
		<div class="flex flex-col items-center justify-center p-16 text-[var(--text-muted)] gap-2">
			<Spinner size={32} />
			<span class="text-xs">{t('common.loading')}</span>
		</div>
	{:else if enabledRepos.length === 0}
		<EmptyState
			icon={Globe}
			title={t('models.discover.noReposTitle')}
			description={t('models.discover.noReposDesc')}
		>
			{#snippet actions()}
				<div class="flex items-center gap-3">
					<Button variant="primary" onclick={enableHuggingFace}>
						<Sparkles size={14} />
						<span>{t('models.discover.enableHuggingFace')}</span>
					</Button>
				</div>
			{/snippet}
		</EmptyState>
	{:else}
		<!-- Search and Filter Controls -->
		<Card padding="sm">
			<div class="flex flex-col md:flex-row items-stretch md:items-center gap-3">
				<!-- Repo Selector -->
				<div class="w-full md:w-56 shrink-0">
					<Select
						bind:value={selectedRepoId}
						onchange={executeSearch}
						options={enabledRepos.map((r) => ({ value: r.id, label: r.name }))}
					/>
				</div>

				<!-- Search Input -->
				<div class="flex-1">
					<Input
						bind:value={searchQuery}
						onkeydown={(e) => e.key === 'Enter' && executeSearch()}
						placeholder={t('models.discover.searchPlaceholder')}
						clearable
					>
						{#snippet prefix()}
							<Search size={15} />
						{/snippet}
					</Input>
				</div>

				<!-- Sort Selector -->
				<div class="w-full md:w-44 shrink-0">
					<Select bind:value={selectedSort} onchange={executeSearch} options={sortOptions} />
				</div>

				<Button variant="primary" size="md" onclick={executeSearch} disabled={isSearching}>
					{#if isSearching}
						<Spinner size={14} />
					{:else}
						<Search size={14} />
					{/if}
					<span>{t('models.discover.search')}</span>
				</Button>
			</div>
		</Card>

		<!-- Modality Filter Pills -->
		<Tabs
			variant="pills"
			items={modalityTabItems}
			bind:value={selectedModality}
			onchange={() => executeSearch()}
		/>

		<!-- Results Grid -->
		{#if isSearching}
			<div class="flex flex-col items-center justify-center p-16 text-[var(--text-muted)] gap-3">
				<Spinner size={32} />
				<span class="text-xs">{t('common.loading')}</span>
			</div>
		{:else if searchResults.length === 0}
			<EmptyState
				icon={Search}
				title={t('models.noMatchingModels')}
				description={t('models.noMatchingModelsDesc')}
			/>
		{:else}
			<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3.5">
				{#each searchResults as model (model.id)}
					<Card padding="md" class="relative overflow-hidden flex flex-col justify-between h-full">
						{#if isModelInstalled(model)}
							<DiagonalLabel text={t('common.installed')} />
						{:else if model.requires_auth}
							<DiagonalLabel variant="warning" text={t('models.discover.authRequired')} />
						{/if}

						<div class="flex flex-col gap-4">
							<div class="flex items-start justify-between gap-2 pr-12">
								<div
									class="text-sm font-semibold text-[var(--text-primary)] leading-tight m-0 line-clamp-2"
									title={model.id}
								>
									{#if model.author || model.id.includes('/')}
										<span class="text-[var(--text-muted)]"
											>{model.author || model.id.split('/')[0]}/</span
										>{getOnlyModelName(model)}
									{:else}
										{getOnlyModelName(model)}
									{/if}
								</div>
							</div>

							<div class="flex gap-2 flex-wrap">
								<Badge variant="primary" rounded="rounded-md">
									{model.modality}
								</Badge>
								{#if model.pipeline_tag}
									<Badge variant="muted" rounded="rounded-md">
										{model.pipeline_tag}
									</Badge>
								{/if}
							</div>

							<div class="flex justify-between items-center gap-4 text-xs text-[var(--text-muted)]">
								<span class="flex items-center gap-1.5" title="Likes">
									<Heart size={13} class="text-[var(--danger-text)]" />
									<span class="font-medium">{(model.likes ?? 0).toLocaleString()}</span>
								</span>
								<span class="flex items-center gap-1.5" title="Downloads">
									<Download size={13} class="text-[var(--primary)]" />
									<span class="font-medium">{(model.downloads ?? 0).toLocaleString()}</span>
								</span>
							</div>
						</div>

						<div class="pt-3 mt-3 border-t border-[var(--border-color)]/60 flex justify-end">
							<Button variant="secondary" size="sm" onclick={() => inspectFiles(model)}>
								<FileDown size={13} />
								<span>{t('models.discover.inspectFiles')}</span>
							</Button>
						</div>
					</Card>
				{/each}
			</div>
		{/if}
	{/if}
</div>

<!-- Files & Quants Modal -->
{#if inspectingModel}
	<Modal
		icon={Box}
		open={!!inspectingModel}
		title={inspectingModel.id}
		onclose={() => (inspectingModel = null)}
	>
		<div class="space-y-4">
			<div class="text-xs text-[var(--text-muted)]">
				{t('models.discover.selectQuantPrompt')}
			</div>

			{#if isLoadingDetails}
				<div class="flex flex-col items-center justify-center p-10 text-[var(--text-muted)] gap-2">
					<Spinner size={24} />
					<span class="text-xs">{t('common.loading')}</span>
				</div>
			{:else if modelFiles.length === 0}
				<div class="text-xs text-[var(--text-muted)] p-6 text-center italic">
					{t('models.discover.noFilesAvailable')}
				</div>
			{:else}
				<div class="space-y-2 max-h-80 overflow-y-auto pr-1">
					{#each modelFiles as file}
						{@const fileSize = getEffectiveFileSize(file)}
						{@const fit = getPerformanceFit(fileSize)}
						{@const downloadedModel = findDownloadedModelForFile(file, inspectingModel)}
						{@const isDownloading = enqueuedFiles[file.rfilename]}

						<Card
							padding="sm"
							class="transition-all {downloadedModel
								? 'border border-[var(--primary)] bg-[var(--primary-light)] shadow-xs'
								: 'border border-[var(--border-color)]/70'}"
						>
							<div class="flex items-center justify-between gap-3 text-xs">
								<div class="min-w-0 flex-1">
									<div class="flex items-center gap-2 flex-wrap">
										<span
											class="w-2.5 h-2.5 rounded-full shrink-0 shadow-xs {fit.color}"
											title={fit.title}
										></span>
										<span class="font-mono text-[var(--text-primary)] font-medium truncate block">
											{file.rfilename.split('/').pop()}
										</span>
										{#if file.quant}
											<Badge variant="info" class="font-mono text-[10px]">
												{file.quant}
											</Badge>
										{/if}
									</div>
									<div class="flex items-center gap-2 mt-1">
										{#if fileSize > 0}
											<span class="text-[11px] text-[var(--text-muted)] font-mono">
												{formatBytes(fileSize)}
											</span>
											<span class="text-[10px] text-[var(--border-color)]">•</span>
										{/if}
										<span
											class="text-[11px] text-[var(--text-muted)] cursor-help truncate"
											title={fit.title}
										>
											{fit.title}
										</span>
									</div>
								</div>

								<div class="shrink-0 flex items-center gap-2">
									{#if isDownloading}
										<Button variant="success" size="sm" disabled>
											<ArrowDownToLine size={13} class="animate-pulse text-[var(--success-text)]" />
											<span>{t('common.downloading')}</span>
										</Button>
									{:else if downloadedModel}
										<Button
											variant="danger"
											size="sm"
											onclick={() => handleDeleteDownloaded(downloadedModel)}
											title={t('common.delete')}
										>
											<Trash size={13} />
											<span>{t('common.delete')}</span>
										</Button>
									{:else}
										<Button
											variant="primary"
											size="sm"
											onclick={() => handleDownload(file)}
											title={t('common.download')}
										>
											<Download size={13} />
											<span>{t('common.download')}</span>
										</Button>
									{/if}
								</div>
							</div>
						</Card>
					{/each}
				</div>
			{/if}
		</div>
	</Modal>
{/if}

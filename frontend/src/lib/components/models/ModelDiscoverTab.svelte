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
		Select,
		Spinner
	} from '$lib/components/ui';
	import { showAlert } from '$lib/confirmStore';
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
	import { getModalityBadgeVariant, getModalityOptions } from '$lib/utils/model';
	import {
		Box,
		ChevronLeft,
		ChevronRight,
		Download,
		Globe,
		Heart,
		Info,
		Search,
		Sparkles
	} from '@lucide/svelte';
	import { onMount } from 'svelte';
	import Divider from '../ui/atoms/Divider.svelte';
	import ModelDetailsModal from './ModelDetailsModal.svelte';

	let repositories = $state<RepositoryRecord[]>([]);
	let selectedRepoId = $state<string>('');
	let searchQuery = $state('');
	let selectedModality = $state<string>('all');
	let selectedSort = $state<'downloads' | 'likes' | 'lastModified'>('downloads');
	let currentPage = $state(0);

	let isLoadingRepos = $state(true);
	let isSearching = $state(false);
	let searchResults = $state<HubModelItem[]>([]);
	let errorMessage = $state<string | null>(null);

	// Registered local models for installed detection
	let registeredModels = $state<ModelRecord[]>([]);

	// Inspecting model state (delegated to ModelDetailsModal)
	let inspectingModel = $state<HubModelItem | null>(null);

	let enabledRepos = $derived(repositories.filter((r) => r.enabled));

	let lastCompletedCount = $state<number | null>(null);

	$effect(() => {
		const current = downloadStore.completedCount;
		if (lastCompletedCount !== null && current > lastCompletedCount) {
			loadRegisteredModels();
		}
		lastCompletedCount = current;
	});

	let modalityOptions = $derived(getModalityOptions());

	let sortOptions = $derived([
		{ value: 'downloads', label: t('models.discover.sortDownloads') },
		{ value: 'likes', label: t('models.discover.sortLikes') },
		{ value: 'lastModified', label: t('models.discover.sortUpdated') }
	]);

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

			if (model) {
				const normModelId = model.id.toLowerCase();
				const baseId = (model.id.split('/').pop() || model.id).toLowerCase();
				if (m.source_repo && m.source_repo.toLowerCase() === normModelId) return true;
				const rel = m.relative_path.toLowerCase();
				if (rel.includes(`/${normModelId}/`) || rel.includes(`/${baseId}/`)) return true;
				if (GENERIC_FILENAMES.has(baseName)) return false;
			}

			return !GENERIC_FILENAMES.has(baseName);
		});
	}

	function isModelInstalled(model: HubModelItem): boolean {
		const normId = model.id.toLowerCase();
		const baseId = (model.id.split('/').pop() || model.id).toLowerCase();

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

		if (model.siblings && model.siblings.length > 0) {
			return model.siblings.some((s) => !!findDownloadedModelForFile(s, model));
		}

		return false;
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

		if (currentPage > 0) {
			url.searchParams.set('page', String(currentPage + 1));
		} else {
			url.searchParams.delete('page');
		}

		// Clean installed-specific params
		url.searchParams.delete('status');

		const rawSearch = url.searchParams.toString().replace(/%20/g, '+');
		const target = `${url.pathname}${rawSearch ? '?' + rawSearch : ''}`;
		if (target !== `${page.url.pathname}${page.url.search}`) {
			goto(target, { replaceState: true, noScroll: true, keepFocus: true });
		}
	}

	async function executeSearch(resetPage = false) {
		if (resetPage) {
			currentPage = 0;
		}
		if (!selectedRepoId) return;

		updateUrlParams();

		isSearching = true;
		errorMessage = null;
		try {
			searchResults = await hubApi.searchModels({
				repo_id: selectedRepoId,
				query: searchQuery.trim() || undefined,
				modality: selectedModality === 'all' ? undefined : selectedModality,
				sort: selectedSort,
				page: currentPage
			});
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			errorMessage = msg;
		} finally {
			isSearching = false;
		}
	}

	function handlePreviousPage() {
		if (currentPage > 0) {
			currentPage--;
			executeSearch(false);
		}
	}

	function handleNextPage() {
		currentPage++;
		executeSearch(false);
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
		const pageParam = p.get('page');
		if (pageParam) {
			const parsed = parseInt(pageParam, 10);
			if (!isNaN(parsed) && parsed > 0) {
				currentPage = parsed - 1;
			}
		}

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
			<div class="flex flex-col xl:flex-row items-stretch xl:items-center gap-2">
				<!-- Search Input -->
				<div class="flex-1 min-w-[200px]">
					<Input
						bind:value={searchQuery}
						onkeydown={(e) => e.key === 'Enter' && executeSearch(true)}
						placeholder={t('models.discover.searchPlaceholder')}
						clearable
					>
						{#snippet prefix()}
							<Search size={15} />
						{/snippet}
					</Input>
				</div>

				<!-- Filter Dropdowns & Action Button -->
				<div class="flex flex-col sm:flex-row gap-2">
					<!-- Modality Filter -->
					<div class="grow-1">
						<Select
							bind:value={selectedModality}
							onchange={() => executeSearch(true)}
							options={modalityOptions}
						/>
					</div>

					<!-- Repo Filter -->
					<div class="grow-1">
						<Select
							bind:value={selectedRepoId}
							onchange={() => executeSearch(true)}
							options={enabledRepos.map((r) => ({ value: r.id, label: r.name }))}
						/>
					</div>

					<!-- Sort Filter -->
					<div class="grow-1">
						<Select
							bind:value={selectedSort}
							onchange={() => executeSearch(true)}
							options={sortOptions}
						/>
					</div>
				</div>
			</div>
		</Card>

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
					<Card class="overflow-hidden">
						{#if isModelInstalled(model)}
							<DiagonalLabel text={t('common.installed')} />
						{:else if model.requires_auth}
							<DiagonalLabel variant="warning" text={t('models.discover.authRequired')} />
						{/if}

						<div class="flex h-full flex-col justify-between gap-2">
							<div class="flex items-center justify-start gap-2 pr-12">
								<div class="w-5 h-5 flex items-center justify-center">
									<Box class="text-[var(--primary)]" />
								</div>
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

							<div class="flex gap-1 flex-wrap">
								<Badge
									variant={getModalityBadgeVariant(model.modality)}
									rounded="rounded-md"
									size="sm"
								>
									{model.modality}
								</Badge>
								{#if model.pipeline_tag}
									<Badge variant="muted" rounded="rounded-md" size="sm">
										{model.pipeline_tag}
									</Badge>
								{/if}
							</div>

							<Divider class="mt-auto" />

							<div class="flex justify-between gap-2">
								<div class="flex justify-start items-center gap-3 text-xs text-[var(--text-muted)]">
									<span class="flex items-center gap-1" title="Likes">
										<Heart size={13} class="text-[var(--danger-text)]" />
										<span class="font-medium">{(model.likes ?? 0).toLocaleString()}</span>
									</span>
									<span class="flex items-center gap-1" title="Downloads">
										<Download size={13} class="text-[var(--primary)]" />
										<span class="font-medium">{(model.downloads ?? 0).toLocaleString()}</span>
									</span>
								</div>

								<Button variant="secondary" size="sm" onclick={() => inspectFiles(model)}>
									<Info size={13} />
									<span>{t('common.details')}</span>
								</Button>
							</div>
						</div>
					</Card>
				{/each}
			</div>

			<!-- Pagination Controls -->
			{#if searchResults.length > 0 || currentPage > 0}
				<div class="flex items-center justify-between gap-2 pt-2 px-1">
					<Button
						variant="primary"
						size="sm"
						disabled={currentPage === 0 || isSearching}
						onclick={handlePreviousPage}
					>
						<ChevronLeft size={14} />
						<span>{t('common.previous')}</span>
					</Button>

					<span class="text-xs font-medium text-[var(--text-muted)]">
						{t('common.page')}
						{currentPage + 1}
					</span>

					<Button
						variant="primary"
						size="sm"
						disabled={searchResults.length < 30 || isSearching}
						onclick={handleNextPage}
					>
						<span>{t('common.next')}</span>
						<ChevronRight size={14} />
					</Button>
				</div>
			{/if}
		{/if}
	{/if}
</div>

<!-- Model Details Modal (README + Files) -->
{#if inspectingModel}
	<ModelDetailsModal
		open={!!inspectingModel}
		modelId={inspectingModel.id}
		title={inspectingModel.id}
		repoId={selectedRepoId}
		installedModels={registeredModels}
		onclose={() => (inspectingModel = null)}
		onModelUpdated={loadRegisteredModels}
	/>
{/if}

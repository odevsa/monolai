<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { modelsApi } from '$lib/api/models';
	import {
		ModelAuxiliaryCard,
		ModelCard,
		ModelDetailsModal,
		ModelDiscoverTab,
		ModelDownloadsTab
	} from '$lib/components/models';
	import {
		Alert,
		Badge,
		Button,
		Card,
		EmptyState,
		Input,
		PageHeader,
		Select,
		Spinner,
		Tabs
	} from '$lib/components/ui';
	import Divider from '$lib/components/ui/atoms/Divider.svelte';
	import { askConfirm, showAlert } from '$lib/confirmStore';
	import { refreshFeatures } from '$lib/featuresStore';
	import { t } from '$lib/i18n';
	import { runningModelsState } from '$lib/runningModelsStore';
	import { downloadStore } from '$lib/state/downloads.svelte';
	import type { ModelRecord, RunningModelStatus } from '$lib/types/models';
	import { copyToClipboard } from '$lib/utils/clipboard';
	import { getModalityOptions } from '$lib/utils/model';
	import { ArrowDownToLine, Box, HardDrive, RefreshCw, Search, Sparkles } from '@lucide/svelte';
	import { onMount } from 'svelte';

	type TabType = 'discover' | 'installed' | 'downloads';

	let activeTab = $state<TabType>('installed');
	let registeredModels = $state<ModelRecord[]>([]);
	let isLoadingData = $state(true);
	let error = $state<string | null>(null);

	// Filter and search states for Installed tab
	let searchQuery = $state('');
	let selectedModalityFilter = $state<string>('all');
	let selectedStatusFilter = $state<'all' | 'running' | 'missing'>('all');
	let copiedId = $state<string | null>(null);

	// Details & action loading states
	let selectedDetailsModel = $state<ModelRecord | null>(null);
	let deletingModelId = $state<string | null>(null);
	let isRefreshing = $state(false);

	let tabItems = $derived([
		{ id: 'discover', label: t('models.tabs.discover'), icon: Sparkles },
		{
			id: 'installed',
			label: t('models.tabs.installed'),
			icon: HardDrive,
			badge: registeredModels.length || undefined
		},
		{
			id: 'downloads',
			label: t('models.tabs.downloads'),
			icon: ArrowDownToLine,
			badge: downloadStore.activeCount || undefined
		}
	]);

	let modalityOptions = $derived(getModalityOptions());

	const statusOptions = [
		{ value: 'all', label: t('models.allStatuses') },
		{ value: 'running', label: t('models.runningStatus') },
		{ value: 'missing', label: t('models.missingStatus') }
	];

	// Running models map for instant lookup
	let runningMap = $derived.by(() => {
		const map = new Map<string, RunningModelStatus>();
		for (const rm of runningModelsState.runningModels) {
			map.set(rm.model_id, rm);
		}
		return map;
	});

	// Cache query params per tab to preserve each tab's independent state
	let tabQueryMap: Record<TabType, string> = {
		discover: '',
		installed: '',
		downloads: ''
	};

	// URL Query param tab synchronization
	$effect(() => {
		const tabParam = page.url.searchParams.get('tab');
		if (tabParam === 'discover' || tabParam === 'installed' || tabParam === 'downloads') {
			if (activeTab !== tabParam) {
				activeTab = tabParam;
				if (tabParam === 'installed') {
					searchQuery = page.url.searchParams.get('q') || '';
					selectedModalityFilter = page.url.searchParams.get('modality') || 'all';
					const st = page.url.searchParams.get('status') || 'all';
					selectedStatusFilter = st === 'running' || st === 'missing' ? st : 'all';
				}
			}
		}
	});

	function updateInstalledUrlParams() {
		if (activeTab !== 'installed') return;
		const url = new URL(page.url);
		url.searchParams.set('tab', 'installed');

		const trimmed = searchQuery.trim();
		if (trimmed) {
			url.searchParams.set('q', trimmed);
		} else {
			url.searchParams.delete('q');
		}

		if (selectedModalityFilter && selectedModalityFilter !== 'all') {
			url.searchParams.set('modality', selectedModalityFilter);
		} else {
			url.searchParams.delete('modality');
		}

		if (selectedStatusFilter && selectedStatusFilter !== 'all') {
			url.searchParams.set('status', selectedStatusFilter);
		} else {
			url.searchParams.delete('status');
		}

		// Clean discover-specific params
		url.searchParams.delete('sort');
		url.searchParams.delete('repo');
		url.searchParams.delete('page');

		const rawSearch = url.searchParams.toString().replace(/%20/g, '+');
		const target = `${url.pathname}${rawSearch ? '?' + rawSearch : ''}`;
		if (target !== `${page.url.pathname}${page.url.search}`) {
			goto(target, { replaceState: true, noScroll: true, keepFocus: true });
		}
	}

	function setTab(newTab: TabType) {
		const oldTab = (page.url.searchParams.get('tab') as TabType) || activeTab;

		// 1. Save current tab's query params (excluding tab parameter itself)
		const currentUrl = new URL(page.url);
		currentUrl.searchParams.delete('tab');
		tabQueryMap[oldTab] = currentUrl.searchParams.toString();

		// 2. Switch tab
		activeTab = newTab;

		// 3. Restore target tab's query params
		const newParams = new URLSearchParams(tabQueryMap[newTab] || '');
		newParams.set('tab', newTab);

		const target = `${page.url.pathname}?${newParams.toString()}`;
		goto(target, { replaceState: true, noScroll: true });

		// 4. If switching to installed, sync filter state
		if (newTab === 'installed') {
			const q = newParams.get('q') || '';
			searchQuery = q;
			const mod = newParams.get('modality') || 'all';
			selectedModalityFilter = mod;
			const st = newParams.get('status') || 'all';
			selectedStatusFilter = st === 'running' || st === 'missing' ? st : 'all';
			loadModels(true);
		}
	}

	// Filtered models list
	let filteredModels = $derived.by(() => {
		let list = registeredModels;

		const q = searchQuery.trim().toLowerCase();
		if (q) {
			list = list.filter((m) => {
				const path = (m.relative_path || m.filename || '').toLowerCase();
				return (
					m.id.toLowerCase().includes(q) ||
					(m.name && m.name.toLowerCase().includes(q)) ||
					path.includes(q) ||
					(m.architecture && m.architecture.toLowerCase().includes(q))
				);
			});
		}

		if (selectedModalityFilter !== 'all') {
			list = list.filter((m) => m.modality.toLowerCase() === selectedModalityFilter.toLowerCase());
		}

		if (selectedStatusFilter === 'running') {
			list = list.filter((m) => runningMap.has(m.id));
		} else if (selectedStatusFilter === 'missing') {
			list = list.filter((m) => m.file_exists === false);
		}

		return list;
	});

	let primaryModels = $derived(filteredModels.filter((m) => !m.is_auxiliary));
	let auxiliaryModels = $derived(filteredModels.filter((m) => m.is_auxiliary));

	let lastCompletedDownloads = $state<number | null>(null);

	$effect(() => {
		const current = downloadStore.completedCount;
		if (lastCompletedDownloads !== null && current > lastCompletedDownloads) {
			loadModels(true);
			refreshFeatures(true);
		}
		lastCompletedDownloads = current;
	});

	async function loadModels(isSilent = false, sync = false) {
		if (!isSilent) isLoadingData = true;
		error = null;

		try {
			registeredModels = await modelsApi.list(false, sync);
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			error = msg || t('models.errorLoading');
			console.error('Error loading models:', err);
		} finally {
			isLoadingData = false;
		}
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

	async function deleteModel(id: string) {
		const confirmed = await askConfirm(
			t('models.deleteConfirmMsg', { id }),
			t('models.deleteConfirmTitle'),
			'danger',
			t('common.delete'),
			t('common.cancel')
		);
		if (!confirmed) return;

		deletingModelId = id;
		try {
			await modelsApi.delete(id, true);
			await loadModels(true);
			refreshFeatures(true);
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			await showAlert(msg || t('models.deleteError'), t('common.error'), 'danger');
		} finally {
			deletingModelId = null;
		}
	}

	async function deleteAuxiliaryFile(id: string, filename: string) {
		const confirmed = await askConfirm(
			t('models.auxiliary.deleteConfirmMsg', { filename }),
			t('models.auxiliary.deleteConfirmTitle'),
			'danger',
			t('common.delete'),
			t('common.cancel')
		);
		if (!confirmed) return;

		deletingModelId = id;
		try {
			await modelsApi.delete(id, true);
			await loadModels(true);
			refreshFeatures(true);
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			await showAlert(msg || t('models.deleteError'), t('common.error'), 'danger');
		} finally {
			deletingModelId = null;
		}
	}

	onMount(() => {
		const p = page.url.searchParams;
		const tabParam = p.get('tab');
		if (tabParam === 'discover' || tabParam === 'installed' || tabParam === 'downloads') {
			activeTab = tabParam;
		}

		// Initialize query map for active tab
		const initialParams = new URLSearchParams(p);
		initialParams.delete('tab');
		tabQueryMap[activeTab] = initialParams.toString();

		if (activeTab === 'installed') {
			const q = p.get('q');
			if (q) searchQuery = q;
			const mod = p.get('modality');
			if (mod) selectedModalityFilter = mod;
			const status = p.get('status');
			if (status === 'running' || status === 'missing') {
				selectedStatusFilter = status;
			}
		}

		loadModels();
	});
</script>

<svelte:head>
	<title>{t('app.title')} - {t('models.pageTitle')}</title>
</svelte:head>

<div class="w-full px-4 sm:px-8 py-5 md:py-8 pt-16 md:pt-16 max-w-7xl mx-auto box-border space-y-6">
	<!-- Page Header -->
	<PageHeader title={t('models.pageTitle')} subtitle={t('models.pageSubtitle')} icon={Box} />

	<!-- Sub-tabs Navigation Bar -->
	<Tabs
		variant="pills"
		items={tabItems}
		bind:value={activeTab}
		onchange={(val) => setTab(val as TabType)}
	/>

	{#if error}
		<Alert variant="error" title={t('common.connectionFailure')}>
			<div class="flex items-center justify-between gap-3">
				<span>{error}</span>
				<Button variant="secondary" size="sm" onclick={() => loadModels()}>
					{t('common.refresh')}
				</Button>
			</div>
		</Alert>
	{/if}

	<!-- Tab Views -->
	{#if activeTab === 'discover'}
		<ModelDiscoverTab />
	{:else if activeTab === 'downloads'}
		<ModelDownloadsTab />
	{:else if activeTab === 'installed'}
		<!-- Filter & Search Bar for Installed Models -->
		<Card padding="sm">
			<div class="flex flex-col xl:flex-row items-stretch xl:items-center gap-2">
				<!-- Search Input -->
				<div class="flex-1 min-w-[200px]">
					<Input
						bind:value={searchQuery}
						oninput={updateInstalledUrlParams}
						placeholder={t('models.searchPlaceholder')}
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
							bind:value={selectedModalityFilter}
							onchange={updateInstalledUrlParams}
							options={modalityOptions}
						/>
					</div>

					<!-- Status Filter -->
					<div class="grow-1">
						<Select
							bind:value={selectedStatusFilter}
							onchange={updateInstalledUrlParams}
							options={statusOptions}
						/>
					</div>
				</div>
			</div>
		</Card>

		<!-- Installed Models Catalog Grid -->
		{#if isLoadingData && registeredModels.length === 0}
			<div class="flex flex-col items-center justify-center p-16 text-[var(--text-muted)] gap-3">
				<Spinner size={32} />
				<span class="text-xs">{t('common.loading')}</span>
			</div>
		{:else if registeredModels.length === 0}
			<EmptyState
				title={t('models.noModelsFound')}
				description={t('models.noModelsFoundDesc')}
				icon={Box}
			>
				{#snippet actions()}
					<Button variant="primary" onclick={() => setTab('discover')}>
						<Sparkles size={15} />
						<span>{t('models.discoverModels')}</span>
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
							selectedModalityFilter = 'all';
							selectedStatusFilter = 'all';
						}}
					>
						{t('common.clear')}
					</Button>
				{/snippet}
			</EmptyState>
		{:else}
			<div class="space-y-6">
				{#if primaryModels.length > 0}
					<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3.5">
						{#each primaryModels as model (model.id)}
							<ModelCard
								{model}
								runningInfo={runningMap.get(model.id)}
								{copiedId}
								isDeleting={deletingModelId === model.id}
								onCopyId={copyId}
								onDetails={(m) => (selectedDetailsModel = m)}
								onModelUpdated={() => loadModels(true)}
								onDelete={deleteModel}
							/>
						{/each}
					</div>
				{/if}

				<Divider />

				<!-- Auxiliary Models Grid -->
				<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
					{#each auxiliaryModels as model (model.id)}
						<ModelAuxiliaryCard {model} onDelete={deleteAuxiliaryFile} />
					{/each}
				</div>
			</div>
		{/if}
	{/if}
</div>

{#if selectedDetailsModel}
	<ModelDetailsModal
		open={!!selectedDetailsModel}
		modelId={selectedDetailsModel.source_repo ||
			(selectedDetailsModel.id.includes('/')
				? selectedDetailsModel.id.split(':')[0]
				: selectedDetailsModel.id)}
		title={selectedDetailsModel.name || selectedDetailsModel.id}
		installedModels={registeredModels}
		onclose={() => (selectedDetailsModel = null)}
		onModelUpdated={() => loadModels(true)}
	/>
{/if}

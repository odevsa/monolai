<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { modelsApi } from '$lib/api/models';
	import { ModelCard, ModelDiscoverTab, ModelDownloadsTab } from '$lib/components/models';
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
	import { askConfirm, showAlert } from '$lib/confirmStore';
	import { refreshFeatures } from '$lib/featuresStore';
	import { t } from '$lib/i18n';
	import { runningModelsState } from '$lib/runningModelsStore';
	import { downloadStore } from '$lib/state/downloads.svelte';
	import type { ModelRecord, RunningModelStatus } from '$lib/types/models';
	import { copyToClipboard } from '$lib/utils/clipboard';
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

	const modalityOptions = [
		{ value: 'all', label: t('models.allModalities') },
		{ value: 'text-to-text', label: 'Text (LLM)' },
		{ value: 'text-to-image', label: 'Image (Diffusion)' },
		{ value: 'text-to-speech', label: 'Audio TTS' },
		{ value: 'speech-to-text', label: 'Audio ASR' },
		{ value: 'embeddings', label: 'Embeddings' }
	];

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

	// URL Query param tab synchronization
	$effect(() => {
		const tabParam = page.url.searchParams.get('tab');
		if (tabParam === 'discover' || tabParam === 'installed' || tabParam === 'downloads') {
			if (activeTab !== tabParam) {
				activeTab = tabParam;
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

		const rawSearch = url.searchParams.toString().replace(/%20/g, '+');
		const target = `${url.pathname}${rawSearch ? '?' + rawSearch : ''}`;
		if (target !== `${page.url.pathname}${page.url.search}`) {
			goto(target, { replaceState: true, noScroll: true, keepFocus: true });
		}
	}

	function setTab(tab: TabType) {
		activeTab = tab;
		const url = new URL(page.url);
		url.searchParams.set('tab', tab);

		// Clean query params of other tab
		url.searchParams.delete('q');
		url.searchParams.delete('modality');
		url.searchParams.delete('sort');
		url.searchParams.delete('repo');
		url.searchParams.delete('status');

		const rawSearch = url.searchParams.toString();
		goto(`${url.pathname}?${rawSearch}`, { replaceState: true, noScroll: true });
		if (tab === 'installed') {
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

		try {
			await modelsApi.delete(id, true);
			await loadModels(true);
			refreshFeatures(true);
		} catch (err: unknown) {
			const msg = err instanceof Error ? err.message : String(err);
			await showAlert(msg || t('models.deleteError'), t('common.error'), 'danger');
		}
	}

	onMount(() => {
		const p = page.url.searchParams;
		const tabParam = p.get('tab');
		if (tabParam === 'discover' || tabParam === 'installed' || tabParam === 'downloads') {
			activeTab = tabParam;
		}

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
	<PageHeader title={t('models.pageTitle')} subtitle={t('models.pageSubtitle')} icon={Box}>
		<Button
			variant="secondary"
			size="md"
			onclick={() => loadModels(false, true)}
			disabled={isLoadingData}
		>
			<RefreshCw size={14} class={isLoadingData ? 'animate-spin text-[var(--primary)]' : ''} />
			<span>{t('common.refresh')}</span>
		</Button>
	</PageHeader>

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
			<div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-3">
				<div class="flex-1">
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

				<div class="flex items-center gap-2">
					<div class="w-44">
						<Select
							bind:value={selectedModalityFilter}
							onchange={updateInstalledUrlParams}
							options={modalityOptions}
						/>
					</div>

					<div class="w-44">
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
			<div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3.5">
				{#each filteredModels as model (model.id)}
					<ModelCard
						{model}
						runningInfo={runningMap.get(model.id)}
						{copiedId}
						onCopyId={copyId}
						onModelUpdated={() => loadModels(true)}
						onDelete={deleteModel}
					/>
				{/each}
			</div>
		{/if}
	{/if}
</div>
